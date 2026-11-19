//! Heading levels for the whole document (V0 `sift/build/headings.py`).

use crate::build::blocks::Block;
use crate::py::{self};

crate::pyre_static!(NUMBERED, ReStart, r"(?:\\?\*\*)?((?:\d{1,2}|[A-Z])(?:\.\d{1,2}){0,5})\.?\s+\S");
crate::pyre_static!(STRIP_MD, Re, r"[*_\\]");

pub fn numbering_depth(text: &str) -> Option<usize> {
    crate::pyre_static!(LETTER, ReFull, r"[A-Z]");
    let m = NUMBERED.m(text)?;
    let num = m.get(1).unwrap().as_str();
    if LETTER.is(num) {
        return None;
    }
    Some(num.matches('.').count() + 1)
}

/// Why a line labelled as a heading is body text ("lead-in", "sentence", "note"), or None.
pub fn body_text_reason(text: &str) -> Option<&'static str> {
    crate::pyre_static!(NOTE, ReStart, r"notes?\s*:", "i");
    let t0 = STRIP_MD.replace_all(text, "");
    let t = py::strip(&t0);
    if t.is_empty() || numbering_depth(t).is_some() {
        return None;
    }
    let words = py::split_ws(t);
    if t.ends_with(':') {
        return Some("lead-in");
    }
    if t.ends_with('.') && !t.ends_with("..") && words.len() >= 5 {
        return Some("sentence");
    }
    if NOTE.is(t) && words.len() > 1 {
        return Some("note");
    }
    None
}

/// An unnumbered heading sits at most one level below the heading before it.
pub fn fix_nesting(heads: &mut [&mut Block]) -> usize {
    let mut changed = 0;
    let mut prev: Option<i64> = None;
    let (mut gap_at, mut gap): (Option<i64>, i64) = (None, 0);
    for h in heads.iter_mut() {
        let unnumbered = h.level != 1 && numbering_depth(&h.text).is_none();
        if !unnumbered || gap_at.is_some_and(|g| h.level < g) {
            gap_at = None;
            gap = 0;
        }
        if unnumbered {
            let mut level = h.level - gap;
            let cap = prev.map(|p| p + 1).unwrap_or(2);
            if level > cap {
                if gap_at.is_none() {
                    gap_at = Some(h.level);
                }
                gap += level - cap;
                level = cap;
            }
            if level != h.level {
                h.level = level;
                changed += 1;
            }
        }
        prev = Some(h.level);
    }
    changed
}

fn clusters(sizes: &[f64], tol: f64) -> Vec<f64> {
    let mut s = sizes.to_vec();
    s.sort_by(|a, b| py::fcmp(*b, *a));
    let mut out: Vec<Vec<f64>> = Vec::new();
    for v in s {
        if let Some(last) = out.last_mut()
            && (last.last().unwrap() - v).abs() <= tol
        {
            last.push(v);
            continue;
        }
        out.push(vec![v]);
    }
    out.iter().map(|c| py::median(c)).collect()
}

pub fn assign_levels(heads: &mut [&mut Block]) {
    if heads.is_empty() {
        return;
    }
    let depths: Vec<Option<usize>> = heads.iter().map(|h| numbering_depth(&h.text)).collect();
    let mut by_depth: Vec<(usize, Vec<f64>)> = Vec::new(); // insertion-ordered dict
    for (h, d) in heads.iter().zip(&depths) {
        if let Some(d) = d {
            match by_depth.iter_mut().find(|(k, _)| k == d) {
                Some((_, v)) => v.push(h.size),
                None => by_depth.push((*d, vec![h.size])),
            }
        }
    }
    let depth_size: Vec<(usize, f64)> = by_depth.iter().map(|(d, s)| (*d, py::median(s))).collect();
    // max(depth_size.values(), default=0.0)
    let max_numbered =
        if depth_size.is_empty() { 0.0 } else { depth_size.iter().map(|(_, s)| *s).fold(f64::NEG_INFINITY, f64::max) };
    let first_page = heads[0].page;
    let mut title: Option<u64> = None;
    let first: Vec<usize> = (0..heads.len()).filter(|&i| heads[i].page == first_page).collect();
    if let Some(big) = py::first_max_by(first.iter().cloned(), |&i| heads[i].size)
        && heads[big].size > max_numbered + 0.5
        && depths[big].is_none()
    {
        title = Some(heads[big].id);
    }
    let cl_sizes: Vec<f64> =
        heads.iter().zip(&depths).filter(|(h, d)| d.is_none() && Some(h.id) != title).map(|(h, _)| h.size).collect();
    let cl = clusters(&cl_sizes, 0.8);
    for (h, d) in heads.iter_mut().zip(&depths) {
        if Some(h.id) == title {
            h.level = 1;
        } else if let Some(d) = d {
            h.level = (*d as i64 + 1).min(6);
        } else if !depth_size.is_empty() {
            let best = py::first_min_by(depth_size.iter(), |(k, s)| ((s - h.size).abs(), *k)).unwrap().0;
            h.level = (best as i64 + 1).min(6);
        } else {
            let idx =
                if cl.is_empty() { 0 } else { py::first_min_by(0..cl.len(), |&i| (cl[i] - h.size).abs()).unwrap() };
            h.level = (idx as i64 + 2).min(6);
        }
    }
}

/// The heading path at the current point of the document ("3 RF > 3.2 PA").
#[derive(Default, Clone)]
pub struct SectionTracker {
    chain: Vec<(usize, String)>,
}

impl SectionTracker {
    pub fn update(&mut self, text: &str) {
        let t0 = STRIP_MD.replace_all(text, "");
        let t = py::strip(&t0).to_string();
        if t.is_empty() {
            return;
        }
        let d = match numbering_depth(&t) {
            Some(d) => d,
            None => {
                let d = match self.chain.last() {
                    Some((ld, _)) if *ld < 9 => ld + 1,
                    _ => 1,
                };
                if let Some((_, lt)) = self.chain.last()
                    && numbering_depth(lt).is_none()
                {
                    self.chain.pop();
                }
                d
            }
        };
        self.chain.retain(|c| c.0 < d);
        self.chain.push((d, t));
    }

    pub fn current(&self) -> String {
        let n = self.chain.len();
        self.chain[n.saturating_sub(3)..].iter().map(|(_, t)| t.as_str()).collect::<Vec<_>>().join(" > ")
    }
}
