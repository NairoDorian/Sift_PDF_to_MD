//! The PDF text layer (D10: the ground truth where it is intact).
//!
//! Two parts, both ported 1:1 so V1 reads exactly the characters, spans and lines V0 read
//! (spike S1, checked against V0's export on the seven reference PDFs):
//! 1. pdftext 0.7.1 (`pdf/chars.py`, `pdf/pages.py`, `postprocessing.py`): characters
//!    from pdfium, de-duplicated words, spans, lines, super/subscripts.
//! 2. V0's `sift/pdf/textlayer.py`: ink boxes, bold/italic like marker, row fragments
//!    back into their line, script checks, script runs inside a span, the document's own
//!    vocabulary for line-end hyphens (D28).

use std::collections::HashMap;

use anyhow::Result;
use serde::Serialize;

use crate::pdf::pdfium::{Document, RawPage};
#[allow(unused_imports)]
use crate::py::ReStart as _ReStart;
use crate::py::{self};

pub type BBox = [f64; 4];

#[derive(Clone, Debug, Serialize)]
pub struct Char {
    pub c: char,
    pub bbox: BBox,
}

#[derive(Clone, Debug, Serialize)]
pub struct Span {
    pub text: String,
    pub bbox: BBox,
    pub font: String,
    pub weight: i64,
    pub bold: bool,
    pub italic: bool,
    pub superscript: bool,
    pub subscript: bool,
    pub chars: Vec<Char>,
    /// pdftext char_start_idx: the PDF's own reading order
    pub pos: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Line {
    pub spans: Vec<Span>,
    pub bbox: BBox,
    /// the document vocabulary (V0's `line.vocab`); None for lines that aren't the PDF's
    #[serde(skip)]
    pub vocab: Option<std::sync::Arc<Vocabulary>>,
}

impl Line {
    pub fn text(&self) -> String {
        self.spans.iter().map(|s| s.text.as_str()).collect()
    }
    pub fn height(&self) -> f64 {
        self.bbox[3] - self.bbox[1]
    }
    pub fn pos(&self) -> i64 {
        self.spans.iter().map(|s| s.pos).min().unwrap_or(0)
    }
    pub fn bold(&self) -> bool {
        let inked: Vec<&Span> = self.spans.iter().filter(|s| py::has_ink(&s.text)).collect();
        !inked.is_empty() && inked.iter().all(|s| s.bold)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct TextPage {
    pub index: usize,
    pub width: f64,
    pub height: f64,
    pub rotation: i32,
    pub lines: Vec<Line>,
}

impl TextPage {
    pub fn text(&self) -> String {
        self.lines.iter().map(|l| l.text()).collect::<Vec<_>>().join("\n")
    }
}

// ---------------------------------------------------------------------------------------
// pdftext
// ---------------------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
struct FontKey {
    name: String,
    flags: i32,
    size: f64,
    weight: i32,
}

struct PdChar {
    code: u32,
    ch: char,
    rot: f64,
    b: BBox,
    font: usize,
    idx: i64,
}

struct PdSpan {
    bbox: BBox,
    text: String, // raw (before postprocess)
    font: usize,
    rotation: f64,
    char_start_idx: i64,
    chars: Vec<(char, BBox)>,
    superscript: bool,
    subscript: bool,
}

struct PdLine {
    spans: Vec<PdSpan>,
    bbox: BBox,
    rotation: f64,
}

fn to_char(code: u32) -> char {
    char::from_u32(code).unwrap_or('\u{FFFD}')
}

fn get_chars(raw: &RawPage, fonts: &mut Vec<FontKey>) -> Vec<PdChar> {
    let [x_start, y_start, x_end, y_end] = raw.bbox;
    let page_width = (x_end - x_start).abs().ceil();
    let page_height = (y_end - y_start).abs().ceil();
    let mut out = Vec::with_capacity(raw.chars.len());
    for (i, c) in raw.chars.iter().enumerate() {
        let key = FontKey { name: c.font_name.clone(), flags: c.font_flags, size: c.font_size, weight: c.font_weight };
        let font = match fonts.iter().position(|f| *f == key) {
            Some(i) => i,
            None => {
                fonts.push(key);
                fonts.len() - 1
            }
        };
        let cx_start = c.l - x_start;
        let cx_end = c.r - x_start;
        let ty_start = page_height - (c.b - y_start);
        let ty_end = page_height - (c.t - y_start);
        let mut x0 = cx_start.min(cx_end);
        let mut y0 = ty_start.min(ty_end);
        let mut x1 = cx_start.max(cx_end);
        let mut y1 = ty_start.max(ty_end);
        if raw.rotation != 0 {
            let (nx0, ny0, nx1, ny1) = match raw.rotation {
                90 => (page_height - y1, x0, page_height - y0, x1),
                180 => (page_width - x1, page_height - y1, page_width - x0, page_height - y0),
                270 => (y0, page_width - x1, y1, page_width - x0),
                _ => (x0, y0, x1, y1),
            };
            x0 = nx0.min(nx1);
            y0 = ny0.min(ny1);
            x1 = nx0.max(nx1);
            y1 = ny0.max(ny1);
        }
        out.push(PdChar {
            code: c.code,
            ch: to_char(c.code),
            rot: c.angle as f64,
            b: [x0, y0, x1, y1],
            font,
            idx: i as i64,
        });
    }
    out
}

fn norm0(x: f64) -> i64 {
    // round(x, 0) as a hashable key (-0.0 == 0.0 in Python)
    let r = py::round_half_even(x);
    if r == 0.0 { 0 } else { r.to_bits() as i64 }
}

fn deduplicate_chars(chars: Vec<PdChar>) -> Vec<PdChar> {
    let n = chars.len();
    if n == 0 {
        return chars;
    }
    let mut starts = Vec::new();
    for j in 0..n {
        if j == 0 {
            starts.push(0);
            continue;
        }
        let p = &chars[j - 1];
        let c = &chars[j];
        if matches!(p.code, 2 | 10 | 32) || c.font != p.font || c.rot != p.rot {
            starts.push(j);
        }
    }
    let mut seen: std::collections::HashSet<(i64, i64, i64, i64, String, u64, usize)> = Default::default();
    let mut keep_word = Vec::with_capacity(starts.len());
    for (w, &s) in starts.iter().enumerate() {
        let e = if w + 1 < starts.len() { starts[w + 1] } else { n };
        let mut b = chars[s].b;
        for c in &chars[s + 1..e] {
            b[0] = b[0].min(c.b[0]);
            b[1] = b[1].min(c.b[1]);
            b[2] = b[2].max(c.b[2]);
            b[3] = b[3].max(c.b[3]);
        }
        let text: String = chars[s..e].iter().map(|c| c.ch).collect();
        let rot = chars[s].rot;
        let key = (
            norm0(b[0]),
            norm0(b[1]),
            norm0(b[2]),
            norm0(b[3]),
            text,
            if rot == 0.0 { 0 } else { rot.to_bits() },
            chars[s].font,
        );
        keep_word.push(seen.insert(key));
    }
    if keep_word.iter().all(|k| *k) {
        return chars;
    }
    let mut out = Vec::with_capacity(n);
    let mut w = 0;
    for (j, c) in chars.into_iter().enumerate() {
        while w + 1 < starts.len() && starts[w + 1] <= j {
            w += 1;
        }
        if keep_word[w] {
            out.push(c);
        }
    }
    out
}

fn get_spans(chars: &[PdChar], sup_h: f64, line_d: f64) -> Vec<PdSpan> {
    let n = chars.len();
    if n == 0 {
        return vec![];
    }
    let mut bounds: Vec<(usize, usize, BBox)> = Vec::new();
    let mut start = 0;
    let mut bb = chars[0].b;
    for j in 1..n {
        let prev_code = chars[j - 1].code;
        let height = bb[3] - bb[1];
        let c = &chars[j];
        let brk = c.font != chars[start].font
            || c.rot != chars[start].rot
            || prev_code == 2
            || prev_code == 10
            || (c.b[1] < bb[1] - height * line_d && c.b[3] < height * sup_h + bb[1] && c.b[0] > bb[2]);
        if brk {
            bounds.push((start, j, bb));
            start = j;
            bb = c.b;
        } else {
            if c.b[0] < bb[0] {
                bb[0] = c.b[0];
            }
            if c.b[1] < bb[1] {
                bb[1] = c.b[1];
            }
            if c.b[2] > bb[2] {
                bb[2] = c.b[2];
            }
            if c.b[3] > bb[3] {
                bb[3] = c.b[3];
            }
        }
    }
    bounds.push((start, n, bb));
    bounds
        .into_iter()
        .map(|(s, e, b)| PdSpan {
            bbox: b,
            text: chars[s..e].iter().map(|c| c.ch).collect(),
            font: chars[s].font,
            rotation: chars[s].rot,
            char_start_idx: chars[s].idx,
            chars: chars[s..e].iter().map(|c| (c.ch, c.b)).collect(),
            superscript: false,
            subscript: false,
        })
        .collect()
}

fn merge_into(a: &mut BBox, b: &BBox) {
    if b[0] < a[0] {
        a[0] = b[0];
    }
    if b[1] < a[1] {
        a[1] = b[1];
    }
    if b[2] > a[2] {
        a[2] = b[2];
    }
    if b[3] > a[3] {
        a[3] = b[3];
    }
}

fn get_lines(spans: Vec<PdSpan>) -> Vec<PdLine> {
    let mut lines: Vec<PdLine> = Vec::new();
    let r45 = 45f64.to_radians();
    let r135 = 135f64.to_radians();
    for span in spans {
        let Some(line) = lines.last_mut() else {
            lines.push(PdLine { bbox: span.bbox, rotation: span.rotation, spans: vec![span] });
            continue;
        };
        let last = &line.spans.last().unwrap().text;
        if last.ends_with('\n') || last.ends_with('\x02') {
            lines.push(PdLine { bbox: span.bbox, rotation: span.rotation, spans: vec![span] });
            continue;
        }
        if span.rotation != line.rotation {
            let two_pi = 2.0 * std::f64::consts::PI;
            let mut d = (span.rotation - line.rotation).abs() % two_pi;
            d = d.min(two_pi - d);
            if r45 <= d && d <= r135 {
                lines.push(PdLine { bbox: span.bbox, rotation: span.rotation, spans: vec![span] });
                continue;
            }
        }
        if span.bbox[1] > line.bbox[3] {
            lines.push(PdLine { bbox: span.bbox, rotation: span.rotation, spans: vec![span] });
            continue;
        }
        let b = span.bbox;
        line.spans.push(span);
        merge_into(&mut line.bbox, &b);
    }
    lines
}

fn top2(v: &[f64]) -> (f64, isize, f64) {
    let (mut m1, mut m2, mut i1) = (f64::NEG_INFINITY, f64::NEG_INFINITY, -1isize);
    for (i, &x) in v.iter().enumerate() {
        if x > m1 {
            m2 = m1;
            m1 = x;
            i1 = i as isize;
        } else if x > m2 {
            m2 = x;
        }
    }
    (m1, i1, m2)
}

fn bottom2(v: &[f64]) -> (f64, isize, f64) {
    let (mut m1, mut m2, mut i1) = (f64::INFINITY, f64::INFINITY, -1isize);
    for (i, &x) in v.iter().enumerate() {
        if x < m1 {
            m2 = m1;
            m1 = x;
            i1 = i as isize;
        } else if x < m2 {
            m2 = x;
        }
    }
    (m1, i1, m2)
}

fn is_math_symbol(s: &str) -> bool {
    let mut it = s.chars();
    matches!((it.next(), it.next()), (Some(c), None) if py::category_name(c) == "Sm")
}

fn assign_scripts(lines: &mut [PdLine], ht: f64, ld: f64) {
    for line in lines.iter_mut() {
        let n = line.spans.len();
        if n < 2 {
            continue;
        }
        let lb = line.bbox;
        let line_height = lb[3] - lb[1];
        if line_height > lb[2] - lb[0] {
            continue;
        }
        let heights: Vec<f64> = line.spans.iter().map(|s| s.bbox[3] - s.bbox[1]).collect();
        let ys: Vec<f64> = line.spans.iter().map(|s| s.bbox[1]).collect();
        let ye: Vec<f64> = line.spans.iter().map(|s| s.bbox[3]).collect();
        let va: Vec<f64> = (0..n).map(|i| ys[i] - heights[i] * ld).collect();
        let vb: Vec<f64> = (0..n).map(|i| ye[i] + heights[i] * ld).collect();
        let (am1, am1i, am2) = top2(&va);
        let (bm1, bm1i, bm2) = bottom2(&vb);
        let max_lh = line_height.max(1.0);
        let last_idx = n - 1;
        let texts: Vec<String> = line.spans.iter().map(|s| s.text.clone()).collect();
        for i in 0..n {
            let is_first = i == 0 || !py::has_ink(&texts[i - 1]);
            let is_last = i == last_idx || !py::has_ink(&texts[i + 1]);
            let h = heights[i];
            let line_full = h / max_lh <= ht;
            let next_full = is_last || h / heights[i + 1].max(1.0) <= ht;
            let prev_full = is_first || h / heights[i - 1].max(1.0) <= ht;
            let above = ys[i] < (if i as isize == am1i { am2 } else { am1 });
            let prev_above = is_first || ys[i] < ys[i - 1];
            let next_above = is_last || ys[i] < ys[i + 1];
            let below = ye[i] > (if i as isize == bm1i { bm2 } else { bm1 });
            let prev_below = is_first || ye[i] > ye[i - 1];
            let next_below = is_last || ye[i] > ye[i + 1];
            let st = py::strip(&texts[i]);
            let ok = (py::char_len(st) == 1 || py::str_isdigit(st)) && (py::str_isalnum(st) || is_math_symbol(st));
            if (prev_full || next_full) && (prev_above || next_above) && above && line_full && ok {
                line.spans[i].superscript = true;
            } else if (prev_full || next_full) && (prev_below || next_below) && below && line_full && ok {
                line.spans[i].subscript = true;
            }
        }
    }
}

fn keep_char(c: char) -> bool {
    !py::category_name(c).starts_with('C') || c == '\x02' || matches!(c, '\n' | '\r' | '\x0c' | '\t' | ' ')
}

/// pdftext postprocess_text + handle_hyphens(keep_hyphens=True)
fn postprocess_text(t: &str) -> String {
    let t = t.replace("\r\n", "\n");
    let mut s = String::with_capacity(t.len());
    for c in t.chars() {
        match c {
            ' ' | '\u{fffe}' | '\u{feff}' | '\u{a0}' => s.push(' '),
            '\n' | '\r' => s.push('\n'),
            '\t' => s.push('\t'),
            'ﬀ' => s.push_str("ff"),
            'ﬃ' => s.push_str("ffi"),
            'ﬄ' => s.push_str("ffl"),
            'ﬁ' => s.push_str("fi"),
            'ﬂ' => s.push_str("fl"),
            'ﬆ' | 'ﬅ' => s.push_str("st"),
            c => s.push(c),
        }
    }
    let s: String = s.chars().filter(|&c| keep_char(c)).collect();
    s.replace('\x02', "-\n")
}

/// pdftext.extraction.dictionary_output(keep_chars=True, flatten_pdf=True,
/// quote_loosebox=False, disable_links=True) for one page, before V0's rules.
struct PdPage {
    width: f64,
    height: f64,
    rotation: i32,
    lines: Vec<PdLine>,
    fonts: Vec<FontKey>,
}

fn pdftext_page(raw: &RawPage) -> PdPage {
    let mut fonts = Vec::new();
    let chars = deduplicate_chars(get_chars(raw, &mut fonts));
    let spans = get_spans(&chars, 0.7, 0.1);
    let mut lines = get_lines(spans);
    assign_scripts(&mut lines, 0.7, 0.1);
    let [l, b, r, t] = raw.bbox;
    let mut width = (r - l).abs().ceil();
    let mut height = (b - t).abs().ceil();
    if raw.rotation == 90 || raw.rotation == 270 {
        std::mem::swap(&mut width, &mut height);
    }
    PdPage { width, height, rotation: raw.rotation, lines, fonts }
}

// ---------------------------------------------------------------------------------------
// V0's rules on top (sift/pdf/textlayer.py)
// ---------------------------------------------------------------------------------------

fn font_flags_to_format(flags: i32) -> (bool, bool, bool) {
    // returns (plain, italic, bold) as marker's PdfProvider.font_flags_to_format
    let names: [(u32, &str); 10] = [
        (1, "FixedPitch"),
        (2, "Serif"),
        (3, "Symbolic"),
        (4, "Script"),
        (6, "Nonsymbolic"),
        (7, "Italic"),
        (17, "AllCap"),
        (18, "SmallCap"),
        (19, "ForceBold"),
        (20, "UseExternAttr"),
    ];
    let mut set: Vec<&str> = names.iter().filter(|(bit, _)| flags & (1 << (bit - 1)) != 0).map(|(_, n)| *n).collect();
    if set.is_empty() {
        set.push("Plain");
    }
    set.sort();
    let is = |v: &[&str]| {
        let mut v = v.to_vec();
        v.sort();
        v == set
    };
    if is(&["Symbolic", "Italic"])
        || is(&["Symbolic", "Italic", "UseExternAttr"])
        || is(&["UseExternAttr"])
        || is(&["Plain"])
    {
        return (true, false, false);
    }
    let italic = set.contains(&"Italic");
    let bold = set.contains(&"ForceBold");
    let plain = set.iter().any(|n| {
        matches!(*n, "FixedPitch" | "Serif" | "Script" | "Nonsymbolic" | "AllCap" | "SmallCap" | "UseExternAttr")
    });
    (plain, italic, bold)
}

pub fn union(boxes: &[BBox]) -> BBox {
    let mut u = boxes[0];
    for b in &boxes[1..] {
        u[0] = u[0].min(b[0]);
        u[1] = u[1].min(b[1]);
        u[2] = u[2].max(b[2]);
        u[3] = u[3].max(b[3]);
    }
    u
}

fn normalize_spaces(t: &str) -> String {
    let mut s = String::with_capacity(t.len());
    for c in t.chars() {
        match c {
            '\u{2003}' | '\u{2002}' | '\u{a0}' | '\u{3000}' => s.push(' '),
            '\u{200b}' => {}
            c => s.push(c),
        }
    }
    s
}

fn v0_page(index: usize, pg: PdPage) -> TextPage {
    let mut lines: Vec<Line> = Vec::new();
    for pl in pg.lines {
        let mut spans: Vec<Span> = Vec::new();
        for sp in pl.spans {
            let text_pp = postprocess_text(&sp.text);
            if text_pp.is_empty() {
                continue;
            }
            let f = &pg.fonts[sp.font];
            let (_, it, bo) = font_flags_to_format(f.flags);
            let low = f.name.to_lowercase();
            let name_bold = !f.name.is_empty() && low.contains("bold");
            let name_italic = !f.name.is_empty() && low.contains("ital");
            let weight = f.weight as i64;
            let mut text = normalize_spaces(&text_pp).replace(['\r', '\n'], "");
            if text.is_empty() {
                continue;
            }
            if sp.superscript || sp.subscript {
                text = py::strip(&text).to_string();
            }
            let chars: Vec<Char> = sp.chars.iter().map(|(c, b)| Char { c: *c, bbox: *b }).collect();
            let ink: Vec<BBox> = chars.iter().filter(|c| !py::is_space(c.c)).map(|c| c.bbox).collect();
            let bbox = if ink.is_empty() { sp.bbox } else { union(&ink) };
            spans.push(Span {
                text,
                bbox,
                font: if f.name.is_empty() { "Unknown".into() } else { f.name.clone() },
                weight,
                bold: bo || name_bold || weight >= 600,
                italic: it || name_italic,
                superscript: sp.superscript,
                subscript: sp.subscript,
                chars,
                pos: sp.char_start_idx,
            });
        }
        let inked: Vec<BBox> = spans.iter().filter(|s| py::has_ink(&s.text)).map(|s| s.bbox).collect();
        if !inked.is_empty() {
            let bb = union(&inked);
            lines.push(Line { spans, bbox: bb, vocab: None });
        }
    }
    let mut lines = merge_row_fragments(lines);
    for l in lines.iter_mut() {
        check_scripts(l);
        split_char_scripts(l);
    }
    TextPage { index, width: pg.width, height: pg.height, rotation: pg.rotation, lines }
}

fn merge_row_fragments(lines: Vec<Line>) -> Vec<Line> {
    let mut out: Vec<Line> = Vec::new();
    for mut l in lines {
        if let Some(p) = out.last_mut() {
            let gap = l.bbox[0] - p.bbox[2];
            let ov = p.bbox[3].min(l.bbox[3]) - p.bbox[1].max(l.bbox[1]);
            if (-2.0..6.0).contains(&gap) && ov >= 0.5 * p.height().min(l.height()) {
                let pc = (p.bbox[1] + p.bbox[3]) / 2.0;
                let lc = (l.bbox[1] + l.bbox[3]) / 2.0;
                if l.height() < 0.8 * p.height() && py::char_len(py::strip(&l.text())) <= 4 {
                    let ph = p.height();
                    for s in l.spans.iter_mut() {
                        if py::has_ink(&s.text) {
                            s.superscript = lc < pc - 0.1 * ph;
                            s.subscript = lc > pc + 0.1 * ph;
                            s.text = py::strip(&s.text).to_string();
                        }
                    }
                }
                let lb = l.bbox;
                p.spans.extend(l.spans);
                p.bbox = union(&[p.bbox, lb]);
                continue;
            }
        }
        out.push(l);
    }
    out
}

fn split_char_scripts(line: &mut Line) {
    let ink: Vec<&Char> = line
        .spans
        .iter()
        .filter(|s| !(s.superscript || s.subscript))
        .flat_map(|s| s.chars.iter())
        .filter(|c| !py::is_space(c.c))
        .collect();
    if ink.len() < 4 {
        return;
    }
    let mut hs: Vec<f64> = ink.iter().map(|c| c.bbox[3] - c.bbox[1]).collect();
    hs.sort_by(|a, b| py::fcmp(*a, *b));
    let hh = hs[hs.len() / 2];
    if hh <= 0.0 {
        return;
    }
    let normal: Vec<&&Char> = ink.iter().filter(|c| c.bbox[3] - c.bbox[1] >= 0.9 * hh).collect();
    if (normal.len() as f64) < 0.5 * ink.len() as f64 {
        return;
    }
    let mut bottoms: Vec<f64> = normal.iter().map(|c| c.bbox[3]).collect();
    bottoms.sort_by(|a, b| py::fcmp(*a, *b));
    let bb = bottoms[bottoms.len() / 2];
    let nspans = line.spans.len();
    let mut out: Vec<Span> = Vec::new();
    let mut changed = false;
    for s in line.spans.iter() {
        let chars: Vec<Char> = s.chars.iter().filter(|c| c.c != '\r' && c.c != '\n').cloned().collect();
        if s.superscript || s.subscript || chars.is_empty() || chars.len() != py::char_len(&s.text) {
            out.push(s.clone());
            continue;
        }
        let mut kinds: Vec<i32> = chars
            .iter()
            .map(|c| {
                let h = c.bbox[3] - c.bbox[1];
                if py::is_space(c.c) || h >= 0.85 * hh {
                    0
                } else if c.bbox[3] < bb - 0.12 * hh {
                    1
                } else if c.bbox[3] > bb + 0.08 * hh {
                    -1
                } else {
                    0
                }
            })
            .collect();
        for i in 0..chars.len() {
            if py::is_space(chars[i].c) {
                let prev = (0..i).rev().find(|&j| !py::is_space(chars[j].c)).map(|j| kinds[j]).unwrap_or(0);
                let nxt = (i + 1..chars.len()).find(|&j| !py::is_space(chars[j].c)).map(|j| kinds[j]).unwrap_or(0);
                if prev == nxt && nxt != 0 {
                    kinds[i] = prev;
                }
            }
        }
        let mut runs: Vec<(usize, usize, i32)> = Vec::new();
        let mut start = 0;
        for i in 1..=chars.len() {
            if i == chars.len() || kinds[i] != kinds[start] {
                runs.push((start, i, kinds[start]));
                start = i;
            }
        }
        let tchars: Vec<char> = s.text.chars().collect();
        let sub = |a: usize, b: usize| -> String { tchars[a.min(tchars.len())..b.min(tchars.len())].iter().collect() };
        let ok: Vec<(usize, usize, i32)> = runs
            .iter()
            .cloned()
            .filter(|&(a, b, k)| {
                let t = sub(a, b);
                k != 0 && t.chars().any(py::is_alnum) && py::char_len(py::strip(&t)) <= 10
            })
            .collect();
        if ok.is_empty() || (runs.len() == 1 && nspans == 1) {
            out.push(s.clone());
            continue;
        }
        let mut merged: Vec<(usize, usize, i32)> = Vec::new();
        for &(a, b, k) in &runs {
            let k = if ok.iter().any(|&(oa, ob, _)| oa == a && ob == b) { k } else { 0 };
            if let Some(last) = merged.last_mut()
                && last.2 == k
                && k == 0
            {
                last.1 = b;
                continue;
            }
            merged.push((a, b, k));
        }
        for (a, b, k) in merged {
            let cs: Vec<Char> = chars[a..b].to_vec();
            let inked: Vec<BBox> = cs.iter().filter(|c| !py::is_space(c.c)).map(|c| c.bbox).collect();
            out.push(Span {
                text: sub(a, b),
                bbox: if inked.is_empty() { s.bbox } else { union(&inked) },
                font: s.font.clone(),
                weight: s.weight,
                bold: s.bold,
                italic: s.italic,
                superscript: k == 1,
                subscript: k == -1,
                chars: cs,
                pos: s.pos + a as i64,
            });
        }
        changed = true;
    }
    if changed {
        line.spans = out;
    }
}

fn check_scripts(line: &mut Line) {
    let idx: Vec<usize> = (0..line.spans.len()).filter(|&i| py::has_ink(&line.spans[i].text)).collect();
    for (k, &i) in idx.iter().enumerate() {
        let s = &line.spans[i];
        if !(s.superscript || s.subscript) {
            continue;
        }
        let mut refs: Vec<usize> = Vec::new();
        if k > 0 {
            refs.push(idx[k - 1]);
        }
        if k + 1 < idx.len() {
            refs.push(idx[k + 1]);
        }
        let refs: Vec<&Span> =
            refs.into_iter().map(|j| &line.spans[j]).filter(|n| !(n.superscript || n.subscript)).collect();
        if refs.is_empty() {
            continue;
        }
        let rh = refs.iter().map(|n| n.bbox[3] - n.bbox[1]).fold(f64::NEG_INFINITY, f64::max);
        let rc = refs.iter().map(|n| (n.bbox[1] + n.bbox[3]) / 2.0).sum::<f64>() / refs.len() as f64;
        let h = s.bbox[3] - s.bbox[1];
        let c = (s.bbox[1] + s.bbox[3]) / 2.0;
        let small = h < 0.85 * rh;
        let (sup, sub) = (s.superscript, s.subscript);
        let s = &mut line.spans[i];
        if sup && !(small && c < rc - 0.15 * rh) {
            s.superscript = false;
        }
        if sub && !(small && c > rc + 0.15 * rh) {
            s.subscript = false;
        }
    }
}

// ---------------------------------------------------------------------------------------
// Vocabulary (D28)
// ---------------------------------------------------------------------------------------

/// The document's own words, to decide what a hyphen at a line end is (MASTER_PLAN D28).
#[derive(Default, Debug)]
pub struct Vocabulary {
    pub words: HashMap<String, usize>,
    pub hyphenated: HashMap<String, usize>,
    pub prefixes: HashMap<String, usize>,
    pub tokens: HashMap<String, usize>,
}

crate::pyre_static!(WORD, Re, r"[A-Za-z]+(?:-[A-Za-z]+)*");
crate::pyre_static!(BREAK, Re, r"(\S*?)([A-Za-z]+)-\s*$");
crate::pyre_static!(TOKEN, Re, r"[A-Za-z0-9_]+");
crate::pyre_static!(TAIL_ALPHA, Re, r"[A-Za-z]+$");

fn inc(m: &mut HashMap<String, usize>, k: String) {
    *m.entry(k).or_insert(0) += 1;
}

impl Vocabulary {
    pub fn new(pages: &[TextPage]) -> Vocabulary {
        let mut v = Vocabulary::default();
        for tp in pages {
            let mut skip_first = false;
            let texts: Vec<String> = tp.lines.iter().map(|l| l.text()).collect();
            for (i, text) in texts.iter().enumerate() {
                for t in TOKEN.find_all(text) {
                    inc(&mut v.tokens, t.to_lowercase());
                }
                let toks = WORD.find_all(text);
                let nxt = if i + 1 < texts.len() { py::lstrip(&texts[i + 1]).to_string() } else { String::new() };
                let first: String = nxt.chars().take(1).collect();
                let brk = BREAK.is_match(text) && py::str_islower(&first);
                for (k, t) in toks.iter().enumerate() {
                    if (k == 0 && skip_first) || (brk && k == toks.len() - 1) {
                        continue;
                    }
                    if t.contains('-') {
                        let low = t.to_lowercase();
                        inc(&mut v.hyphenated, low.clone());
                        let parts: Vec<&str> = low.split('-').collect();
                        for p in &parts[..parts.len() - 1] {
                            inc(&mut v.prefixes, p.to_string());
                        }
                    }
                    for p in t.split('-') {
                        inc(&mut v.words, p.to_lowercase());
                    }
                }
                skip_first = brk;
            }
        }
        v
    }

    fn n(m: &HashMap<String, usize>, k: &str) -> usize {
        m.get(k).copied().unwrap_or(0)
    }

    pub fn word(&self, k: &str) -> usize {
        Self::n(&self.words, k)
    }
    pub fn token(&self, k: &str) -> usize {
        Self::n(&self.tokens, k)
    }

    /// `before`: the line-end token without its hyphen; `second`: the next line's first word.
    pub fn keep_hyphen(&self, before: &str, second: &str) -> bool {
        let Some(first) = TAIL_ALPHA.search(before) else { return false };
        if second.is_empty() {
            return false;
        }
        let a = first.get(0).unwrap().as_str();
        let b = second;
        if Self::n(&self.words, &format!("{a}{b}").to_lowercase()) > 0 {
            return false;
        }
        if Self::n(&self.hyphenated, &format!("{a}-{b}").to_lowercase()) > 0 || before.contains('-') {
            return true;
        }
        if py::str_isupper(a) {
            return true;
        }
        if py::char_len(b) <= 2 && Self::n(&self.words, &b.to_lowercase()) > 0 {
            return true;
        }
        if Self::n(&self.prefixes, &a.to_lowercase()) > 0 {
            return true;
        }
        py::char_len(a) >= 3
            && py::char_len(b) >= 3
            && Self::n(&self.words, &a.to_lowercase()) > 0
            && Self::n(&self.words, &b.to_lowercase()) > 0
    }
}

/// The whole document's text layer, as V0's `textlayer.extract(path)`.
pub struct TextLayer {
    pub pages: Vec<TextPage>,
    pub vocab: std::sync::Arc<Vocabulary>,
}

pub fn extract(doc: &Document) -> Result<TextLayer> {
    let mut pages = Vec::with_capacity(doc.page_count);
    for i in 0..doc.page_count {
        let raw = doc.raw_chars(i, false)?;
        pages.push(v0_page(i, pdftext_page(&raw)));
    }
    let vocab = std::sync::Arc::new(Vocabulary::new(&pages));
    for tp in pages.iter_mut() {
        for l in tp.lines.iter_mut() {
            l.vocab = Some(vocab.clone());
        }
    }
    Ok(TextLayer { pages, vocab })
}
