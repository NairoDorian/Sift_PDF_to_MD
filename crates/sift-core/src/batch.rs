//! A command: one PDF or a folder of PDFs (D3, D11; V0 `sift/batch.py`).
//! Output mirrors the input subfolders: OUT/<subfolders>/<pdf name>/<pdf name>.md + images/.
//! In folder mode an up-to-date .md (newer than its PDF, converted with the same options:
//! `.sift-options` next to it) is skipped unless `force`. One failing PDF doesn't stop
//! the others. Files overlap: the next PDF is prepared while the previous one finalizes.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Result, anyhow};
use serde_json::json;

use crate::answers::AnswerStore;
use crate::config::Config;
use crate::events::Events;
use crate::pipeline::{FileJob, FileResult, Options, Runtime};

#[derive(Clone, Default)]
pub struct BatchOptions {
    pub output: Option<PathBuf>,
    pub recursive: bool,
    pub force: bool,
    pub pipeline: Options,
    /// Cancel (the app's button): queued work stops, then the servers; no model starts after it
    pub cancel: Option<Arc<std::sync::atomic::AtomicBool>>,
}

pub fn find_pdfs(inp: &Path, recursive: bool) -> Vec<PathBuf> {
    if inp.is_file() {
        return vec![inp.to_path_buf()];
    }
    let mut out = Vec::new();
    fn walk(d: &Path, rec: bool, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        let mut entries: Vec<PathBuf> = rd.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                if rec {
                    walk(&p, rec, out);
                }
            } else if p.extension().is_some_and(|e| e.eq_ignore_ascii_case("pdf")) {
                out.push(p);
            }
        }
    }
    walk(inp, recursive, &mut out);
    out.sort();
    out
}

/// The options a file was converted with, kept next to its Markdown: in folder mode a file is up
/// to date only when it is newer than its PDF AND was converted with the same options (a quick
/// `--pages 1-2 --no-llm` run must not stand in for the real conversion later).
const STAMP: &str = ".sift-options";

fn options_stamp(cfg: &Config, opts: &BatchOptions) -> String {
    format!(
        "pages={}\nmode={}\ntables={}\nkeep_headers={}\nimage_dpi={}\ndescriber={}\nllm={}\nsurya={}\n",
        cfg.convert.pages,
        cfg.convert.mode,
        cfg.convert.tables,
        cfg.convert.keep_headers,
        cfg.convert.image_dpi,
        opts.pipeline.describer,
        cfg.llm.enabled,
        opts.pipeline.use_surya
    )
}

fn is_done(pdf: &Path, out_dir: &Path, stamp: &str) -> bool {
    let md = out_dir.join(format!("{}.md", pdf.file_stem().unwrap_or_default().to_string_lossy()));
    let fresh =
        match (std::fs::metadata(&md).and_then(|m| m.modified()), std::fs::metadata(pdf).and_then(|m| m.modified())) {
            (Ok(a), Ok(b)) => a >= b,
            _ => false,
        };
    // no stamp: made before Sift recorded its options, trusted as before (no surprise redo of a library)
    fresh && std::fs::read_to_string(out_dir.join(STAMP)).map(|s| s == stamp).unwrap_or(true)
}

pub fn run(
    cfg: Config,
    inp: &Path,
    opts: BatchOptions,
    events: Arc<Events>,
    store: Arc<AnswerStore>,
) -> Result<(i32, Vec<FileResult>)> {
    let inp = std::fs::canonicalize(inp).map_err(|_| anyhow!("{} not found", inp.display()))?;
    let out_root = match &opts.output {
        Some(o) => {
            std::fs::create_dir_all(o)?;
            std::fs::canonicalize(o)?
        }
        None => {
            if inp.is_dir() {
                inp.clone()
            } else {
                inp.parent().unwrap().to_path_buf()
            }
        }
    };
    let out_root = dunce(&out_root);
    let inp = dunce(&inp);
    let pdfs = find_pdfs(&inp, opts.recursive);
    let base = if inp.is_dir() { inp.clone() } else { inp.parent().unwrap().to_path_buf() };
    let mut todo: Vec<(PathBuf, PathBuf)> = pdfs
        .into_iter()
        .map(|p| {
            let rel = p.parent().and_then(|d| d.strip_prefix(&base).ok()).map(Path::to_path_buf).unwrap_or_default();
            let stem = p.file_stem().unwrap_or_default().to_string_lossy().to_string();
            let o = out_root.join(rel).join(stem);
            (p, o)
        })
        .collect();
    let mut skipped = 0;
    let stamp = options_stamp(&cfg, &opts);
    if inp.is_dir() && !opts.force {
        let before = todo.len();
        todo.retain(|(p, o)| !is_done(p, o, &stamp));
        skipped = before - todo.len();
    }
    let t0 = Instant::now();
    if todo.is_empty() {
        events.emit("job_start", json!({"files": 0, "output": out_root.display().to_string(), "mode": cfg.convert.mode, "describer": opts.pipeline.describer, "skipped": skipped}));
        events.emit("job_done", json!({"files": 0, "ok": 0, "failed": 0, "skipped": skipped, "seconds": 0.0}));
        return Ok((0, vec![]));
    }
    let mode = cfg.convert.mode.clone();
    let mut rt =
        Runtime::new(cfg, opts.pipeline.clone(), events.clone(), store.clone(), out_root.join(".sift").join("logs"))?;
    rt.cancel = opts.cancel.clone();
    events.emit("job_start", json!({"files": todo.len(), "output": out_root.display().to_string(), "mode": mode, "describer": opts.pipeline.describer, "engine": rt.engine, "skipped": skipped}));
    let mut results = Vec::new();
    let n = todo.len();
    let done = |r: &FileResult| {
        if let Some(e) = &r.error {
            // `cancelled`: stopped by Cancel, nothing written for it (Runtime::check_cancel)
            events
                .emit("file_error", json!({"file": r.pdf.display().to_string(), "error": e, "cancelled": r.cancelled}));
        } else {
            if let Some(dir) = r.md.as_deref().and_then(Path::parent) {
                // best effort: without it the file only counts as up to date by its date, as before
                let _ = std::fs::write(dir.join(STAMP), &stamp);
            }
            events.emit("file_done", json!({"file": r.pdf.display().to_string(), "md": r.md.as_ref().map(|p| p.display().to_string()), "images": r.images,
                "report": r.report.as_ref().map(|p| p.display().to_string()), "seconds": r.seconds,
                "stats": r.stats.iter().map(|(k, v)| (k.clone(), json!(v))).collect::<serde_json::Map<_, _>>(),
                "stages": r.profile.iter().map(|(k, v)| (k.clone(), json!(v))).collect::<serde_json::Map<_, _>>()}));
        }
    };
    let all_done = std::sync::atomic::AtomicBool::new(false);
    let cancelled = || opts.cancel.as_ref().is_some_and(|c| c.load(std::sync::atomic::Ordering::Relaxed));
    std::thread::scope(|s| {
        // Cancel: stop the services (queued work fails fast; a waiting model never starts)
        if opts.cancel.is_some() {
            let (rt, all_done, events) = (&rt, &all_done, &events);
            s.spawn(move || {
                while !all_done.load(std::sync::atomic::Ordering::Relaxed) {
                    if cancelled() {
                        events.emit("warning", json!({"message": "Cancelled: stopping the models"}));
                        rt.shutdown();
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(150));
                }
            });
        }
        let mut pending: Option<std::thread::ScopedJoinHandle<FileResult>> = None;
        for (i, (pdf, out)) in todo.iter().enumerate() {
            if cancelled() {
                rt.finish_surya();
                break;
            }
            let mut job = FileJob::new(&rt, pdf, out, i, n);
            let prep = job.prepare();
            if let Some(h) = pending.take() {
                let r = h
                    .join()
                    .unwrap_or_else(|_| FileResult { error: Some("finalize panicked".into()), ..Default::default() });
                done(&r);
                results.push(r);
            }
            match prep {
                Ok(()) => {
                    if i + 1 == n {
                        rt.finish_surya();
                    }
                    pending = Some(s.spawn(move || job.finalize()));
                }
                Err(e) => {
                    let r = FileResult {
                        pdf: pdf.clone(),
                        cancelled: e.is::<crate::pipeline::Cancelled>(),
                        error: Some(format!("{e:#}")),
                        ..Default::default()
                    };
                    done(&r);
                    results.push(r);
                    if i + 1 == n {
                        rt.finish_surya();
                    }
                }
            }
        }
        if let Some(h) = pending.take() {
            let r = h
                .join()
                .unwrap_or_else(|_| FileResult { error: Some("finalize panicked".into()), ..Default::default() });
            done(&r);
            results.push(r);
        }
        all_done.store(true, std::sync::atomic::Ordering::Relaxed);
    });
    rt.finish();
    rt.shutdown();
    let failed = results.iter().filter(|r| r.error.is_some()).count();
    let st = &store.stats;
    events.emit("job_done", json!({"files": n, "ok": results.len() - failed, "failed": failed, "skipped": skipped,
        "seconds": (t0.elapsed().as_secs_f64() * 100.0).round() / 100.0, "trace": rt.trace_path.as_ref().map(|p| p.display().to_string()),
        "answers": {"hits": st.hits.load(std::sync::atomic::Ordering::Relaxed), "misses": st.misses.load(std::sync::atomic::Ordering::Relaxed), "written": st.writes.load(std::sync::atomic::Ordering::Relaxed)}}));
    if opts.pipeline.profile {
        for r in &results {
            events.emit("log", json!({"message": format!("profile {}: {:?} stats {:?}", r.pdf.file_name().unwrap_or_default().to_string_lossy(), r.profile, r.stats)}));
        }
    }
    let miss = store.misses.lock().unwrap();
    if !miss.is_empty() {
        events.emit("warning", json!({"message": format!("{} model requests were not in the answer store (replay): first {:?}", miss.len(), &miss[..miss.len().min(5)])}));
    }
    Ok((if failed > 0 { 1 } else { 0 }, results))
}

/// Strip the \\?\ prefix canonicalize adds on Windows (paths in Markdown and logs).
fn dunce(p: &Path) -> PathBuf {
    let s = p.display().to_string();
    match s.strip_prefix(r"\\?\") {
        Some(r) => PathBuf::from(r),
        None => p.to_path_buf(),
    }
}
