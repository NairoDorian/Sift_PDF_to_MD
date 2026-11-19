//! Where Sift keeps things (MASTER_PLAN section 5).
//!
//! - **Engine folder** and **models folder**: chosen by the user (settings `[folders]`, the
//!   window's folder pickers, `sift engine folder` / `sift models folder`). There is no default:
//!   nothing is downloaded before a folder is chosen, and never into `%APPDATA%`.
//! - **Settings file**: `%APPDATA%\com.nairodorian.sift\settings.toml` (the app's config folder),
//!   or `Data\settings.toml` next to the executable when a file named `portable` is there
//!   (ZER0's portable mode). `SIFT_SETTINGS` overrides it (the golden test points it at nothing).
//! - Development overrides for one process: `SIFT_ENGINE_DIR`, `SIFT_MODELS_DIR`.

use std::path::PathBuf;

use anyhow::{Result, anyhow};

/// The app's identifier (tauri.conf.json): the name of its config folder.
pub const APP_ID: &str = "com.nairodorian.sift";

fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe().ok().and_then(|e| e.parent().map(PathBuf::from))
}

/// Portable mode: a file named `portable` next to the executable.
pub fn portable_dir() -> Option<PathBuf> {
    exe_dir().filter(|d| d.join("portable").exists()).map(|d| d.join("Data"))
}

/// The folder of the settings file (and the window's session log).
pub fn config_dir() -> PathBuf {
    if let Some(d) = portable_dir() {
        return d;
    }
    match std::env::var_os("APPDATA") {
        Some(a) => PathBuf::from(a).join(APP_ID),
        None => exe_dir().unwrap_or_default().join("Data"),
    }
}

pub fn settings_file() -> PathBuf {
    match std::env::var_os("SIFT_SETTINGS") {
        Some(p) => PathBuf::from(p),
        None => config_dir().join("settings.toml"),
    }
}

/// The saved `[folders]` (environment overrides first).
fn folder(env: &str, pick: impl Fn(&crate::config::Folders) -> String, what: &str, cmd: &str) -> Result<PathBuf> {
    if let Some(p) = std::env::var_os(env).filter(|p| !p.is_empty()) {
        return Ok(PathBuf::from(p));
    }
    let saved = crate::config::Config::load_saved().map(|c| pick(&c.folders)).unwrap_or_default();
    if saved.trim().is_empty() {
        return Err(anyhow!(
            "No {what} folder set. Choose one in the app (Settings), or run: sift {cmd} folder <path>"
        ));
    }
    Ok(PathBuf::from(saved.trim()))
}

/// The folder holding the llama.cpp builds (`<backend>-<tag>\`, `current.txt`, `previous.txt`).
pub fn engine_dir() -> Result<PathBuf> {
    folder("SIFT_ENGINE_DIR", |f| f.engine_dir.clone(), "engine", "engine")
}

/// The folder holding the models (`surya-ocr-2.5-gguf\`, `surya_layout2\`, `gemma-4-E4B-it-qat-GGUF\`).
pub fn models_dir() -> Result<PathBuf> {
    folder("SIFT_MODELS_DIR", |f| f.models_dir.clone(), "models", "models")
}

/// One model's folder inside the models folder.
pub fn find_model_dir(folder: &str) -> Result<PathBuf> {
    Ok(models_dir()?.join(folder))
}

/// Shipped prompts (surya's prompts word for word, description prompts): `SIFT_PROMPTS_DIR`,
/// else `prompts/` next to the executable or in an ancestor folder (development: the repository's
/// `prompts/`).
pub fn prompts_dir() -> PathBuf {
    if let Ok(p) = std::env::var("SIFT_PROMPTS_DIR") {
        return PathBuf::from(p);
    }
    if let Ok(exe) = std::env::current_exe() {
        for anc in exe.ancestors().skip(1).take(6) {
            let p = anc.join("prompts");
            if p.join("surya-2.5").is_dir() {
                return p;
            }
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../prompts")
}

/// The model catalog (`catalog/*.toml`: pinned files of surya, its layout model and Gemma).
pub fn catalog_dir() -> PathBuf {
    if let Ok(p) = std::env::var("SIFT_CATALOG_DIR") {
        return PathBuf::from(p);
    }
    if let Ok(exe) = std::env::current_exe() {
        for anc in exe.ancestors().skip(1).take(6) {
            let p = anc.join("catalog");
            if p.join("surya-2.5.toml").is_file() {
                return p;
            }
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../catalog")
}

/// Read a text file as Python's `read_text` does (universal newlines: CRLF and CR -> LF).
pub fn read_text(p: &std::path::Path) -> std::io::Result<String> {
    Ok(std::fs::read_to_string(p)?.replace("\r\n", "\n").replace('\r', "\n"))
}
