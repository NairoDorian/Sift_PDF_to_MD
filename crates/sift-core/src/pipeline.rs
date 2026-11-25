//! One PDF -> <name>.md + images/ + descriptions + report (MASTER_PLAN section 8).
//!
//! `Runtime` holds what every file of one command shares: settings, events, the answer store,
//! the layout model and the model services. `FileJob::prepare` reads the text layer, lays out
//! the pages batch by batch, builds the blocks and queues every model request (surya starts
//! on the first table while layout goes on, as in V0); `finalize` applies the answers in a
//! fixed order (V0 applied them as threads finished), runs Gemma, cleans up and writes.

use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use anyhow::{Context, Result, anyhow};
use serde_json::{Value, json};

use crate::answers::AnswerStore;
use crate::brain::{self, passes};
use crate::build::blocks::{Block, Page};
use crate::build::cleanup;
use crate::build::headings::{self, SectionTracker};
use crate::build::page::{FigureTask, SuryaTask, attach_captions, build_page, nearby_text, rare_font_families};
use crate::build::tables as xcheck;
use crate::build::text as T;
use crate::config::Config;
use crate::describe::{FigureContext, build_prompt, kind_from_text};
use crate::events::Events;
use crate::imaging::{Rgb, crop_pt};
use crate::layout::{FIGURE_LABELS, LayoutBox, LayoutDetector, LayoutResult, relabel};
use crate::llm::{Completion, Gate, LlamaServer, ModelService, StateFn};
use crate::pdf::pdfium::Document;
use crate::pdf::quality::page_text_ok;
use crate::pdf::textlayer::{Line, TextPage, extract};
use crate::py::{self};
use crate::surya::client::{self as sclient, Prepared, prepare_image};
use crate::surya::html::{self, Part};

pub const LAYOUT_DPI: f64 = 96.0;
pub const OCR_DPI: f64 = 192.0;
pub const FULL_PAGE_MAX_TOKENS: u32 = 12288;

#[derive(Clone, Default)]
pub struct Options {
    pub use_surya: bool,
    /// gemma | off
    pub describer: String,
    /// golden test: layout comes from <dir>/<pdf stem>/layout.json (tests/golden/layout)
    pub layout_replay: Option<PathBuf>,
    pub profile: bool,
    /// measurement: ignore the text layer, every page is read as if scanned (score the result
    /// against the text layer with bench/score.py)
    pub mask_text_layer: bool,
}

pub struct Runtime {
    pub cfg: Config,
    pub opts: Options,
    pub events: Arc<Events>,
    pub store: Arc<AnswerStore>,
    pub log_dir: PathBuf,
    /// surya 2.5: tables, formulas, broken or scanned text, labels inside figures
    pub surya: Option<Arc<ModelService>>,
    /// opened when surya stopped: Gemma may start (one model at a time on the GPU)
    pub brain_gate: Option<Arc<Gate>>,
    /// Gemma 4 E4B: descriptions and the LLM pass
    pub brain: Option<Arc<ModelService>>,
    pub detector: Option<Arc<LayoutDetector>>,
    pub engine: String,
    pub need_brain: bool,
    trace: Mutex<Option<std::fs::File>>,
    pub trace_path: Option<PathBuf>,
    t0: Instant,
}

fn on_state(events: Arc<Events>) -> StateFn {
    Arc::new(move |name: &str, state: &str, secs: Option<f64>| {
        events.emit("server", json!({"model": name, "state": state, "seconds": secs}))
    })
}

impl Runtime {
    pub fn new(
        cfg: Config,
        opts: Options,
        events: Arc<Events>,
        store: Arc<AnswerStore>,
        log_dir: PathBuf,
    ) -> Result<Runtime> {
        let llm_wanted = cfg.llm.enabled;
        let need_brain = opts.describer == "gemma" || (llm_wanted && opts.describer == "off");
        let live = !store.replay_only;
        let mut engine = String::new();
        let mut build: Option<PathBuf> = None;
        if live && (opts.use_surya || need_brain) {
            let b = crate::engine::current_build()?;
            engine = b.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            build = Some(b);
        }
        // one model at a time (V0's measured default on one 8 GB GPU): surya first, Gemma after
        // surya stopped; surya's service opens Gemma's gate when it stops
        let together = cfg.scheduler.models == "together";
        let mut surya_done: Option<Arc<Gate>> = None;
        let surya = if opts.use_surya {
            let next = Arc::new(Gate::default());
            let server = match &build {
                Some(b) => {
                    let dir = crate::paths::find_model_dir("surya-ocr-2.5-gguf")?;
                    let file = if cfg.surya.model == "bf16" { "surya-2.5.gguf" } else { "surya-2.5-Q8_0.gguf" };
                    let (m, mm) = (dir.join(file), dir.join("surya-2.5-mmproj.gguf"));
                    for f in [&m, &mm] {
                        if !f.exists() {
                            return Err(anyhow!(
                                "{} is missing. Download it: sift models download surya-2.5",
                                f.display()
                            ));
                        }
                    }
                    Some(Arc::new(sclient::make_server(
                        &m,
                        &mm,
                        cfg.surya.parallel,
                        cfg.surya.ctx_per_slot,
                        b,
                        &log_dir,
                        on_state(events.clone()),
                    )))
                }
                None => None,
            };
            let svc = ModelService::new("surya", cfg.surya.parallel, server, None, Some(next.clone()));
            surya_done = Some(next);
            Some(svc)
        } else {
            None
        };
        let gate = if together { None } else { surya_done };
        let brain = if need_brain {
            let server = match &build {
                Some(b) => {
                    crate::paths::models_dir()?;
                    let v = brain::variant(&cfg.gemma.model);
                    let missing = v.missing(&cfg.gemma);
                    if !missing.is_empty() {
                        return Err(anyhow!(
                            "{} files missing in {}: {:?}. Download them: sift models download gemma-4-e4b-qat",
                            v.name,
                            v.folder.display(),
                            missing
                        ));
                    }
                    Some(Arc::new(brain::make_server(&cfg.gemma, b, &log_dir, on_state(events.clone()))))
                }
                None => None,
            };
            Some(ModelService::new("gemma", cfg.gemma.parallel, server, gate.clone(), None))
        } else {
            None
        };
        let detector = if opts.layout_replay.is_none() {
            let dir = crate::paths::find_model_dir("surya_layout2")?;
            Some(Arc::new(LayoutDetector::load(&dir).with_context(|| format!("layout model in {}", dir.display()))?))
        } else {
            None
        };
        std::fs::create_dir_all(&log_dir)?;
        let trace_path = log_dir.join(format!("{}-sift-trace.jsonl", crate::util::stamp()));
        let rt = Runtime {
            cfg,
            opts,
            events,
            store,
            log_dir,
            surya,
            brain_gate: gate,
            brain,
            detector,
            engine,
            need_brain,
            trace: Mutex::new(std::fs::File::create(&trace_path).ok()),
            trace_path: Some(trace_path),
            t0: Instant::now(),
        };
        rt.trace(json!({"event": "job", "engine": rt.engine, "surya": rt.cfg.surya, "gemma": rt.cfg.gemma, "replay_only": rt.store.replay_only}));
        Ok(rt)
    }

    pub fn trace(&self, rec: Value) {
        let mut v = json!({"t": (self.t0.elapsed().as_secs_f64() * 1000.0).round() / 1000.0});
        if let (Value::Object(a), Value::Object(b)) = (&mut v, rec) {
            a.extend(b);
        }
        if let Some(f) = self.trace.lock().unwrap().as_mut() {
            let _ = writeln!(f, "{v}");
        }
    }

    pub fn llm_on(&self, what: &str) -> bool {
        let l = &self.cfg.llm;
        let flag = match what {
            "tables" => l.tables,
            "headings" => l.headings,
            "table_merge" => l.table_merge,
            "equations" => l.equations,
            _ => true,
        };
        l.enabled && flag && self.brain.is_some()
    }

    /// No more reading work: surya stops once its queue is empty (and opens Gemma's gate).
    pub fn finish_surya(&self) {
        if let Some(s) = &self.surya {
            s.finish();
        }
    }
    pub fn finish(&self) {
        self.finish_surya();
        if let Some(s) = &self.brain {
            s.finish();
        }
    }
    pub fn shutdown(&self) {
        if let Some(s) = &self.brain {
            s.shutdown();
        }
        if let Some(s) = &self.surya {
            s.shutdown();
        }
    }

    /// Gemma may start only after surya stopped (one model at a time on 8 GB).
    pub fn wait_brain_gate(&self) {
        if let Some(g) = &self.brain_gate {
            g.wait();
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct FileResult {
    pub pdf: PathBuf,
    pub md: Option<PathBuf>,
    pub images: usize,
    pub report: Option<PathBuf>,
    pub seconds: f64,
    pub error: Option<String>,
    pub profile: Vec<(String, f64)>,
    pub stats: Vec<(String, i64)>,
}

enum SuryaOut {
    Block { task: SuryaTask, page: usize, c: Completion, max_tokens: u32, png: Prepared, blank: bool },
    FigureOcr { fig: u64, c: Completion },
    FullPage { page: usize, c: Completion, img: Arc<Rgb>, fallback: Vec<(u64, Completion)> },
}

struct FigJob {
    task: FigureTask,
    page: usize,
    crop: Arc<Rgb>,
    png: Arc<Vec<u8>>,
    describe: bool,
    ocr_all: String,
}

pub struct FileJob<'a> {
    rt: &'a Runtime,
    pub pdf: PathBuf,
    pub out_dir: PathBuf,
    pub stem: String,
    images_dir: PathBuf,
    index: usize,
    total: usize,
    doc: Option<Document>,
    text_pages: Vec<TextPage>,
    pages: Vec<Page>,
    report: Vec<String>,
    stats: Vec<(String, i64)>,
    profile: Vec<(String, f64)>,
    t0: Instant,
    last_tick: f64,
    seq: usize,
    tx: Sender<(usize, SuryaOut)>,
    rx: Receiver<(usize, SuryaOut)>,
    submitted: usize,
    figs: Vec<FigJob>,
    fig_extra: usize,
}

fn page_key(line: &str) -> (i64, String) {
    crate::pyre_static!(P, ReStart, r"page (\d+)");
    match P.m(line) {
        Some(m) => (m[1].parse().unwrap_or(i64::MAX), line.to_string()),
        None => (1_000_000_000, line.to_string()),
    }
}

pub fn parse_pages(spec: &str, n: usize) -> Vec<usize> {
    let s: String = spec.chars().filter(|c| *c != ' ').collect();
    if s.is_empty() {
        return (0..n).collect();
    }
    let mut out = Vec::new();
    for part in s.split(',') {
        if part.is_empty() {
            continue;
        }
        let (a, b, dash) = match part.split_once('-') {
            Some((a, b)) => (a, b, true),
            None => (part, "", false),
        };
        let lo: usize = if a.is_empty() { 1 } else { a.parse().unwrap_or(1) };
        let hi: usize = if !b.is_empty() {
            b.parse().unwrap_or(n)
        } else if dash {
            n
        } else {
            lo
        };
        for i in lo.max(1)..=hi.min(n) {
            out.push(i - 1);
        }
    }
    out.sort();
    out.dedup();
    if out.is_empty() { (0..n).collect() } else { out }
}

fn inter(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    (a[2].min(b[2]) - a[0].max(b[0])).max(0.0) * (a[3].min(b[3]) - a[1].max(b[1])).max(0.0)
}
fn area(b: &[f64; 4]) -> f64 {
    (b[2] - b[0]).max(0.0) * (b[3] - b[1]).max(0.0)
}
fn iou(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    let i = inter(a, b);
    let u = (a[2] - a[0]) * (a[3] - a[1]) + (b[2] - b[0]) * (b[3] - b[1]) - i;
    if u > 0.0 { i / u } else { 0.0 }
}

fn norm_num(s: &str) -> String {
    let t = s.replace(',', ".");
    let t = t.trim_start_matches('+');
    match t.parse::<f64>() {
        Ok(f) => py::float_repr(f),
        Err(_) => t.to_string(),
    }
}

/// V0's layout.json -> LayoutResult (development replay for exact A/B).
fn layout_from_json(p: &Value) -> LayoutResult {
    let sz = p["image_size"]
        .as_array()
        .map(|a| (a[0].as_u64().unwrap_or(1) as usize, a[1].as_u64().unwrap_or(1) as usize))
        .unwrap_or((1, 1));
    let boxes = p["boxes"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|b| {
            let bb: Vec<f64> = b["bbox"].as_array().unwrap().iter().map(|v| v.as_f64().unwrap()).collect();
            LayoutBox {
                bbox: [bb[0], bb[1], bb[2], bb[3]],
                label: b["label"].as_str().unwrap_or("").into(),
                raw_label: b["raw_label"].as_str().unwrap_or("").into(),
                position: b["position"].as_u64().unwrap_or(0) as usize,
                confidence: b["confidence"].as_f64().unwrap_or(0.0),
            }
        })
        .collect();
    LayoutResult {
        boxes,
        image_size: sz,
        order_model_used: p["order_model_used"].as_bool().unwrap_or(false),
        raw: vec![],
    }
}

impl<'a> FileJob<'a> {
    pub fn new(rt: &'a Runtime, pdf: &Path, out_dir: &Path, index: usize, total: usize) -> FileJob<'a> {
        let stem = pdf.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let (tx, rx) = channel();
        FileJob {
            rt,
            pdf: pdf.into(),
            out_dir: out_dir.into(),
            images_dir: out_dir.join("images"),
            stem,
            index,
            total,
            doc: None,
            text_pages: vec![],
            pages: vec![],
            report: vec![],
            stats: vec![],
            profile: vec![],
            t0: Instant::now(),
            last_tick: 0.0,
            seq: 0,
            tx,
            rx,
            submitted: 0,
            figs: vec![],
            fig_extra: 0,
        }
    }

    fn emit(&self, kind: &str, mut data: Value) {
        data["file"] = json!(self.pdf.display().to_string());
        self.rt.events.emit(kind, data);
    }

    fn stat(&mut self, k: &str, n: i64) {
        match self.stats.iter_mut().find(|(a, _)| a == k) {
            Some(e) => e.1 += n,
            None => self.stats.push((k.to_string(), n)),
        }
    }

    fn tick(&mut self, name: &str) {
        let now = self.t0.elapsed().as_secs_f64();
        let v = ((now - self.last_tick) * 1000.0).round() / 1000.0;
        match self.profile.iter_mut().find(|(a, _)| a == name) {
            Some(e) => e.1 += v,
            None => self.profile.push((name.into(), v)),
        }
        self.last_tick = now;
    }

    /// The page image surya sees (192 DPI).
    fn ocr_render(&self, page: usize) -> Result<Rgb> {
        self.doc.as_ref().unwrap().render_rgb(page, OCR_DPI / 72.0)
    }

    fn page_lines(&self, page: usize) -> &[Line] {
        self.text_pages.get(page).map(|t| t.lines.as_slice()).unwrap_or(&[])
    }

    fn submit_surya(&mut self, job: impl FnOnce(Option<&LlamaServer>, &AnswerStore) -> SuryaOut + Send + 'static) {
        let Some(svc) = &self.rt.surya else { return };
        let seq = self.seq;
        self.seq += 1;
        let tx = self.tx.clone();
        let store = self.rt.store.clone();
        if svc
            .submit(move |srv| {
                let out = job(srv, &store);
                let _ = tx.send((seq, out));
            })
            .is_ok()
        {
            self.submitted += 1;
        }
    }

    // ---- prepare -------------------------------------------------------------------------
    pub fn prepare(&mut self) -> Result<()> {
        let doc = Document::open(&self.pdf, true)?;
        let n = doc.page_count;
        let selected = parse_pages(&self.rt.cfg.convert.pages, n);
        self.stats = [
            "pages",
            "scanned_pages",
            "tables",
            "equations",
            "ocr_blocks",
            "figures",
            "described",
            "cached",
            "table_values_corrected",
            "table_values_unresolved",
        ]
        .iter()
        .map(|k| (k.to_string(), 0))
        .collect();
        self.stats[0].1 = selected.len() as i64;
        self.emit("file_start", json!({"index": self.index, "total": self.total, "pages": selected.len()}));
        self.emit("stage", json!({"name": "text", "done": 0, "total": n}));
        let tl = extract(&doc)?;
        self.text_pages = tl.pages;
        if self.rt.opts.mask_text_layer {
            for tp in self.text_pages.iter_mut() {
                tp.lines.clear();
            }
        }
        let mode = self.rt.cfg.convert.mode.clone();
        let has_text: Vec<bool> = self
            .text_pages
            .iter()
            .map(|tp| !tp.lines.is_empty() && page_text_ok(&tp.lines.iter().map(|l| l.text()).collect::<Vec<_>>()))
            .collect();
        self.emit("stage", json!({"name": "text", "done": n, "total": n}));
        self.tick("text");
        let replay: Option<Vec<Value>> = match &self.rt.opts.layout_replay {
            Some(dir) => {
                let p = dir.join(&self.stem).join("layout.json");
                let v: Value = serde_json::from_slice(
                    &std::fs::read(&p).with_context(|| format!("layout replay {}", p.display()))?,
                )?;
                Some(v["pages"].as_array().cloned().unwrap_or_default())
            }
            None => None,
        };
        let batch = self.rt.cfg.convert.layout_batch.max(1);
        let mut tracker = SectionTracker::default();
        let rare = rare_font_families(&self.text_pages, 0.01);
        std::fs::create_dir_all(&self.images_dir)?;
        self.emit("stage", json!({"name": "layout", "done": 0, "total": selected.len()}));
        let workers = match self.rt.cfg.scheduler.layout_workers {
            0 => crate::layout::default_workers(),
            w => w,
        };
        self.doc = Some(doc);
        let mut layout_secs = 0.0;
        for (bi, idx) in selected.chunks(batch).enumerate() {
            let t0 = Instant::now();
            let layouts: Vec<LayoutResult> = match &replay {
                Some(pages) => idx
                    .iter()
                    .map(|&i| {
                        pages
                            .iter()
                            .find(|p| p["page"].as_u64() == Some(i as u64))
                            .map(layout_from_json)
                            .ok_or_else(|| anyhow!("page {} missing in layout replay", i + 1))
                    })
                    .collect::<Result<_>>()?,
                None => {
                    let doc = self.doc.as_ref().unwrap();
                    let imgs: Vec<Rgb> =
                        idx.iter().map(|&i| doc.render_rgb(i, LAYOUT_DPI / 72.0)).collect::<Result<_>>()?;
                    let use_order: Vec<bool> = idx.iter().map(|&i| !has_text[i]).collect();
                    self.rt.detector.as_ref().unwrap().detect_many(&imgs, &use_order, workers)?
                }
            };
            layout_secs += t0.elapsed().as_secs_f64();
            self.emit("stage", json!({"name": "layout", "done": bi * batch + idx.len(), "total": selected.len()}));
            for (&i, lay) in idx.iter().zip(layouts) {
                let (w, h) = self.doc.as_ref().unwrap().page_sizes[i];
                let scanned = mode == "full-ocr" || !has_text[i];
                let tp = if has_text[i] { Some(&self.text_pages[i]) } else { None };
                let work = build_page(i, tp, &lay, w, h, self.rt.cfg.convert.keep_headers, &mut tracker, &rare);
                let mut page = work.page;
                page.scanned = scanned;
                if scanned {
                    // a scanned page: surya reads the whole page (box by box as a fallback)
                    self.stat("scanned_pages", 1);
                    self.pages.push(page);
                    self.queue_full_page(i, &lay);
                } else {
                    self.pages.push(page);
                    self.queue_blocks(i, &work.surya)?;
                }
                self.queue_figures(i, work.figures)?;
                self.emit("stage", json!({"name": "build", "done": self.pages.len(), "total": selected.len()}));
            }
        }
        self.profile.push(("layout".into(), (layout_secs * 1000.0).round() / 1000.0));
        self.tick("build");
        Ok(())
    }

    fn queue_blocks(&mut self, page: usize, tasks: &[SuryaTask]) -> Result<()> {
        if tasks.is_empty() {
            return Ok(());
        }
        if self.rt.surya.is_none() {
            for t in tasks {
                let pi = self.pages.len() - 1;
                if let Some(b) = self.pages[pi].blocks.iter_mut().find(|b| b.id == t.block) {
                    fallback_text(b);
                }
            }
            return Ok(());
        }
        let img = self.ocr_render(page)?;
        let prompt = Arc::new(sclient::load_prompt("block"));
        for t in tasks {
            let key = match t.kind {
                "table" => "tables",
                "equation" => "equations",
                _ => "ocr_blocks",
            };
            let crop = crop_pt(&img, t.bbox, OCR_DPI, 1.5);
            let blank = crop.is_one_colour();
            if blank && t.kind == "ocr" {
                // nothing is drawn there, so there is nothing to read: asked to read a blank
                // image, surya answers with a text of its own (docs/LEARNINGS.md, "A page
                // keeps its size"). The block keeps what the PDF gave it, if anything.
                self.stat("ocr_blank_areas_not_read", 1);
                self.rt.trace(json!({"event": "surya", "file": self.stem, "route": "ocr", "page": page + 1,
                    "bbox": t.bbox.map(|v| (v * 10.0).round() / 10.0), "blank_crop": true, "asked": false}));
                let kept_pdf_text = self
                    .pages
                    .last()
                    .and_then(|p| p.blocks.iter().find(|b| b.id == t.block))
                    .is_some_and(|b| py::has_ink(&b.text) || !b.items.is_empty());
                if kept_pdf_text {
                    self.report.push(format!(
                        "page {}, text: the PDF text here looks broken and nothing is drawn at its place \
                         (invisible text); kept the PDF text",
                        page + 1
                    ));
                }
                continue;
            }
            self.stat(key, 1);
            let png = prepare_image(&crop);
            let task = t.clone();
            let prompt = prompt.clone();
            self.submit_surya(move |srv, store| {
                let mut c = sclient::generate(srv, store, &png, &prompt, task.max_tokens);
                let mut mt = task.max_tokens;
                if c.finish_reason.as_deref() == Some("length") && task.max_tokens < 8192 {
                    c = sclient::generate(srv, store, &png, &prompt, 8192);
                    mt = 8192;
                }
                SuryaOut::Block { task, page, c, max_tokens: mt, png, blank }
            });
        }
        Ok(())
    }

    fn queue_full_page(&mut self, page: usize, layout: &LayoutResult) {
        if self.rt.surya.is_none() {
            return;
        }
        let Ok(img) = self.ocr_render(page) else { return };
        let img = Arc::new(img);
        let prompt = sclient::load_prompt("high_accuracy_bbox");
        let block_prompt = sclient::load_prompt("block");
        // box-by-box fallback crops, prepared now (blocks of the built page)
        let pi = self.pages.len() - 1;
        let fallback: Vec<(u64, String, Prepared, u32)> = self.pages[pi]
            .blocks
            .iter()
            .filter(|b| b.kind != "figure")
            .map(|b| {
                let s = OCR_DPI / 72.0;
                let (x0, y0, x1, y1) =
                    ((b.bbox[0] * s) as i64, (b.bbox[1] * s) as i64, (b.bbox[2] * s) as i64, (b.bbox[3] * s) as i64);
                let c = img.crop(
                    (x0 - 4).max(0) as usize,
                    (y0 - 4).max(0) as usize,
                    ((x1 + 4).min(img.width as i64)) as usize,
                    ((y1 + 4).min(img.height as i64)) as usize,
                );
                (b.id, b.kind.clone(), prepare_image(&c), if b.kind != "table" { 2048 } else { 4096 })
            })
            .collect();
        let _ = layout;
        let img2 = img.clone();
        self.submit_surya(move |srv, store| {
            let png = prepare_image(&img2);
            let c = sclient::generate(srv, store, &png, &prompt, FULL_PAGE_MAX_TOKENS);
            let ok = c.error.is_none()
                && c.finish_reason.as_deref() != Some("length")
                && !c.text.is_empty()
                && !crate::llm::detect_repeat_token(&c.text, 0);
            let parsed_ok = ok && {
                let p = html::parse_full_page(&c.text);
                !p.is_empty() && !(p.last().unwrap().truncated && c.finish_reason.as_deref() != Some("stop"))
            };
            let mut fb = Vec::new();
            if !parsed_ok {
                for (id, _, png, mt) in &fallback {
                    fb.push((*id, sclient::generate(srv, store, png, &block_prompt, *mt)));
                }
            }
            SuryaOut::FullPage { page, c, img: img2, fallback: fb }
        });
    }

    fn queue_figures(&mut self, page: usize, figs: Vec<FigureTask>) -> Result<()> {
        if figs.is_empty() {
            return Ok(());
        }
        let dpi = self.rt.cfg.convert.image_dpi as f64;
        let img = self.doc.as_ref().unwrap().render_rgb(page, dpi / 72.0)?;
        for ft in figs {
            let crop = crop_pt(&img, ft.bbox, dpi, 2.0);
            self.queue_one_figure(ft, page, crop, true);
        }
        Ok(())
    }

    fn queue_one_figure(&mut self, mut ft: FigureTask, page: usize, crop: Rgb, allow_ocr: bool) {
        let pi = self.pages.iter().position(|p| p.index == page).unwrap();
        let label =
            self.pages[pi].blocks.iter().find(|b| b.id == ft.block).map(|b| b.label.clone()).unwrap_or_default();
        let kind_name = match label.as_str() {
            "Figure" => "figure",
            "Picture" => "picture",
            _ => "diagram",
        };
        let name = format!("p{:03}-{kind_name}-{:02}.png", page + 1, ft.index);
        if let Some(b) = self.pages[pi].blocks.iter_mut().find(|b| b.id == ft.block) {
            b.image = Some(format!("images/{name}"));
        }
        self.stat("figures", 1);
        let (bw, bh) = (ft.bbox[2] - ft.bbox[0], ft.bbox[3] - ft.bbox[1]);
        let describe =
            !(!self.rt.need_brain || self.rt.opts.describer == "off" || bw < 40.0 || bh < 20.0 || bw * bh < 3000.0);
        let ocr = allow_ocr
            && describe
            && !py::has_ink(&ft.context.figure_text)
            && self.rt.surya.is_some()
            && self.rt.cfg.describe.figure_ocr;
        let png = Arc::new(crop.png(false));
        let _ = std::fs::write(self.images_dir.join(&name), png.as_slice());
        let crop = Arc::new(crop);
        if ocr {
            self.stat("figure_ocr", 1);
            let prep = prepare_image(&crop);
            let fig = ft.block;
            let prompt = sclient::load_prompt("block");
            self.submit_surya(move |srv, store| SuryaOut::FigureOcr {
                fig,
                c: sclient::generate(srv, store, &prep, &prompt, 2048),
            });
        }
        if ft.context.figure_text_source.is_empty() {
            ft.context.figure_text_source = "pdf".into();
        }
        self.figs.push(FigJob { task: ft, page, crop, png, describe, ocr_all: String::new() });
    }

    // ---- applying surya's answers ----------------------------------------------------------
    /// One trace line per surya answer: the route (table, equation, ocr, figure labels, full
    /// page), the region and the start of the answer, so a line of the Markdown can be
    /// followed back to the request that wrote it.
    fn trace_surya(&self, route: &str, page: usize, bbox: Option<[f64; 4]>, c: &Completion, more: Value) {
        let mut rec = json!({
            "event": "surya",
            "file": self.stem,
            "route": route,
            "page": page + 1,
            "bbox": bbox.map(|b| b.map(|v| (v * 10.0).round() / 10.0)),
            "tokens": c.tokens,
            "retries": c.retries,
            "stored": c.cached,
            "finish": c.finish_reason,
            "error": c.error,
            "chars": py::char_len(&c.text),
            "text": c.text.chars().take(160).collect::<String>(),
        });
        if let (Value::Object(a), Value::Object(b)) = (&mut rec, more) {
            a.extend(b);
        }
        self.rt.trace(rec);
    }

    fn block_mut(&mut self, page: usize, id: u64) -> Option<&mut Block> {
        self.pages.iter_mut().find(|p| p.index == page).and_then(|p| p.blocks.iter_mut().find(|b| b.id == id))
    }

    fn apply_block(&mut self, task: SuryaTask, page: usize, c: Completion, max_tokens0: u32, png: Prepared) {
        let page_lines: Vec<Line> = self.page_lines(page).to_vec();
        let where_ = format!("page {}, {}", page + 1, task.kind);
        let mut reports: Vec<String> = Vec::new();
        let mut stats: Vec<(&str, i64)> = Vec::new();
        let mut turn_figure: Option<String> = None;
        {
            let Some(b) = self.block_mut(page, task.block) else { return };
            b.source = "surya".into();
            if task.kind == "table" || task.kind == "equation" {
                b.meta.png = Some(png.rgb.clone());
            }
            if b.label == "Form" {
                b.meta.review = true;
            }
            if c.error.is_some() || !py::has_ink(&c.text) {
                reports.push(format!(
                    "{where_}: surya gave no answer ({}); used the PDF text layer",
                    c.error.clone().or(c.finish_reason.clone()).unwrap_or_else(|| "None".into())
                ));
                if task.kind == "table" {
                    b.meta.review = true;
                }
                fallback_text(b);
            } else {
                if c.finish_reason.as_deref() == Some("length") {
                    reports.push(format!(
                        "{where_}: surya's answer was cut at {} tokens; the block may be incomplete",
                        task.max_tokens
                    ));
                }
                match task.kind {
                    "table" => {
                        let parts = html::parse_tables(&c.text);
                        let n_tables = parts.iter().filter(|p| matches!(p, Part::Table(_))).count();
                        if n_tables == 0 && c.text.contains("<img") {
                            turn_figure = Some(c.text.clone());
                        } else if n_tables == 0 {
                            let alt = html::block_to_markdown(&c.text, None);
                            if py::has_ink(&alt) && b.label == "Form" {
                                b.kind = "paragraph".into();
                                b.text = alt;
                            } else {
                                b.meta.review = true;
                                b.meta.xcheck_report =
                                    Some(vec![format!("{where_}: surya returned no table; used the PDF text layer")]);
                                fallback_text(b);
                            }
                        } else {
                            let mut blocks: Vec<Block> = Vec::new();
                            let mut k = 0;
                            let mut first = true;
                            for p in parts {
                                match p {
                                    Part::Table(t) if first => {
                                        first = false;
                                        k = blocks.len();
                                        b.table = Some(t);
                                        blocks.push(Block::new("__self__", page, b.bbox, &b.label));
                                    }
                                    Part::Table(t) => {
                                        let mut nb = Block::new("table", page, b.bbox, &b.label);
                                        nb.source = "surya".into();
                                        nb.table = Some(t);
                                        blocks.push(nb);
                                    }
                                    Part::Text(s) => {
                                        let mut nb = Block::new("paragraph", page, b.bbox, "Text");
                                        nb.source = "surya".into();
                                        nb.text = s;
                                        blocks.push(nb);
                                    }
                                }
                            }
                            if n_tables > 1 {
                                b.meta.png = None;
                                stats.push(("tables_split_from_one_box", n_tables as i64 - 1));
                            }
                            let tag = format!("page {}, table", page + 1);
                            let lines_empty = b.lines_or_empty().is_empty();
                            if !page_lines.is_empty() {
                                let mut reps: Vec<String> = Vec::new();
                                let bbox = b.bbox;
                                let check = |t: &mut crate::surya::html::Table,
                                             stats: &mut Vec<(&str, i64)>,
                                             reps: &mut Vec<String>| {
                                    let merged = xcheck::merge_centred_cells(t);
                                    let math = xcheck::fix_math_cells(t, &page_lines, &bbox);
                                    let res = xcheck::cross_check(t, &page_lines, &bbox);
                                    stats.push(("table_cells_merged", merged as i64));
                                    stats.push(("table_math_cells_from_pdf", math.len() as i64));
                                    stats.push(("table_values_verified", res.verified as i64));
                                    stats.push(("table_values_corrected", res.corrected.len() as i64));
                                    stats.push((
                                        "table_values_unresolved",
                                        (res.unresolved.len() + res.ambiguous.len()) as i64,
                                    ));
                                    reps.extend(xcheck::report_lines(&tag, &res));
                                };
                                for (i, x) in blocks.iter_mut().enumerate() {
                                    if i == k {
                                        check(b.table.as_mut().unwrap(), &mut stats, &mut reps);
                                    } else if x.kind == "table" {
                                        check(x.table.as_mut().unwrap(), &mut stats, &mut reps);
                                    }
                                }
                                let texts: Vec<String> = blocks
                                    .iter()
                                    .enumerate()
                                    .filter(|(i, x)| *i != k && x.kind != "table")
                                    .map(|(_, x)| x.text.clone())
                                    .collect();
                                let mut tabs: Vec<crate::surya::html::Table> = Vec::new();
                                for (i, x) in blocks.iter().enumerate() {
                                    if i == k {
                                        tabs.push(b.table.clone().unwrap());
                                    } else if x.kind == "table" {
                                        tabs.push(x.table.clone().unwrap());
                                    }
                                }
                                let (placed, missing) = xcheck::fill_missing(&mut tabs, &page_lines, &bbox, &texts);
                                let mut ti = 0;
                                for (i, x) in blocks.iter_mut().enumerate() {
                                    if i == k {
                                        b.table = Some(tabs[ti].clone());
                                        ti += 1;
                                    } else if x.kind == "table" {
                                        x.table = Some(tabs[ti].clone());
                                        ti += 1;
                                    }
                                }
                                stats.push(("table_values_restored", placed.len() as i64));
                                b.meta.xcheck_restored = placed;
                                if !missing.is_empty() {
                                    stats.push(("table_texts_missing", missing.len() as i64));
                                    reps.push(format!(
                                        "{tag}: PDF text not in surya's table: {}",
                                        missing.iter().take(8).map(|m| py::repr_str(m)).collect::<Vec<_>>().join(", ")
                                    ));
                                }
                                for (i, x) in blocks.iter_mut().enumerate() {
                                    let t = if i == k {
                                        b.table.as_ref()
                                    } else if x.kind == "table" {
                                        x.table.as_ref()
                                    } else {
                                        None
                                    };
                                    let Some(t) = t else { continue };
                                    let g = xcheck::geometry_score(t, &page_lines, &bbox);
                                    if i == k {
                                        b.meta.geometry = g
                                    } else {
                                        x.meta.geometry = g
                                    }
                                    if let Some(g) = g
                                        && g < 0.8
                                    {
                                        stats.push(("tables_layout_doubtful", 1));
                                        reps.push(format!(
                                            "{tag}: structure doesn't match the PDF layout well ({} consistent)",
                                            py::fmt_pct0(g)
                                        ));
                                    }
                                }
                                if !reps.is_empty() {
                                    b.meta.review = true;
                                }
                                b.meta.xcheck_report = Some(reps);
                            } else if lines_empty {
                                b.meta.xcheck_report =
                                    Some(vec![format!("{tag}: not verified (scanned page, no text layer)")]);
                                b.meta.review = true;
                            }
                            let more = blocks.split_off(k + 1);
                            blocks.pop(); // the placeholder for b
                            b.meta.before = blocks;
                            b.meta.more = more;
                        }
                    }
                    "equation" => {
                        let parts = html::equation_parts(&c.text);
                        match parts.iter().position(|(k, _)| k == "math") {
                            None => match html::math_latex(&c.text) {
                                Some(l) => b.latex = Some(l),
                                None => {
                                    b.kind = "paragraph".into();
                                    b.text = html::block_to_markdown(&c.text, None);
                                }
                            },
                            Some(first) => {
                                b.latex = Some(parts[first].1.clone());
                                let mk = |kind: &str, v: &str, b: &Block| {
                                    let mut x = Block::new(
                                        kind,
                                        page,
                                        b.bbox,
                                        if kind == "equation" { "Equation" } else { "Text" },
                                    );
                                    x.source = "surya".into();
                                    if kind == "equation" {
                                        x.latex = Some(v.into())
                                    } else {
                                        x.text = v.into()
                                    }
                                    x
                                };
                                b.meta.before = parts[..first].iter().map(|(_, v)| mk("paragraph", v, b)).collect();
                                b.meta.more = parts[first + 1..]
                                    .iter()
                                    .map(|(k, v)| mk(if k == "math" { "equation" } else { "paragraph" }, v, b))
                                    .collect();
                            }
                        }
                    }
                    _ => {
                        if b.kind == "heading" {
                            heading_or_text(b, &T::escape_md(&html::block_plain_text(&c.text)));
                        } else {
                            b.kind = "paragraph".into();
                            b.items.clear();
                            b.text = html::block_to_markdown(&c.text, None);
                        }
                        // the PDF gives no usable text for this block, but it does for the rest
                        // of the page: what surya read is held against the page's own words,
                        // and the words the page does not have go to the report
                        let unknown = xcheck::words_not_on_page(&html::block_words(&c.text), &page_lines);
                        if let Some(line) = xcheck::text_report_line(&format!("page {}, text", page + 1), &unknown) {
                            stats.push(("ocr_words_not_in_pdf", unknown.len() as i64));
                            reports.push(line);
                        }
                    }
                }
            }
        }
        let _ = max_tokens0;
        for (k, n) in stats {
            self.stat(k, n);
        }
        self.report.extend(reports);
        if let Some(h) = turn_figure {
            if !self.table_to_figure(page, task.block, &h) {
                // page not found: treat like "no table"
            }
        }
    }

    /// A drawing the layout called a table, which surya answered with an image description.
    fn table_to_figure(&mut self, page: usize, id: u64, html_text: &str) -> bool {
        let Some(pi) = self.pages.iter().position(|p| p.index == page) else { return false };
        let bb = match self.pages[pi].blocks.iter().find(|b| b.id == id) {
            Some(b) => b.bbox,
            None => return false,
        };
        let covered = self.pages[pi]
            .blocks
            .iter()
            .any(|f| f.id != id && f.kind == "figure" && inter(&f.bbox, &bb) >= 0.6 * area(&f.bbox).min(area(&bb)));
        if covered {
            let b = self.pages[pi].blocks.iter_mut().find(|b| b.id == id).unwrap();
            b.kind = "paragraph".into();
            b.table = None;
            b.text.clear();
            b.meta.review = false;
            return true;
        }
        self.fig_extra += 1;
        let index = 50 + self.fig_extra;
        let (section, lines) = {
            let b = self.pages[pi].blocks.iter_mut().find(|b| b.id == id).unwrap();
            b.kind = "figure".into();
            b.label = "Diagram".into();
            b.table = None;
            b.text.clear();
            b.meta.review = false;
            b.meta.png = None;
            (b.meta.section.clone().unwrap_or_default(), b.lines_or_empty().to_vec())
        };
        let mut ctx = FigureContext {
            label: "Diagram".into(),
            figure_text: lines
                .iter()
                .map(|l| py::strip(&l.text()).to_string())
                .filter(|t| !t.is_empty())
                .collect::<Vec<_>>()
                .join(" | "),
            section,
            figure_text_source: "pdf".into(),
            ..Default::default()
        };
        if ctx.figure_text.is_empty() {
            ctx.figure_text = html::figure_labels(html_text).join(" | ");
            ctx.figure_text_source = "ocr".into();
        }
        attach_captions(&mut self.pages[pi]);
        let b = self.pages[pi].blocks.iter().find(|b| b.id == id).cloned().unwrap();
        ctx.caption = b.caption.clone().unwrap_or_default();
        ctx.nearby_text = nearby_text(&self.pages[pi], &b);
        let ft = FigureTask { block: id, bbox: bb, index, context: ctx };
        let dpi = self.rt.cfg.convert.image_dpi as f64;
        let Ok(img) = self.doc.as_ref().unwrap().render_rgb(page, dpi / 72.0) else { return false };
        let crop = crop_pt(&img, bb, dpi, 2.0);
        self.stat("tables_turned_figures", 1);
        let before = self.figs.len();
        self.queue_one_figure(ft, page, crop, false);
        if self.figs.len() > before {
            self.figs.last_mut().unwrap().ocr_all = html::figure_all_text(html_text);
        }
        true
    }

    fn apply_full_page(&mut self, page: usize, c: Completion, img: Arc<Rgb>, fallback: Vec<(u64, Completion)>) {
        let Some(pi) = self.pages.iter().position(|p| p.index == page) else { return };
        let parsed = if c.error.is_none()
            && c.finish_reason.as_deref() != Some("length")
            && !c.text.is_empty()
            && !crate::llm::detect_repeat_token(&c.text, 0)
        {
            let p = html::parse_full_page(&c.text);
            if p.is_empty() || (p.last().unwrap().truncated && c.finish_reason.as_deref() != Some("stop")) {
                None
            } else {
                Some(p)
            }
        } else {
            None
        };
        let lines: Vec<Line> = self.page_lines(page).to_vec();
        let keep_headers = self.rt.cfg.convert.keep_headers;
        let (pw, ph) = (self.pages[pi].width, self.pages[pi].height);
        let figures: Vec<Block> = self.pages[pi].blocks.iter().filter(|b| b.kind == "figure").cloned().collect();
        let mut blocks: Vec<Block> = Vec::new();
        if let Some(parsed) = parsed {
            let (sx, sy) = (pw / 1000.0, ph / 1000.0);
            let mut used: HashSet<u64> = HashSet::new();
            for item in parsed {
                let lbl = relabel(&item.label);
                let bb = [item.bbox[0] * sx, item.bbox[1] * sy, item.bbox[2] * sx, item.bbox[3] * sy];
                if FIGURE_LABELS.contains(&lbl.as_str()) || lbl == "ChemicalBlock" {
                    if let Some(best) = py::first_max_by(figures.iter(), |f| iou(&f.bbox, &bb))
                        && !used.contains(&best.id)
                        && iou(&best.bbox, &bb) > 0.2
                    {
                        used.insert(best.id);
                        blocks.push(best.clone());
                    }
                    continue;
                }
                if (lbl == "PageHeader" || lbl == "PageFooter") && !keep_headers {
                    continue;
                }
                if lbl == "BlankPage" {
                    continue;
                }
                let mut nb = self.block_from_html(&lbl, bb, &item.html, page, &lines);
                if nb.kind == "table" || nb.kind == "equation" || matches!(lbl.as_str(), "Table" | "Form" | "Equation")
                {
                    let s = OCR_DPI / 72.0;
                    let crop = img.crop(
                        (bb[0] * s) as usize,
                        (bb[1] * s) as usize,
                        (bb[2] * s) as usize,
                        (bb[3] * s) as usize,
                    );
                    nb.meta.png = Some(prepare_image(&crop).rgb);
                    nb.meta.review = (lbl == "Table" || lbl == "Form") && lines.is_empty();
                }
                blocks.push(nb);
            }
            for f in figures {
                if !used.contains(&f.id) {
                    let at = blocks.iter().position(|b| b.bbox[1] > f.bbox[1]).unwrap_or(blocks.len());
                    blocks.insert(at, f);
                }
            }
        } else {
            self.report.push(format!(
                "page {}: full-page OCR failed ({}); used box-by-box OCR",
                page + 1,
                c.error.clone().or(c.finish_reason.clone()).unwrap_or_else(|| "loop/parse".into())
            ));
            let old = self.pages[pi].blocks.clone();
            for b in old {
                if b.kind == "figure" {
                    blocks.push(b);
                    continue;
                }
                let Some((_, cc)) = fallback.iter().find(|(id, _)| *id == b.id) else { continue };
                if cc.error.is_some() || !py::has_ink(&cc.text) {
                    continue;
                }
                blocks.push(self.block_from_html(&b.label, b.bbox, &sclient::strip_end_marker(&cc.text), page, &lines));
            }
        }
        self.pages[pi].blocks = blocks;
    }

    fn block_from_html(&mut self, label: &str, bbox: [f64; 4], h: &str, page: usize, lines: &[Line]) -> Block {
        let mut b = Block::new("paragraph", page, bbox, label);
        b.source = "surya-page".into();
        match label {
            "SectionHeader" => {
                b.size = 1f64.max((bbox[3] - bbox[1]) / (h.matches("<br").count() + 1) as f64 / 1.2);
                heading_or_text(&mut b, &T::escape_md(&html::block_plain_text(h)));
            }
            "Table" | "Form" => match html::parse_table(h) {
                Some(mut t) => {
                    if !lines.is_empty() {
                        let res = xcheck::cross_check(&mut t, lines, &b.bbox);
                        self.stat("table_values_corrected", res.corrected.len() as i64);
                        b.meta.xcheck_report = Some(xcheck::report_lines(&format!("page {}, table", page + 1), &res));
                    } else {
                        b.meta.xcheck_report =
                            Some(vec![format!("page {}, table: not verified (scanned page, no text layer)", page + 1)]);
                    }
                    b.kind = "table".into();
                    b.table = Some(t);
                }
                None => b.text = html::block_to_markdown(h, None),
            },
            "Equation" => match html::math_latex(h) {
                Some(l) => {
                    b.kind = "equation".into();
                    b.latex = Some(l);
                }
                None => b.text = html::block_to_markdown(h, None),
            },
            "Caption" => {
                b.kind = "caption".into();
                b.text = html::block_to_markdown(h, None);
            }
            _ => b.text = html::block_to_markdown(h, None),
        }
        b
    }

    fn apply_figure_ocr(&mut self, fig: u64, c: Completion) {
        let Some(fj) = self.figs.iter_mut().find(|f| f.task.block == fig) else { return };
        if c.error.is_none() {
            let labels = html::figure_labels(&c.text);
            fj.ocr_all = html::figure_all_text(&c.text);
            if !labels.is_empty() {
                fj.task.context.figure_text = labels.join(" | ");
                fj.task.context.figure_text_source = "ocr".into();
            }
        }
    }

    // ---- finalize --------------------------------------------------------------------------
    pub fn finalize(&mut self) -> FileResult {
        let t0 = Instant::now();
        let r = self.finalize_inner();
        let mut res = FileResult { pdf: self.pdf.clone(), ..Default::default() };
        match r {
            Ok((md, images, report)) => {
                res.md = Some(md);
                res.images = images;
                res.report = report;
            }
            Err(e) => res.error = Some(format!("{e:#}")),
        }
        let _ = t0;
        res.seconds = (self.t0.elapsed().as_secs_f64() * 100.0).round() / 100.0;
        res.stats = self.stats.clone();
        res.profile = self.profile.clone();
        self.rt.trace(json!({"event": "file", "file": self.pdf.file_name().map(|s| s.to_string_lossy().to_string()), "seconds": res.seconds,
            "stages": res.profile.iter().map(|(k, v)| (k.clone(), json!(v))).collect::<serde_json::Map<_, _>>(),
            "stats": res.stats.iter().map(|(k, v)| (k.clone(), json!(v))).collect::<serde_json::Map<_, _>>(), "error": res.error}));
        self.doc = None;
        res
    }

    fn finalize_inner(&mut self) -> Result<(PathBuf, usize, Option<PathBuf>)> {
        // 1. surya's answers, in submission order
        let total = self.submitted;
        if total > 0 {
            self.emit("stage", json!({"name": "surya", "done": 0, "total": total}));
        }
        let mut outs: Vec<(usize, SuryaOut)> = Vec::with_capacity(total);
        while outs.len() < total {
            match self.rx.recv() {
                Ok(o) => {
                    outs.push(o);
                    self.emit("stage", json!({"name": "surya", "done": outs.len(), "total": total}));
                }
                Err(_) => break,
            }
        }
        outs.sort_by_key(|o| o.0);
        for (_, o) in outs {
            match o {
                SuryaOut::Block { task, page, c, max_tokens, png, blank } => {
                    let own = self
                        .pages
                        .iter()
                        .find(|p| p.index == page)
                        .and_then(|p| p.blocks.iter().find(|b| b.id == task.block))
                        .map(|b| b.lines_or_empty().len());
                    let in_box: usize =
                        xcheck::lines_from_textlayer(self.page_lines(page), &task.bbox).iter().map(|l| l.0.len()).sum();
                    self.trace_surya(
                        task.kind,
                        page,
                        Some(task.bbox),
                        &c,
                        json!({"pdf_lines": own, "pdf_words_in_box": in_box, "blank_crop": blank}),
                    );
                    self.apply_block(task, page, c, max_tokens, png)
                }
                SuryaOut::FigureOcr { fig, c } => {
                    let at = self.figs.iter().find(|f| f.task.block == fig).map(|f| (f.page, f.task.bbox));
                    if let Some((page, bbox)) = at {
                        self.trace_surya("figure labels", page, Some(bbox), &c, json!({}));
                    }
                    self.apply_figure_ocr(fig, c)
                }
                SuryaOut::FullPage { page, c, img, fallback } => {
                    self.trace_surya("full page", page, None, &c, json!({"fallback_blocks": fallback.len()}));
                    self.apply_full_page(page, c, img, fallback)
                }
            }
        }
        self.expand_parts();
        // 2. Gemma: table reviews, equations, descriptions (V0: refine + describe)
        self.run_brain();
        self.tick("models");
        let r = self.assemble_and_write()?;
        self.tick("write");
        Ok(r)
    }

    fn expand_parts(&mut self) {
        crate::pyre_static!(NW, Re, r"\W+");
        let key = |s: &str| NW.replace_all(&s.to_lowercase(), "");
        for p in self.pages.iter_mut() {
            let mut out: Vec<Block> = Vec::new();
            let snapshot: Vec<(u64, String)> = p.blocks.iter().map(|b| (b.id, b.text.clone())).collect();
            let blocks = std::mem::take(&mut p.blocks);
            for (i, mut b) in blocks.into_iter().enumerate() {
                let before = std::mem::take(&mut b.meta.before);
                let more = std::mem::take(&mut b.meta.more);
                let lo = i.saturating_sub(1);
                let hi = (i + 2).min(snapshot.len());
                let mut neighbours: HashSet<String> = snapshot[lo..hi]
                    .iter()
                    .filter(|(id, t)| *id != b.id && !t.is_empty())
                    .map(|(_, t)| key(t))
                    .collect();
                if let Some(c) = &b.caption
                    && !c.is_empty()
                {
                    neighbours.insert(key(c));
                }
                for x in before {
                    if x.kind == "paragraph" && neighbours.contains(&key(&x.text)) {
                        continue;
                    }
                    out.push(x);
                }
                out.push(b);
                for x in more {
                    if x.kind == "paragraph" && neighbours.contains(&key(&x.text)) {
                        continue;
                    }
                    out.push(x);
                }
            }
            p.blocks = out;
        }
    }

    fn run_brain(&mut self) {
        let store = self.rt.store.clone();
        let think = self.rt.cfg.llm.reasoning;
        let slots = self.rt.cfg.gemma.parallel.max(1);
        let server: Option<Arc<LlamaServer>> =
            self.rt.brain.as_ref().and_then(|s| s.server.clone()).filter(|_| !store.replay_only);
        // the server is started once, by the first request that needs it (D23)
        let ready = std::sync::OnceLock::<Option<Arc<LlamaServer>>>::new();
        let live =
            || ready.get_or_init(|| server.as_ref().and_then(|s| s.wait_ready(None).ok().map(|_| s.clone()))).clone();

        // table reviews and equations (V0 _queue_refine), on Gemma's slots
        struct RJob {
            pi: usize,
            bi: usize,
            review: bool,
            block: Block,
            lines: Arc<Vec<Line>>,
            scanned: bool,
            index: usize,
        }
        let mut jobs: Vec<RJob> = Vec::new();
        if self.rt.brain.is_some() {
            for pi in 0..self.pages.len() {
                let (index, scanned) = (self.pages[pi].index, self.pages[pi].scanned);
                let lines = Arc::new(if !scanned { self.page_lines(index).to_vec() } else { vec![] });
                for bi in 0..self.pages[pi].blocks.len() {
                    let b = &self.pages[pi].blocks[bi];
                    let review = b.meta.review && self.rt.llm_on("tables");
                    let eq = b.kind == "equation"
                        && b.latex.is_none()
                        && b.meta.png.is_some()
                        && self.rt.llm_on("equations");
                    if review || eq {
                        jobs.push(RJob { pi, bi, review, block: b.clone(), lines: lines.clone(), scanned, index });
                    }
                }
            }
        }
        let reviews = jobs.iter().filter(|j| j.review).count() as i64;
        // progress of the LLM pass and of the descriptions (docs/events.md: `refine`, `describe`);
        // events only, nothing here changes what is written
        let (rt, file) = (self.rt, self.pdf.display().to_string());
        let stage = |name: &str, done: usize, total: usize| {
            rt.events.emit("stage", json!({"name": name, "done": done, "total": total, "file": file}));
        };
        let (n_jobs, refined) = (jobs.len(), AtomicUsize::new(0));
        if n_jobs > 0 {
            stage("refine", 0, n_jobs);
        }
        let done: Vec<(RJob, Option<String>)> = par_map(jobs, slots, |mut j| {
            let srv = live();
            let ctx = passes::Ctx { server: srv.as_deref(), store: &store, think, slots };
            let line = if j.review {
                let where_ = format!("page {}, table", j.index + 1);
                passes::review_table(&ctx, &mut j.block, &j.lines, j.scanned || j.lines.is_empty(), &where_)
            } else {
                passes::equation(&ctx, &mut j.block);
                None
            };
            stage("refine", refined.fetch_add(1, Ordering::Relaxed) + 1, n_jobs);
            (j, line)
        });
        let mut fixed = 0;
        for (mut j, line) in done {
            if let Some(line) = line {
                if line.ends_with("checked against the PDF)") || line.contains("scanned page, values could not") {
                    j.block.meta.llm_fixed = true;
                    fixed += 1;
                }
                self.report.push(line);
            }
            self.pages[j.pi].blocks[j.bi] = j.block;
        }
        if reviews > 0 {
            self.stat("llm_table_reviews", reviews);
        }
        if fixed > 0 {
            self.stat("llm_table_corrections", fixed);
        }

        // descriptions, on Gemma's slots; applied in figure order
        let figs = std::mem::take(&mut self.figs);
        let cfg = self.rt.cfg.gemma.clone();
        let idx: Vec<usize> = (0..figs.len()).filter(|&i| figs[i].describe).collect();
        let describe_one = |i: usize| -> Result<Completion, String> {
            let fj = &figs[i];
            let ctx = &fj.task.context;
            let mut kind = kind_from_text(&ctx.caption, &ctx.figure_text).map(String::from);
            if kind.is_none() && ctx.label == "Picture" {
                kind = Some("photo".into());
            }
            let kind = match kind {
                Some(k) => k,
                None => match brain::classify(live().as_deref(), &store, &fj.crop, &fj.png) {
                    Some(k) => k,
                    None => return Err("no figure kind".to_string()),
                },
            };
            let prompt = build_prompt(&kind, ctx);
            Ok(brain::describe(live().as_deref(), &store, &fj.crop, &fj.png, &prompt, &cfg))
        };
        let (n_figs, figs_done) = (idx.len(), AtomicUsize::new(0));
        if n_figs > 0 {
            stage("describe", 0, n_figs);
        }
        let results: Vec<(usize, Result<Completion, String>)> = par_map(idx, slots, |i| {
            let r = describe_one(i);
            stage("describe", figs_done.fetch_add(1, Ordering::Relaxed) + 1, n_figs);
            (i, r)
        });
        let mut described = 0;
        let mut cached = 0;
        for (i, r) in results {
            let fj = &figs[i];
            let c = match r {
                Ok(c) => c,
                Err(e) => {
                    self.report.push(format!("page {}, {}: no description ({e})", fj.page + 1, self.image_of(fj)));
                    continue;
                }
            };
            if c.error.is_some() || c.text.is_empty() {
                self.report.push(format!(
                    "page {}, {}: no description ({})",
                    fj.page + 1,
                    self.image_of(fj),
                    c.error.clone().or(c.finish_reason.clone()).unwrap_or_else(|| "None".into())
                ));
                continue;
            }
            if c.cached {
                cached += 1;
            }
            described += 1;
            let text = c.text.clone();
            if let Some(b) = self.block_mut(fj.page, fj.task.block) {
                b.description = Some(text.clone());
            }
            self.check_numbers(fj, &text);
        }
        self.stat("described", described);
        self.stat("cached", cached);
        self.figs = figs;
    }

    fn image_of(&self, fj: &FigJob) -> String {
        self.pages
            .iter()
            .find(|p| p.index == fj.page)
            .and_then(|p| p.blocks.iter().find(|b| b.id == fj.task.block))
            .and_then(|b| b.image.clone())
            .unwrap_or_default()
    }

    fn check_numbers(&mut self, fj: &FigJob, text: &str) {
        crate::pyre_static!(NUM, Re, r"(?<![\w.])[-+]?\d+(?:[.,]\d+)*(?![\w.]*\d)");
        crate::pyre_static!(MARKERS, Re, r"(?m)^\s*\d{1,2}[.)]\s");
        let ctx = &fj.task.context;
        if !py::has_ink(&ctx.figure_text) {
            return;
        }
        let source = format!("{} {} {}", ctx.figure_text, ctx.caption, fj.ocr_all);
        let have: HashSet<String> = NUM.find_all(&source).into_iter().map(norm_num).collect();
        let body = MARKERS.replace_all(text, "");
        let mut missing: Vec<String> = NUM
            .find_all(&body)
            .into_iter()
            .filter(|m| !have.contains(&norm_num(m)))
            .map(String::from)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        missing.sort_by(|a, b| (py::char_len(a), a.as_str()).cmp(&(py::char_len(b), b.as_str())));
        // Gemma's grade (bench/score.py): numbers in its descriptions the figure's own text confirms
        let all: HashSet<String> = NUM.find_all(&body).into_iter().map(String::from).collect();
        self.stat("description_numbers", all.len() as i64);
        self.stat("description_numbers_unconfirmed", missing.len() as i64);
        if !missing.is_empty() {
            let w = if ctx.source_is_ocr() {
                "the PDF text or surya's reading of the image"
            } else {
                "the figure's PDF text"
            };
            let line = format!(
                "page {}, {}: please check these description numbers, not found in {w}: {}",
                fj.page + 1,
                self.image_of(fj),
                missing.iter().take(12).cloned().collect::<Vec<_>>().join(", ")
            );
            self.report.push(line);
        }
    }

    fn assemble_and_write(&mut self) -> Result<(PathBuf, usize, Option<PathBuf>)> {
        let keep_headers = self.rt.cfg.convert.keep_headers;
        let dropped = if !keep_headers { cleanup::drop_repeated_margins(&mut self.pages) } else { 0 };
        let mut blocks: Vec<Block> = self.pages.iter_mut().flat_map(|p| std::mem::take(&mut p.blocks)).collect();
        for b in blocks.iter_mut() {
            if (b.kind == "table" && b.table.is_none()) || (b.kind == "equation" && b.latex.is_none()) {
                fallback_text(b);
            }
        }
        for b in &blocks {
            if !b.meta.llm_fixed {
                self.report.extend(b.meta.xcheck_report.clone().unwrap_or_default());
            }
        }
        {
            let mut heads: Vec<&mut Block> =
                blocks.iter_mut().filter(|b| b.kind == "heading" && !b.text.is_empty()).collect();
            headings::assign_levels(&mut heads);
            headings::fix_nesting(&mut heads);
            self.rt.trace(json!({"event": "headings", "file": self.stem, "stage": "font sizes",
                "headings": heads.iter().map(|h| json!([h.page + 1, (h.size * 10.0).round() / 10.0, h.level, h.text.chars().take(80).collect::<String>()])).collect::<Vec<_>>()}));
            if !heads.is_empty() && self.rt.llm_on("headings") {
                let store = self.rt.store.clone();
                let server = self
                    .rt
                    .brain
                    .as_ref()
                    .and_then(|s| s.server.clone())
                    .filter(|_| !store.replay_only)
                    .and_then(|s| s.wait_ready(None).ok().map(|_| s));
                let ctx = passes::Ctx {
                    server: server.as_deref(),
                    store: &store,
                    think: self.rt.cfg.llm.reasoning,
                    slots: self.rt.cfg.gemma.parallel,
                };
                let changes = passes::heading_levels(&ctx, &mut heads);
                let nested = headings::fix_nesting(&mut heads);
                self.rt.trace(json!({"event": "headings", "file": self.stem, "stage": "llm", "changes": changes.len(), "nesting_fixed_after": nested}));
                let n = changes.len() as i64;
                drop(heads);
                self.stat("llm_heading_levels_changed", n);
            }
        }
        blocks = cleanup::merge_split_tables(blocks);
        if self.rt.llm_on("table_merge") {
            let before = blocks.len();
            let store = self.rt.store.clone();
            let server = self
                .rt
                .brain
                .as_ref()
                .and_then(|s| s.server.clone())
                .filter(|_| !store.replay_only)
                .and_then(|s| s.wait_ready(None).ok().map(|_| s));
            let ctx = passes::Ctx {
                server: server.as_deref(),
                store: &store,
                think: self.rt.cfg.llm.reasoning,
                slots: self.rt.cfg.gemma.parallel,
            };
            blocks = passes::merge_tables(&ctx, blocks);
            self.stat("llm_tables_merged", (before - blocks.len()) as i64);
        }
        let before = blocks.len();
        let sizes: HashMap<usize, (f64, f64)> = self.pages.iter().map(|p| (p.index, (p.width, p.height))).collect();
        blocks = cleanup::join_paragraphs(blocks, &sizes);
        self.stat("paragraphs_joined", (before - blocks.len()) as i64);
        let text = crate::render::markdown::render(&blocks, &self.rt.cfg.convert.tables);
        std::fs::create_dir_all(&self.out_dir)?;
        let md_path = self.out_dir.join(format!("{}.md", self.stem));
        let tmp = md_path.with_extension("md.tmp");
        std::fs::write(&tmp, text.as_bytes())?;
        std::fs::rename(&tmp, &md_path)?;
        let images = blocks.iter().filter(|b| b.kind == "figure" && b.image.is_some()).count();
        let rep = self.out_dir.join(format!("{}.sift-report.txt", self.stem));
        let mut report = None;
        if !self.report.is_empty() {
            let mut lines: Vec<String> = self.report.iter().cloned().collect::<HashSet<_>>().into_iter().collect();
            lines.sort_by_key(|l| page_key(l));
            let name = self.pdf.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            let mut out = vec![
                format!("Sift report for {name} ({})", crate::util::now_minutes()),
                "Items Sift could not resolve automatically. Everything else was verified or corrected.".into(),
                String::new(),
            ];
            out.extend(lines);
            std::fs::write(&rep, out.join("\n") + "\n")?;
            report = Some(rep);
        } else if rep.exists() {
            let _ = std::fs::remove_file(&rep);
        }
        self.stat("margin_blocks_dropped", dropped as i64);
        let misses = self.rt.store.misses.lock().unwrap().len();
        if misses > 0 {
            self.rt.trace(json!({"event": "answer_store_misses", "file": self.stem, "count": misses}));
        }
        Ok((md_path, images, report))
    }
}

fn heading_or_text(b: &mut Block, text: &str) {
    match headings::body_text_reason(text) {
        Some("lead-in") => {
            b.kind = "paragraph".into();
            b.text = format!("**{text}**");
        }
        Some(_) => {
            b.kind = "paragraph".into();
            b.text = text.into();
        }
        None => {
            b.kind = "heading".into();
            b.text = text.into();
        }
    }
}

/// surya never answered (or there's no surya): keep the text layer.
pub fn fallback_text(b: &mut Block) {
    let lines = b.lines_or_empty().to_vec();
    if b.kind == "table" {
        b.kind = "paragraph".into();
        b.table = None;
        b.text = lines
            .iter()
            .map(|l| l.text())
            .filter(|t| py::has_ink(t))
            .map(|t| T::escape_md(py::strip(&t)))
            .collect::<Vec<_>>()
            .join("  \n");
    } else if b.kind == "equation" {
        b.kind = "paragraph".into();
        b.text = T::paragraph(&lines);
    }
}

/// Map `f` over `items` on `workers` threads; results in input order.
pub fn par_map<T: Send, R: Send>(items: Vec<T>, workers: usize, f: impl Fn(T) -> R + Sync) -> Vec<R> {
    let n = items.len();
    let queue: Mutex<std::collections::VecDeque<(usize, T)>> = Mutex::new(items.into_iter().enumerate().collect());
    let out: Mutex<Vec<Option<R>>> = Mutex::new((0..n).map(|_| None).collect());
    std::thread::scope(|s| {
        for _ in 0..workers.clamp(1, n.max(1)) {
            s.spawn(|| {
                loop {
                    let next = queue.lock().unwrap().pop_front();
                    let Some((i, t)) = next else { break };
                    let r = f(t);
                    out.lock().unwrap()[i] = Some(r);
                }
            });
        }
    });
    out.into_inner().unwrap().into_iter().map(|r| r.expect("every item mapped")).collect()
}
