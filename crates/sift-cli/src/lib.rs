//! The `sift` command line (MASTER_PLAN section 10), shared by the `sift` binary and the app
//! binary (which runs it headless when it gets a subcommand). Every option of the window is a
//! flag here; flags never change the saved settings, except the commands that are about them
//! (`setup`, `engine folder`, `models folder`, `config set`).

use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{Result, anyhow, bail};
use clap::{Parser, Subcommand};
use sift_core::config::Config;

#[derive(Parser)]
#[command(name = "sift", version, about = "PDF to Markdown with surya 2.5 and Gemma 4, all local (official llama.cpp)")]
pub struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Convert a PDF or a folder of PDFs to Markdown (+ images, descriptions, report).
    /// `sift <pdf|folder>` works too.
    Convert(ConvertArgs),
    /// A new machine in one command: set the folders, install the newest llama.cpp build,
    /// download and verify surya, its layout model and Gemma
    Setup {
        /// where the llama.cpp builds go (saved in the settings)
        #[arg(long)]
        engine_dir: Option<PathBuf>,
        /// where the models go (saved in the settings)
        #[arg(long)]
        models_dir: Option<PathBuf>,
        /// also download llama.cpp's CUDA runtime package (default: only when no CUDA toolkit is found)
        #[arg(long)]
        with_cudart: bool,
    },
    /// The llama.cpp engine: official builds only, in the engine folder you chose
    Engine {
        #[command(subcommand)]
        what: EngineCmd,
    },
    /// surya 2.5, its layout model and Gemma 4 E4B: pinned, sha256-checked, in the models folder you chose
    Models {
        #[command(subcommand)]
        what: ModelsCmd,
    },
    /// The settings file shared with the window
    Config {
        #[command(subcommand)]
        what: ConfigCmd,
    },
    /// Development tools
    Dev {
        #[command(subcommand)]
        what: Dev,
    },
}

#[derive(clap::Args)]
struct ConvertArgs {
    /// a PDF, or a folder of PDFs
    input: PathBuf,
    /// output folder (default: next to the input)
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// hybrid (default): text layer + surya where needed; full-ocr: surya reads every page
    #[arg(long, value_parser = ["hybrid", "full-ocr"])]
    mode: Option<String>,
    /// folder mode: include subfolders
    #[arg(short, long)]
    recursive: bool,
    /// folder mode: convert again even if the .md is up to date (newer than its PDF, same options)
    #[arg(long)]
    force: bool,
    /// only these pages, 1-based, e.g. 1-5,137
    #[arg(long)]
    pages: Option<String>,
    /// DPI of extracted images (default 300)
    #[arg(long)]
    image_dpi: Option<u32>,
    /// auto: HTML only for merged cells; markdown: always pipe tables
    #[arg(long, value_parser = ["auto", "markdown"])]
    tables: Option<String>,
    /// keep page headers and footers
    #[arg(long)]
    keep_headers: bool,
    /// image descriptions: gemma (default) or off
    #[arg(long, value_parser = ["gemma", "off"])]
    describer: Option<String>,
    /// same as --describer off
    #[arg(long)]
    no_describe: bool,
    /// never start surya (tables and broken text fall back to the text layer)
    #[arg(long)]
    no_surya: bool,
    /// no LLM pass beyond image descriptions (heading levels, split tables, table review, formulas)
    #[arg(long)]
    no_llm: bool,
    /// machine-readable progress on stdout (docs/events.md)
    #[arg(long)]
    json: bool,
    /// print time per stage
    #[arg(long)]
    profile: bool,
    /// an extra settings file on top of the saved settings (V0's sift.toml works as is)
    #[arg(long)]
    config: Option<PathBuf>,
    /// keep every model answer in this folder; a re-run reuses them
    #[arg(long, env = "SIFT_ANSWERS")]
    answers: Option<PathBuf>,
    /// development: read-only answer stores tried first (the golden set); repeatable
    #[arg(long)]
    replay: Vec<PathBuf>,
    /// development: never call a model; every answer must come from the stores
    #[arg(long)]
    replay_only: bool,
    /// development: layout boxes from <dir>/<pdf stem>/layout.json (the golden set)
    #[arg(long)]
    layout_from: Option<PathBuf>,
    /// measurement: ignore the text layer, every page is read as if scanned (score the result
    /// against the text layer with bench/score.py)
    #[arg(long)]
    mask_text_layer: bool,
}

#[derive(Subcommand)]
enum EngineCmd {
    /// Active and previous build, backend, folder
    Status,
    /// Is there a newer official build? (no download; exit code 10 = update available)
    Check {
        /// cuda-13.5 | vulkan | cpu | auto (default: the settings)
        #[arg(long)]
        backend: Option<String>,
    },
    /// Download the newest official build (or --tag), verify it, make it active
    Update {
        #[arg(long)]
        tag: Option<String>,
        /// cuda-13.5 | vulkan | cpu | auto (default: the settings)
        #[arg(long)]
        backend: Option<String>,
        /// also download llama.cpp's CUDA runtime package
        #[arg(long)]
        with_cudart: bool,
    },
    /// Same as update (first install)
    Install {
        #[arg(long)]
        tag: Option<String>,
        #[arg(long)]
        backend: Option<String>,
        #[arg(long)]
        with_cudart: bool,
    },
    /// The installed builds
    List,
    /// Verify an installed build and make it active
    Use { build: String },
    /// The previous build becomes active
    Rollback,
    /// Delete an installed build (not the active or the previous one)
    Remove { build: String },
    /// Delete the CUDA runtime a build carries, when an installed CUDA toolkit provides it
    RemoveRuntime { build: String },
    /// Show or set the engine folder
    Folder { path: Option<PathBuf> },
}

#[derive(Subcommand)]
enum ModelsCmd {
    /// Each model: size, revision, installed or what is missing
    Status,
    /// Newer revisions upstream (shown only; Sift keeps its pinned, tested revision)
    Check,
    /// Download a model (or all) at its pinned revision, resumable, sha256-checked
    Download {
        /// all | surya-2.5 | surya-layout | gemma-4-e4b-qat
        #[arg(default_value = "all")]
        id: String,
        /// an optional variant, e.g. surya-2.5 --quant BF16
        #[arg(long)]
        quant: Option<String>,
    },
    /// Re-check the sha256 of the files on disk
    Verify {
        #[arg(default_value = "all")]
        id: String,
    },
    /// Delete a model's files
    Delete { id: String },
    /// Show or set the models folder
    Folder { path: Option<PathBuf> },
}

#[derive(Subcommand)]
enum ConfigCmd {
    /// Print the saved settings
    Show,
    /// Where the settings file is
    Path,
    /// Set one key, e.g. sift config set engine.check_at_startup true
    Set { key: String, value: String },
}

#[derive(Subcommand)]
enum Dev {
    /// Write a crop as surya sees it (raw and scale_to_fit) as PNG: page (1-based) x0 y0 x1 y1 (points)
    Crop { pdf: PathBuf, page: usize, x0: f64, y0: f64, x1: f64, y1: f64, out: PathBuf },
    /// The PDF's text layer as JSON on stdout (pages: index, width, height, lines: text, bbox):
    /// the ground truth bench/score.py and bench/compare_marker.py score against
    Textlayer { pdf: PathBuf },
}

const COMMANDS: [&str; 10] = ["convert", "setup", "engine", "models", "config", "dev", "help", "-h", "--help", "-V"];

/// Parse `args` (argv[0] included) and run the command; returns the process exit code.
pub fn run_cli<I: IntoIterator<Item = std::ffi::OsString>>(args: I) -> Result<i32> {
    sift_core::layout::init_threads();
    let mut args: Vec<std::ffi::OsString> = args.into_iter().collect();
    // V0's form: `sift <pdf|folder> [flags]` = `sift convert ...`
    if let Some(first) = args.get(1).map(|a| a.to_string_lossy().to_string())
        && !COMMANDS.contains(&first.as_str())
        && !first.starts_with('-')
    {
        args.insert(1, "convert".into());
    }
    let cli = Cli::parse_from(args);
    match cli.cmd {
        Cmd::Convert(a) => return convert(a),
        Cmd::Setup { engine_dir, models_dir, with_cudart } => setup(engine_dir, models_dir, with_cudart)?,
        Cmd::Engine { what } => return engine(what),
        Cmd::Models { what } => models(what)?,
        Cmd::Config { what } => config(what)?,
        Cmd::Dev { what } => match what {
            Dev::Crop { pdf, page, x0, y0, x1, y1, out } => {
                let doc = sift_core::pdf::pdfium::Document::open(&pdf, true)?;
                let pg = doc.render_rgb(page - 1, 192.0 / 72.0)?;
                let c = sift_core::imaging::crop_pt(&pg, [x0, y0, x1, y1], 192.0, 1.5);
                let s = sift_core::imaging::scale_to_fit(&c);
                std::fs::write(out.with_extension("raw.png"), c.png(true))?;
                std::fs::write(out.with_extension("fit.png"), s.png(true))?;
                println!("crop {}x{} -> {}x{}", c.width, c.height, s.width, s.height);
            }
            Dev::Textlayer { pdf } => {
                let doc = sift_core::pdf::pdfium::Document::open(&pdf, true)?;
                let tl = sift_core::pdf::textlayer::extract(&doc)?;
                let pages: Vec<serde_json::Value> = tl
                    .pages
                    .iter()
                    .map(|p| {
                        let lines: Vec<serde_json::Value> =
                            p.lines.iter().map(|l| serde_json::json!({"text": l.text(), "bbox": l.bbox})).collect();
                        serde_json::json!({"index": p.index, "width": p.width, "height": p.height, "lines": lines})
                    })
                    .collect();
                use std::io::Write;
                let mut out = std::io::stdout().lock();
                serde_json::to_writer(&mut out, &serde_json::json!({"pages": pages}))?;
                out.flush()?;
            }
        },
    }
    Ok(0)
}

fn log(s: &str) {
    eprintln!("{s}");
}

/// Progress lines every 50 MB.
fn mb_progress() -> impl Fn(u64, u64) {
    let last = std::sync::atomic::AtomicU64::new(u64::MAX);
    move |done: u64, total: u64| {
        let step = done / 50_000_000;
        if step != last.swap(step, std::sync::atomic::Ordering::Relaxed) {
            eprintln!("  {:.0} / {:.0} MB", done as f64 / 1e6, total as f64 / 1e6);
        }
    }
}

/// Set a folder in the saved settings (created if needed; must be writable).
fn set_folder(path: &std::path::Path, engine: bool) -> Result<PathBuf> {
    std::fs::create_dir_all(path).map_err(|e| anyhow!("cannot use {}: {e}", path.display()))?;
    let abs = std::path::absolute(path)?;
    let probe = abs.join(".sift-write-test");
    std::fs::write(&probe, b"").map_err(|e| anyhow!("{} is not writable: {e}", abs.display()))?;
    let _ = std::fs::remove_file(&probe);
    let mut cfg = Config::load_saved()?;
    if engine {
        cfg.folders.engine_dir = abs.display().to_string();
    } else {
        cfg.folders.models_dir = abs.display().to_string();
    }
    cfg.save()?;
    Ok(abs)
}

fn install_engine(tag: Option<&str>, backend: Option<String>, with_cudart: bool) -> Result<()> {
    let cfg = Config::load(None)?;
    let backend = sift_core::engine::wanted_backend(&backend.unwrap_or(cfg.engine.backend.clone()));
    let with_cudart = with_cudart || cfg.engine.include_cudart == "always";
    sift_core::engine::install(tag, Some(&backend), with_cudart, &log, &mb_progress())?;
    Ok(())
}

fn engine(what: EngineCmd) -> Result<i32> {
    use sift_core::engine as e;
    match what {
        EngineCmd::Status => {
            let s = e::status();
            println!(
                "engine folder: {}",
                s.dir
                    .as_ref()
                    .map(|d| d.display().to_string())
                    .unwrap_or_else(|| "not set (sift engine folder <path>)".into())
            );
            println!("active:   {}", s.active.as_deref().unwrap_or("none (sift engine install)"));
            println!("previous: {}", s.previous.as_deref().unwrap_or("none"));
            if let Some(v) = s.version {
                println!("version:  {v}");
            }
            println!(
                "backend:  {} (detected on this machine: {})",
                Config::load(None)?.engine.backend,
                e::detect_backend()
            );
        }
        EngineCmd::Check { backend } => {
            let b = backend.unwrap_or(Config::load(None)?.engine.backend);
            let c = e::check(&b)?;
            println!("active: {}", c.active.as_deref().unwrap_or("none"));
            println!("newest: {} ({}, {:.0} MB)", c.newest, c.published, c.size as f64 / 1e6);
            if let Some(n) = &c.newer_cuda {
                println!(
                    "note:   official builds now also use {n}; switch with --backend {n} (sift config set engine.backend {n})"
                );
            }
            if c.update_available {
                println!("update available: sift engine update");
                return Ok(10);
            }
            println!("up to date");
        }
        EngineCmd::Update { tag, backend, with_cudart } | EngineCmd::Install { tag, backend, with_cudart } => {
            install_engine(tag.as_deref(), backend, with_cudart)?
        }
        EngineCmd::List => {
            let l = e::list()?;
            if l.is_empty() {
                println!("no build installed (sift engine install)");
            }
            for i in l {
                let mark = if i.active {
                    "active"
                } else if i.previous {
                    "previous"
                } else {
                    ""
                };
                let rt = if i.cuda_runtime_mb > 0 {
                    format!(", CUDA runtime {} MB", i.cuda_runtime_mb)
                } else {
                    String::new()
                };
                println!("{:<24} {:>5} MB{rt}  {mark}", i.name, i.size_mb);
            }
        }
        EngineCmd::Use { build } => {
            e::use_build(&build, &log)?;
        }
        EngineCmd::Rollback => {
            e::rollback(&log)?;
        }
        EngineCmd::Remove { build } => {
            e::remove(&build)?;
            println!("deleted {build}");
        }
        EngineCmd::RemoveRuntime { build } => println!("freed {} MB from {build}", e::remove_cuda_runtime(&build)?),
        EngineCmd::Folder { path } => match path {
            Some(p) => println!("engine folder: {}", set_folder(&p, true)?.display()),
            None => println!("{}", sift_core::paths::engine_dir()?.display()),
        },
    }
    Ok(0)
}

fn models(what: ModelsCmd) -> Result<()> {
    use sift_core::models as m;
    let gb = |b: u64| b as f64 / 1e9;
    let pick = |id: &str| -> Result<Vec<m::Entry>> { if id == "all" { m::all() } else { Ok(vec![m::get(id)?]) } };
    let progress = || {
        let last = Mutex::new((String::new(), 0u64));
        move |file: &str, phase: &str, done: u64, total: u64| {
            let step = done / 200_000_000;
            let mut l = last.lock().unwrap();
            let key = format!("{file}{phase}");
            if l.0 != key || l.1 != step || done == total {
                *l = (key, step);
                eprintln!("  {file}: {phase} {:.2} / {:.2} GB", gb(done), gb(total));
            }
        }
    };
    match what {
        ModelsCmd::Status => {
            for e in m::all()? {
                let s = e.status();
                let state =
                    if s.installed { "installed".to_string() } else { format!("missing {}", s.missing.join(", ")) };
                println!("{:<16} {:>5.2} GB  {:<10} {}  [{}]", s.id, gb(s.size), state, s.name, s.license);
            }
            match sift_core::paths::models_dir() {
                Ok(d) => println!("models folder: {}", d.display()),
                Err(e) => println!("{e}"),
            }
        }
        ModelsCmd::Check => {
            for e in m::all()? {
                match m::upstream_revision(&e) {
                    Ok(r) if r == e.revision => {
                        println!("{:<16} pinned revision is the newest ({})", e.id, &r[..8.min(r.len())])
                    }
                    Ok(r) => println!(
                        "{:<16} newer revision upstream: {} (pinned: {}); adopting it needs the golden test + scorecard",
                        e.id,
                        &r[..8.min(r.len())],
                        &e.revision[..8]
                    ),
                    Err(err) => println!("{:<16} {err}", e.id),
                }
            }
        }
        ModelsCmd::Download { id, quant } => {
            sift_core::paths::models_dir()?;
            for e in pick(&id)? {
                eprintln!("{} ({:.2} GB)", e.name, gb(e.size()));
                m::download(&e, quant.as_deref(), &progress())?;
                println!("{} ready in {}", e.name, e.dir()?.display());
            }
        }
        ModelsCmd::Verify { id } => {
            let mut bad = 0;
            for e in pick(&id)? {
                for (f, ok) in m::verify(&e, &|_, _, _, _| {})? {
                    bad += !ok as usize;
                    println!("{:<16} {:<40} {}", e.id, f, if ok { "ok" } else { "MISSING OR DAMAGED" });
                }
            }
            if bad > 0 {
                bail!("{bad} file(s) missing or damaged: sift models download");
            }
        }
        ModelsCmd::Delete { id } => {
            let e = m::get(&id)?;
            m::delete(&e)?;
            println!("deleted {}", e.name);
        }
        ModelsCmd::Folder { path } => match path {
            Some(p) => println!("models folder: {}", set_folder(&p, false)?.display()),
            None => println!("{}", sift_core::paths::models_dir()?.display()),
        },
    }
    Ok(())
}

fn config(what: ConfigCmd) -> Result<()> {
    match what {
        ConfigCmd::Show => {
            print!("{}", toml::to_string_pretty(&Config::load_saved()?)?);
            // the file is what is shown; conversions also see these (docs/settings.md, priority)
            let mut env: Vec<(String, String)> = std::env::vars().filter(|(k, _)| k.starts_with("SIFT_")).collect();
            env.sort();
            if !env.is_empty() {
                println!("\n# set in the environment, used by conversions on top of this file:");
                for (k, v) in env {
                    println!("# {k}={v}");
                }
            }
        }
        ConfigCmd::Path => println!("{}", sift_core::paths::settings_file().display()),
        ConfigCmd::Set { key, value } => {
            let mut cfg = Config::load_saved()?;
            cfg.set(&key, &value)?;
            cfg.save()?;
            println!("{key} = {value}  ({})", sift_core::paths::settings_file().display());
        }
    }
    Ok(())
}

fn setup(engine_dir: Option<PathBuf>, models_dir: Option<PathBuf>, with_cudart: bool) -> Result<()> {
    if let Some(p) = engine_dir {
        println!("engine folder: {}", set_folder(&p, true)?.display());
    }
    if let Some(p) = models_dir {
        println!("models folder: {}", set_folder(&p, false)?.display());
    }
    let (e, m) = (sift_core::paths::engine_dir(), sift_core::paths::models_dir());
    if e.is_err() || m.is_err() {
        bail!("choose both folders first: sift setup --engine-dir <path> --models-dir <path>");
    }
    eprintln!("== llama.cpp");
    install_engine(None, None, with_cudart)?;
    eprintln!("== models");
    models(ModelsCmd::Download { id: "all".into(), quant: None })?;
    models(ModelsCmd::Verify { id: "all".into() })?;
    println!("Sift is ready: sift convert <pdf>");
    Ok(())
}

fn convert(a: ConvertArgs) -> Result<i32> {
    use std::sync::Arc;
    let mut cfg = Config::load(a.config.as_deref())?;
    // flags override the settings for this run only
    if let Some(m) = a.mode {
        cfg.convert.mode = m;
    }
    if let Some(p) = a.pages {
        cfg.convert.pages = p;
    }
    if let Some(d) = a.image_dpi {
        cfg.convert.image_dpi = d;
    }
    if let Some(t) = a.tables {
        cfg.convert.tables = t;
    }
    if a.keep_headers {
        cfg.convert.keep_headers = true;
    }
    if a.no_llm {
        cfg.llm.enabled = false;
    }
    let describer =
        if a.no_describe { "off".to_string() } else { a.describer.unwrap_or_else(|| cfg.convert.describer.clone()) };
    let store = Arc::new(sift_core::answers::AnswerStore::new(a.answers, a.replay, a.replay_only));
    let events = Arc::new(sift_core::events::Events::new(a.json));
    let opts = sift_core::batch::BatchOptions {
        output: a.output,
        recursive: a.recursive,
        force: a.force,
        pipeline: sift_core::pipeline::Options {
            use_surya: !a.no_surya,
            describer,
            layout_replay: a.layout_from,
            profile: a.profile,
            mask_text_layer: a.mask_text_layer,
        },
        cancel: None,
    };
    let (code, _) = sift_core::batch::run(cfg, &a.input, opts, events, store)?;
    Ok(code)
}
