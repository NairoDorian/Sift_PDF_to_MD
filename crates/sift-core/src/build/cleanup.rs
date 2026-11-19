//! Document-level clean-up after every page is built (V0 `sift/build/cleanup.py`):
//! repeated margins (D12), running titles, paragraphs cut by a column or page break,
//! tables split by a page break.

use std::collections::{HashMap, HashSet};

use crate::build::blocks::{Block, Page};
use crate::build::text::{TERMINAL, join_lines, vocab_of};
use crate::py::{self};

fn key(text: &str) -> String {
    crate::pyre_static!(WS, Re, r"\s+");
    crate::pyre_static!(DIG, Re, r"\d+");
    py::strip(&DIG.replace_all(&WS.replace_all(&text.to_lowercase(), " "), "#")).to_string()
}

fn in_margin(b: &Block, p: &Page) -> bool {
    b.bbox[3] < p.height * 0.10 || b.bbox[1] > p.height * 0.90
}

fn spot6(b: &Block) -> [i64; 4] {
    [0, 1, 2, 3].map(|i| py::round_half_even(b.bbox[i] / 6.0) as i64)
}

pub fn drop_repeated_margins(pages: &mut [Page]) -> usize {
    if pages.len() < 3 {
        return 0;
    }
    let mut texts: HashMap<String, usize> = HashMap::new();
    let mut spots: HashMap<[i64; 4], usize> = HashMap::new();
    for p in pages.iter() {
        let (mut st, mut ss) = (HashSet::new(), HashSet::new());
        for b in &p.blocks {
            if !in_margin(b, p) {
                continue;
            }
            if (b.kind == "paragraph" || b.kind == "heading") && py::char_len(&b.text) < 200 {
                st.insert(key(&b.text));
            } else if b.kind == "figure" {
                ss.insert(spot6(b));
            }
        }
        for t in st {
            *texts.entry(t).or_insert(0) += 1;
        }
        for s in ss {
            *spots.entry(s).or_insert(0) += 1;
        }
    }
    let need = 3usize.max((0.3 * pages.len() as f64) as usize);
    let need_heading = 3;
    let mut dropped = 0;
    for p in pages.iter_mut() {
        let (h, blocks) = (p.height, std::mem::take(&mut p.blocks));
        let pg = Page { index: p.index, width: p.width, height: h, blocks: vec![], scanned: p.scanned };
        for b in blocks {
            let tcount = texts.get(&key(&b.text)).copied().unwrap_or(0);
            if in_margin(&b, &pg)
                && ((b.kind == "paragraph" && tcount >= need)
                    || (b.kind == "heading" && tcount >= need_heading)
                    || (b.kind == "figure" && spots.get(&spot6(&b)).copied().unwrap_or(0) >= need))
            {
                dropped += 1;
                continue;
            }
            p.blocks.push(b);
        }
    }
    dropped + drop_running_titles(pages)
}

fn title_key(text: &str) -> String {
    crate::pyre_static!(MD, Re, r"[*_#\\`]");
    crate::pyre_static!(NUM, Re, r"^(?:\d+(?:\.\d+)*\.?|[A-Z]\.)\s+");
    crate::pyre_static!(WS, Re, r"\s+");
    let t = py::strip(&MD.replace_all(text, "")).to_string();
    let t = NUM.0.replacen(&t, 1, "").into_owned();
    WS.replace_all(&t, " ").to_lowercase()
}

fn drop_running_titles(pages: &mut [Page]) -> usize {
    let mut heads: HashSet<String> = HashSet::new();
    for p in pages.iter() {
        for b in &p.blocks {
            if b.kind == "heading" && !in_margin(b, p) {
                heads.insert(title_key(&b.text));
            }
        }
    }
    heads.remove("");
    let spot = |b: &Block| -> (i64, i64, i64) {
        let bb = b.lines.as_ref().and_then(|l| l.first()).map(|l| l.bbox).unwrap_or(b.bbox);
        (
            py::round_half_even(bb[0] / 3.0) as i64,
            py::round_half_even(bb[1] / 3.0) as i64,
            py::round_half_even((bb[3] - bb[1]) / 2.0) as i64,
        )
    };
    let mut echoes: Vec<(usize, u64, (i64, i64, i64))> = Vec::new();
    for p in pages.iter() {
        for b in &p.blocks {
            if (b.kind == "paragraph" || b.kind == "heading")
                && py::char_len(&b.text) < 200
                && in_margin(b, p)
                && heads.contains(&title_key(&b.text))
            {
                echoes.push((p.index, b.id, spot(b)));
            }
        }
    }
    let uniq: HashSet<((i64, i64, i64), usize)> = echoes.iter().map(|(pi, _, s)| (*s, *pi)).collect();
    let mut per_spot: HashMap<(i64, i64, i64), usize> = HashMap::new();
    for (s, _) in uniq {
        *per_spot.entry(s).or_insert(0) += 1;
    }
    let gone: HashSet<u64> = echoes.iter().filter(|(_, _, s)| per_spot[s] >= 2).map(|(_, id, _)| *id).collect();
    if gone.is_empty() {
        return 0;
    }
    for p in pages.iter_mut() {
        p.blocks.retain(|b| !gone.contains(&b.id));
    }
    gone.len()
}

pub fn join_paragraphs(blocks: Vec<Block>, page_sizes: &HashMap<usize, (f64, f64)>) -> Vec<Block> {
    let mut out: Vec<Block> = Vec::new();
    for b in blocks {
        if let Some(a) = out.last_mut() {
            let ok = b.kind == "paragraph"
                && a.kind == "paragraph"
                && !a.text.is_empty()
                && !b.text.is_empty()
                && !py::rstrip(&a.text).ends_with(|c| TERMINAL.contains(c))
                && (py::str_islower(&b.text.chars().take(1).collect::<String>())
                    || (!a.text.ends_with('*') && !b.text.starts_with('*') && continues(a, &b, page_sizes)));
            if ok {
                a.text = join_lines(&[a.text.clone(), b.text.clone()], vocab_of(a.lines_or_empty()));
                let mut ls = a.lines.clone().unwrap_or_default();
                ls.extend(b.lines.clone().unwrap_or_default());
                a.lines = Some(ls);
                continue;
            }
        }
        out.push(b);
    }
    out
}

fn continues(a: &Block, b: &Block, sizes: &HashMap<usize, (f64, f64)>) -> bool {
    crate::pyre_static!(HYPHEN, ReStart, r".*[a-z\d][-—¬]\s?$");
    let (la, lb) = (a.lines_or_empty(), b.lines_or_empty());
    if la.len() < 2 || lb.is_empty() {
        return false;
    }
    let brk = if b.page == a.page {
        b.bbox[1].floor() <= a.bbox[1].ceil() && b.bbox[0] > a.bbox[2] + 0.02 * (a.bbox[2] - a.bbox[0])
    } else if b.page == a.page + 1 && sizes.contains_key(&b.page) {
        let (w, h) = sizes[&b.page];
        b.bbox[0] < w / 2.0 && b.bbox[1] < h / 2.0
    } else {
        return false;
    };
    if !brk {
        return false;
    }
    let indented = lb[0].bbox[0] > lb.iter().map(|l| l.bbox[0]).fold(f64::INFINITY, f64::min).ceil();
    let full = la[la.len() - 1].bbox[2] >= la.iter().map(|l| l.bbox[2]).fold(f64::NEG_INFINITY, f64::max).floor();
    let last = la[la.len() - 1].text();
    let hyphen = HYPHEN.is(py::strip(&last));
    (full || hyphen) && !indented
}

pub fn merge_split_tables(blocks: Vec<Block>) -> Vec<Block> {
    let mut out: Vec<Block> = Vec::new();
    for b in blocks {
        let mut j = out.len() as isize - 1;
        while j >= 0
            && out[j as usize].kind == "caption"
            && py::lstrip_chars(&out[j as usize].text, "*_ ").to_lowercase().starts_with("fig")
        {
            j -= 1;
        }
        if j >= 0 {
            let prev = &mut out[j as usize];
            let ok = b.kind == "table"
                && prev.kind == "table"
                && b.table.is_some()
                && prev.table.is_some()
                && b.page == prev.page + 1
                && b.table.as_ref().unwrap().n_cols() == prev.table.as_ref().unwrap().n_cols()
                && !py::lstrip_chars(b.caption.as_deref().unwrap_or(""), "*_ ").to_lowercase().starts_with("tab");
            if ok {
                let bt = b.table.as_ref().unwrap();
                let pt = prev.table.as_mut().unwrap();
                let mut rows = bt.rows.clone();
                let ph = pt.header_rows();
                let first_same =
                    !rows.is_empty() && rows[0].iter().map(|c| &c.text).eq(pt.rows[0].iter().map(|c| &c.text));
                if ph > 0 && first_same {
                    rows = rows[ph.min(rows.len())..].to_vec();
                }
                pt.rows.extend(rows);
                if prev.meta.merged_pages.is_empty() {
                    prev.meta.merged_pages.push(prev.page);
                }
                prev.meta.merged_pages.push(b.page);
                continue;
            }
        }
        out.push(b);
    }
    out
}
