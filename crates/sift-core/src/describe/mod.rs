//! Figure descriptions by Gemma (D13, D26): context, kind, prompts (V0
//! `sift/describe/prompts.py`); the request itself is in `brain::describe`.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use crate::py::{self};

pub fn kind_name(kind: &str) -> &'static str {
    match kind {
        "block_diagram" => "block diagram or schematic",
        "pinout" => "pinout or package drawing",
        "graph" => "graph or chart",
        "timing" => "timing diagram",
        _ => "photo or rendering",
    }
}
pub const KINDS: [&str; 5] = ["block_diagram", "pinout", "graph", "timing", "photo"];

crate::pyre_static!(
    PINOUT,
    Re,
    r"\bpin[- ]?(out|assignment|map|definition|configuration|description|diagram)|\bpackage\b|footprint|\bdimension|\boutline\b|\bQFN\b|\bLGA\b|\bBGA\b|top view|bottom view|land pattern",
    "i"
);
crate::pyre_static!(TIMING, Re, r"timing|waveform|\bt(su|h|hd|pd|r|f|rise|fall|setup|hold)\b", "i");
crate::pyre_static!(
    GRAPH,
    Re,
    r"\bvs\.?\b|versus|\bcurve|\bgraph|\bplot|characteristic|\bover (temperature|frequency|voltage|supply)|sensitivity|\bspectrum",
    "i"
);
crate::pyre_static!(
    BLOCK,
    Re,
    r"block diagram|schematic|circuit|application|architecture|reference design|topology|connection|interface|\bblock\b|flow ?chart|state (machine|diagram)",
    "i"
);

/// Kind from caption keywords, then figure-text keywords; None when they don't say.
pub fn kind_from_text(caption: &str, figure_text: &str) -> Option<&'static str> {
    for text in [caption, figure_text] {
        if PINOUT.is_match(text) {
            return Some("pinout");
        }
        if TIMING.is_match(text) && !GRAPH.is_match(text) {
            return Some("timing");
        }
        if GRAPH.is_match(text) {
            return Some("graph");
        }
        if BLOCK.is_match(text) {
            return Some("block_diagram");
        }
    }
    None
}

#[derive(Clone, Debug, Default)]
pub struct FigureContext {
    pub label: String,
    pub caption: String,
    pub section: String,
    pub figure_text: String,
    pub nearby_text: String,
    /// "pdf": the PDF text layer; "ocr": read from the image by surya
    pub figure_text_source: String,
}

impl FigureContext {
    pub fn source_is_ocr(&self) -> bool {
        self.figure_text_source == "ocr"
    }
}

static PROMPTS: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(Default::default);

pub fn load_prompt(name: &str) -> String {
    let mut m = PROMPTS.lock().unwrap();
    if let Some(s) = m.get(name) {
        return s.clone();
    }
    let p = crate::paths::prompts_dir().join("describe").join(format!("{name}.txt"));
    let s =
        py::strip(&crate::paths::read_text(&p).unwrap_or_else(|e| panic!("prompt {}: {e}", p.display()))).to_string();
    m.insert(name.to_string(), s.clone());
    s
}

fn clip(s: &str, n: usize) -> String {
    let s = py::squash_ws(s);
    if py::char_len(&s) <= n {
        return s;
    }
    let cut: String = s.chars().take(n - 3).collect();
    let head = match cut.rfind(' ') {
        Some(i) => cut[..i].to_string(),
        None => cut,
    };
    format!("{head}...")
}

fn or_none(s: String) -> String {
    if s.is_empty() { "none".into() } else { s }
}

/// Python str.format on the _context template.
fn fill(template: &str, vals: &[(&str, String)]) -> String {
    let mut out = template.to_string();
    for (k, v) in vals {
        out = out.replace(&format!("{{{k}}}"), v);
    }
    out
}

pub fn build_prompt(kind: &str, ctx: &FigureContext) -> String {
    let context = fill(
        &load_prompt("_context"),
        &[
            ("kind_name", kind_name(kind).to_string()),
            ("section", or_none(clip(&ctx.section, 200))),
            ("caption", or_none(clip(&ctx.caption, 300))),
            ("figure_text", or_none(clip(&ctx.figure_text, 1500))),
            (
                "figure_text_source",
                if ctx.source_is_ocr() {
                    "read from this image by a specialised OCR model; use these spellings".into()
                } else {
                    "exact, from the PDF text layer".into()
                },
            ),
            ("nearby_text", or_none(clip(&ctx.nearby_text, 600))),
        ],
    );
    format!("{}\n\n{context}\n\n{}", load_prompt(kind), load_prompt("_rules"))
}

pub const CLASSIFY_PROMPT: &str = "What kind of figure is this? Answer with exactly one of: block diagram, schematic, pinout, package drawing, graph, chart, timing diagram, photo, other.";

pub fn classify_answer(ans: &str) -> &'static str {
    let a = py::strip(ans).to_lowercase();
    for (key, kind) in [
        ("timing", "timing"),
        ("pinout", "pinout"),
        ("package", "pinout"),
        ("schematic", "block_diagram"),
        ("block", "block_diagram"),
        ("graph", "graph"),
        ("chart", "graph"),
        ("photo", "photo"),
    ] {
        if a.contains(key) {
            return kind;
        }
    }
    "block_diagram"
}
