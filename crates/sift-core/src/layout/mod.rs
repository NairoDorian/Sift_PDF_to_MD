//! Layout: boxes, labels and reading order per page (V0 `sift/layout/detector.py`).
//!
//! Same steps as surya's fast layout: detect at 448 px, threshold 0.4, reading order from the
//! order head on pages without a text layer (raster order otherwise), then merge same-label
//! boxes contained >= 90 % in a larger one.

pub mod nn;
pub mod order;
pub mod rfdetr;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::imaging::Rgb;

pub const CONTAINMENT_THRESHOLD: f64 = 0.9;

/// surya's layout labels -> canonical names (surya/layout/label.py).
pub fn relabel(raw: &str) -> String {
    match raw {
        "Equation-Block" => "Equation",
        "List-Group" => "ListGroup",
        "Page-Header" => "PageHeader",
        "Page-Footer" => "PageFooter",
        "Image" => "Picture",
        "Section-Header" => "SectionHeader",
        "Complex-Block" => "Figure",
        "Code-Block" => "Code",
        "Table-Of-Contents" => "TableOfContents",
        "Chemical-Block" => "ChemicalBlock",
        "Blank-Page" => "BlankPage",
        other => other,
    }
    .to_string()
}

pub const TEXT_LABELS: [&str; 8] =
    ["Text", "SectionHeader", "PageHeader", "PageFooter", "Caption", "Footnote", "Code", "Bibliography"];
pub const FIGURE_LABELS: [&str; 3] = ["Figure", "Picture", "Diagram"];

#[derive(Clone, Debug, Serialize)]
pub struct LayoutBox {
    /// x0 y0 x1 y1 in pixels of the image given to the detector
    pub bbox: [f64; 4],
    pub label: String,
    pub raw_label: String,
    pub position: usize,
    pub confidence: f64,
}

impl LayoutBox {
    pub fn area(&self) -> f64 {
        (self.bbox[2] - self.bbox[0]).max(0.0) * (self.bbox[3] - self.bbox[1]).max(0.0)
    }
    pub fn intersection_area(&self, o: &LayoutBox) -> f64 {
        let x0 = self.bbox[0].max(o.bbox[0]);
        let y0 = self.bbox[1].max(o.bbox[1]);
        let x1 = self.bbox[2].min(o.bbox[2]);
        let y1 = self.bbox[3].min(o.bbox[3]);
        (x1 - x0).max(0.0) * (y1 - y0).max(0.0)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RawDet {
    pub label: String,
    pub score: f64,
    pub bbox: [f64; 4],
    pub position: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct LayoutResult {
    pub boxes: Vec<LayoutBox>,
    pub image_size: (usize, usize),
    pub order_model_used: bool,
    pub raw: Vec<RawDet>,
}

/// surya's _merge_contained_boxes.
pub fn merge_contained_boxes(boxes: Vec<LayoutBox>, threshold: f64) -> Vec<LayoutBox> {
    if threshold > 1.0 || boxes.len() < 2 {
        return boxes;
    }
    let mut sorted = boxes;
    // sorted(..., key=area, reverse=True) is stable: equal areas keep their order
    sorted.sort_by(|a, b| b.area().partial_cmp(&a.area()).unwrap_or(std::cmp::Ordering::Equal));
    let mut survivors: Vec<LayoutBox> = Vec::new();
    for b in sorted {
        let mut absorbed = false;
        for s in survivors.iter_mut() {
            if s.label != b.label {
                continue;
            }
            if b.area() > 0.0 && s.intersection_area(&b) / b.area() >= threshold {
                s.bbox = [
                    s.bbox[0].min(b.bbox[0]),
                    s.bbox[1].min(b.bbox[1]),
                    s.bbox[2].max(b.bbox[2]),
                    s.bbox[3].max(b.bbox[3]),
                ];
                absorbed = true;
                break;
            }
        }
        if !absorbed {
            survivors.push(b);
        }
    }
    survivors.sort_by_key(|b| b.position);
    for (rank, b) in survivors.iter_mut().enumerate() {
        b.position = rank;
    }
    survivors
}

#[derive(Deserialize)]
struct Cat {
    id: i64,
    name: String,
}
#[derive(Deserialize)]
struct LayoutConfig {
    categories: Vec<Cat>,
    weights: Option<String>,
}

pub struct LayoutDetector {
    pub model: rfdetr::RfDetr,
    pub order: Option<order::OrderModel>,
    id2label: Vec<String>,
    pub dir: PathBuf,
}

impl LayoutDetector {
    /// `dir`: the surya_layout2 folder (config.json, rfdetr_layout.pth, order/order_ar.pt).
    pub fn load(dir: &Path) -> Result<LayoutDetector> {
        let cfg: LayoutConfig =
            serde_json::from_slice(&std::fs::read(dir.join("config.json")).context("layout config.json")?)?;
        let mut cats = cfg.categories;
        cats.sort_by_key(|c| c.id);
        let weights = dir.join(cfg.weights.unwrap_or_else(|| "rfdetr_layout.pth".into()));
        let model = rfdetr::RfDetr::load(&weights)?;
        let op = dir.join("order").join("order_ar.pt");
        let order = if op.exists() { Some(order::OrderModel::load(&op)?) } else { None };
        Ok(LayoutDetector {
            model,
            order,
            id2label: cats.into_iter().map(|c| c.name).collect(),
            dir: dir.to_path_buf(),
        })
    }

    fn label_name(&self, id: usize) -> String {
        self.id2label.get(id).cloned().unwrap_or_else(|| id.to_string())
    }

    /// One page. `use_order`: the order head (pages without a text layer), else raster order.
    pub fn detect(&self, img: &Rgb, use_order: bool) -> Result<LayoutResult> {
        let res = self.model.detect(img, rfdetr::CONFIDENCE_THRESHOLD)?;
        let names: Vec<String> = res.dets.iter().map(|d| self.label_name(d.label)).collect();
        let boxes32: Vec<[f32; 4]> = res.dets.iter().map(|d| d.bbox).collect();
        let (positions, used) = match (&self.order, use_order && !res.dets.is_empty()) {
            (Some(o), true) => {
                let labels: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
                (o.order_page(&res.features, &boxes32, &labels, img.width as f32, img.height as f32)?, true)
            }
            _ => (order::raster_positions(&boxes32), false),
        };
        let raw: Vec<RawDet> = res
            .dets
            .iter()
            .zip(&names)
            .zip(&positions)
            .map(|((d, n), &p)| RawDet {
                label: n.clone(),
                score: d.score as f64,
                bbox: [d.bbox[0] as f64, d.bbox[1] as f64, d.bbox[2] as f64, d.bbox[3] as f64],
                position: p,
            })
            .collect();
        let boxes: Vec<LayoutBox> = raw
            .iter()
            .map(|r| LayoutBox {
                bbox: r.bbox,
                label: relabel(&r.label),
                raw_label: r.label.clone(),
                position: r.position,
                confidence: r.score,
            })
            .collect();
        let mut boxes = merge_contained_boxes(boxes, CONTAINMENT_THRESHOLD);
        boxes.sort_by_key(|b| b.position);
        Ok(LayoutResult { boxes, image_size: (img.width, img.height), order_model_used: used, raw })
    }
}

impl LayoutDetector {
    /// Several pages at once, one page per worker thread (each single-threaded: the matrix
    /// code scales far better across pages than inside one page, measured 2026-11-06).
    /// Results in input order.
    pub fn detect_many(&self, imgs: &[Rgb], use_order: &[bool], workers: usize) -> Result<Vec<LayoutResult>> {
        let n = imgs.len();
        // on the GPU the backbone queues on one stream: two pages in flight keep it busy while
        // the CPU runs the other's decoder
        let workers = if self.model.dev.is_cpu() { workers } else { workers.min(2) };
        let workers = workers.clamp(1, n.max(1));
        let next = std::sync::atomic::AtomicUsize::new(0);
        let slots: Vec<std::sync::Mutex<Option<Result<LayoutResult>>>> =
            (0..n).map(|_| std::sync::Mutex::new(None)).collect();
        std::thread::scope(|s| {
            for _ in 0..workers {
                s.spawn(|| {
                    loop {
                        let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if i >= n {
                            break;
                        }
                        let r = self.detect(&imgs[i], use_order[i]);
                        *slots[i].lock().unwrap() = Some(r);
                    }
                });
            }
        });
        slots.into_iter().map(|m| m.into_inner().unwrap().unwrap()).collect()
    }
}

/// Worker count for page-parallel layout: the physical-ish core count, leaving the rest of the
/// machine (llama-server's CPU side, pdfium, the window) some room.
pub fn default_workers() -> usize {
    std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).saturating_sub(2).clamp(1, 16)
}

/// The matrix code must be single-threaded per page (candle reads RAYON_NUM_THREADS).
/// Call once at start-up, before any layout work.
pub fn init_threads() {
    if std::env::var_os("RAYON_NUM_THREADS").is_none() {
        // SAFETY: called at start-up, before any other thread exists.
        unsafe { std::env::set_var("RAYON_NUM_THREADS", "1") };
    }
}
