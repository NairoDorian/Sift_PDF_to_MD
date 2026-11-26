//! Settings (MASTER_PLAN section 5): one `settings.toml` shared by the window and the command
//! line (`paths::settings_file`), in V0's `sift.toml` format plus `[folders]` and `[engine]`.
//! Priority: command line > environment (SIFT_<SECTION>_<KEY>) > `--config` file > settings file
//! > defaults. Flags never change the saved settings.

use std::path::Path;

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(default)]
pub struct Convert {
    pub mode: String,
    pub image_dpi: u32,
    pub tables: String,
    pub keep_headers: bool,
    pub describer: String,
    pub layout_batch: usize,
    pub pages: String,
}
impl Default for Convert {
    fn default() -> Self {
        Convert {
            mode: "hybrid".into(),
            image_dpi: 300,
            tables: "auto".into(),
            keep_headers: false,
            describer: "gemma".into(),
            layout_batch: 8,
            pages: String::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(default)]
pub struct Surya {
    pub model: String,
    pub parallel: usize,
    pub ctx_per_slot: usize,
}
impl Default for Surya {
    fn default() -> Self {
        Surya { model: "q8_0".into(), parallel: 6, ctx_per_slot: 12288 }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(default)]
pub struct Gemma {
    pub model: String,
    pub tuned: String,
    pub parallel: usize,
    pub ctx_per_slot: usize,
    pub mtp: bool,
    pub draft: Option<String>,
    pub spec_draft_n_max: u32,
    pub kv_type: String,
    pub batch: u32,
    pub ubatch: u32,
    pub temperature: f64,
    pub top_p: f64,
    pub top_k: u32,
    pub min_p: f64,
    pub repeat_penalty: Option<f64>,
    pub max_tokens: u32,
    pub image_min_tokens: Option<u32>,
    pub image_max_tokens: Option<u32>,
    pub reasoning: String,
}
impl Default for Gemma {
    fn default() -> Self {
        Gemma {
            model: "e4b".into(),
            tuned: "not yet".into(),
            parallel: 2,
            ctx_per_slot: 8192,
            mtp: true,
            draft: None,
            spec_draft_n_max: 4,
            kv_type: "f16".into(),
            batch: 2048,
            ubatch: 512,
            temperature: 0.05,
            top_p: 0.35,
            top_k: 64,
            min_p: 0.0,
            repeat_penalty: None,
            max_tokens: 900,
            image_min_tokens: None,
            image_max_tokens: None,
            reasoning: "off".into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(default)]
pub struct Llm {
    pub enabled: bool,
    pub tables: bool,
    pub headings: bool,
    pub table_merge: bool,
    pub equations: bool,
    pub reasoning: bool,
}
impl Default for Llm {
    fn default() -> Self {
        Llm { enabled: true, tables: true, headings: true, table_merge: true, equations: true, reasoning: true }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(default)]
pub struct Describe {
    pub figure_ocr: bool,
}
impl Default for Describe {
    fn default() -> Self {
        Describe { figure_ocr: true }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(default)]
pub struct Scheduler {
    /// sequential (surya first, then Gemma) | together (V0 [budget] models)
    pub models: String,
    pub together_min_free_mib: u64,
    /// page-parallel layout workers (0 = auto)
    pub layout_workers: usize,
}
impl Default for Scheduler {
    fn default() -> Self {
        Scheduler { models: "sequential".into(), together_min_free_mib: 6300, layout_workers: 0 }
    }
}

/// Where the engine and the models live: chosen by the user, empty until then (VD7). Nothing is
/// ever downloaded into a default folder.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(default)]
pub struct Folders {
    pub engine_dir: String,
    pub models_dir: String,
}

/// The llama.cpp engine (MASTER_PLAN section 6).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(default)]
pub struct EngineSettings {
    /// cuda-13.5 | vulkan | cpu | auto (detected)
    pub backend: String,
    /// check GitHub for a newer official build when Sift starts (asked in onboarding)
    pub check_at_startup: bool,
    /// auto: download the CUDA runtime package only when no CUDA 13 toolkit is found | always | never
    pub include_cudart: String,
}
impl Default for EngineSettings {
    fn default() -> Self {
        EngineSettings { backend: "cuda-13.5".into(), check_at_startup: false, include_cudart: "auto".into() }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type), serde(rename = "Settings"))]
#[serde(default)]
pub struct Config {
    pub folders: Folders,
    pub engine: EngineSettings,
    pub convert: Convert,
    pub surya: Surya,
    pub gemma: Gemma,
    pub llm: Llm,
    pub describe: Describe,
    /// V0's [budget] is [scheduler]; both names are read
    #[serde(alias = "budget")]
    pub scheduler: Scheduler,
}

/// Deep-merge `top` into `base` (tables merge key by key, anything else replaces).
fn merge(base: &mut toml::Table, top: toml::Table) {
    for (k, v) in top {
        match (base.get_mut(&k), v) {
            (Some(toml::Value::Table(b)), toml::Value::Table(t)) => merge(b, t),
            (_, v) => {
                base.insert(k, v);
            }
        }
    }
}

fn read_table(p: &Path) -> Result<toml::Table> {
    let s = std::fs::read_to_string(p).with_context(|| format!("reading {}", p.display()))?;
    toml::from_str::<toml::Table>(&s).with_context(|| format!("parsing {}", p.display()))
}

impl Config {
    /// The saved settings, then `extra` (a `--config` file, V0's `sift.toml` works as is) on
    /// top, then the environment.
    pub fn load(extra: Option<&Path>) -> Result<Config> {
        let mut t = toml::Table::new();
        let saved = crate::paths::settings_file();
        if saved.exists() {
            t = read_table(&saved)?;
        }
        if let Some(p) = extra {
            if !p.exists() {
                return Err(anyhow!("settings file {} not found", p.display()));
            }
            merge(&mut t, read_table(p)?);
        }
        let mut cfg: Config = toml::Value::Table(t).try_into()?;
        cfg.apply_env();
        Ok(cfg)
    }

    /// Only the saved settings (no environment): what the window shows and saves.
    pub fn load_saved() -> Result<Config> {
        let saved = crate::paths::settings_file();
        if !saved.exists() {
            return Ok(Config::default());
        }
        Ok(toml::Value::Table(read_table(&saved)?).try_into()?)
    }

    pub fn save(&self) -> Result<()> {
        let p = crate::paths::settings_file();
        if let Some(d) = p.parent() {
            std::fs::create_dir_all(d)?;
        }
        std::fs::write(&p, toml::to_string_pretty(self)?).with_context(|| format!("writing {}", p.display()))
    }

    /// `section.key = value` with the key's own type (`sift config set`).
    pub fn set(&mut self, key: &str, val: &str) -> Result<()> {
        let (sec, k) = key.split_once('.').ok_or_else(|| anyhow!("use section.key, e.g. folders.models_dir"))?;
        let mut v = serde_json::to_value(&*self)?;
        let obj = v.get_mut(sec).and_then(|s| s.as_object_mut()).ok_or_else(|| anyhow!("no section [{sec}]"))?;
        let cur = obj.get(k).ok_or_else(|| anyhow!("no key {k} in [{sec}]"))?;
        obj.insert(k.to_string(), typed(cur, val));
        *self = serde_json::from_value(v)?;
        Ok(())
    }

    /// SIFT_<SECTION>_<KEY> overrides (V0 config.load).
    fn apply_env(&mut self) {
        let mut v = serde_json::to_value(&*self).unwrap_or_default();
        for (k, val) in std::env::vars() {
            let Some(rest) = k.strip_prefix("SIFT_") else { continue };
            let lower = rest.to_lowercase();
            let Some((sec, key)) = lower.split_once('_') else { continue };
            let sec = if sec == "budget" { "scheduler" } else { sec };
            let Some(obj) = v.get_mut(sec).and_then(|s| s.as_object_mut()) else { continue };
            let Some(cur) = obj.get(key) else { continue };
            let new = typed(cur, &val);
            obj.insert(key.to_string(), new);
        }
        if let Ok(c) = serde_json::from_value(v) {
            *self = c;
        }
    }
}

fn typed(cur: &serde_json::Value, val: &str) -> serde_json::Value {
    match cur {
        serde_json::Value::Bool(_) => {
            serde_json::json!(matches!(val.trim().to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        }
        serde_json::Value::Number(n) if n.is_f64() => {
            val.parse::<f64>().map(serde_json::Value::from).unwrap_or(cur.clone())
        }
        serde_json::Value::Number(_) => val.parse::<i64>().map(serde_json::Value::from).unwrap_or(cur.clone()),
        _ => serde_json::Value::String(val.to_string()),
    }
}
