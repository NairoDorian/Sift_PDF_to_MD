//! The answer store (docs/GOLDEN.md): every model answer, keyed by what produced it.
//!
//! Key = sha256(role \0 prompt \0 image pixel hashes, comma-joined \0 max_tokens \0 extra),
//! the key the golden set (tests/golden/answers) was recorded with, so a run can be replayed for an
//! exact A/B of everything after the models (llama.cpp isn't byte-repeatable with several slots).
//! The pixel hash (imaging::Rgb::pixel_hash) is over decoded RGB, never over PNG bytes.
//!
//! With `--answers <dir>` (or `SIFT_ANSWERS`) the store also keeps answers across runs, so a
//! re-run reuses them; off when unset.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Value, json};

use crate::imaging::sha256_hex;

pub fn request_key(role: &str, prompt: &str, pixel_hashes: &[String], max_tokens: u32, extra: &str) -> String {
    let blob = format!("{role}\0{prompt}\0{}\0{max_tokens}\0{extra}", pixel_hashes.join(","));
    sha256_hex(blob.as_bytes())
}

#[derive(Default)]
pub struct Stats {
    pub hits: AtomicUsize,
    pub misses: AtomicUsize,
    pub writes: AtomicUsize,
}

pub struct AnswerStore {
    /// read-only stores tried first (e.g. V0's oracle), then `dir`
    pub replay: Vec<PathBuf>,
    pub dir: Option<PathBuf>,
    /// replay only: never call a model (a miss is reported, not filled)
    pub replay_only: bool,
    pub stats: Stats,
    pub misses: std::sync::Mutex<Vec<String>>,
}

impl AnswerStore {
    pub fn new(dir: Option<PathBuf>, replay: Vec<PathBuf>, replay_only: bool) -> AnswerStore {
        AnswerStore { replay, dir, replay_only, stats: Stats::default(), misses: Default::default() }
    }

    pub fn disabled() -> AnswerStore {
        AnswerStore::new(None, vec![], false)
    }

    fn path(root: &Path, key: &str) -> PathBuf {
        root.join(&key[..2]).join(format!("{key}.json"))
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        for root in self.replay.iter().chain(self.dir.iter()) {
            if let Ok(s) = std::fs::read_to_string(Self::path(root, key))
                && let Ok(v) = serde_json::from_str::<Value>(&s)
            {
                self.stats.hits.fetch_add(1, Ordering::Relaxed);
                return v.get("answer").cloned();
            }
        }
        self.stats.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    pub fn note_miss(&self, what: String) {
        if let Ok(mut m) = self.misses.lock() {
            m.push(what);
        }
    }

    /// Write one answer (temp file + rename; a failed write never fails the job).
    pub fn put(&self, key: &str, role: &str, prompt: &str, max_tokens: u32, extra: &str, answer: &Value) {
        let Some(root) = &self.dir else { return };
        let p = Self::path(root, key);
        let rec = json!({"role": role, "max_tokens": max_tokens, "extra": extra, "prompt": prompt, "answer": answer});
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let tmp = p.with_extension(
            format!("{}.{:?}.tmp", std::process::id(), std::thread::current().id()).replace(['(', ')'], ""),
        );
        if std::fs::write(&tmp, serde_json::to_vec(&rec).unwrap_or_default()).is_ok()
            && std::fs::rename(&tmp, &p).is_ok()
        {
            self.stats.writes.fetch_add(1, Ordering::Relaxed);
        } else {
            let _ = std::fs::remove_file(&tmp);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_matches_v0_formula() {
        // python: hashlib.sha256("\0".join(["surya", "p", "a,b", "64", ""]).encode()).hexdigest()
        let k = request_key("surya", "p", &["a".into(), "b".into()], 64, "");
        assert_eq!(k, sha256_hex("surya\0p\0a,b\064\0".as_bytes()));
    }
}
