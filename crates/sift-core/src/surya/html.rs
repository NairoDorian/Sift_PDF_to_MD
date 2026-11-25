//! surya's HTML answers -> tables, Markdown,
//! LaTeX, figure labels. Port of V0's `sift/surya/html.py` (itself surya's parsers.py and
//! merged_cells.py on a small DOM).

use serde::{Deserialize, Serialize};

use super::compact;
use super::dom::{self, Dom, NodeId};
use crate::py::{self, Re};

pub fn strip_fences(text: &str) -> String {
    crate::pyre_static!(OPEN, Re, r"^```[a-zA-Z]*\n");
    crate::pyre_static!(CLOSE, Re, r"\n```\s*$");
    let mut c = py::strip(text).to_string();
    if c.starts_with("```") {
        c = OPEN.0.replacen(&c, 1, "").into_owned();
        c = CLOSE.0.replacen(&c, 1, "").into_owned();
    }
    c
}

fn has_unclosed_cells(html: &str) -> bool {
    crate::pyre_static!(OPEN, Re, r"<(?:td|th|tr)(?=[\s>/])", "i");
    crate::pyre_static!(CLOSE, Re, r"</(?:td|th|tr)\s*>", "i");
    OPEN.find_all(html).len() > CLOSE.find_all(html).len()
}

pub fn clean_block_html(text: &str) -> String {
    crate::pyre_static!(UNKNOWN, Re, r"<\|[^<>|\n]{0,24}\|>");
    let mut c = py::strip(&strip_fences(text)).to_string();
    if c.contains("<|") || has_unclosed_cells(&c) {
        c = compact::decode(&c);
    }
    UNKNOWN.replace_all(&c, "")
}

pub fn own_rows(d: &Dom, table: NodeId) -> Vec<NodeId> {
    fn walk(d: &Dom, n: NodeId, out: &mut Vec<NodeId>) {
        for &c in &d.n(n).children {
            let t = d.n(c).tag.as_str();
            if t == "tr" {
                out.push(c);
            } else if !matches!(t, "#text" | "table" | "td" | "th") {
                walk(d, c, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(d, table, &mut out);
    out
}

pub fn top_tables(d: &Dom, node: NodeId) -> Vec<NodeId> {
    let mut out = Vec::new();
    for &c in &d.n(node).children {
        let t = d.n(c).tag.as_str();
        if t == "table" {
            out.push(c);
        } else if t != "#text" {
            out.extend(top_tables(d, c));
        }
    }
    out
}

pub fn convert_merged_cells(d: &mut Dom) {
    for table in d.find_all(Dom::ROOT, &["table"], true) {
        let rows = own_rows(d, table);
        if rows.is_empty() {
            continue;
        }
        let grid: Vec<Vec<NodeId>> = rows
            .iter()
            .map(|&tr| {
                d.n(tr).children.iter().cloned().filter(|&c| matches!(d.n(c).tag.as_str(), "td" | "th")).collect()
            })
            .collect();
        let max_cols = grid.iter().map(|r| r.len()).max().unwrap_or(0);
        if max_cols == 0 {
            continue;
        }
        let cell = |r: usize, c: usize| -> Option<NodeId> { grid[r].get(c).cloned() };
        let mut processed = vec![vec![false; max_cols]; rows.len()];
        for r in 0..rows.len() {
            for c in 0..max_cols {
                let Some(node) = cell(r, c) else { continue };
                if processed[r][c] {
                    continue;
                }
                if d.attr(node, "data-kind") != Some("parent") {
                    processed[r][c] = true;
                    continue;
                }
                let (mut colspan, mut cc) = (1, c + 1);
                while cc < max_cols && cell(r, cc).is_some_and(|x| d.attr(x, "data-kind") == Some("ghost")) {
                    colspan += 1;
                    cc += 1;
                }
                let (mut rowspan, mut rr) = (1, r + 1);
                while rr < rows.len()
                    && (0..colspan).all(|dc| cell(rr, c + dc).is_some_and(|x| d.attr(x, "data-kind") == Some("ghost")))
                {
                    rowspan += 1;
                    rr += 1;
                }
                if colspan > 1 {
                    d.set_attr(node, "colspan", &colspan.to_string());
                }
                if rowspan > 1 {
                    d.set_attr(node, "rowspan", &rowspan.to_string());
                }
                for k in ["data-cs", "data-rs", "data-kind"] {
                    d.pop_attr(node, k);
                }
                for dr in 0..rowspan {
                    for dc in 0..colspan {
                        if r + dr < rows.len() && c + dc < max_cols {
                            processed[r + dr][c + dc] = true;
                        }
                    }
                }
            }
        }
        for row in &grid {
            for &node in row {
                if d.attr(node, "data-kind") == Some("ghost") {
                    d.remove(node);
                }
            }
        }
    }
}

const BALANCED: [&str; 14] =
    ["div", "table", "math", "pre", "code", "p", "h1", "h2", "h3", "h4", "h5", "ul", "ol", "li"];
crate::pyre_static!(PARTIAL_COMPACT_HEADER, Re, r"<\|[A-Za-z-]+\|>(?:<\|\d{1,4}\|>){0,3}$|<\|[^<>|]*$");

pub fn html_looks_truncated(html: &str) -> bool {
    for tag in BALANCED {
        let o = Re::new_flags(&format!(r"<{tag}(?=[\s>/])"), "i");
        let c = Re::new_flags(&format!(r"</{tag}\s*>"), "i");
        if o.find_all(html).len() > c.find_all(html).len() {
            return true;
        }
    }
    crate::pyre_static!(OPEN_TAG, Re, r"<[^<>]*$");
    OPEN_TAG.is_match(html) || PARTIAL_COMPACT_HEADER.is_match(html)
}

#[derive(Clone, Debug)]
pub struct FullPageBlock {
    pub label: String,
    pub bbox: [f64; 4],
    pub html: String,
    pub truncated: bool,
}

pub fn parse_full_page(text: &str) -> Vec<FullPageBlock> {
    let mut cleaned = py::strip(&strip_fences(text)).to_string();
    let partial = PARTIAL_COMPACT_HEADER.is_match(&cleaned);
    if cleaned.contains("<|") {
        cleaned = compact::decode(&cleaned);
    }
    if cleaned.is_empty() {
        return vec![];
    }
    let mut d = dom::parse(&cleaned);
    let mut out = Vec::new();
    let divs: Vec<NodeId> = d.n(Dom::ROOT).children.iter().cloned().filter(|&c| d.n(c).tag == "div").collect();
    for div in divs {
        let (Some(label), Some(bbox)) =
            (d.attr(div, "data-label").map(String::from), d.attr(div, "data-bbox").map(String::from))
        else {
            continue;
        };
        if label.is_empty() || bbox.is_empty() {
            continue;
        }
        let parts: Result<Vec<f64>, _> = py::split_ws(&bbox).iter().map(|x| x.parse::<f64>()).collect();
        let Ok(parts) = parts else { continue };
        if parts.len() != 4 {
            continue;
        }
        for n in d.find_all(div, &["td", "th", "tr", "div", "p", "img", "table", "span"], true) {
            d.pop_attr(n, "data-bbox");
            d.pop_attr(n, "data-label");
        }
        let html: String = d.n(div).children.iter().map(|&c| d.to_html(c)).collect();
        out.push(FullPageBlock {
            label,
            bbox: [parts[0], parts[1], parts[2], parts[3]],
            html: py::strip(&html).to_string(),
            truncated: false,
        });
    }
    if !out.is_empty() && (partial || html_looks_truncated(&cleaned)) {
        out.last_mut().unwrap().truncated = true;
    }
    out
}

// ---- tables -------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cell {
    pub html: String,
    pub text: String,
    pub header: bool,
    pub colspan: usize,
    pub rowspan: usize,
}

impl Cell {
    pub fn new(text: &str) -> Cell {
        Cell { html: dom::escape(text, false), text: text.to_string(), header: false, colspan: 1, rowspan: 1 }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Table {
    pub rows: Vec<Vec<Cell>>,
}

impl Table {
    pub fn has_merged(&self) -> bool {
        self.rows.iter().flatten().any(|c| c.colspan > 1 || c.rowspan > 1)
    }
    pub fn n_cols(&self) -> usize {
        self.rows.iter().map(|r| r.iter().map(|c| c.colspan).sum::<usize>()).max().unwrap_or(0)
    }
    pub fn header_rows(&self) -> usize {
        let mut n = 0;
        for r in &self.rows {
            if !r.is_empty() && r.iter().all(|c| c.header) {
                n += 1;
            } else {
                break;
            }
        }
        n
    }
}

const INLINE_KEEP: [&str; 13] =
    ["b", "strong", "i", "em", "sup", "sub", "br", "u", "code", "math", "del", "small", "big"];

fn inline_html(d: &Dom, node: NodeId) -> String {
    let mut out: Vec<String> = Vec::new();
    for &c in &d.n(node).children {
        let t = d.n(c).tag.as_str();
        if t == "#text" {
            out.push(dom::escape(&d.n(c).text, false));
        } else if t == "br" {
            out.push("<br>".into());
        } else if INLINE_KEEP.contains(&t) {
            let tag = match t {
                "strong" => "b",
                "em" => "i",
                o => o,
            };
            if tag == "math" {
                out.push(format!("${}$", py::strip(&d.get_text(c))));
            } else {
                out.push(format!("<{tag}>{}</{tag}>", inline_html(d, c)));
            }
        } else if matches!(t, "p" | "div" | "li") {
            let inner = py::strip(&inline_html(d, c)).to_string();
            if !inner.is_empty() {
                out.push(format!("{}{inner}", if out.is_empty() { "" } else { "<br>" }));
            }
        } else if t == "table" {
            out.push(nested_table_html(d, c));
        } else {
            out.push(inline_html(d, c));
        }
    }
    out.concat()
}

fn nested_table_html(d: &Dom, table: NodeId) -> String {
    let mut rows = Vec::new();
    for tr in own_rows(d, table) {
        let mut cells = Vec::new();
        for &c in &d.n(tr).children {
            let t = d.n(c).tag.as_str();
            if t == "td" || t == "th" {
                let attrs: String = ["colspan", "rowspan"]
                    .iter()
                    .filter_map(|k| {
                        d.attr(c, k).filter(|v| !v.is_empty() && *v != "1").map(|v| format!(" {k}=\"{v}\""))
                    })
                    .collect();
                cells.push(format!("<{t}{attrs}>{}</{t}>", norm_ws(&inline_html(d, c))));
            }
        }
        if !cells.is_empty() {
            rows.push(format!("<tr>{}</tr>", cells.concat()));
        }
    }
    if rows.is_empty() { String::new() } else { format!("<table>{}</table>", rows.concat()) }
}

const TEXT_BREAKS: [&str; 17] =
    ["p", "div", "li", "tr", "td", "th", "table", "thead", "tbody", "ul", "ol", "h1", "h2", "h3", "h4", "h5", "h6"];

pub fn cell_text(d: &Dom, node: NodeId) -> String {
    let n = d.n(node);
    if n.tag == "#text" {
        return n.text.clone();
    }
    if n.tag == "br" {
        return "\n".into();
    }
    let inner: String = n.children.iter().map(|&c| cell_text(d, c)).collect();
    if TEXT_BREAKS.contains(&n.tag.as_str()) { format!(" {inner} ") } else { inner }
}

pub fn norm_ws(s: &str) -> String {
    crate::pyre_static!(WS, Re, r"[ \t\r\n]+");
    py::strip(&WS.replace_all(s, " ")).to_string()
}

pub fn parse_table(html: &str) -> Option<Table> {
    if !html.contains("<table") {
        return None;
    }
    let mut d = dom::parse(&clean_block_html(html));
    convert_merged_cells(&mut d);
    let t = d.find(Dom::ROOT, "table")?;
    table_from_node(&d, t)
}

/// One part of a table answer: a table, or the Markdown text between tables.
#[derive(Clone, Debug)]
pub enum Part {
    Table(Table),
    Text(String),
}

pub fn parse_tables(html: &str) -> Vec<Part> {
    if !html.contains("<table") {
        return vec![];
    }
    let mut d = dom::parse(&clean_block_html(html));
    convert_merged_cells(&mut d);
    let mut out: Vec<Part> = Vec::new();
    let mut pending: Vec<NodeId> = Vec::new();
    let mut loose: Vec<NodeId> = Vec::new();
    let root_children = d.n(Dom::ROOT).children.clone();

    fn flush(d: &Dom, pending: &mut Vec<NodeId>, out: &mut Vec<Part>) {
        if !pending.is_empty() {
            let html: String = pending.iter().map(|&n| d.to_html(n)).collect();
            let md = py::strip(&block_to_markdown(&html, None)).to_string();
            if !md.is_empty() {
                out.push(Part::Text(md));
            }
            pending.clear();
        }
    }
    fn flush_loose(d: &mut Dom, loose: &mut Vec<NodeId>, out: &mut Vec<Part>) {
        if !loose.is_empty() {
            let t = d.detached("table", loose.clone());
            if let Some(p) = table_from_node(d, t) {
                out.push(Part::Table(p));
            }
            loose.clear();
        }
    }

    for node in root_children {
        let tag = d.n(node).tag.clone();
        if matches!(tag.as_str(), "thead" | "tbody" | "tr") {
            flush(&d, &mut pending, &mut out);
            loose.push(node);
            continue;
        }
        if tag == "#text" && !py::has_ink(&d.n(node).text) && !loose.is_empty() {
            continue;
        }
        flush_loose(&mut d, &mut loose, &mut out);
        let tables = if tag == "table" {
            vec![node]
        } else if tag != "#text" {
            top_tables(&d, node)
        } else {
            vec![]
        };
        if tables.is_empty() {
            pending.push(node);
            continue;
        }
        flush(&d, &mut pending, &mut out);
        for t in tables {
            if let Some(p) = table_from_node(&d, t) {
                out.push(Part::Table(p));
            }
        }
    }
    flush_loose(&mut d, &mut loose, &mut out);
    flush(&d, &mut pending, &mut out);
    out
}

fn table_from_node(d: &Dom, table: NodeId) -> Option<Table> {
    crate::pyre_static!(TRAIL_BR, Re, r"(<br>\s*)+$");
    let mut rows = Vec::new();
    for tr in own_rows(d, table) {
        let mut cells = Vec::new();
        for &c in &d.n(tr).children {
            let t = d.n(c).tag.as_str();
            if t != "td" && t != "th" {
                continue;
            }
            // capped at the HTML standard's own limits (browsers clamp the same way): a span the
            // model made up cannot ask for a grid of billions of cells; real tables never get near
            let span = |k: &str, max: i64| {
                d.attr(c, k)
                    .filter(|v| !v.is_empty())
                    .map(|v| py::strip(v).parse::<i64>().map(|x| x.clamp(1, max) as usize).unwrap_or(1))
                    .unwrap_or(1)
            };
            let in_thead = d.n(tr).parent.is_some_and(|p| d.n(p).tag == "thead");
            cells.push(Cell {
                html: TRAIL_BR.replace_all(&norm_ws(&inline_html(d, c)), ""),
                text: norm_ws(&cell_text(d, c)),
                header: t == "th" || in_thead,
                colspan: span("colspan", 1000),
                rowspan: span("rowspan", 65534),
            });
        }
        if !cells.is_empty() {
            rows.push(cells);
        }
    }
    if rows.is_empty() || !rows.iter().flatten().any(|c| !c.text.is_empty()) {
        return None;
    }
    Some(Table { rows })
}

// ---- OCR'd text blocks -> Markdown --------------------------------------------------

fn md_inline(d: &Dom, node: NodeId) -> String {
    crate::pyre_static!(WS, Re, r"\s+");
    let mut out = String::new();
    for &c in &d.n(node).children {
        let t = d.n(c).tag.as_str();
        match t {
            "#text" => out.push_str(&WS.replace_all(&d.n(c).text, " ")),
            "br" => out.push_str("  \n"),
            "b" | "strong" => {
                let s = py::strip(&md_inline(d, c)).to_string();
                if !s.is_empty() {
                    out.push_str(&format!("**{s}**"));
                }
            }
            "i" | "em" => {
                let s = py::strip(&md_inline(d, c)).to_string();
                if !s.is_empty() {
                    out.push_str(&format!("*{s}*"));
                }
            }
            "sup" | "sub" => out.push_str(&format!("<{t}>{}</{t}>", py::strip(&md_inline(d, c)))),
            "math" => out.push_str(&format!("${}$", py::strip(&d.get_text(c)))),
            "code" => out.push_str(&format!("`{}`", d.get_text(c))),
            "img" => {}
            _ => out.push_str(&md_inline(d, c)),
        }
    }
    out
}

/// Markdown for one OCR'd block (headings, paragraphs, lists, math, code).
pub fn block_to_markdown(html: &str, heading_level: Option<usize>) -> String {
    let mut d = dom::parse(&clean_block_html(html));
    convert_merged_cells(&mut d);
    let mut parts: Vec<String> = Vec::new();
    walk_md(&d, Dom::ROOT, 0, heading_level, &mut parts);
    parts.into_iter().filter(|p| py::has_ink(p)).collect::<Vec<_>>().join("\n\n")
}

fn walk_md(d: &Dom, node: NodeId, depth: usize, heading_level: Option<usize>, parts: &mut Vec<String>) {
    crate::pyre_static!(OL_NUM, ReStart, r"(\d{1,9})([.)])\s+");
    crate::pyre_static!(OL_MARK, ReStart, r"\(?(?:[a-zA-Z]|[ivxIVX]{1,4}|\d{1,9})[.)]\s");
    for &c in &d.n(node).children {
        let tag = d.n(c).tag.as_str();
        match tag {
            "#text" => {
                let t = norm_ws(&d.n(c).text);
                if !t.is_empty() {
                    parts.push(t);
                }
            }
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let t = norm_ws(&md_inline(d, c));
                match crate::build::headings::body_text_reason(&t) {
                    Some("lead-in") => parts.push(if t.starts_with("**") { t } else { format!("**{t}**") }),
                    Some(_) => parts.push(t),
                    None => {
                        let lvl = heading_level.unwrap_or_else(|| tag[1..].parse().unwrap_or(1));
                        parts.push(format!("{} {t}", "#".repeat(lvl)));
                    }
                }
            }
            "ul" | "ol" => {
                let mut items = Vec::new();
                let mut n: i64 = d
                    .attr(c, "start")
                    .filter(|s| !s.is_empty())
                    .map(|s| py::strip(s).parse::<i64>().map(|v| v - 1).unwrap_or(0))
                    .unwrap_or(0);
                for &li in d.n(c).children.iter().filter(|&&x| d.n(x).tag == "li") {
                    let mut t = norm_ws(&md_inline(d, li));
                    n += 1;
                    let mut marker = "-".to_string();
                    if tag == "ol" {
                        if let Some(m) = OL_NUM.m(&t) {
                            let num: i64 = m.get(1).unwrap().as_str().parse().unwrap_or(n);
                            n = num;
                            marker = format!("{num}{}", m.get(2).unwrap().as_str());
                            let end = m.get(0).unwrap().end();
                            t = t[end..].to_string();
                        } else if !OL_MARK.is(&t) {
                            marker = format!("{n}.");
                        }
                    }
                    items.push(format!("{}{marker} {t}", "  ".repeat(depth)));
                }
                parts.push(items.join("\n"));
            }
            "math" => {
                let tex = py::strip(&d.get_text(c)).to_string();
                parts.push(if d.attr(c, "display") == Some("block") || d.n(node).tag.is_empty() {
                    format!("$$\n{tex}\n$$")
                } else {
                    format!("${tex}$")
                });
            }
            "pre" => parts.push(format!("```\n{}\n```", d.get_text(c).trim_matches('\n'))),
            "table" => parts.push(d.to_html(c)),
            "p" | "div" | "span" | "caption" | "small" | "big" => {
                let nested = d.n(c).children.iter().any(|&x| {
                    matches!(
                        d.n(x).tag.as_str(),
                        "p" | "div" | "ul" | "ol" | "table" | "h1" | "h2" | "h3" | "h4" | "h5" | "pre"
                    )
                });
                if nested {
                    walk_md(d, c, depth, heading_level, parts);
                } else {
                    let t = py::strip(&md_inline(d, c)).to_string();
                    if !t.is_empty() {
                        parts.push(t);
                    }
                }
            }
            "img" | "hr" | "input" => {}
            _ => {
                let t = py::strip(&md_inline(d, c)).to_string();
                if !t.is_empty() {
                    parts.push(t);
                }
            }
        }
    }
}

pub fn block_plain_text(html: &str) -> String {
    let d = dom::parse(&clean_block_html(html));
    norm_ws(&d.get_text(Dom::ROOT))
}

/// The words of a block answer as plain text: like `block_plain_text`, with paragraphs, list
/// items and cells kept apart (their last and first words must not run together when the
/// words are looked up in the PDF text).
pub fn block_words(html: &str) -> String {
    let d = dom::parse(&clean_block_html(html));
    norm_ws(&cell_text(&d, Dom::ROOT))
}

/// ("math", LaTeX) / ("text", Markdown) parts of an equation block, in order.
pub fn equation_parts(html: &str) -> Vec<(String, String)> {
    let d = dom::parse(&clean_block_html(html));
    let mut parts: Vec<(String, String)> = Vec::new();
    for &c in &d.n(Dom::ROOT).children {
        let n = d.n(c);
        if n.tag == "#text" && !py::has_ink(&n.text) {
            continue;
        }
        let inner: Vec<NodeId> = if n.tag == "p" || n.tag == "div" {
            n.children.iter().cloned().filter(|&x| d.n(x).tag != "#text" || py::has_ink(&d.n(x).text)).collect()
        } else {
            vec![]
        };
        let only_math = if n.tag == "math" {
            Some(c)
        } else if inner.len() == 1 && d.n(inner[0]).tag == "math" {
            Some(inner[0])
        } else {
            None
        };
        if let Some(m) = only_math {
            let tex = py::strip(&d.get_text(m)).to_string();
            if !tex.is_empty() {
                parts.push(("math".into(), tex));
            }
            continue;
        }
        let md = py::strip(&block_to_markdown(&d.to_html(c), None)).to_string();
        if !md.is_empty() {
            if let Some(last) = parts.last_mut()
                && last.0 == "text"
            {
                last.1 = format!("{}\n\n{md}", last.1);
                continue;
            }
            parts.push(("text".into(), md));
        }
    }
    parts
}

pub fn math_latex(html: &str) -> Option<String> {
    let d = dom::parse(&clean_block_html(html));
    let maths = d.find_all(Dom::ROOT, &["math"], true);
    if maths.is_empty() {
        return None;
    }
    let s = maths
        .iter()
        .map(|&m| py::strip(&d.get_text(m)).to_string())
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if s.is_empty() { None } else { Some(s) }
}

pub fn figure_all_text(html: &str) -> String {
    let mut d = dom::parse(&clean_block_html(html));
    for t in d.find_all(Dom::ROOT, &["table"], true) {
        for cell in d.find_all(t, &["td", "th"], true) {
            if d.get_text(cell).contains('~') {
                d.nodes[cell].children.clear();
            }
        }
    }
    fn walk(d: &Dom, n: NodeId, texts: &mut Vec<String>) {
        for &c in &d.n(n).children {
            if d.n(c).tag == "#text" {
                texts.push(d.n(c).text.clone());
            } else {
                if d.n(c).tag == "img"
                    && let Some(a) = d.attr(c, "alt")
                    && !a.is_empty()
                {
                    texts.push(a.to_string());
                }
                walk(d, c, texts);
            }
        }
    }
    let mut texts = Vec::new();
    walk(&d, Dom::ROOT, &mut texts);
    norm_ws(&texts.join(" "))
}

pub fn figure_labels(html: &str) -> Vec<String> {
    let d = dom::parse(&clean_block_html(html));
    let mut out: Vec<String> = Vec::new();
    for p in d.find_all(Dom::ROOT, &["p"], true) {
        let cls = d.attr(p, "class").unwrap_or("").to_string();
        if cls.contains("chart-axes") {
            for x in d.get_text(p).split(';') {
                if py::has_ink(x) {
                    out.push(norm_ws(x));
                }
            }
            continue;
        }
        if !cls.contains("img-text") {
            continue;
        }
        let mut cur: Vec<String> = Vec::new();
        for &c in &d.n(p).children {
            if d.n(c).tag == "br" {
                let j = cur.concat();
                if py::has_ink(&j) {
                    out.push(norm_ws(&j));
                }
                cur.clear();
            } else {
                cur.push(d.get_text(c));
            }
        }
        let j = cur.concat();
        if py::has_ink(&j) {
            out.push(norm_ws(&j));
        }
    }
    let mut seen = std::collections::HashSet::new();
    out.into_iter().filter(|t| !t.is_empty() && seen.insert(t.clone())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Spans come from the model's text: capped at the HTML standard's limits, and anything that
    /// is not a positive number counts as 1, as before.
    #[test]
    fn spans_are_capped_and_cleaned() {
        let t = parse_table(
            r#"<table><tr><td colspan="5000" rowspan="99999">a</td><td colspan="99999999999999999999">b</td><td colspan="0">c</td><td colspan="-3">d</td><td colspan=" 2 ">e</td><td colspan="x">f</td></tr></table>"#,
        )
        .expect("a table");
        let c = &t.rows[0];
        assert_eq!((c[0].colspan, c[0].rowspan), (1000, 65534));
        assert_eq!([c[1].colspan, c[2].colspan, c[3].colspan, c[4].colspan, c[5].colspan], [1, 1, 1, 2, 1]);
    }

    #[test]
    fn ordinary_spans_are_kept() {
        let t = parse_table(
            r#"<table><tr><th colspan="3">h</th></tr><tr><td rowspan="2">a</td><td>b</td><td>c</td></tr></table>"#,
        )
        .expect("a table");
        assert_eq!(t.rows[0][0].colspan, 3);
        assert_eq!(t.rows[1][0].rowspan, 2);
        assert!(t.rows[0][0].header);
    }
}
