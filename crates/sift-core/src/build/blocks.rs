//! Sift's block model (docs/blocks.schema.json; V0 `sift/build/blocks.py`).
//!
//! A document is a list of pages; a page is a list of blocks in reading order. V0 kept
//! per-block working data in a `meta` dict; here it is typed (`Meta`). Python compared blocks
//! by identity (`is`): every block has a unique `id` for that.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

use crate::imaging::Rgb;
use crate::pdf::textlayer::Line;
use crate::surya::html::Table;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn new_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ListItem {
    pub text: String,
    pub level: i64,
    pub ordered: bool,
    pub marker: String,
}

impl ListItem {
    pub fn plain(text: String) -> ListItem {
        ListItem { text, ..Default::default() }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct XCheck {
    pub merged: usize,
    pub math: Vec<(String, String)>,
    pub corrected: Vec<(String, String)>,
    pub unresolved: Vec<String>,
    pub ambiguous: Vec<(String, Vec<String>)>,
}

#[derive(Clone, Debug, Default)]
pub struct Meta {
    pub needs_ocr: bool,
    pub section: Option<String>,
    /// LLM pass: review this table (marker's LLMTableProcessor)
    pub review: bool,
    /// the crop sent to surya (scale_to_fit, the LLM pass reuses it)
    pub png: Option<Arc<Rgb>>,
    pub caption_block: Option<u64>,
    /// blocks surya's answer added before / after this one (_expand_parts)
    pub before: Vec<Block>,
    pub more: Vec<Block>,
    pub xcheck_report: Option<Vec<String>>,
    pub xcheck: Option<XCheck>,
    pub xcheck_restored: Vec<String>,
    pub geometry: Option<f64>,
    pub llm_fixed: bool,
    pub merged_pages: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub id: u64,
    pub kind: String,
    pub page: usize,
    pub bbox: [f64; 4],
    pub label: String,
    pub text: String,
    pub level: i64,
    pub items: Vec<ListItem>,
    pub table: Option<Table>,
    pub latex: Option<String>,
    pub image: Option<String>,
    pub caption: Option<String>,
    pub description: Option<String>,
    pub source: String,
    pub size: f64,
    /// meta["lines"]: the text-layer lines of the block (None: made from a model's answer)
    pub lines: Option<Vec<Line>>,
    pub meta: Meta,
}

impl Block {
    pub fn new(kind: &str, page: usize, bbox: [f64; 4], label: &str) -> Block {
        Block {
            id: new_id(),
            kind: kind.into(),
            page,
            bbox,
            label: label.into(),
            text: String::new(),
            level: 0,
            items: vec![],
            table: None,
            latex: None,
            image: None,
            caption: None,
            description: None,
            source: "text".into(),
            size: 0.0,
            lines: None,
            meta: Meta::default(),
        }
    }

    /// `b.meta.get("lines") or []`
    pub fn lines_or_empty(&self) -> &[Line] {
        self.lines.as_deref().unwrap_or(&[])
    }
}

#[derive(Clone, Debug)]
pub struct Page {
    pub index: usize,
    pub width: f64,
    pub height: f64,
    pub blocks: Vec<Block>,
    pub scanned: bool,
}
