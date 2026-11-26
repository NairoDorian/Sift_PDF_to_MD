//! Sift's window (MASTER_PLAN section 9): Tauri 3 + SolidJS 2, the ZER0 way.
//!
//! - With a subcommand (`sift-app convert ...`, `sift-app engine update`), the binary runs
//!   headless: the very same command line as `sift`, no window.
//! - Without one it opens the window. Commands never block the webview (ZER0's rule): long work
//!   runs on its own thread and reports through events (`ipc.rs`; `sift-event` is docs/events.md).
//! - The commands and events are typed: tauri-specta writes them into `app/src/lib/bindings.ts`
//!   (a debug start-up rewrites it when it changed; the `bindings_are_current` test checks it).
//! - Settings: the one `settings.toml` shared with the command line (`sift_core::paths`); the
//!   engine and the models live only in the folders the user picks here.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ipc;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

use ipc::*;
use sift_core::config::Config;
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use tauri_specta::Event;

static BUSY: AtomicBool = AtomicBool::new(false);
static CANCEL: LazyLock<Mutex<Option<Arc<AtomicBool>>>> = LazyLock::new(|| Mutex::new(None));

fn err(e: anyhow::Error) -> String {
    format!("{e:#}")
}

/// Run `job` on its own thread unless another long job (conversion, engine install) runs.
fn exclusive(job: impl FnOnce() + Send + 'static) -> Result<(), String> {
    if BUSY.swap(true, Ordering::SeqCst) {
        return Err("Busy: wait for the current job to finish.".into());
    }
    std::thread::spawn(move || {
        job();
        BUSY.store(false, Ordering::SeqCst);
    });
    Ok(())
}

// ---- conversion ------------------------------------------------------------------------

#[tauri::command]
#[specta::specta]
fn start_convert(app: AppHandle, req: ConvertRequest) -> Result<(), String> {
    let cancel = Arc::new(AtomicBool::new(false));
    let c2 = cancel.clone();
    exclusive(move || {
        *CANCEL.lock().unwrap() = Some(c2.clone());
        let r = (|| -> anyhow::Result<i32> {
            let mut cfg = Config::load(None)?;
            // set, never merged: an option switched off on the page is off for this run
            if let Some(p) = req.pages.filter(|p| !p.trim().is_empty()) {
                cfg.convert.pages = p;
            }
            cfg.convert.mode = req.mode;
            cfg.convert.tables = req.tables;
            cfg.convert.keep_headers = req.keep_headers;
            cfg.convert.describer = req.describer.clone();
            if let Some(dpi) = req.image_dpi.filter(|d| (72..=1200).contains(d)) {
                cfg.convert.image_dpi = dpi;
            }
            cfg.llm.enabled = req.llm;
            let events = Arc::new(sift_core::events::Events::new(false));
            let a2 = app.clone();
            events.sinks.lock().unwrap().push(Box::new(move |v| {
                let _ = ConvertEvent(v.clone()).emit(&a2);
            }));
            let store = Arc::new(sift_core::answers::AnswerStore::new(None, vec![], false));
            let opts = sift_core::batch::BatchOptions {
                output: req.output.filter(|o| !o.trim().is_empty()).map(Into::into),
                recursive: req.recursive,
                force: req.force,
                pipeline: sift_core::pipeline::Options {
                    use_surya: true,
                    describer: req.describer.clone(),
                    layout_replay: None,
                    profile: false,
                    mask_text_layer: false,
                },
                cancel: Some(c2.clone()),
            };
            Ok(sift_core::batch::run(cfg, Path::new(&req.input), opts, events, store)?.0)
        })();
        let cancelled = c2.load(Ordering::Relaxed);
        let done = match r {
            Ok(code) => ConvertDone { ok: code == 0 && !cancelled, cancelled, error: None },
            Err(e) => ConvertDone { ok: false, cancelled, error: Some(format!("{e:#}")) },
        };
        let _ = done.emit(&app);
        *CANCEL.lock().unwrap() = None;
    })
}

/// Cancel: queued work stops first, then the model servers; no model starts afterwards.
#[tauri::command]
#[specta::specta]
fn cancel_convert() -> bool {
    match CANCEL.lock().unwrap().as_ref() {
        Some(c) => {
            c.store(true, Ordering::Relaxed);
            true
        }
        None => false,
    }
}

// ---- settings and folders ------------------------------------------------------------------

#[tauri::command]
#[specta::specta]
async fn settings_get() -> Result<Config, String> {
    Config::load_saved().map_err(err)
}

#[tauri::command]
#[specta::specta]
async fn settings_save(settings: Config) -> Result<(), String> {
    settings.save().map_err(err)
}

/// A folder picker (the dialog plugin, called from Rust: the page needs no plugin permission).
#[tauri::command]
#[specta::specta]
async fn pick_folder(app: AppHandle, title: String) -> Option<String> {
    app.dialog()
        .file()
        .set_title(title)
        .blocking_pick_folder()
        .and_then(|p| p.into_path().ok())
        .map(|p| p.display().to_string())
}

/// A one-PDF picker for the Convert page.
#[tauri::command]
#[specta::specta]
async fn pick_pdf(app: AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .set_title("Choose a PDF")
        .add_filter("PDF", &["pdf"])
        .blocking_pick_file()
        .and_then(|p| p.into_path().ok())
        .map(|p| p.display().to_string())
}

/// What the Convert page shows about its input: a PDF (pages, size) or a folder (how many PDFs,
/// at its top level and with its subfolders: the same list a conversion walks, `batch::find_pdfs`).
#[tauri::command]
#[specta::specta]
async fn input_info(path: String) -> Result<InputInfo, String> {
    let p = PathBuf::from(path.trim());
    tauri::async_runtime::spawn_blocking(move || {
        let meta = std::fs::metadata(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| p.display().to_string());
        let dir = p.parent().map(|d| d.display().to_string()).unwrap_or_default();
        let path = p.display().to_string();
        if meta.is_dir() {
            let pdfs = sift_core::batch::find_pdfs(&p, false).len();
            let pdfs_all = sift_core::batch::find_pdfs(&p, true).len();
            return Ok(InputInfo::Folder { name, dir, path, pdfs, pdfs_all });
        }
        if !p.extension().is_some_and(|e| e.eq_ignore_ascii_case("pdf")) {
            return Err(format!("{name} is not a PDF"));
        }
        // the page count is a nicety: a PDF pdfium cannot open still goes to the converter,
        // which reports the reason
        let pages = sift_core::pdf::pdfium::Document::open(&p, false).ok().map(|d| d.page_count);
        Ok(InputInfo::Pdf { name, dir, path, size: meta.len(), pages })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// The first lines of a converted Markdown file, for the result card.
#[tauri::command]
#[specta::specta]
async fn md_preview(path: String, max_bytes: Option<usize>) -> Result<String, String> {
    let p = PathBuf::from(&path);
    if !p.extension().is_some_and(|e| e.eq_ignore_ascii_case("md")) {
        return Err("not a Markdown file".into());
    }
    use std::io::Read;
    let limit = max_bytes.unwrap_or(6000).min(64_000) as u64;
    let mut bytes = Vec::new();
    std::fs::File::open(&p)
        .and_then(|f| f.take(limit).read_to_end(&mut bytes))
        .map_err(|e| format!("{}: {e}", p.display()))?;
    // a cut inside a multi-byte character leaves a replacement character at the end: drop it
    Ok(String::from_utf8_lossy(&bytes).trim_end_matches('\u{FFFD}').to_string())
}

/// Set the engine or models folder. `move_files`: move what the current folder holds into the
/// new one (same drive: a rename); otherwise the new folder is used as it is. The old folder is
/// never deleted here.
#[tauri::command]
#[specta::specta]
async fn set_folder(kind: FolderKind, path: String, move_files: bool) -> Result<FolderMove, String> {
    let new = PathBuf::from(path.trim());
    std::fs::create_dir_all(&new).map_err(|e| format!("cannot use {}: {e}", new.display()))?;
    let probe = new.join(".sift-write-test");
    std::fs::write(&probe, b"").map_err(|e| format!("{} is not writable: {e}", new.display()))?;
    let _ = std::fs::remove_file(&probe);
    let mut cfg = Config::load_saved().map_err(err)?;
    let old = match kind {
        FolderKind::Engine => cfg.folders.engine_dir.clone(),
        FolderKind::Models => cfg.folders.models_dir.clone(),
    };
    let mut moved = Vec::new();
    if move_files && !old.trim().is_empty() && Path::new(&old) != new.as_path() {
        for e in std::fs::read_dir(&old).map_err(|e| e.to_string())?.flatten() {
            let to = new.join(e.file_name());
            if to.exists() {
                continue;
            }
            std::fs::rename(e.path(), &to).map_err(|x| {
                format!(
                    "cannot move {} to {} ({x}); move the files yourself, then choose 'use as it is'",
                    e.path().display(),
                    to.display()
                )
            })?;
            moved.push(e.file_name().to_string_lossy().to_string());
        }
    }
    let s = new.display().to_string();
    match kind {
        FolderKind::Engine => cfg.folders.engine_dir = s,
        FolderKind::Models => cfg.folders.models_dir = s,
    }
    cfg.save().map_err(err)?;
    Ok(FolderMove { moved, old })
}

/// The model servers' command lines as Sift would start them now (Engine page, read-only).
#[tauri::command]
#[specta::specta]
async fn server_commands() -> Result<Vec<ServerCommand>, String> {
    let cfg = Config::load(None).map_err(err)?;
    let lines = sift_core::pipeline::server_command_lines(&cfg).map_err(err)?;
    Ok(lines.into_iter().map(|(model, args)| ServerCommand { model, args }).collect())
}

/// The end of a model server's newest log in a run's output folder (`<output>\.sift\logs`): the
/// Engine page shows it while a server runs, and the last run's afterwards.
#[tauri::command]
#[specta::specta]
async fn server_log(model: String, output: String) -> Result<ServerLog, String> {
    use std::io::{Read, Seek, SeekFrom};
    if model != "surya" && model != "gemma" {
        return Err(format!("no model server named {model}"));
    }
    let dir = Path::new(&output).join(".sift").join("logs");
    let newest = std::fs::read_dir(&dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .flatten()
        .filter(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            n.starts_with(&format!("{model}-")) && n.ends_with(".log")
        })
        .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
    let Some(entry) = newest else { return Ok(ServerLog { file: None, lines: vec![] }) };
    let mut f = std::fs::File::open(entry.path()).map_err(|e| e.to_string())?;
    let len = f.metadata().map(|m| m.len()).unwrap_or(0);
    // the last 48 KB is plenty for the lines shown, whatever the log's size
    f.seek(SeekFrom::Start(len.saturating_sub(48_000))).map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&buf);
    let mut lines: Vec<&str> = text.lines().collect();
    if len > 48_000 && !lines.is_empty() {
        lines.remove(0); // a partial first line
    }
    let tail = lines.iter().rev().take(80).rev().map(|l| l.to_string()).collect();
    Ok(ServerLog { file: Some(entry.path().display().to_string()), lines: tail })
}

/// Open a folder in Explorer, or a file with its default app: only what Sift writes (the
/// converted `.md`, the `.txt` report). Explorer *runs* a program it is handed, so nothing else
/// goes through here.
#[tauri::command]
#[specta::specta]
async fn open_path(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("{path} does not exist"));
    }
    let ours = p
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("md") || e.eq_ignore_ascii_case("txt"));
    if !p.is_dir() && !ours {
        return Err(format!("{path}: only folders, Markdown and reports open from here"));
    }
    std::process::Command::new("explorer").arg(&path).spawn().map(|_| ()).map_err(|e| e.to_string())
}

/// Show a file selected in its folder.
#[tauri::command]
#[specta::specta]
async fn reveal_path(path: String) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    if !Path::new(&path).exists() {
        return Err(format!("{path} does not exist"));
    }
    // explorer parses `/select,"<path>"` itself: the argument goes as written, not re-quoted
    std::process::Command::new("explorer")
        .raw_arg(format!("/select,\"{path}\""))
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// The About page's links (the repository, the licenses), in the default browser.
#[tauri::command]
#[specta::specta]
async fn open_url(url: String) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("only https links open".into());
    }
    std::process::Command::new("explorer").arg(&url).spawn().map(|_| ()).map_err(|e| e.to_string())
}

/// What the window needs to know at a glance (onboarding, footer, menu dot).
#[tauri::command]
#[specta::specta]
async fn overview() -> Overview {
    let cfg = Config::load_saved().unwrap_or_default();
    Overview {
        version: env!("CARGO_PKG_VERSION").to_string(),
        settings_file: sift_core::paths::settings_file().display().to_string(),
        portable: sift_core::paths::portable_dir().is_some(),
        engine_dir: cfg.folders.engine_dir,
        models_dir: cfg.folders.models_dir,
        engine_active: sift_core::engine::status().active,
        models: sift_core::models::all().unwrap_or_default().iter().map(|m| m.status()).collect(),
        check_at_startup: cfg.engine.check_at_startup,
    }
}

// ---- the engine (ZER0's llama.cpp page) ----------------------------------------------------

#[tauri::command]
#[specta::specta]
async fn engine_status() -> EngineStatus {
    let s = sift_core::engine::status();
    let cfg = Config::load_saved().unwrap_or_default();
    let cuda = if cfg.engine.backend.starts_with("cuda-") { cfg.engine.backend.as_str() } else { "cuda-13.5" };
    EngineStatus {
        active: s.active,
        previous: s.previous,
        version: s.version,
        dir: s.dir.map(|d| d.display().to_string()),
        detected: sift_core::engine::detect_backend(),
        driver_cuda: sift_core::engine::driver_cuda().map(|(a, b)| format!("{a}.{b}")),
        toolkit: sift_core::engine::system_cudart_dir(cuda).map(|p| p.display().to_string()),
        builds: sift_core::engine::list().unwrap_or_default(),
        backend: cfg.engine.backend,
        include_cudart: cfg.engine.include_cudart,
        check_at_startup: cfg.engine.check_at_startup,
    }
}

/// Read the release list (no download) and tell the window whether a newer build exists.
#[tauri::command]
#[specta::specta]
async fn engine_check(app: AppHandle) -> Result<sift_core::engine::Check, String> {
    let backend = Config::load_saved().map_err(err)?.engine.backend;
    let c = tauri::async_runtime::spawn_blocking(move || sift_core::engine::check(&backend))
        .await
        .map_err(|e| e.to_string())?
        .map_err(err)?;
    let _ = EngineChecked(c.clone()).emit(&app);
    Ok(c)
}

/// Install or update: download, verify, switch (`tag`: a given build instead of the newest).
#[tauri::command]
#[specta::specta]
fn engine_install(app: AppHandle, tag: Option<String>) -> Result<(), String> {
    let cfg = Config::load_saved().map_err(err)?;
    sift_core::paths::engine_dir().map_err(err)?;
    exclusive(move || {
        let a = app.clone();
        let log = move |s: &str| {
            let _ = EngineLog { message: s.to_string() }.emit(&a);
        };
        let a2 = app.clone();
        let progress = move |done: u64, total: u64| {
            let _ = EngineProgress { done, total }.emit(&a2);
        };
        let backend = sift_core::engine::wanted_backend(&cfg.engine.backend);
        let r = sift_core::engine::install(
            tag.as_deref(),
            Some(&backend),
            cfg.engine.include_cudart == "always",
            &log,
            &progress,
        );
        let done = match r {
            Ok(p) => EngineDone { ok: true, build: Some(p.display().to_string()), message: None, error: None },
            Err(e) => EngineDone { ok: false, build: None, message: None, error: Some(format!("{e:#}")) },
        };
        let _ = done.emit(&app);
    })
}

/// Use / Roll back / Delete / Remove the bundled CUDA runtime of an installed build.
#[tauri::command]
#[specta::specta]
fn engine_action(app: AppHandle, action: EngineAction, build: Option<String>) -> Result<(), String> {
    exclusive(move || {
        let a = app.clone();
        let log = move |s: &str| {
            let _ = EngineLog { message: s.to_string() }.emit(&a);
        };
        let b = build.unwrap_or_default();
        let r: anyhow::Result<String> = match action {
            EngineAction::Use => sift_core::engine::use_build(&b, &log).map(|_| format!("{b} is active")),
            EngineAction::Rollback => sift_core::engine::rollback(&log).map(|p| format!("{} is active", p.display())),
            EngineAction::Remove => sift_core::engine::remove(&b).map(|_| format!("deleted {b}")),
            EngineAction::RemoveRuntime => {
                sift_core::engine::remove_cuda_runtime(&b).map(|mb| format!("freed {mb} MB from {b}"))
            }
        };
        let done = match r {
            Ok(m) => EngineDone { ok: true, build: None, message: Some(m), error: None },
            Err(e) => EngineDone { ok: false, build: None, message: None, error: Some(format!("{e:#}")) },
        };
        let _ = done.emit(&app);
    })
}

// ---- the models ------------------------------------------------------------------------

#[tauri::command]
#[specta::specta]
fn model_download(app: AppHandle, id: String) -> Result<(), String> {
    sift_core::paths::models_dir().map_err(err)?;
    let entries = if id == "all" {
        sift_core::models::all().map_err(err)?
    } else {
        vec![sift_core::models::get(&id).map_err(err)?]
    };
    std::thread::spawn(move || {
        for e in entries {
            let a = app.clone();
            let last = Mutex::new(0u64);
            let r = sift_core::models::download(&e, None, &|file, phase, done, total| {
                let step = done / 20_000_000;
                let mut l = last.lock().unwrap();
                if *l != step || done == total {
                    *l = step;
                    let p = ModelProgress {
                        id: e.id.clone(),
                        file: file.to_string(),
                        phase: phase.to_string(),
                        done,
                        total,
                    };
                    let _ = p.emit(&a);
                }
            });
            let _ = ModelDone { id: e.id.clone(), ok: r.is_ok(), error: r.err().map(|x| format!("{x:#}")) }.emit(&app);
        }
    });
    Ok(())
}

#[tauri::command]
#[specta::specta]
async fn model_verify(id: String) -> Result<Vec<ModelFile>, String> {
    let e = sift_core::models::get(&id).map_err(err)?;
    let r = tauri::async_runtime::spawn_blocking(move || sift_core::models::verify(&e, &|_, _, _, _| {}))
        .await
        .map_err(|e| e.to_string())?
        .map_err(err)?;
    Ok(r.into_iter().map(|(file, ok)| ModelFile { file, ok }).collect())
}

#[tauri::command]
#[specta::specta]
async fn model_delete(id: String) -> Result<(), String> {
    sift_core::models::delete(&sift_core::models::get(&id).map_err(err)?).map_err(err)
}

#[tauri::command]
#[specta::specta]
async fn models_check() -> Result<Vec<ModelRevision>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        sift_core::models::all().map(|all| {
            all.iter()
                .map(|e| ModelRevision {
                    id: e.id.clone(),
                    pinned: e.revision.clone(),
                    upstream: sift_core::models::upstream_revision(e).ok(),
                })
                .collect::<Vec<_>>()
        })
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(err)
}

// ---- about -----------------------------------------------------------------------------

#[tauri::command]
#[specta::specta]
async fn about_info() -> AboutInfo {
    let mut gpus = Vec::new();
    if let Ok(o) = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=name,memory.total,memory.used,driver_version", "--format=csv,noheader,nounits"])
        .output()
    {
        for l in String::from_utf8_lossy(&o.stdout).lines() {
            let f: Vec<&str> = l.split(',').map(str::trim).collect();
            if f.len() >= 4 {
                gpus.push(Gpu {
                    name: f[0].to_string(),
                    total_mib: f[1].parse().unwrap_or(0),
                    used_mib: f[2].parse().unwrap_or(0),
                    driver: f[3].to_string(),
                });
            }
        }
    }
    AboutInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        settings: sift_core::paths::settings_file().display().to_string(),
        exe_dir: std::env::current_exe()
            .ok()
            .and_then(|e| e.parent().map(|d| d.display().to_string()))
            .unwrap_or_default(),
        gpus,
        driver_cuda: sift_core::engine::driver_cuda().map(|(a, b)| format!("{a}.{b}")),
        cpu_threads: std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1),
    }
}

/// The footer's meters (ZER0's `system_monitor.rs`): one thread for the app's lifetime, a sample
/// a second while the window shows, one event each; while it is hidden or minimized only a 3 s
/// wake to look again (and keep the CPU baseline fresh): no query, no event.
fn start_meters(app: AppHandle) {
    use tauri::Manager;
    let run = move || {
        let mut sampler = sift_core::meters::Sampler::new();
        loop {
            let shown = app
                .get_webview_window("main")
                .is_some_and(|w| w.is_visible().unwrap_or(false) && !w.is_minimized().unwrap_or(false));
            if shown {
                let _ = MetersUpdate(sampler.sample()).emit(&app);
                std::thread::sleep(std::time::Duration::from_secs(1));
            } else {
                sampler.refresh_cpu();
                std::thread::sleep(std::time::Duration::from_secs(3));
            }
        }
    };
    if let Err(e) = std::thread::Builder::new().name("meters".into()).spawn(run) {
        eprintln!("meters: {e}");
    }
}

// ---- the typed bindings ------------------------------------------------------------------

/// The page code's view of the commands and events (`app/src/lib/bindings.ts`).
#[cfg(any(debug_assertions, test))]
const BINDINGS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../src/lib/bindings.ts");

/// Every command and event the window uses, for the invoke handler and the bindings alike.
fn ipc_builder() -> tauri_specta::Builder<tauri::DynRuntime> {
    tauri_specta::Builder::<tauri::DynRuntime>::new()
        // byte counts (a model is 5 GB) and counters: far below 2^53, exact as numbers
        .dangerously_cast_bigints_to_number()
        // `mockCommands`, typed against the commands: the browser preview (lib/mock.ts)
        .mocks()
        .commands(tauri_specta::collect_commands![
            start_convert,
            cancel_convert,
            settings_get,
            settings_save,
            pick_folder,
            pick_pdf,
            input_info,
            md_preview,
            server_commands,
            server_log,
            set_folder,
            open_path,
            reveal_path,
            open_url,
            overview,
            engine_status,
            engine_check,
            engine_install,
            engine_action,
            model_download,
            model_verify,
            model_delete,
            models_check,
            about_info,
        ])
        .events(tauri_specta::collect_events![
            ConvertEvent,
            ConvertDone,
            MetersUpdate,
            EngineLog,
            EngineProgress,
            EngineDone,
            EngineChecked,
            ModelProgress,
            ModelDone,
        ])
}

/// The bindings as `builder` writes them.
#[cfg(any(debug_assertions, test))]
fn render_bindings(builder: &tauri_specta::Builder<tauri::DynRuntime>) -> Result<String, String> {
    let tmp = std::env::temp_dir().join(format!("sift-bindings-{}.ts", std::process::id()));
    builder.export(specta_typescript::Typescript::default(), &tmp).map_err(|e| e.to_string())?;
    let text = std::fs::read_to_string(&tmp).map_err(|e| e.to_string());
    let _ = std::fs::remove_file(&tmp);
    text
}

// A release build is a window program (no console of its own), so its command-line mode borrows
// the console it was started from; without this every line it prints is lost.
#[cfg(all(windows, not(debug_assertions)))]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn AttachConsole(process_id: u32) -> i32;
}

fn main() {
    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    if args.len() > 1 {
        // headless: the `sift` command line
        #[cfg(all(windows, not(debug_assertions)))]
        unsafe {
            const ATTACH_PARENT_PROCESS: u32 = u32::MAX;
            AttachConsole(ATTACH_PARENT_PROCESS);
        }
        match sift_cli::run_cli(args) {
            Ok(code) => std::process::exit(code),
            Err(e) => {
                eprintln!("error: {e:#}");
                std::process::exit(2);
            }
        }
    }
    sift_core::layout::init_threads();
    let ipc = ipc_builder();
    // development: the page code always sees the commands as they are (an unchanged file is not
    // rewritten, so Vite does not reload)
    #[cfg(debug_assertions)]
    match render_bindings(&ipc) {
        Ok(fresh) if std::fs::read_to_string(BINDINGS).ok().as_deref() != Some(fresh.as_str()) => {
            if let Err(e) = std::fs::write(BINDINGS, fresh) {
                eprintln!("bindings: {e}");
            }
        }
        Ok(_) => {}
        Err(e) => panic!("the typed bindings cannot be written: {e}"),
    }
    let handler = ipc.invoke_handler();
    tauri::Builder::default()
        .runtime(tauri_runtime_wry::Wry::default())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(handler)
        .setup(move |app| {
            // before any event: a typed event finds its name through what is mounted here
            ipc.mount_events(app);
            // the start-up check only when the user turned it on (asked in onboarding)
            if Config::load_saved()
                .map(|c| c.engine.check_at_startup && !c.folders.engine_dir.is_empty())
                .unwrap_or(false)
            {
                let h = app.handle().clone();
                std::thread::spawn(move || {
                    let backend = Config::load_saved().map(|c| c.engine.backend).unwrap_or_default();
                    if let Ok(c) = sift_core::engine::check(&backend) {
                        let _ = EngineChecked(c).emit(&h);
                    }
                });
            }
            start_meters(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Sift");
}

#[cfg(test)]
mod tests {
    /// `app/src/lib/bindings.ts` is what the commands and events generate. After changing one,
    /// start the window once in a debug build (`bun run tauri dev`), or run this test with
    /// `SIFT_WRITE_BINDINGS=1`.
    #[test]
    fn bindings_are_current() {
        let fresh = super::render_bindings(&super::ipc_builder()).expect("export");
        if std::env::var_os("SIFT_WRITE_BINDINGS").is_some() {
            std::fs::write(super::BINDINGS, &fresh).expect("write bindings.ts");
            return;
        }
        let committed = std::fs::read_to_string(super::BINDINGS).unwrap_or_default();
        assert!(
            committed == fresh,
            "app/src/lib/bindings.ts is out of date: start the window once in a debug build, or run \
             `SIFT_WRITE_BINDINGS=1 cargo test -p sift-app bindings_are_current`"
        );
    }
}
