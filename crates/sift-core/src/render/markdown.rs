//! Blocks -> Markdown (V0 `sift/render/markdown.py`). D9: pipe tables, plain HTML tables
//! only for merged cells (or nested tables), displayed by VS Code, Obsidian and GitHub.

use std::collections::HashMap;

use crate::build::blocks::{Block, ListItem};
use crate::py::{self};
use crate::surya::html::Table;

fn pipe_cell(html: &str) -> String {
    let s = html.replace('|', "\\|").replace('\n', " ");
    let t = py::strip(&s);
    if t.is_empty() { " ".into() } else { t.to_string() }
}

fn expand(table: &Table) -> Vec<Vec<String>> {
    let mut grid: Vec<Vec<String>> = Vec::new();
    let mut carry: HashMap<(usize, usize), String> = HashMap::new();
    for (r, row) in table.rows.iter().enumerate() {
        let mut out = Vec::new();
        let mut c = 0;
        let mut it = row.iter();
        loop {
            if let Some(v) = carry.remove(&(r, c)) {
                out.push(v);
                c += 1;
                continue;
            }
            let Some(cell) = it.next() else { break };
            for dc in 0..cell.colspan {
                out.push(cell.html.clone());
                for dr in 1..cell.rowspan {
                    carry.insert((r + dr, c + dc), cell.html.clone());
                }
            }
            c += cell.colspan;
        }
        while let Some(v) = carry.remove(&(r, c)) {
            out.push(v);
            c += 1;
        }
        grid.push(out);
    }
    let width = grid.iter().map(|r| r.len()).max().unwrap_or(0);
    grid.into_iter()
        .map(|mut r| {
            r.resize(width, String::new());
            r
        })
        .collect()
}

pub fn pipe_table(table: &Table) -> String {
    let grid = expand(table);
    if grid.is_empty() {
        return String::new();
    }
    let hdr_rows = match table.header_rows() {
        0 => 1,
        n => n,
    };
    let header: Vec<String> = if hdr_rows > 1 {
        let width = grid[0].len();
        (0..width)
            .map(|ci| {
                let mut seen: Vec<String> = Vec::new();
                for r in grid.iter().take(hdr_rows) {
                    let x = &r[ci];
                    if py::has_ink(x) && !seen.contains(x) {
                        seen.push(x.clone());
                    }
                }
                seen.join("<br>")
            })
            .collect()
    } else {
        grid[0].clone()
    };
    let body = &grid[hdr_rows.min(grid.len())..];
    let mut lines = vec![
        format!("| {} |", header.iter().map(|h| pipe_cell(h)).collect::<Vec<_>>().join(" | ")),
        format!("|{}|", header.iter().map(|_| " --- ").collect::<Vec<_>>().join("|")),
    ];
    for row in body {
        lines.push(format!("| {} |", row.iter().map(|c| pipe_cell(c)).collect::<Vec<_>>().join(" | ")));
    }
    lines.join("\n")
}

pub fn html_table(table: &Table) -> String {
    let hdr = table.header_rows();
    let mut out = vec!["<table>".to_string()];
    if hdr > 0 {
        out.push("<thead>".into());
    }
    for (i, row) in table.rows.iter().enumerate() {
        if hdr > 0 && i == hdr {
            out.push("</thead>".into());
            out.push("<tbody>".into());
        } else if hdr == 0 && i == 0 {
            out.push("<tbody>".into());
        }
        let cells: String = row
            .iter()
            .map(|c| {
                let tag = if c.header || i < hdr { "th" } else { "td" };
                let mut attrs = String::new();
                if c.colspan > 1 {
                    attrs.push_str(&format!(" colspan=\"{}\"", c.colspan));
                }
                if c.rowspan > 1 {
                    attrs.push_str(&format!(" rowspan=\"{}\"", c.rowspan));
                }
                format!("<{tag}{attrs}>{}</{tag}>", py::strip(&c.html.replace('\n', " ")))
            })
            .collect();
        out.push(format!("<tr>{cells}</tr>"));
    }
    out.push(if hdr == table.rows.len() { "</thead>".into() } else { "</tbody>".into() });
    out.push("</table>".into());
    out.join("\n")
}

pub fn table_markdown(table: &Table, mode: &str) -> String {
    let nested = table.rows.iter().flatten().any(|c| c.html.contains("<table"));
    if mode == "markdown" || !(table.has_merged() || nested) { pipe_table(table) } else { html_table(table) }
}

pub fn list_markdown(items: &[ListItem]) -> String {
    crate::pyre_static!(NUM_MARKER, ReFull, r"(\d{1,9})([.)])");
    let mut out: Vec<String> = Vec::new();
    let mut counters: Vec<(i64, i64)> = Vec::new(); // insertion-ordered dict
    let mut starts: Vec<(i64, usize)> = Vec::new();
    for it in items {
        let lvl = it.level;
        let mut text = it.text.clone();
        counters.retain(|(k, _)| *k <= lvl);
        starts.retain(|(k, _)| *k <= lvl);
        let marker: String;
        if it.ordered {
            let v = counters.iter().find(|(k, _)| *k == lvl).map(|(_, v)| *v).unwrap_or(0) + 1;
            set(&mut counters, lvl, v);
            if let Some(m) = NUM_MARKER.m(&it.marker) {
                let n: i64 = m[1].parse().unwrap_or(v);
                set(&mut counters, lvl, n);
                marker = format!("{n}{}", &m[2]);
            } else if !it.marker.is_empty() {
                marker = "-".into();
                text = format!("{} {text}", it.marker);
            } else {
                marker = format!("{v}.");
            }
        } else {
            counters.retain(|(k, _)| *k != lvl);
            marker = "-".into();
        }
        let lower: Vec<i64> = starts.iter().filter(|(k, _)| *k < lvl).map(|(k, _)| *k).collect();
        let pad = match lower.iter().max() {
            Some(mx) => starts.iter().find(|(k, _)| k == mx).unwrap().1,
            None => 0,
        };
        let st = pad + py::char_len(&marker) + 1;
        set_u(&mut starts, lvl, st);
        let sep = format!("  \n{}", " ".repeat(st));
        let text = text.split('\n').map(|t| py::strip(t).to_string()).collect::<Vec<_>>().join(&sep);
        out.push(format!("{}{marker} {text}", " ".repeat(pad)));
    }
    out.join("\n")
}

fn set(v: &mut Vec<(i64, i64)>, k: i64, val: i64) {
    match v.iter_mut().find(|(a, _)| *a == k) {
        Some(e) => e.1 = val,
        None => v.push((k, val)),
    }
}
fn set_u(v: &mut Vec<(i64, usize)>, k: i64, val: usize) {
    match v.iter_mut().find(|(a, _)| *a == k) {
        Some(e) => e.1 = val,
        None => v.push((k, val)),
    }
}

fn alt(text: &str) -> String {
    crate::pyre_static!(WS, Re, r"\s+");
    crate::pyre_static!(MD, Re, r"[*_`]");
    let t = py::strip(&WS.replace_all(text, " ")).to_string();
    let t = MD.replace_all(&t, "").replace('[', "(").replace(']', ")");
    t.chars().take(300).collect()
}

fn quote(text: &str) -> String {
    py::splitlines(py::strip(text))
        .iter()
        .map(|l| {
            let l = py::rstrip(l);
            if l.is_empty() { ">".to_string() } else { format!("> {l}") }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn figure_markdown(b: &Block) -> String {
    let mut a = alt(b.caption.as_deref().unwrap_or(""));
    if a.is_empty() {
        a = format!("{} on page {}", b.label, b.page + 1);
    }
    let mut parts = vec![format!("![{a}]({})", b.image.as_deref().unwrap_or(""))];
    if let Some(d) = &b.description
        && !d.is_empty()
    {
        let desc = py::strip(d);
        let (first, rest) = match desc.split_once('\n') {
            Some((f, r)) => (f, Some(r)),
            None => (desc, None),
        };
        let body = match rest {
            Some(r) if !r.is_empty() => format!("**Image description:** {first}\n{r}"),
            _ => format!("**Image description:** {first}"),
        };
        parts.push(quote(&body));
    }
    parts.join("\n\n")
}

pub fn block_markdown(b: &Block, tables: &str) -> String {
    let t = py::strip(&b.text).to_string();
    match b.kind.as_str() {
        "heading" => {
            if t.is_empty() {
                String::new()
            } else {
                let lvl = if b.level == 0 { 2 } else { b.level }.clamp(1, 6);
                format!("{} {t}", "#".repeat(lvl as usize))
            }
        }
        "paragraph" | "footnote" | "toc" => t,
        "caption" => {
            if t.starts_with('*') {
                t
            } else if t.is_empty() {
                t
            } else {
                format!("*{t}*")
            }
        }
        "code" => {
            if t.is_empty() {
                String::new()
            } else {
                format!("```\n{}\n```", py::rstrip_chars(&b.text, "\n"))
            }
        }
        "list" => list_markdown(&b.items),
        "table" => match &b.table {
            Some(tb) => table_markdown(tb, tables),
            None => t,
        },
        "equation" => match &b.latex {
            Some(l) if !l.is_empty() => format!("$$\n{}\n$$", py::strip(l)),
            _ => t,
        },
        "figure" => {
            if b.image.as_deref().is_some_and(|s| !s.is_empty()) {
                figure_markdown(b)
            } else {
                String::new()
            }
        }
        _ => t,
    }
}

pub fn render(blocks: &[Block], tables: &str) -> String {
    crate::pyre_static!(NL3, Re, r"\n{3,}");
    let parts: Vec<String> = blocks.iter().map(|b| block_markdown(b, tables)).filter(|p| py::has_ink(p)).collect();
    let md = parts.join("\n\n");
    format!("{}\n", py::strip(&NL3.replace_all(&md, "\n\n")))
}
