//! What the window and the Rust side exchange: the commands' arguments and results, and the
//! events. Every type here (and the sift-core types it names, behind sift-core's `specta`
//! feature) is written into `app/src/lib/bindings.ts` by tauri-specta, so the page code and
//! these definitions cannot drift apart; `main.rs` checks the committed file in a test.

use serde::{Deserialize, Serialize};
use sift_core::{engine, meters, models};
use specta::Type;
use tauri_specta::Event;

// ---- command arguments -----------------------------------------------------------------

/// One run from the Convert page. The options are the page's switches, sent as they are shown
/// (the page also saves them, so the window and the command line share one set of defaults);
/// `pages`, `recursive`, `force` and `output` belong to this run only.
#[derive(Deserialize, Type)]
pub struct ConvertRequest {
    pub input: String,
    pub output: Option<String>,
    pub pages: Option<String>,
    pub recursive: bool,
    pub force: bool,
    /// `gemma` or `off`
    pub describer: String,
    /// the LLM pass (`[llm] enabled`)
    pub llm: bool,
    /// `hybrid` or `full-ocr`
    pub mode: String,
    /// `auto` or `markdown`
    pub tables: String,
    pub keep_headers: bool,
    pub image_dpi: Option<u32>,
}

/// Which folder `set_folder` changes.
#[derive(Deserialize, Type, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FolderKind {
    Engine,
    Models,
}

/// What the Engine page asks of an installed build.
#[derive(Deserialize, Type, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum EngineAction {
    /// make this build the active one
    Use,
    /// back to the previous build
    Rollback,
    /// delete this build
    Remove,
    /// delete the CUDA runtime this build carries (a toolkit provides it)
    RemoveRuntime,
}

// ---- command results ---------------------------------------------------------------------

/// What the window needs to know at a glance (onboarding, footer, menu dot).
#[derive(Serialize, Type)]
pub struct Overview {
    pub version: String,
    pub settings_file: String,
    pub portable: bool,
    pub engine_dir: String,
    pub models_dir: String,
    pub engine_active: Option<String>,
    pub models: Vec<models::Status>,
    pub check_at_startup: bool,
}

/// The Engine page: the active build, the builds in the engine folder, the machine's CUDA.
#[derive(Serialize, Type)]
pub struct EngineStatus {
    pub active: Option<String>,
    pub previous: Option<String>,
    /// `llama-server --version` of the active build
    pub version: Option<String>,
    pub dir: Option<String>,
    /// the setting (`cuda-13.5`, `vulkan`, `cpu`, `auto`)
    pub backend: String,
    /// what this machine supports
    pub detected: String,
    /// the CUDA version the NVIDIA driver supports
    pub driver_cuda: Option<String>,
    /// a CUDA toolkit's runtime folder, when one is installed
    pub toolkit: Option<String>,
    pub include_cudart: String,
    pub check_at_startup: bool,
    pub builds: Vec<engine::Installed>,
}

/// What the Convert page shows about its input, before converting.
#[derive(Serialize, Type)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InputInfo {
    Pdf {
        name: String,
        dir: String,
        path: String,
        size: u64,
        /// `None` when pdfium cannot open it (the converter then reports why)
        pages: Option<usize>,
    },
    Folder {
        name: String,
        dir: String,
        path: String,
        /// PDFs at the folder's top level
        pdfs: usize,
        /// with its subfolders (what a recursive run converts)
        pdfs_all: usize,
    },
}

/// What `set_folder` moved from the previous folder (nothing when used as it is).
#[derive(Serialize, Type)]
pub struct FolderMove {
    pub moved: Vec<String>,
    pub old: String,
}

/// One model server's command line, as Sift would start it now.
#[derive(Serialize, Type)]
pub struct ServerCommand {
    pub model: String,
    pub args: Vec<String>,
}

/// The end of a model server's newest log.
#[derive(Serialize, Type)]
pub struct ServerLog {
    pub file: Option<String>,
    pub lines: Vec<String>,
}

/// One file of a model, checked against its sha256.
#[derive(Serialize, Type)]
pub struct ModelFile {
    pub file: String,
    pub ok: bool,
}

/// A model's pinned revision against its repository's newest (`None`: could not check).
#[derive(Serialize, Type)]
pub struct ModelRevision {
    pub id: String,
    pub pinned: String,
    pub upstream: Option<String>,
}

/// One NVIDIA GPU as `nvidia-smi` reports it (the About page).
#[derive(Serialize, Type)]
pub struct Gpu {
    pub name: String,
    pub total_mib: u64,
    pub used_mib: u64,
    pub driver: String,
}

/// The About page: versions, where things are, the machine.
#[derive(Serialize, Type)]
pub struct AboutInfo {
    pub version: String,
    pub settings: String,
    pub exe_dir: String,
    pub gpus: Vec<Gpu>,
    pub driver_cuda: Option<String>,
    pub cpu_threads: usize,
}

// ---- events (docs/events.md) ---------------------------------------------------------------

/// One line of the conversion's progress (`sift-event`): the JSON the command line prints with
/// `--json`, documented in docs/events.md and typed on the page side (`SiftEvent` in api.ts).
#[derive(Serialize, Clone, Type, Event)]
#[tauri_specta(event_name = "sift-event")]
pub struct ConvertEvent(pub serde_json::Value);

/// The conversion ended (`sift-done`).
#[derive(Serialize, Clone, Type, Event)]
#[tauri_specta(event_name = "sift-done")]
pub struct ConvertDone {
    pub ok: bool,
    pub cancelled: bool,
    pub error: Option<String>,
}

/// The footer's meters, once a second while the window shows (`meters`).
#[derive(Serialize, Clone, Type, Event)]
#[tauri_specta(event_name = "meters")]
pub struct MetersUpdate(pub meters::Sample);

/// A line of the engine's install or switch (`engine-log`).
#[derive(Serialize, Clone, Type, Event)]
#[tauri_specta(event_name = "engine-log")]
pub struct EngineLog {
    pub message: String,
}

/// The engine download's bytes (`engine-progress`).
#[derive(Serialize, Clone, Type, Event)]
#[tauri_specta(event_name = "engine-progress")]
pub struct EngineProgress {
    pub done: u64,
    pub total: u64,
}

/// An engine install or action ended (`engine-done`).
#[derive(Serialize, Clone, Type, Event)]
#[tauri_specta(event_name = "engine-done")]
pub struct EngineDone {
    pub ok: bool,
    /// the installed build's folder (install)
    pub build: Option<String>,
    /// what was done (actions)
    pub message: Option<String>,
    pub error: Option<String>,
}

/// The release list was read (`engine-check`): from the Check button, or at start-up when the
/// user turned that on.
#[derive(Serialize, Clone, Type, Event)]
#[tauri_specta(event_name = "engine-check")]
pub struct EngineChecked(pub engine::Check);

/// A model download's progress, about every 20 MB (`model-progress`).
#[derive(Serialize, Clone, Type, Event)]
#[tauri_specta(event_name = "model-progress")]
pub struct ModelProgress {
    pub id: String,
    pub file: String,
    pub phase: String,
    pub done: u64,
    pub total: u64,
}

/// A model's download ended, verified or not (`model-done`).
#[derive(Serialize, Clone, Type, Event)]
#[tauri_specta(event_name = "model-done")]
pub struct ModelDone {
    pub id: String,
    pub ok: bool,
    pub error: Option<String>,
}
