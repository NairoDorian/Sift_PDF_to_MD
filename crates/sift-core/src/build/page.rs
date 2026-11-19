//! One page: layout boxes + text-layer lines -> blocks, plus the model work they need
//! (V0 `sift/build/page.py`; CURRENT_ARCHITECTURE.md step 4, "the routing").

use std::collections::{HashMap, HashSet};

use crate::build::blocks::{Block, Page};
use crate::build::headings::{self as H, SectionTracker};
use crate::build::text as T;
use crate::describe::FigureContext;
use crate::layout::{FIGURE_LABELS, LayoutResult};
use crate::pdf::quality::block_text_bad;
use crate::pdf::textlayer::{Line, TextPage};
use crate::py::{self};

const CONTAINERS: [&str; 7] = ["Figure", "Picture", "Diagram", "Table", "Form", "Equation", "ChemicalBlock"];
const TEXTUAL: [&str; 10] = [
    "Text",
    "SectionHeader",
    "Caption",
    "Footnote",
    "ListGroup",
    "Code",
    "Bibliography",
    "TableOfContents",
    "PageHeader",
    "PageFooter",
];
const STREAM_EXEMPT: [&str; 7] = CONTAINERS;

fn priority(label: &str) -> i64 {
    match label {
        "Table" | "Form" | "Equation" | "Figure" | "Picture" | "Diagram" | "ChemicalBlock" => 5,
        "SectionHeader" | "Caption" | "PageHeader" | "PageFooter" => 4,
        "Footnote" | "ListGroup" | "TableOfContents" | "Code" => 3,
        "Bibliography" => 2,
        _ => 1,
    }
}

crate::pyre_static!(
    CAPTION_START,
    ReStart,
    r"\s*(?i:fig(?:ure)?\.?|table|equation|eq\.)\s*[A-Z]?\d+(?:[.-]\d+)*(?:\s*[:.\-–]|\s+[A-Z])"
);
crate::pyre_static!(
    CAPTION_LABEL,
    ReStart,
    r"\s*(?i:fig(?:ure)?\.?|table|equation|eq\.)\s*[A-Z]?\d+(?:[.-]\d+)*\s*[:.]?\s*$"
);
crate::pyre_static!(FIG_REF, Re, r"\b(fig(?:ure)?\.?|table)\s*(\d+(?:[.-]\d+)?)", "i");
crate::pyre_static!(BULLET_GLYPH, ReStart, r"\s*[•●▪■◦○·‣⁃∙►▸➢✓]");

#[derive(Clone, Debug)]
pub struct SuryaTask {
    pub block: u64,
    pub kind: &'static str, // table | equation | ocr
    pub bbox: [f64; 4],
    pub max_tokens: u32,
}

#[derive(Clone, Debug)]
pub struct FigureTask {
    pub block: u64,
    pub bbox: [f64; 4],
    pub index: usize,
    pub context: FigureContext,
}

pub struct PageWork {
    pub page: Page,
    pub surya: Vec<SuryaTask>,
    pub figures: Vec<FigureTask>,
}

#[derive(Clone)]
struct Bx {
    id: usize,
    bbox: [f64; 4],
    label: String,
    position: usize,
    lines: Vec<Line>,
    orphan: bool,
}

impl Bx {
    fn area(&self) -> f64 {
        area(&self.bbox)
    }
}

fn inter(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    (a[2].min(b[2]) - a[0].max(b[0])).max(0.0) * (a[3].min(b[3]) - a[1].max(b[1])).max(0.0)
}
fn area(b: &[f64; 4]) -> f64 {
    (b[2] - b[0]).max(0.0) * (b[3] - b[1]).max(0.0)
}

fn line_share(line: &Line, b: &[f64; 4]) -> f64 {
    let [x0, y0, x1, y1] = line.bbox;
    let w = (x1 - x0).max(1e-6);
    let cy = (y0 + y1) / 2.0;
    let half = ((y1 - y0) * 0.25).max(0.5);
    let ox = (x1.min(b[2]) - x0.max(b[0])).max(0.0) / w;
    let oy = ((cy + half).min(b[3]) - (cy - half).max(b[1])).max(0.0) / (2.0 * half);
    ox * oy
}

fn split_straddling(lines: Vec<Line>, boxes: &[Bx]) -> Vec<Line> {
    let textual: Vec<&Bx> = boxes.iter().filter(|b| TEXTUAL.contains(&b.label.as_str())).collect();
    if textual.len() < 2 {
        return lines;
    }
    let mut out = Vec::new();
    for l in lines {
        out.extend(split_line_by_boxes(l, &textual));
    }
    out
}

fn split_line_by_boxes(l: Line, boxes: &[&Bx]) -> Vec<Line> {
    let (x0, x1) = (l.bbox[0], l.bbox[2]);
    let w = (x1 - x0).max(1e-6);
    let mut hits: Vec<(&Bx, f64)> = Vec::new();
    for b in boxes {
        let oy = line_share(&l, &[-1e9, b.bbox[1], 1e9, b.bbox[3]]);
        let ox = (x1.min(b.bbox[2]) - x0.max(b.bbox[0])).max(0.0) / w;
        if oy >= 0.5 && ox > 0.0 {
            hits.push((b, ox));
        }
    }
    if hits.len() < 2 || hits.iter().map(|h| h.1).fold(f64::NEG_INFINITY, f64::max) >= 0.9 {
        return vec![l];
    }
    hits.sort_by(|a, b| py::fcmp(a.0.bbox[0], b.0.bbox[0]));
    for pair in hits.windows(2) {
        let (a, b) = (pair[0].0, pair[1].0);
        if a.bbox[2] > b.bbox[0] + 0.25 * (a.bbox[2] - a.bbox[0]).min(b.bbox[2] - b.bbox[0]) {
            continue;
        }
        let lo = a.bbox[2].min(b.bbox[0]) - l.height();
        let hi = a.bbox[2].max(b.bbox[0]) + l.height();
        let mut ink: Vec<(f64, f64)> = l
            .spans
            .iter()
            .flat_map(|s| s.chars.iter())
            .filter(|c| !py::is_space(c.c))
            .map(|c| (c.bbox[0], c.bbox[2]))
            .collect();
        ink.sort_by(|p, q| p.partial_cmp(q).unwrap_or(std::cmp::Ordering::Equal));
        let gaps: Vec<(f64, f64)> = ink
            .windows(2)
            .map(|w| (w[1].0 - w[0].1, (w[0].1 + w[1].0) / 2.0))
            .filter(|(_, m)| lo <= *m && *m <= hi)
            .collect();
        if gaps.is_empty() {
            continue;
        }
        let (g, x) = gaps.iter().cloned().fold((f64::NEG_INFINITY, f64::NEG_INFINITY), |acc, v| {
            if v.partial_cmp(&acc) == Some(std::cmp::Ordering::Greater) { v } else { acc }
        });
        if g < 0.25 * l.height() {
            continue;
        }
        let Some((left, right)) = T::split_line_at(&l, x) else { continue };
        let mut v = split_line_by_boxes(left, boxes);
        v.extend(split_line_by_boxes(right, boxes));
        return v;
    }
    vec![l]
}

fn orphan_groups(lines: Vec<Line>) -> Vec<Vec<Line>> {
    let mut lines = lines;
    lines.sort_by(|a, b| {
        (py::round_half_even(a.bbox[1]), a.bbox[0])
            .partial_cmp(&(py::round_half_even(b.bbox[1]), b.bbox[0]))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut groups: Vec<Vec<Line>> = Vec::new();
    for l in lines {
        if let Some(g) = groups.last_mut() {
            let last = g.last().unwrap();
            let gap = l.bbox[1] - last.bbox[3];
            let gx1 = g.iter().map(|x| x.bbox[2]).fold(f64::NEG_INFINITY, f64::max);
            let gx0 = g.iter().map(|x| x.bbox[0]).fold(f64::INFINITY, f64::min);
            let overlap_x = l.bbox[2].min(gx1) - l.bbox[0].max(gx0);
            if gap < 3f64.max(0.9 * last.height()) && overlap_x > 0.0 {
                g.push(l);
                continue;
            }
        }
        groups.push(vec![l]);
    }
    groups
}

fn union_lines(lines: &[Line]) -> [f64; 4] {
    let b: Vec<[f64; 4]> = lines.iter().map(|l| l.bbox).collect();
    crate::pdf::textlayer::union(&b)
}

pub fn token_budget_text(lines: &[Line], floor: u32) -> u32 {
    let chars: usize = lines.iter().map(|l| py::char_len(&l.text())).sum();
    (8192f64).min((floor as f64).max(chars as f64 * 0.6 + 6.0 * lines.len() as f64 + 100.0)) as u32
}

pub fn token_budget_table(lines: &[Line]) -> u32 {
    let chars: usize = lines.iter().map(|l| py::char_len(&l.text())).sum();
    let est = (chars as f64 * 0.7 + 8.0 * lines.len() as f64) as i64;
    (8192i64).min(2048i64.max(est) + 100) as u32
}

#[allow(clippy::too_many_arguments)]
pub fn build_page(
    index: usize,
    text_page: Option<&TextPage>,
    layout: &LayoutResult,
    page_w: f64,
    page_h: f64,
    keep_headers: bool,
    tracker: &mut SectionTracker,
    rare_fonts: &HashSet<String>,
) -> PageWork {
    crate::pyre_static!(FURNITURE, ReFull, r"[\d\s/|.-]*(page\s*\d+.*)?", "i");
    let scale = page_w / layout.image_size.0 as f64;
    let lines: Vec<Line> = text_page.map(|t| t.lines.clone()).unwrap_or_default();
    let mut page = Page { index, width: page_w, height: page_h, blocks: vec![], scanned: false };
    let mut work_surya: Vec<SuryaTask> = Vec::new();
    let mut work_figs: Vec<FigureTask> = Vec::new();

    let mut boxes: Vec<Bx> = layout
        .boxes
        .iter()
        .filter(|b| b.label != "BlankPage")
        .enumerate()
        .map(|(i, b)| Bx {
            id: i,
            bbox: b.bbox.map(|v| v * scale),
            label: b.label.clone(),
            position: b.position,
            lines: vec![],
            orphan: false,
        })
        .collect();
    let containers: Vec<[f64; 4]> =
        boxes.iter().filter(|b| CONTAINERS.contains(&b.label.as_str())).map(|b| b.bbox).collect();
    boxes.retain(|b| {
        CONTAINERS.contains(&b.label.as_str())
            || !containers.iter().any(|c| b.area() != 0.0 && inter(&b.bbox, c) / b.area() >= 0.8)
    });

    let lines = split_straddling(lines, &boxes);
    let mut orphans: Vec<Line> = Vec::new();
    for l in &lines {
        let mut best: Option<usize> = None;
        let mut best_key: Option<(bool, i64, f64, f64)> = None;
        for (bi, b) in boxes.iter().enumerate() {
            let frac = line_share(l, &b.bbox);
            if frac <= 0.0 {
                continue;
            }
            let key = (frac >= 0.5, priority(&b.label), frac, -b.area());
            if best_key.is_none_or(|bk| key.partial_cmp(&bk) == Some(std::cmp::Ordering::Greater)) {
                best = Some(bi);
                best_key = Some(key);
            }
        }
        match (best, best_key) {
            (Some(bi), Some(k)) if k.2 >= 0.4 => boxes[bi].lines.push(l.clone()),
            _ => orphans.push(l.clone()),
        }
    }

    if !lines.is_empty() {
        let filled: Vec<(usize, [f64; 4])> =
            boxes.iter().filter(|b| !b.lines.is_empty()).map(|b| (b.id, b.bbox)).collect();
        boxes.retain(|b| {
            let covered = b.area() != 0.0
                && filled.iter().filter(|(id, _)| *id != b.id).map(|(_, f)| inter(&b.bbox, f)).sum::<f64>() / b.area()
                    >= 0.5;
            !b.lines.is_empty() || CONTAINERS.contains(&b.label.as_str()) || !covered
        });
    }

    let mut next_id = boxes.len() + 1000;
    for g in orphan_groups(orphans) {
        let bb = union_lines(&g);
        let txt = py::strip(&g.iter().map(|x| x.text()).collect::<Vec<_>>().join(" ")).to_string();
        let in_margin = bb[3] < page_h * 0.07 || bb[1] > page_h * 0.93;
        if !keep_headers && in_margin && (FURNITURE.is(&txt) || py::char_len(&txt) < 4) {
            continue;
        }
        let n = boxes.len();
        boxes.push(Bx { id: next_id, bbox: bb, label: "Text".into(), position: 10_000 + n, lines: g, orphan: true });
        next_id += 1;
    }

    for b in boxes.iter_mut() {
        b.lines.sort_by(|a, c| {
            (a.pos(), a.bbox[1], a.bbox[0])
                .partial_cmp(&(c.pos(), c.bbox[1], c.bbox[0]))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    let has_text = !lines.is_empty();
    if has_text {
        let mut raster: Vec<Bx> = boxes.clone();
        raster.sort_by(|a, b| {
            (py::round_n(a.bbox[1], 1), a.bbox[0])
                .partial_cmp(&(py::round_n(b.bbox[1], 1), b.bbox[0]))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let streamed: Vec<([f64; 4], i64)> = boxes
            .iter()
            .filter(|b| {
                !b.lines.is_empty()
                    && !STREAM_EXEMPT.contains(&b.label.as_str())
                    && b.label != "PageHeader"
                    && b.label != "PageFooter"
                    && !b.orphan
            })
            .map(|b| (b.bbox, b.lines.iter().map(|l| l.pos()).min().unwrap()))
            .collect();
        let mut order: Vec<(i64, usize, Bx)> = Vec::new();
        let mut last_pos: i64 = -1;
        for (i, b) in raster.into_iter().enumerate() {
            let pos = b.lines.iter().map(|l| l.pos()).min();
            if let Some(p) = pos
                && !STREAM_EXEMPT.contains(&b.label.as_str())
            {
                last_pos = p;
                order.push((p, i, b));
                continue;
            }
            let above: Vec<i64> = streamed
                .iter()
                .filter(|(x, _)| x[3] <= b.bbox[1] + 2.0 && x[2].min(b.bbox[2]) - x[0].max(b.bbox[0]) > 0.0)
                .map(|(_, p)| *p)
                .collect();
            let k = above.iter().cloned().max().unwrap_or(last_pos);
            order.push((k, i, b));
        }
        order.sort_by_key(|t| (t.0, t.1));
        boxes = order.into_iter().map(|t| t.2).collect();
    } else {
        boxes.sort_by_key(|b| b.position);
    }

    let mut fig_no = 0;
    let figure_boxes: Vec<[f64; 4]> =
        boxes.iter().filter(|f| FIGURE_LABELS.contains(&f.label.as_str())).map(|f| f.bbox).collect();
    for b in &boxes {
        let Some(mut blk) = make_block(b, index, keep_headers) else { continue };
        if blk.kind == "heading" && !rare_fonts.is_empty() && drawing_label(b, &figure_boxes, rare_fonts) {
            blk.kind = "paragraph".into();
            blk.text = T::paragraph(&b.lines);
        }
        if blk.kind == "heading" {
            tracker.update(&blk.text);
        }
        let lbl = b.label.as_str();
        if lbl == "Table" || lbl == "Form" {
            blk.meta.section = Some(tracker.current());
            work_surya.push(SuryaTask {
                block: blk.id,
                kind: "table",
                bbox: b.bbox,
                max_tokens: token_budget_table(&b.lines),
            });
        } else if lbl == "Equation" {
            work_surya.push(SuryaTask {
                block: blk.id,
                kind: "equation",
                bbox: b.bbox,
                max_tokens: token_budget_text(&b.lines, 512),
            });
        } else if FIGURE_LABELS.contains(&lbl) || lbl == "ChemicalBlock" {
            fig_no += 1;
            let ctx = FigureContext {
                label: if FIGURE_LABELS.contains(&lbl) { lbl.to_string() } else { "Diagram".into() },
                figure_text: b
                    .lines
                    .iter()
                    .map(|l| py::strip(&l.text()).to_string())
                    .filter(|t| !t.is_empty())
                    .collect::<Vec<_>>()
                    .join(" | "),
                section: tracker.current(),
                ..Default::default()
            };
            work_figs.push(FigureTask { block: blk.id, bbox: b.bbox, index: fig_no, context: ctx });
        } else if blk.meta.needs_ocr {
            let mt = if b.lines.is_empty() { 2048 } else { token_budget_text(&b.lines, 256) };
            work_surya.push(SuryaTask { block: blk.id, kind: "ocr", bbox: b.bbox, max_tokens: mt });
        }
        page.blocks.push(blk);
    }

    attach_captions(&mut page);
    for ft in work_figs.iter_mut() {
        let blk = page.blocks.iter().find(|x| x.id == ft.block);
        ft.context.caption = blk.and_then(|b| b.caption.clone()).unwrap_or_default();
        ft.context.nearby_text = blk.map(|b| nearby_text(&page, b)).unwrap_or_default();
    }
    PageWork { page, surya: work_surya, figures: work_figs }
}

fn make_block(b: &Bx, page_index: usize, keep_headers: bool) -> Option<Block> {
    crate::pyre_static!(BULLET_STRIP, Re, r"^[•●▪■◦○·‣⁃∙►▸➢✓]\s*");
    let lines = &b.lines;
    let mut lbl = b.label.clone();
    let mut blk = Block::new("paragraph", page_index, b.bbox, &lbl);
    if !lines.is_empty() {
        let hs: Vec<f64> = lines.iter().map(|l| l.height()).collect();
        blk.size = py::median(&hs);
    }
    blk.lines = Some(lines.clone());
    match lbl.as_str() {
        "PageHeader" | "PageFooter" => {
            if !keep_headers {
                return None;
            }
            blk.text = T::paragraph(lines);
            return if blk.text.is_empty() { None } else { Some(blk) };
        }
        "Table" | "Form" => {
            blk.kind = "table".into();
            return Some(blk);
        }
        "Equation" => {
            blk.kind = "equation".into();
            return Some(blk);
        }
        l if FIGURE_LABELS.contains(&l) || l == "ChemicalBlock" => {
            blk.kind = "figure".into();
            return Some(blk);
        }
        _ => {}
    }
    let raw = lines.iter().map(|l| l.text()).collect::<Vec<_>>().join(" ");
    if lines.is_empty() || block_text_bad(&raw) {
        blk.meta.needs_ocr = true;
    }
    if lbl == "SectionHeader" {
        let p = T::plain(lines);
        if CAPTION_START.is(&p) || CAPTION_LABEL.is(&p) {
            lbl = "Caption".into();
            blk.label = lbl.clone();
        }
    }
    match lbl.as_str() {
        "SectionHeader" => {
            blk.kind = "heading".into();
            blk.text = T::heading_text(lines);
            let why = H::body_text_reason(&blk.text);
            if py::char_len(&blk.text) > 250 || lines.len() > 4 || matches!(why, Some("sentence") | Some("note")) {
                blk.kind = "paragraph".into();
                blk.text = T::paragraph(lines);
            } else if why == Some("lead-in") {
                blk.kind = "paragraph".into();
                blk.text = format!("**{}**", T::heading_text(lines));
            } else if !lines.is_empty() && bullet_start(&lines[0]) && list_like(&blk.text) {
                blk.kind = "list".into();
                blk.text = String::new();
                blk.items = T::list_items(lines);
            } else if !lines.is_empty() && bullet_start(&lines[0]) {
                blk.text = BULLET_STRIP.0.replacen(&blk.text, 1, "").into_owned();
            }
        }
        "ListGroup" => {
            blk.kind = "list".into();
            blk.items = T::list_items(lines);
        }
        "Caption" => {
            blk.kind = "caption".into();
            blk.text = T::paragraph(lines);
        }
        "Footnote" => {
            blk.kind = "footnote".into();
            blk.text = T::paragraph(lines);
            if numbered_lines(lines) {
                blk.kind = "list".into();
                blk.text = String::new();
                blk.items = T::list_items(lines);
            }
        }
        "Code" => {
            blk.kind = "code".into();
            blk.text = T::code_text(lines);
        }
        "TableOfContents" => {
            blk.kind = "toc".into();
            blk.text = T::toc_lines(lines);
        }
        _ => {
            blk.kind = "paragraph".into();
            blk.text = T::paragraph(lines);
            if lines.len() >= 2
                && (lines.iter().filter(|l| T::is_bullet(&l.text())).count() >= 2 || numbered_lines(lines))
            {
                blk.kind = "list".into();
                blk.items = T::list_items(lines);
            }
        }
    }
    if !blk.meta.needs_ocr && blk.text.is_empty() && blk.items.is_empty() {
        return None;
    }
    Some(blk)
}

pub fn font_family(name: &str) -> String {
    let a = name.rsplit('+').next().unwrap_or("");
    let b = a.split(',').next().unwrap_or("");
    b.split('-').next().unwrap_or("").to_string()
}

pub fn rare_font_families(pages: &[TextPage], share: f64) -> HashSet<String> {
    let mut cnt: HashMap<String, usize> = HashMap::new();
    for tp in pages {
        for l in &tp.lines {
            for s in &l.spans {
                if py::has_ink(&s.text) {
                    *cnt.entry(font_family(&s.font)).or_insert(0) += py::char_len(py::strip(&s.text));
                }
            }
        }
    }
    let tot: usize = cnt.values().sum();
    cnt.into_iter().filter(|(_, c)| tot > 0 && (*c as f64) / (tot as f64) < share).map(|(f, _)| f).collect()
}

fn drawing_label(b: &Bx, figures: &[[f64; 4]], rare: &HashSet<String>) -> bool {
    if b.lines.is_empty() || H::numbering_depth(&T::plain(&b.lines)).is_some() {
        return false;
    }
    let mut chars: Vec<(String, usize)> = Vec::new(); // insertion order, as a dict
    for l in &b.lines {
        for s in &l.spans {
            if py::has_ink(&s.text) {
                let f = font_family(&s.font);
                let n = py::char_len(py::strip(&s.text));
                match chars.iter_mut().find(|(k, _)| *k == f) {
                    Some((_, v)) => *v += n,
                    None => chars.push((f, n)),
                }
            }
        }
    }
    let Some(top) = py::first_max_by(chars.iter(), |(_, c)| *c) else { return false };
    if !rare.contains(&top.0) {
        return false;
    }
    let near = 30.0;
    figures.iter().any(|f| {
        if *f == b.bbox && FIGURE_LABELS.contains(&b.label.as_str()) {
            return false;
        }
        let dx = (f[0] - b.bbox[2]).max(b.bbox[0] - f[2]).max(0.0);
        let dy = (f[1] - b.bbox[3]).max(b.bbox[1] - f[3]).max(0.0);
        dx <= near && dy <= near
    })
}

fn list_like(text: &str) -> bool {
    crate::pyre_static!(MD, Re, r"[*_\\]");
    crate::pyre_static!(LEAD, Re, r"^[\W_]+");
    let t0 = MD.replace_all(text, "");
    let t1 = LEAD.0.replacen(&t0, 1, "").into_owned();
    let t = py::strip(&t1);
    !t.is_empty() && (t.contains(':') || t.ends_with('.') || t.ends_with(';') || t.ends_with(','))
}

pub fn numbered_lines(lines: &[Line]) -> bool {
    if lines.len() < 2 || !T::is_numbered_line(&lines[0].text()) {
        return false;
    }
    let starts: Vec<&Line> = lines.iter().filter(|l| T::is_numbered_line(&l.text())).collect();
    starts.len() >= 2
        && starts.iter().map(|l| l.bbox[0]).fold(f64::NEG_INFINITY, f64::max)
            - starts.iter().map(|l| l.bbox[0]).fold(f64::INFINITY, f64::min)
            < 4.0
}

fn bullet_start(line: &Line) -> bool {
    if line.spans.iter().filter(|s| py::has_ink(&s.text)).any(|s| s.bold) {
        return false;
    }
    if BULLET_GLYPH.is(&line.text()) {
        return true;
    }
    if line.spans.len() < 2 {
        return false;
    }
    let sp = &line.spans[0];
    (!py::has_ink(&sp.text) && sp.font == "Unknown")
        || sp.text.chars().next().is_some_and(|c| (0xE000..=0xF8FF).contains(&(c as u32)))
}

fn merge_caption_parts(page: &mut Page) {
    let caps: Vec<usize> = (0..page.blocks.len()).filter(|&i| page.blocks[i].kind == "caption").collect();
    let mut drop: HashSet<u64> = HashSet::new();
    let label_text = |b: &Block| -> String {
        let p = T::plain(b.lines_or_empty());
        if p.is_empty() { py::strip_chars(&b.text, "*_ ").to_string() } else { p }
    };
    for &ai in &caps {
        let a_id = page.blocks[ai].id;
        if drop.contains(&a_id) || !CAPTION_LABEL.is(&label_text(&page.blocks[ai])) {
            continue;
        }
        let ab = page.blocks[ai].bbox;
        let h = ab[3] - ab[1];
        let mut best: Option<usize> = None;
        for &bi in &caps {
            let b = &page.blocks[bi];
            if bi == ai || drop.contains(&b.id) || b.bbox[0] < ab[2] - 1.0 {
                continue;
            }
            let ov = ab[3].min(b.bbox[3]) - ab[1].max(b.bbox[1]);
            if ov < 0.5 * h.min(b.bbox[3] - b.bbox[1]) || CAPTION_START.is(&label_text(b)) {
                continue;
            }
            if best.is_none_or(|x| b.bbox[0] < page.blocks[x].bbox[0]) {
                best = Some(bi);
            }
        }
        let Some(bi) = best else { continue };
        if page.blocks[bi].bbox[0] - ab[2] > 4.0 * h.max(1.0) {
            continue;
        }
        let (bt, bb, bl, bid) =
            (page.blocks[bi].text.clone(), page.blocks[bi].bbox, page.blocks[bi].lines.clone(), page.blocks[bi].id);
        let a = &mut page.blocks[ai];
        let mut lines = a.lines.clone().unwrap_or_default();
        lines.extend(bl.unwrap_or_default());
        a.bbox = [a.bbox[0].min(bb[0]), a.bbox[1].min(bb[1]), a.bbox[2].max(bb[2]), a.bbox[3].max(bb[3])];
        a.text = if lines.is_empty() { format!("{} {bt}", a.text) } else { T::paragraph(&lines) };
        a.lines = Some(lines);
        drop.insert(bid);
    }
    if !drop.is_empty() {
        page.blocks.retain(|b| !drop.contains(&b.id));
    }
}

fn place_next_to_captions(page: &mut Page) {
    let figs: Vec<(u64, u64)> = page
        .blocks
        .iter()
        .filter(|b| b.kind == "figure")
        .filter_map(|b| b.meta.caption_block.map(|c| (b.id, c)))
        .collect();
    for (fid, cid) in figs {
        if !page.blocks.iter().any(|x| x.id == cid) {
            continue;
        }
        let fi = page.blocks.iter().position(|x| x.id == fid).unwrap();
        let fig = page.blocks.remove(fi);
        let k = page.blocks.iter().position(|x| x.id == cid).unwrap();
        let at = if page.blocks[k].bbox[1] >= fig.bbox[1] { k } else { k + 1 };
        page.blocks.insert(at, fig);
    }
}

pub fn attach_captions(page: &mut Page) {
    merge_caption_parts(page);
    let caps: Vec<usize> = (0..page.blocks.len()).filter(|&i| page.blocks[i].kind == "caption").collect();
    let targets: Vec<usize> =
        (0..page.blocks.len()).filter(|&i| matches!(page.blocks[i].kind.as_str(), "figure" | "table")).collect();
    let mut pairs: Vec<(f64, usize, usize)> = Vec::new();
    for &ti in &targets {
        let t = &page.blocks[ti];
        for &ci in &caps {
            let c = &page.blocks[ci];
            let ox = t.bbox[2].min(c.bbox[2]) - t.bbox[0].max(c.bbox[0]);
            if ox < 0.3 * (t.bbox[2] - t.bbox[0]).min(c.bbox[2] - c.bbox[0]) {
                continue;
            }
            let gap = if c.bbox[1] >= t.bbox[1] { c.bbox[1] - t.bbox[3] } else { t.bbox[1] - c.bbox[3] };
            if gap > 60.0 {
                continue;
            }
            let word = py::lstrip_chars(&c.text, "*_ ").to_lowercase();
            if (t.kind == "table" && word.starts_with("fig"))
                || (t.kind == "figure" && word.starts_with("tab"))
                || word.starts_with("eq")
            {
                continue;
            }
            let bonus =
                if (t.kind == "figure" && word.starts_with("fig")) || (t.kind == "table" && word.starts_with("tab")) {
                    -20.0
                } else {
                    0.0
                };
            pairs.push((gap.max(0.0) + bonus, ti, ci));
        }
    }
    pairs.sort_by(|a, b| py::fcmp(a.0, b.0)); // stable
    let mut taken: HashSet<usize> = HashSet::new();
    for (_, ti, ci) in pairs {
        if taken.contains(&ti) || taken.contains(&ci) {
            continue;
        }
        taken.insert(ti);
        taken.insert(ci);
        let p = T::plain(page.blocks[ci].lines_or_empty());
        let cap = if p.is_empty() { page.blocks[ci].text.clone() } else { p };
        let cid = page.blocks[ci].id;
        let t = &mut page.blocks[ti];
        t.caption = Some(cap);
        t.meta.caption_block = Some(cid);
    }
    place_next_to_captions(page);
}

pub fn nearby_text(page: &Page, fig: &Block) -> String {
    let text_of = |p: &Block| -> String {
        let s = T::plain(p.lines_or_empty());
        if s.is_empty() { p.text.clone() } else { s }
    };
    let paras: Vec<&Block> = page.blocks.iter().filter(|b| b.kind == "paragraph" && !b.text.is_empty()).collect();
    if let Some(r) = FIG_REF.search(fig.caption.as_deref().unwrap_or("")) {
        let num = r.get(2).unwrap().as_str().to_string();
        for p in &paras {
            for m in FIG_REF.0.captures_iter(&p.text).filter_map(|c| c.ok()) {
                if m.get(2).unwrap().as_str() == num && p.id != fig.id {
                    return text_of(p);
                }
            }
        }
    }
    let mut before: Option<&Block> = None;
    for b in &page.blocks {
        if b.id == fig.id {
            break;
        }
        if b.kind == "paragraph" && !b.text.is_empty() {
            before = Some(b);
        }
    }
    before.map(text_of).unwrap_or_default()
}
