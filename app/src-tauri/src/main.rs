//! Sift's window (MASTER_PLAN section 9): Tauri 3 + SolidJS 2, the ZER0 way.
//!
//! - With a subcommand (`sift-app convert ...`, `sift-app engine update`), the binary runs
//!   headless: the very same command line as `sift`, no window.
//! - Without one it opens the window. Commands never block the webview (ZER0's rule): long work
//!   runs on its own thread and reports through events (`sift-event`: docs/events.md,
//!   `engine-log`, `engine-progress`, `engine-done`, `engine-check`, `model-progress`,
//!   `model-done`, `sift-done`).
//! - Settings: the one `settings.toml` shared with the command line (`sift_core::paths`); the
//!   engine and the models live only in the folders the user picks here.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

use serde::Deserialize;
use serde_json::{Value, json};
use sift_core::config::Config;
use tauri::{AppHandle, Emitter};
use tauri_plugin_dialog::DialogExt;

static BUSY: AtomicBool = AtomicBool::new(false);
static CANCEL: LazyLock<Mutex<Option<Arc<AtomicBool>>>> = LazyLock::new(|| Mutex::new(None));

fn err(e: anyhow::Error) -> String {
    format!("{e:#}")
}

fn emit(app: &AppHandle, ev: &str, v: Value) {
    let _ = app.emit(ev, v);
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

#[derive(Deserialize)]
struct ConvertRequest {
    input: String,
    output: Option<String>,
    pages: Option<String>,
    no_llm: bool,
    describer: String,
    recursive: bool,
    force: bool,
    mode: Option<String>,
    tables: Option<String>,
    keep_headers: bool,
}

#[tauri::command]
fn start_convert(app: AppHandle, req: ConvertRequest) -> Result<(), String> {
    let cancel = Arc::new(AtomicBool::new(false));
    let c2 = cancel.clone();
    exclusive(move || {
        *CANCEL.lock().unwrap() = Some(c2.clone());
        let r = (|| -> anyhow::Result<i32> {
            let mut cfg = Config::load(None)?;
            // the page's options apply to this run only, like command-line flags
            if let Some(p) = req.pages.filter(|p| !p.trim().is_empty()) {
                cfg.convert.pages = p;
            }
            if let Some(m) = req.mode {
                cfg.convert.mode = m;
            }
            if let Some(t) = req.tables {
                cfg.convert.tables = t;
            }
            cfg.convert.keep_headers |= req.keep_headers;
            if req.no_llm {
                cfg.llm.enabled = false;
            }
            let events = Arc::new(sift_core::events::Events::new(false));
            let a2 = app.clone();
            events.sinks.lock().unwrap().push(Box::new(move |v| emit(&a2, "sift-event", v.clone())));
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
        match r {
            Ok(code) => emit(&app, "sift-done", json!({"ok": code == 0 && !cancelled, "cancelled": cancelled})),
            Err(e) => emit(&app, "sift-done", json!({"ok": false, "cancelled": cancelled, "error": format!("{e:#}")})),
        }
        *CANCEL.lock().unwrap() = None;
    })
}

/// Cancel: queued work stops first, then the model servers; no model starts afterwards.
#[tauri::command]
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
async fn settings_get() -> Result<Value, String> {
    serde_json::to_value(Config::load_saved().map_err(err)?).map_err(|e| e.to_string())
}

#[tauri::command]
async fn settings_save(settings: Value) -> Result<(), String> {
    let c: Config = serde_json::from_value(settings).map_err(|e| e.to_string())?;
    c.save().map_err(err)
}

/// A folder picker (the dialog plugin, called from Rust: the page needs no plugin permission).
#[tauri::command]
async fn pick_folder(app: AppHandle, title: String) -> Option<String> {
    app.dialog()
        .file()
        .set_title(title)
        .blocking_pick_folder()
        .and_then(|p| p.into_path().ok())
        .map(|p| p.display().to_string())
}

/// Set the engine or models folder. `move_files`: move what the current folder holds into the
/// new one (same drive: a rename); otherwise the new folder is used as it is. The old folder is
/// never deleted here.
#[tauri::command]
async fn set_folder(kind: String, path: String, move_files: bool) -> Result<Value, String> {
    let new = PathBuf::from(path.trim());
    std::fs::create_dir_all(&new).map_err(|e| format!("cannot use {}: {e}", new.display()))?;
    let probe = new.join(".sift-write-test");
    std::fs::write(&probe, b"").map_err(|e| format!("{} is not writable: {e}", new.display()))?;
    let _ = std::fs::remove_file(&probe);
    let mut cfg = Config::load_saved().map_err(err)?;
    let old = if kind == "engine" { cfg.folders.engine_dir.clone() } else { cfg.folders.models_dir.clone() };
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
    if kind == "engine" {
        cfg.folders.engine_dir = s;
    } else {
        cfg.folders.models_dir = s;
    }
    cfg.save().map_err(err)?;
    Ok(json!({"moved": moved, "old": old}))
}

#[tauri::command]
async fn open_folder(path: String) -> Result<(), String> {
    std::process::Command::new("explorer").arg(&path).spawn().map(|_| ()).map_err(|e| e.to_string())
}

/// What the window needs to know at a glance (onboarding, footer, menu dot).
#[tauri::command]
async fn overview() -> Value {
    let cfg = Config::load_saved().unwrap_or_default();
    let models: Vec<Value> = sift_core::models::all().unwrap_or_default().iter().map(|m| json!(m.status())).collect();
    let s = sift_core::engine::status();
    json!({
        "settings_file": sift_core::paths::settings_file().display().to_string(),
        "portable": sift_core::paths::portable_dir().is_some(),
        "engine_dir": cfg.folders.engine_dir,
        "models_dir": cfg.folders.models_dir,
        "engine_active": s.active,
        "models": models,
        "check_at_startup": cfg.engine.check_at_startup,
    })
}

// ---- the engine (ZER0's llama.cpp page) ----------------------------------------------------

#[tauri::command]
async fn engine_status() -> Value {
    let s = sift_core::engine::status();
    let cfg = Config::load_saved().unwrap_or_default();
    let builds: Vec<Value> = sift_core::engine::list().unwrap_or_default().iter().map(|b| json!(b)).collect();
    json!({
        "active": s.active, "previous": s.previous, "version": s.version,
        "dir": s.dir.map(|d| d.display().to_string()),
        "backend": cfg.engine.backend, "detected": sift_core::engine::detect_backend(),
        "driver_cuda": sift_core::engine::driver_cuda().map(|(a, b)| format!("{a}.{b}")),
        "toolkit": sift_core::engine::system_cudart_dir(if cfg.engine.backend.starts_with("cuda-") { &cfg.engine.backend } else { "cuda-13.5" }).map(|p| p.display().to_string()),
        "include_cudart": cfg.engine.include_cudart,
        "check_at_startup": cfg.engine.check_at_startup,
        "builds": builds,
    })
}

/// Read the release list (no download) and tell the window whether a newer build exists.
#[tauri::command]
async fn engine_check(app: AppHandle) -> Result<Value, String> {
    let backend = Config::load_saved().map_err(err)?.engine.backend;
    let c = tauri::async_runtime::spawn_blocking(move || sift_core::engine::check(&backend))
        .await
        .map_err(|e| e.to_string())?
        .map_err(err)?;
    let v = json!(c);
    emit(&app, "engine-check", v.clone());
    Ok(v)
}

/// Install or update: download, verify, switch (`tag`: a given build instead of the newest).
#[tauri::command]
fn engine_install(app: AppHandle, tag: Option<String>) -> Result<(), String> {
    let cfg = Config::load_saved().map_err(err)?;
    sift_core::paths::engine_dir().map_err(err)?;
    exclusive(move || {
        let a = app.clone();
        let log = move |s: &str| emit(&a, "engine-log", json!({"message": s}));
        let a2 = app.clone();
        let progress = move |done: u64, total: u64| emit(&a2, "engine-progress", json!({"done": done, "total": total}));
        let backend = sift_core::engine::wanted_backend(&cfg.engine.backend);
        let r = sift_core::engine::install(
            tag.as_deref(),
            Some(&backend),
            cfg.engine.include_cudart == "always",
            &log,
            &progress,
        );
        match r {
            Ok(p) => emit(&app, "engine-done", json!({"ok": true, "build": p.display().to_string()})),
            Err(e) => emit(&app, "engine-done", json!({"ok": false, "error": format!("{e:#}")})),
        }
    })
}

/// Use / Roll back / Delete / Remove the bundled CUDA runtime of an installed build.
#[tauri::command]
fn engine_action(app: AppHandle, action: String, build: Option<String>) -> Result<(), String> {
    exclusive(move || {
        let a = app.clone();
        let log = move |s: &str| emit(&a, "engine-log", json!({"message": s}));
        let b = build.unwrap_or_default();
        let r: anyhow::Result<String> = match action.as_str() {
            "use" => sift_core::engine::use_build(&b, &log).map(|_| format!("{b} is active")),
            "rollback" => sift_core::engine::rollback(&log).map(|p| format!("{} is active", p.display())),
            "remove" => sift_core::engine::remove(&b).map(|_| format!("deleted {b}")),
            "remove_runtime" => sift_core::engine::remove_cuda_runtime(&b).map(|mb| format!("freed {mb} MB from {b}")),
            other => Err(anyhow::anyhow!("unknown action {other}")),
        };
        match r {
            Ok(m) => emit(&app, "engine-done", json!({"ok": true, "message": m})),
            Err(e) => emit(&app, "engine-done", json!({"ok": false, "error": format!("{e:#}")})),
        }
    })
}

// ---- the models ------------------------------------------------------------------------

#[tauri::command]
async fn models_status() -> Result<Value, String> {
    let m: Vec<Value> = sift_core::models::all().map_err(err)?.iter().map(|m| json!(m.status())).collect();
    Ok(json!({"models": m, "dir": sift_core::paths::models_dir().ok().map(|d| d.display().to_string())}))
}

#[tauri::command]
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
                    emit(
                        &a,
                        "model-progress",
                        json!({"id": e.id, "file": file, "phase": phase, "done": done, "total": total}),
                    );
                }
            });
            emit(&app, "model-done", json!({"id": e.id, "ok": r.is_ok(), "error": r.err().map(|x| format!("{x:#}"))}));
        }
    });
    Ok(())
}

#[tauri::command]
async fn model_verify(id: String) -> Result<Value, String> {
    let e = sift_core::models::get(&id).map_err(err)?;
    let r = tauri::async_runtime::spawn_blocking(move || sift_core::models::verify(&e, &|_, _, _, _| {}))
        .await
        .map_err(|e| e.to_string())?
        .map_err(err)?;
    Ok(json!(r.iter().map(|(f, ok)| json!({"file": f, "ok": ok})).collect::<Vec<_>>()))
}

#[tauri::command]
async fn model_delete(id: String) -> Result<(), String> {
    sift_core::models::delete(&sift_core::models::get(&id).map_err(err)?).map_err(err)
}

#[tauri::command]
async fn models_check() -> Result<Value, String> {
    let r = tauri::async_runtime::spawn_blocking(|| {
        sift_core::models::all().map(|all| all.iter().map(|e| json!({"id": e.id, "pinned": e.revision, "upstream": sift_core::models::upstream_revision(e).ok()})).collect::<Vec<_>>())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(err)?;
    Ok(json!(r))
}

// ---- about -----------------------------------------------------------------------------

#[tauri::command]
async fn about_info() -> Value {
    let mut gpus = Vec::new();
    if let Ok(o) = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=name,memory.total,memory.used,driver_version", "--format=csv,noheader,nounits"])
        .output()
    {
        for l in String::from_utf8_lossy(&o.stdout).lines() {
            let f: Vec<&str> = l.split(',').map(str::trim).collect();
            if f.len() >= 4 {
                gpus.push(json!({"name": f[0], "total_mib": f[1].parse::<u64>().unwrap_or(0), "used_mib": f[2].parse::<u64>().unwrap_or(0), "driver": f[3]}));
            }
        }
    }
    json!({
        "version": env!("CARGO_PKG_VERSION"),
        "settings": sift_core::paths::settings_file().display().to_string(),
        "gpus": gpus,
        "driver_cuda": sift_core::engine::driver_cuda().map(|(a, b)| format!("{a}.{b}")),
        "cpu_threads": std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1),
    })
}

fn main() {
    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    if args.len() > 1 {
        // headless: the `sift` command line
        match sift_cli::run_cli(args) {
            Ok(code) => std::process::exit(code),
            Err(e) => {
                eprintln!("error: {e:#}");
                std::process::exit(2);
            }
        }
    }
    sift_core::layout::init_threads();
    tauri::Builder::default()
        .runtime(tauri_runtime_wry::Wry::default())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // the start-up check only when the user turned it on (asked in onboarding)
            if Config::load_saved()
                .map(|c| c.engine.check_at_startup && !c.folders.engine_dir.is_empty())
                .unwrap_or(false)
            {
                let h = app.handle().clone();
                std::thread::spawn(move || {
                    let backend = Config::load_saved().map(|c| c.engine.backend).unwrap_or_default();
                    if let Ok(c) = sift_core::engine::check(&backend) {
                        emit(&h, "engine-check", json!(c));
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_convert,
            cancel_convert,
            settings_get,
            settings_save,
            pick_folder,
            set_folder,
            open_folder,
            overview,
            engine_status,
            engine_check,
            engine_install,
            engine_action,
            models_status,
            model_download,
            model_verify,
            model_delete,
            models_check,
            about_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running Sift");
}
