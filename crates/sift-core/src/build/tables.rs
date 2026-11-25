//! Table cross-check against the PDF text (D10, D27; V0 `sift/build/tables.py`).
//!
//! Every number or identifier surya writes in a table must exist in the PDF text inside
//! the table's area: found -> verified; one close unused PDF token -> replaced by the exact
//! PDF value; otherwise kept and reported. Plus: LaTeX cells take the PDF's text, PDF text no
//! table has is put back where unambiguous, values the PDF centres beside sub-rows get one
//! spanning cell, and a geometry score says whether the structure matches the page.

use std::collections::{HashMap, HashSet};

use crate::pdf::textlayer::{Line, Vocabulary};
use crate::py::{self, Re};
use crate::surya::dom::{escape, unescape};
use crate::surya::html::{Cell, Table};

/// Python's Counter with insertion order (iteration order matters in a few places).
#[derive(Default, Clone, Debug)]
pub struct Counter {
    keys: Vec<String>,
    idx: HashMap<String, usize>,
    vals: Vec<i64>,
    /// joins where the upper line is full (pdf_tokens' `full_wraps`)
    pub full_wraps: HashSet<String>,
}

impl Counter {
    pub fn add(&mut self, k: &str, n: i64) {
        match self.idx.get(k) {
            Some(&i) => self.vals[i] += n,
            None => {
                self.idx.insert(k.to_string(), self.keys.len());
                self.keys.push(k.to_string());
                self.vals.push(n);
            }
        }
    }
    pub fn get(&self, k: &str) -> i64 {
        self.idx.get(k).map(|&i| self.vals[i]).unwrap_or(0)
    }
    pub fn contains(&self, k: &str) -> bool {
        self.idx.contains_key(k)
    }
    pub fn keys(&self) -> &[String] {
        &self.keys
    }
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

const EDGE: &str = "()[]{},;:\"'“”‘’";

pub fn norm(tok: &str) -> String {
    let n = py::nfkc(tok);
    n.chars().map(|c| if "−–—‒﹣－‐".contains(c) { '-' } else { c }).filter(|&c| c != ' ').collect()
}

/// Optimal string alignment distance, capped.
pub fn lev(a: &str, b: &str, cap: usize) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > cap {
        return cap + 1;
    }
    let mut prev2: Option<Vec<usize>> = None;
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, &ca) in a.iter().enumerate() {
        let i = i + 1;
        let mut cur = vec![0; b.len() + 1];
        cur[0] = i;
        let mut best = cur[0];
        for (j, &cb) in b.iter().enumerate() {
            let j = j + 1;
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + (ca != cb) as usize);
            if let Some(p2) = &prev2
                && j > 1
                && ca == b[j - 2]
                && a[i - 2] == cb
            {
                cur[j] = cur[j].min(p2[j - 2] + 1);
            }
            best = best.min(cur[j]);
        }
        if best > cap {
            return cap + 1;
        }
        prev2 = Some(std::mem::replace(&mut prev, cur));
    }
    prev[b.len()]
}

fn plain_word(tok: &str) -> bool {
    py::char_len(tok) >= 3 && py::str_isalpha(tok)
}

fn checkable(tok: &str) -> bool {
    if tok.chars().any(|c| "\\{}$^".contains(c)) {
        return false;
    }
    if tok.chars().any(py::is_digit) {
        return true;
    }
    if py::char_len(tok) < 5 || !tok.chars().any(py::is_alpha) {
        return false;
    }
    let camel = tok.chars().skip(1).any(|c| c.is_uppercase()) && tok.chars().any(|c| c.is_lowercase());
    camel || tok.contains('_')
}

pub fn strip_edges(tok: &str) -> &str {
    py::strip_chars(tok, EDGE)
}

/// table_recon.lines_from_textlayer: [(tokens (text, x0, x1), y0, y1)] inside `bbox`.
pub type TLine = (Vec<(String, f64, f64)>, f64, f64);

pub fn lines_from_textlayer(lines: &[Line], bbox: &[f64; 4]) -> Vec<TLine> {
    crate::pyre_static!(LEADER_ONLY, ReStart, r"[.·•…_\-\s]+$");
    let [bx0, by0, bx1, by1] = *bbox;
    let inside = |b: &[f64; 4]| {
        bx0 <= (b[0] + b[2]) / 2.0
            && (b[0] + b[2]) / 2.0 <= bx1
            && by0 <= (b[1] + b[3]) / 2.0
            && (b[1] + b[3]) / 2.0 <= by1
    };
    let mut out = Vec::new();
    for l in lines {
        let lb = l.bbox;
        if lb[2] < bx0 || lb[0] > bx1 || lb[3] < by0 || lb[1] > by1 {
            continue;
        }
        let mut tokens: Vec<(String, f64, f64)> = Vec::new();
        let mut have_chars = false;
        for s in &l.spans {
            if s.chars.is_empty() {
                continue;
            }
            have_chars = true;
            let mut cur: Option<(String, f64, f64)> = None;
            for c in &s.chars {
                if !inside(&c.bbox) {
                    continue;
                }
                let gap = 0.5 * (c.bbox[3] - c.bbox[1]).max(1.0);
                match &mut cur {
                    None => cur = Some((c.c.to_string(), c.bbox[0], c.bbox[2])),
                    Some(t) if c.bbox[0] - t.2 > gap => {
                        tokens.push(cur.take().unwrap());
                        cur = Some((c.c.to_string(), c.bbox[0], c.bbox[2]));
                    }
                    Some(t) => {
                        t.0.push(c.c);
                        t.2 = c.bbox[2];
                    }
                }
            }
            if let Some(t) = cur {
                tokens.push(t);
            }
        }
        if !have_chars {
            for s in &l.spans {
                let t = py::strip(&s.text);
                if !t.is_empty() && inside(&s.bbox) {
                    tokens.push((t.to_string(), s.bbox[0], s.bbox[2]));
                }
            }
        }
        let toks: Vec<(String, f64, f64)> = tokens
            .into_iter()
            .filter_map(|(t, x0, x1)| {
                let t = py::strip(&t).to_string();
                if !t.is_empty() && !LEADER_ONLY.is(&t) {
                    Some((t, py::round_n(x0, 1), py::round_n(x1, 1)))
                } else {
                    None
                }
            })
            .collect();
        if !toks.is_empty() {
            out.push((toks, py::round_n(lb[1], 1), py::round_n(lb[3], 1)));
        }
    }
    out
}

pub fn pdf_tokens(lines: &[Line], bbox: &[f64; 4]) -> Counter {
    let [x0, y0, x1, y1] = *bbox;
    let mut toks = Counter::default();
    for l in lines {
        let cx = (l.bbox[0] + l.bbox[2]) / 2.0;
        let cy = (l.bbox[1] + l.bbox[3]) / 2.0;
        if !(x0 - 2.0 <= cx && cx <= x1 + 2.0 && y0 - 2.0 <= cy && cy <= y1 + 2.0) {
            continue;
        }
        let text = l.text();
        let raw = py::split_ws(&text);
        for w in raw.iter().map(|x| strip_edges(x)).filter(|w| !w.is_empty()) {
            toks.add(w, 1);
        }
        for n in [2usize, 3] {
            if raw.len() < n {
                continue;
            }
            for i in 0..=raw.len() - n {
                let j = strip_edges(&raw[i..i + n].concat()).to_string();
                if !j.is_empty() {
                    toks.add(&j, 0);
                }
            }
        }
    }
    let groups: Vec<(String, f64, f64, f64, f64)> = lines_from_textlayer(lines, bbox)
        .into_iter()
        .flat_map(|(ts, y0, y1)| ts.into_iter().map(move |(t, a, b)| (t, a, b, y0, y1)))
        .collect();
    for (ut, ux0, ux1, uy0, uy1) in &groups {
        let mut col_right: Option<f64> = None;
        for (lt, lx0, lx1, ly0, _) in &groups {
            if !(uy0 + 2.0 < *ly0 && *ly0 <= uy1 + 6.0) || ux1.min(*lx1) - ux0.max(*lx0) <= 0.0 {
                continue;
            }
            let ulast = py::split_ws(ut).last().copied().unwrap_or("").to_string();
            let lfirst = py::split_ws(lt).first().copied().unwrap_or("").to_string();
            let j = strip_edges(&format!("{ulast}{lfirst}")).to_string();
            if !j.is_empty() {
                toks.add(&j, 0);
                if j.matches('-').count() == 1 && ulast.ends_with('-') {
                    toks.add(&j.replace('-', ""), 0);
                }
                if col_right.is_none() {
                    col_right = Some(
                        groups
                            .iter()
                            .filter(|g| g.1 < *ux1 && g.2 > *ux0)
                            .map(|g| g.2)
                            .fold(f64::NEG_INFINITY, f64::max),
                    );
                }
                let width = (lx1 - lx0) * py::char_len(&lfirst) as f64 / py::char_len(lt).max(1) as f64;
                if col_right.unwrap() - ux1 < width {
                    toks.full_wraps.insert(j);
                }
            }
        }
    }
    toks
}

#[derive(Default, Clone, Debug)]
pub struct CheckResult {
    pub verified: usize,
    pub corrected: Vec<(String, String)>,
    pub unresolved: Vec<String>,
    pub ambiguous: Vec<(String, Vec<String>)>,
}

fn cells_mut(table: &mut Table) -> Vec<&mut Cell> {
    table.rows.iter_mut().flat_map(|r| r.iter_mut()).collect()
}

pub fn cross_check(table: &mut Table, lines: &[Line], bbox: &[f64; 4]) -> CheckResult {
    let mut res = CheckResult::default();
    let pdf = pdf_tokens(lines, bbox);
    if pdf.is_empty() {
        return res;
    }
    let mut pdf_norm: HashSet<String> = HashSet::new();
    for p in pdf.keys() {
        pdf_norm.insert(norm(p));
    }
    let page_words: HashSet<String> = lines
        .iter()
        .flat_map(|l| py::split_ws(&l.text()).into_iter().map(|w| norm(strip_edges(w))).collect::<Vec<_>>())
        .collect();
    let mut used = Counter::default();
    {
        let mut surya = Counter::default();
        for c in table.rows.iter().flatten() {
            for t in py::split_ws(&c.text) {
                let t = strip_edges(t);
                if checkable(t) || plain_word(t) {
                    surya.add(t, 1);
                }
            }
        }
        for t in surya.keys() {
            if pdf.contains(t) {
                used.add(t, surya.get(t));
            }
        }
    }
    let vocab: Option<&Vocabulary> = lines.iter().find_map(|l| l.vocab.as_deref());
    for cell in cells_mut(table) {
        let words: Vec<String> = py::split_ws(&cell.text).into_iter().map(String::from).collect();
        let mut joined_at: HashSet<usize> = HashSet::new();
        for (wi, raw) in words.iter().enumerate() {
            let tok = strip_edges(raw).to_string();
            if tok.is_empty() || joined_at.contains(&wi) {
                continue;
            }
            let plain = !checkable(&tok) && plain_word(&tok);
            if !(plain || checkable(&tok)) {
                continue;
            }
            if pdf.contains(&tok) || pdf_norm.contains(&norm(&tok)) || (plain && page_words.contains(&norm(&tok))) {
                res.verified += 1;
                continue;
            }
            if let Some((old, new, ia, ib)) = join_with_neighbour(&words, wi, &pdf) {
                joined_at.extend(ia..=ib);
                replace_phrase_in_cell(cell, &old, &new);
                res.corrected.push((old, new));
                continue;
            }
            let n = norm(&tok);
            let nl = py::char_len(&n);
            let cap = if nl < 8 { 1 } else { 2 };
            let mut scored: Vec<(String, (usize, usize))> = Vec::new();
            let mut piece = false;
            for p in pdf.keys() {
                if plain && !p.chars().any(py::is_alpha) {
                    continue;
                }
                let np = norm(p);
                let npl = py::char_len(&np);
                if nl.saturating_sub(cap) <= npl && npl < nl && npl >= 2 && (n.starts_with(&np) || n.ends_with(&np)) {
                    piece = true;
                    continue;
                }
                if pdf.get(p) - used.get(p) > 0 || (pdf.get(p) == 0 && !used.contains(p)) {
                    let d = lev(&n, &np, cap);
                    if d <= cap {
                        scored.push((p.clone(), (d, if pdf.get(p) > 0 { 0 } else { 1 })));
                    }
                }
            }
            if piece {
                scored.clear();
            }
            let best = scored.iter().map(|s| s.1).min();
            let mut cands: Vec<String> = scored.iter().filter(|s| Some(s.1) == best).map(|s| s.0.clone()).collect();
            cands.sort();
            if cands.len() == 1 {
                let new = cands[0].clone();
                replace_in_cell(cell, &tok, &new);
                used.add(&new, 1);
                res.corrected.push((tok, new));
            } else if cands.is_empty() {
                res.unresolved.push(tok);
            } else {
                cands.truncate(5);
                res.ambiguous.push((tok, cands));
            }
        }
    }
    for cell in cells_mut(table) {
        for (old, new) in wrapped_identifiers(cell, &pdf, vocab) {
            replace_phrase_in_cell(cell, &old, &new);
            res.corrected.push((old, new));
        }
    }
    res
}

fn wrapped_identifiers(cell: &Cell, pdf: &Counter, vocab: Option<&Vocabulary>) -> Vec<(String, String)> {
    crate::pyre_static!(NAME, ReFull, r"[A-Za-z0-9_]+");
    let mut words: Vec<String> = py::split_ws(&cell.text).into_iter().map(String::from).collect();
    let mut last = words.clone();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < words.len() {
        let a = strip_edges(&last[i]).to_string();
        let b = strip_edges(&words[i + 1]).to_string();
        let whole = format!("{}{b}", strip_edges(&words[i]));
        let j = format!("{a}{b}");
        let Some(v) = vocab else { break };
        let tail = a.rsplit('_').next().unwrap_or("");
        let piece = v.token(&a.to_lowercase()) <= 2
            && (py::char_len(&b) <= 1 || py::char_len(tail) <= 1 || v.token(&b.to_lowercase()) <= 1);
        let b_first: String = b.chars().take(1).collect();
        if strip_edges(&words[i]).contains('_')
            && pdf.contains(&a)
            && pdf.contains(&b)
            && pdf.full_wraps.contains(&j)
            && piece
            && NAME.is(&whole)
            && (!py::str_isupper(&b_first) || py::str_isupper(&a))
        {
            out.push((format!("{} {}", words[i], words[i + 1]), format!("{}{}", words[i], words[i + 1])));
            let joined = format!("{}{}", words[i], words[i + 1]);
            words.splice(i..i + 2, [joined]);
            let l1 = last[i + 1].clone();
            last.splice(i..i + 2, [l1]);
            continue;
        }
        i += 1;
    }
    out
}

fn join_with_neighbour(words: &[String], i: usize, pdf: &Counter) -> Option<(String, String, usize, usize)> {
    let mut found: Vec<(String, String, usize, usize)> = Vec::new();
    for n in [2usize, 3] {
        for a in (i as isize - n as isize + 1)..=(i as isize) {
            if a < 0 {
                continue;
            }
            let a = a as usize;
            let b = a + n - 1;
            if b >= words.len() {
                continue;
            }
            // itertools.product(("_", "", "-"), repeat=n-1), in product order
            let seps = ["_", "", "-"];
            let combos: Vec<Vec<&str>> = if n == 2 {
                seps.iter().map(|s| vec![*s]).collect()
            } else {
                seps.iter().flat_map(|x| seps.iter().map(move |y| vec![*x, *y])).collect()
            };
            for sp in combos {
                let mut joined = words[a].clone();
                for (k, w) in words[a + 1..=b].iter().enumerate() {
                    joined.push_str(sp[k]);
                    joined.push_str(w);
                }
                if pdf.contains(strip_edges(&joined)) {
                    found.push((words[a..=b].join(" "), joined, a, b));
                }
            }
        }
        if !found.is_empty() {
            break;
        }
    }
    let uniq: HashSet<&String> = found.iter().map(|f| &f.1).collect();
    if uniq.len() == 1 { found.into_iter().next() } else { None }
}

fn replace_phrase_in_cell(cell: &mut Cell, old: &str, new: &str) {
    cell.text = cell.text.replacen(old, new, 1);
    let (eo, en) = (escape(old, false), escape(new, false));
    if cell.html.contains(&eo) {
        cell.html = cell.html.replacen(&eo, &en, 1);
    } else {
        let parts: Vec<String> = old.split(' ').map(|x| py::re_escape(&escape(x, false))).collect();
        let rx = Re::new(&parts.join(r"\s*(?:<br>)?\s*"));
        if let Some(m) = rx.0.find(&cell.html).ok().flatten() {
            cell.html = format!("{}{en}{}", &cell.html[..m.start()], &cell.html[m.end()..]);
        }
    }
}

fn replace_in_cell(cell: &mut Cell, old: &str, new: &str) {
    let pat = Re::new(&format!(r"(?<!\S){}(?![^\s,;:)\]])", py::re_escape(old)));
    if let Some(m) = pat.0.find(&cell.text).ok().flatten() {
        cell.text = format!("{}{new}{}", &cell.text[..m.start()], &cell.text[m.end()..]);
    }
    let (eo, en) = (escape(old, false), escape(new, false));
    if cell.html.contains(&eo) {
        cell.html = cell.html.replacen(&eo, &en, 1);
    } else {
        let tags = r"(?:</?(?:sub|sup|b|i|u)>)*";
        let rx = eo.chars().map(|ch| py::re_escape(&ch.to_string())).collect::<Vec<_>>().join(tags);
        let re = Re::new(&rx);
        let mut new_html = cell.html.clone();
        if let Some(m) = re.0.find(&cell.html).ok().flatten() {
            new_html = format!("{}{en}{}", &cell.html[..m.start()], &cell.html[m.end()..]);
        }
        if ["sub", "sup", "b", "i", "u"]
            .iter()
            .any(|t| new_html.matches(&format!("<{t}>")).count() != new_html.matches(&format!("</{t}>")).count())
        {
            new_html = escape(&cell.text, false);
        }
        cell.html = new_html;
    }
}

pub fn geometry_score(table: &Table, lines: &[Line], bbox: &[f64; 4]) -> Option<f64> {
    let mut words: HashMap<String, Vec<[f64; 4]>> = HashMap::new();
    for (toks, y0, y1) in lines_from_textlayer(lines, bbox) {
        for (t, x0, x1) in toks {
            for w in py::split_ws(&t) {
                words.entry(strip_edges(w).to_string()).or_default().push([x0, y0, x1, y1]);
            }
        }
    }
    let mut grid: Vec<Vec<Option<[f64; 4]>>> = Vec::new();
    let mut carry: HashSet<(usize, usize)> = HashSet::new();
    for (r, row) in table.rows.iter().enumerate() {
        let mut out = Vec::new();
        let mut c = 0;
        let mut it = row.iter();
        loop {
            if carry.contains(&(r, c)) {
                out.push(None);
                c += 1;
                continue;
            }
            let Some(cell) = it.next() else { break };
            let ws = py::split_ws(&cell.text);
            let first = ws.first().map(|w| strip_edges(w).to_string()).unwrap_or_default();
            let bx = if !first.is_empty() && words.get(&first).map(|v| v.len()) == Some(1) {
                Some(words[&first][0])
            } else {
                None
            };
            for dc in 0..cell.colspan {
                out.push(if dc == 0 && cell.colspan == 1 && cell.rowspan == 1 { bx } else { None });
                // rows past the table's end are never read back
                for dr in 1..cell.rowspan.min(table.rows.len() - r) {
                    carry.insert((r + dr, c + dc));
                }
            }
            c += cell.colspan;
        }
        grid.push(out);
    }
    let (mut ok, mut total) = (0usize, 0usize);
    for row in &grid {
        let anchors: Vec<&[f64; 4]> = row.iter().flatten().collect();
        for w in anchors.windows(2) {
            total += 1;
            ok += (w[0][3].min(w[1][3]) - w[0][1].max(w[1][1]) > 0.0) as usize;
        }
    }
    let ncols = grid.iter().map(|r| r.len()).max().unwrap_or(0);
    for col in 0..ncols {
        let anchors: Vec<&[f64; 4]> = grid.iter().filter_map(|r| r.get(col).and_then(|x| x.as_ref())).collect();
        for w in anchors.windows(2) {
            total += 1;
            ok += (w[0][2].min(w[1][2]) - w[0][0].max(w[1][0]) > -2.0) as usize;
        }
    }
    if total >= 4 { Some(ok as f64 / total as f64) } else { None }
}

/// A position in a table's grid mapped to another: (row, col) -> (row, index) or the reverse.
type GridMap = HashMap<(usize, usize), (usize, usize)>;

/// (row, col) -> covering cell (r, i in row) and cell -> origin (row, col).
fn grid_of(table: &Table) -> (GridMap, GridMap) {
    let mut cover: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
    let mut origin: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
    for (r, row) in table.rows.iter().enumerate() {
        let mut c = 0;
        for (k, cell) in row.iter().enumerate() {
            while cover.contains_key(&(r, c)) {
                c += 1;
            }
            origin.insert((r, k), (r, c));
            // rows past the table's end are never looked up
            for dr in 0..cell.rowspan.min(table.rows.len() - r) {
                for dc in 0..cell.colspan {
                    cover.entry((r + dr, c + dc)).or_insert((r, k));
                }
            }
            c += cell.colspan;
        }
    }
    (cover, origin)
}

pub fn merge_centred_cells(table: &mut Table) -> usize {
    let (cover, origin) = grid_of(table);
    let n_rows = table.rows.len();
    let hdr = table.header_rows();
    let mut merges = 0;
    let mut moved: HashMap<(usize, usize), usize> = HashMap::new();
    let mut dropped: HashSet<(usize, usize)> = HashSet::new();
    let mut new_rowspan: HashMap<(usize, usize), usize> = HashMap::new();
    let cell = |t: &Table, id: (usize, usize)| -> Cell { t.rows[id.0][id.1].clone() };
    let rowspan_of = |t: &Table, id: (usize, usize), nr: &HashMap<(usize, usize), usize>| {
        nr.get(&id).copied().unwrap_or(t.rows[id.0][id.1].rowspan)
    };
    let n_cols = cover.keys().map(|(_, c)| *c).max().map(|m| m + 1).unwrap_or(1);
    for r in hdr..n_rows {
        let Some(&first) = cover.get(&(r, 0)) else { continue };
        let fc = cell(table, first);
        if origin[&first] != (r, 0) || rowspan_of(table, first, &new_rowspan) < 2 || fc.colspan != 1 {
            continue;
        }
        let span: Vec<usize> = (r..n_rows.min(r + rowspan_of(table, first, &new_rowspan))).collect();
        for c in 1..n_cols {
            let cells: Vec<Option<(usize, usize)>> = span.iter().map(|&rr| cover.get(&(rr, c)).copied()).collect();
            if cells.iter().any(|x| x.is_none()) {
                continue;
            }
            let cells: Vec<(usize, usize)> = cells.into_iter().flatten().collect();
            let uniq: HashSet<&(usize, usize)> = cells.iter().collect();
            if uniq.len() != cells.len() {
                continue;
            }
            if cells.iter().any(|&x| {
                rowspan_of(table, x, &new_rowspan) != 1 || cell(table, x).colspan != 1 || dropped.contains(&x)
            }) {
                continue;
            }
            let full: Vec<(usize, usize)> =
                cells.iter().cloned().filter(|&x| py::has_ink(&cell(table, x).text)).collect();
            if full.len() != 1 {
                continue;
            }
            let keep = full[0];
            let empties: Vec<(usize, usize)> = cells.iter().cloned().filter(|&x| x != keep).collect();
            let row_ids =
                |row: usize| -> Vec<(usize, usize)> { (0..table.rows[row].len()).map(|k| (row, k)).collect() };
            let rows_left_empty: Vec<usize> = empties
                .iter()
                .filter(|&&x| row_ids(origin[&x].0).iter().all(|o| dropped.contains(o) || *o == x))
                .map(|x| origin[x].0)
                .collect();
            let kr = origin[&keep].0;
            let keep_row_empty =
                kr != r && row_ids(kr).iter().all(|o| dropped.contains(o) || *o == keep || empties.contains(o));
            if !rows_left_empty.is_empty() || keep_row_empty {
                continue;
            }
            new_rowspan.insert(keep, cells.len());
            moved.insert(keep, r);
            dropped.extend(empties.iter().cloned());
            merges += 1;
        }
    }
    if merges == 0 {
        return 0;
    }
    for (id, rs) in &new_rowspan {
        table.rows[id.0][id.1].rowspan = *rs;
    }
    let mut placed: HashMap<usize, Vec<(usize, Cell)>> = HashMap::new();
    for (ri, row) in table.rows.iter().enumerate() {
        for (k, c) in row.iter().enumerate() {
            let id = (ri, k);
            if dropped.contains(&id) {
                continue;
            }
            let (r0, c0) = origin[&id];
            placed.entry(moved.get(&id).copied().unwrap_or(r0)).or_default().push((c0, c.clone()));
        }
    }
    table.rows = (0..n_rows)
        .map(|r| {
            let mut v = placed.remove(&r).unwrap_or_default();
            v.sort_by_key(|t| t.0);
            v.into_iter().map(|t| t.1).collect()
        })
        .collect();
    merges
}

// ---- LaTeX in cells -----------------------------------------------------------------

const LATEX_SYMBOLS: [(&str, &str); 19] = [
    (r"\Omega", "Ω"),
    (r"\omega", "ω"),
    (r"\mu", "μ"),
    (r"\pm", "±"),
    (r"\times", "×"),
    (r"\cdot", "·"),
    (r"\leq", "≤"),
    (r"\geq", "≥"),
    (r"\le", "≤"),
    (r"\ge", "≥"),
    (r"\approx", "≈"),
    (r"\circ", "°"),
    (r"\degree", "°"),
    (r"\%", "%"),
    (r"\sim", "~"),
    (r"\Delta", "Δ"),
    (r"\alpha", "α"),
    (r"\beta", "β"),
    (r"\lambda", "λ"),
];
const ARROWS: [(&str, &str); 17] = [
    (r"\rightarrow", "→"),
    (r"\leftarrow", "←"),
    (r"\to", "→"),
    (r"\Rightarrow", "⇒"),
    (r"\leftrightarrow", "↔"),
    (r"\infty", "∞"),
    (r"\neq", "≠"),
    (r"\ne", "≠"),
    (r"\ldots", "…"),
    (r"\dots", "…"),
    (r"\cdots", "⋯"),
    (r"\prime", "′"),
    (r"\max", "max"),
    (r"\min", "min"),
    (r"\log", "log"),
    (r"\ln", "ln"),
    (r"\exp", "exp"),
];

/// sorted(keys, key=len, reverse=True) is stable: equal lengths keep dict order.
fn by_len_desc(v: &[(&'static str, &'static str)]) -> Vec<(&'static str, &'static str)> {
    let mut s = v.to_vec();
    // a stable sort on the reversed length: ties keep their order, as Python's does
    s.sort_by_key(|a| std::cmp::Reverse(py::char_len(a.0)));
    s
}

fn delatex(s: &str) -> String {
    crate::pyre_static!(TEXTCMD, Re, r"\\(?:text|mathrm|mathit|mathbf|operatorname|rm)\s*\{([^{}]*)\}");
    crate::pyre_static!(CMD, Re, r"\\([A-Za-z]+)");
    crate::pyre_static!(JUNK, Re, r"[{}_^$\\]|\s+");
    let mut s = s.replace(r"\_", "\x00");
    s = TEXTCMD.replace_all(&s, "${1}");
    for (k, v) in by_len_desc(&LATEX_SYMBOLS) {
        s = s.replace(k, v);
    }
    s = CMD.replace_all(&s, "${1}");
    JUNK.replace_all(&s, "").replace('\x00', "_")
}

pub fn simple_math_html(tex: &str) -> Option<String> {
    crate::pyre_static!(SPACING, Re, r"\\[,;: !]|~");
    crate::pyre_static!(TEXTCMD, Re, r"\\(?:text|mathrm|mathit|mathbf|operatorname|rm|textrm)\s*\{([^{}]*)\}");
    crate::pyre_static!(DEG, Re, r"\^\{?\s*°\s*\}?");
    crate::pyre_static!(SUBG, Re, r"_\{([^{}]*)\}");
    crate::pyre_static!(SUPG, Re, r"\^\{([^{}]*)\}");
    crate::pyre_static!(SUB1, Re, r"_([A-Za-z0-9])");
    crate::pyre_static!(SUP1, Re, r"\^([A-Za-z0-9+\-])");
    crate::pyre_static!(LEFT, Re, r"[{}_^$]");
    crate::pyre_static!(WS, Re, r"\s+");
    let mut s = py::strip(tex).replace(r"\_", "\x00").replace(r"\%", "%").replace(r"\&", "&").replace(r"\#", "#");
    s = SPACING.replace_all(&s, " ");
    let mut all: Vec<(&str, &str)> = LATEX_SYMBOLS.to_vec();
    for a in ARROWS {
        if let Some(e) = all.iter_mut().find(|(k, _)| *k == a.0) {
            e.1 = a.1;
        } else {
            all.push(a);
        }
    }
    for (k, v) in by_len_desc(&all) {
        let re = Re::new(&format!("{}(?![A-Za-z])", py::re_escape(k)));
        s = re.replace_all_fn(&s, |_| v.to_string());
    }
    s = TEXTCMD.replace_all(&s, "${1}");
    s = DEG.replace_all(&s, "°");
    if s.contains('\\') {
        return None;
    }
    s = escape(&s, false);
    for _ in 0..4 {
        let t = SUBG.replace_all(&s, "<sub>${1}</sub>");
        let t = SUPG.replace_all(&t, "<sup>${1}</sup>");
        if t == s {
            break;
        }
        s = t;
    }
    s = SUB1.replace_all(&s, "<sub>${1}</sub>");
    s = SUP1.replace_all(&s, "<sup>${1}</sup>");
    if LEFT.is_match(&s) {
        return None;
    }
    Some(py::strip(&WS.replace_all(&s, " ")).replace('\x00', "_"))
}

fn math_to_html(whole: &str, tex: &str) -> String {
    crate::pyre_static!(HAS, Re, r"[_^\\]");
    if !HAS.is_match(tex) {
        return whole.to_string();
    }
    simple_math_html(tex).unwrap_or_else(|| whole.to_string())
}

fn html_text(html: &str) -> String {
    crate::pyre_static!(INLINE, Re, r"</?(?:sub|sup|b|i|u|small|big)>");
    crate::pyre_static!(TAG, Re, r"<[^>]+>");
    crate::pyre_static!(WS, Re, r"\s+");
    let t = INLINE.replace_all(html, "");
    py::strip(&WS.replace_all(&unescape(&TAG.replace_all(&t, " ")), " ")).to_string()
}

fn symbol_or_math(whole: &str, inner: &str) -> String {
    crate::pyre_static!(TWO, Re, r"[A-Za-z]{2}");
    let sym = delatex(inner);
    if !sym.is_empty() && py::char_len(&sym) <= 3 && sym.chars().any(|c| c as u32 > 127) && !TWO.is_match(&sym) {
        return sym;
    }
    whole.to_string()
}

crate::pyre_static!(MATH200, Re, r"\$([^$]{1,200})\$");
crate::pyre_static!(MATH40, Re, r"\$([^$]{1,40})\$");
crate::pyre_static!(TAGS, Re, r"<[^>]+>");

pub fn cell_plain(cell: &Cell) -> String {
    let h = MATH200.replace_all_fn(&cell.html, |m| delatex(&m[1]));
    unescape(&TAGS.replace_all(&h, " "))
}

fn mkey(s: &str) -> String {
    crate::pyre_static!(WS, Re, r"\s+");
    norm(&WS.replace_all(s, "")).replace('≤', "<=").replace('≥', ">=").replace('≠', "!=").replace('×', "x")
}

pub fn fix_math_cells(table: &mut Table, lines: &[Line], bbox: &[f64; 4]) -> Vec<(String, String)> {
    crate::pyre_static!(SUBSUP, Re, r"<su[bp]>");
    crate::pyre_static!(WS, Re, r"\s+");
    let groups: Vec<String> = lines_from_textlayer(lines, bbox)
        .into_iter()
        .flat_map(|(ts, _, _)| ts.into_iter().map(|t| py::strip(&t.0).to_string()))
        .filter(|t| !t.is_empty())
        .collect();
    // set(groups): iteration order doesn't change the result (scores are compared, cands sorted)
    let uniq: Vec<String> = {
        let mut seen = HashSet::new();
        groups.iter().filter(|g| seen.insert((*g).clone())).cloned().collect()
    };
    let mut fixed = Vec::new();
    for cell in cells_mut(table) {
        if !cell.html.contains('$') {
            continue;
        }
        let want = mkey(&delatex(&unescape(&TAGS.replace_all(&cell.html, ""))));
        let wl = py::char_len(&want);
        let mut scored: Vec<(usize, String)> = Vec::new();
        let mut piece = false;
        let mut cap = 1usize.max(wl / 5);
        if wl >= 2 {
            for g in &uniq {
                let gn = mkey(g);
                let gl = py::char_len(&gn);
                if wl.saturating_sub(cap) <= gl && gl < wl && (want.starts_with(&gn) || want.ends_with(&gn)) {
                    piece = true;
                    continue;
                }
                cap = 1usize.max(wl / 5);
                let d = lev(&want, &gn, cap);
                if d <= cap {
                    scored.push((d, g.clone()));
                }
            }
        }
        if piece {
            scored.clear();
        }
        let best = scored.iter().map(|s| s.0).min();
        let mut cands: Vec<String> = scored
            .iter()
            .filter(|s| Some(s.0) == best)
            .map(|s| s.1.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        cands.sort();
        if cands.len() != 1 {
            let new = MATH40.replace_all_fn(&cell.html, |m| symbol_or_math(&m[0], &m[1]));
            let new = MATH200.replace_all_fn(&new, |m| math_to_html(&m[0], &m[1]));
            if new != cell.html {
                fixed.push((cell.html.clone(), new.clone()));
                cell.text = html_text(&new);
                cell.html = new;
            }
            continue;
        }
        let old = cell.html.clone();
        let as_html = MATH200.replace_all_fn(&cell.html, |m| math_to_html(&m[0], &m[1]));
        let same = WS.replace_all(&html_text(&as_html), "") == WS.replace_all(&cands[0], "");
        if !as_html.contains('$') && SUBSUP.is_match(&as_html) && same {
            cell.html = as_html;
        } else {
            cell.html = escape(&cands[0], false);
        }
        cell.text = cands[0].clone();
        fixed.push((old, cands[0].clone()));
    }
    fixed
}

// ---- PDF text no table has ------------------------------------------------------------

type Group = (String, f64, f64, f64, f64);

fn squash(s: &str) -> String {
    crate::pyre_static!(WS, Re, r"\s+");
    norm(&WS.replace_all(s, ""))
}
fn words_of(s: &str) -> HashSet<String> {
    crate::pyre_static!(W, Re, r"[\p{L}\p{Nl}\p{No}][\p{L}\p{N}_]{2,}");
    W.find_all(&norm(s)).into_iter().map(String::from).collect()
}
fn overlap(a: &Group, b: &Group) -> bool {
    a.2.min(b.2) - a.1.max(b.1) > 0.3 * 1e-6f64.max((a.2 - a.1).min(b.2 - b.1))
}
fn centre(o: &Group) -> f64 {
    (o.3 + o.4) / 2.0
}
fn toks_of(s: &str) -> Vec<String> {
    crate::pyre_static!(T, Re, r"[-+]?\d+(?:[.,]\d+)*\w*|\w+");
    T.find_all(&norm(s)).into_iter().map(String::from).collect()
}
fn br_parts(html: &str) -> Vec<String> {
    crate::pyre_static!(BR, Re, r"<br\s*/?>");
    BR.split(html).into_iter().map(String::from).collect()
}

fn split_swallowed(tables: &mut [Table], groups: &[Group], gi: usize, h: f64) -> bool {
    let g = &groups[gi];
    let t = g.0.clone();
    for table in tables.iter_mut() {
        let (_, origin) = grid_of(table);
        for r0 in 0..table.rows.len() {
            for k in 0..table.rows[r0].len() {
                let x = &table.rows[r0][k];
                if x.rowspan != 2 || x.colspan != 1 || !py::has_ink(&x.text) {
                    continue;
                }
                let first = squash(&unescape(&TAGS.replace_all(&br_parts(&x.html)[0], "")));
                let own: Vec<usize> = (0..groups.len()).filter(|&i| squash(&groups[i].0) == first).collect();
                if own.len() != 1 || !overlap(&groups[own[0]], g) || centre(g) <= centre(&groups[own[0]]) + 0.5 * h {
                    continue;
                }
                let o = &groups[own[0]];
                let between = (0..groups.len()).any(|i| {
                    i != own[0]
                        && i != gi
                        && overlap(&groups[i], g)
                        && centre(o) + 0.25 * h < centre(&groups[i])
                        && centre(&groups[i]) < centre(g) - 0.25 * h
                });
                if between || r0 + 1 >= table.rows.len() {
                    continue;
                }
                let beside = (0..groups.len())
                    .any(|i| i != gi && !overlap(&groups[i], g) && (centre(&groups[i]) - centre(g)).abs() < h);
                if !beside || (g.2 - g.1) > 1.5 * 1f64.max(o.2 - o.1) {
                    continue;
                }
                let c0 = origin[&(r0, k)].1;
                let header = x.header;
                table.rows[r0][k].rowspan = 1;
                let new = Cell { html: escape(&t, false), text: t.clone(), header, colspan: 1, rowspan: 1 };
                let nr = r0 + 1;
                let at = (0..table.rows[nr].len()).find(|&i| origin[&(nr, i)].1 > c0).unwrap_or(table.rows[nr].len());
                table.rows[nr].insert(at, new);
                return true;
            }
        }
    }
    false
}

/// (placed, missing): PDF text in the table area that the tables lack.
pub fn fill_missing(
    tables: &mut [Table],
    lines: &[Line],
    bbox: &[f64; 4],
    present: &[String],
) -> (Vec<String>, Vec<String>) {
    let mut groups: Vec<Group> = Vec::new();
    for (ts, y0, y1) in lines_from_textlayer(lines, bbox) {
        for (t, x0, x1) in ts {
            let s = py::strip(&t).to_string();
            if !s.is_empty() {
                groups.push((s, x0, x1, y0, y1));
            }
        }
    }
    let cell_plains = |tables: &[Table]| -> Vec<String> {
        tables.iter().flat_map(|t| t.rows.iter().flatten()).map(cell_plain).collect()
    };
    let all_text_of = |tables: &[Table]| -> Vec<String> {
        let mut v: Vec<String> = cell_plains(tables).iter().map(|c| squash(c)).collect();
        v.extend(present.iter().map(|p| squash(p)));
        v
    };
    let token_sets = |tables: &[Table]| -> (HashSet<String>, HashSet<(String, String)>) {
        let mut stream: Vec<String> = Vec::new();
        for s in cell_plains(tables).iter().chain(present.iter()) {
            stream.extend(toks_of(s));
        }
        let pairs = stream.windows(2).map(|w| (w[0].clone(), w[1].clone())).collect();
        (stream.into_iter().collect(), pairs)
    };
    let similar_line = |tables: &[Table], sq: &str| -> bool {
        let cap = 1usize.max(py::char_len(sq) / 5);
        for c in tables.iter().flat_map(|t| t.rows.iter().flatten()) {
            for part in br_parts(&c.html) {
                let p = squash(&unescape(&TAGS.replace_all(&part, "")));
                if !p.is_empty() && lev(&p, sq, cap) <= cap {
                    return true;
                }
            }
        }
        false
    };
    let mut all_text = all_text_of(tables);
    let (mut have1, mut have2) = token_sets(tables);
    let mut counts: HashMap<String, usize> = HashMap::new();
    for g in &groups {
        *counts.entry(g.0.clone()).or_insert(0) += 1;
    }
    let (mut placed, mut missing) = (Vec::new(), Vec::new());
    for gi in 0..groups.len() {
        let g = groups[gi].clone();
        let t = g.0.clone();
        let sq = squash(&t);
        if py::char_len(&sq) < 2 || !sq.chars().any(py::is_alnum) || all_text.iter().any(|a| a.contains(&sq)) {
            continue;
        }
        let tk = toks_of(&t);
        if tk.is_empty() {
            continue;
        }
        if tk.len() == 1 {
            if have1.contains(&tk[0]) {
                continue;
            }
        } else {
            let hits = tk.windows(2).filter(|w| have2.contains(&(w[0].clone(), w[1].clone()))).count();
            if hits as f64 >= 0.5 * (tk.len() - 1) as f64 {
                continue;
            }
        }
        if counts[&t] != 1 || similar_line(tables, &sq) {
            missing.push(t);
            continue;
        }
        let h = 1f64.max(g.4 - g.3);
        if split_swallowed(tables, &groups, gi, h) {
            all_text = all_text_of(tables);
            (have1, have2) = token_sets(tables);
            placed.push(t);
            continue;
        }
        let near: Vec<usize> = (0..groups.len())
            .filter(|&i| {
                i != gi && overlap(&groups[i], &g) && {
                    let d = (centre(&groups[i]) - centre(&g)).abs();
                    d > 0.0 && d <= 1.25 * h
                }
            })
            .collect();
        let mut owners: Vec<(usize, (usize, usize, usize))> = Vec::new(); // (group, (table, row, col))
        for &oi in &near {
            let o = &groups[oi];
            let so = squash(&o.0);
            let mut oc = Vec::new();
            for (ti, tb) in tables.iter().enumerate() {
                for (ri, row) in tb.rows.iter().enumerate() {
                    for (ci, c) in row.iter().enumerate() {
                        if !so.is_empty() && squash(&cell_plain(c)).contains(&so) {
                            oc.push((ti, ri, ci));
                        }
                    }
                }
            }
            if oc.len() != 1 {
                continue;
            }
            let peers = (0..groups.len()).filter(|&p| p != oi && (centre(&groups[p]) - centre(o)).abs() < 0.5 * h);
            if peers.into_iter().any(|p| overlap(&groups[p], &g)) {
                continue;
            }
            owners.push((oi, oc[0]));
        }
        let uniq: HashSet<(usize, usize, usize)> = owners.iter().map(|o| o.1).collect();
        if uniq.len() != 1 {
            missing.push(t);
            continue;
        }
        let (oi, (ti, ri, ci)) = owners[0];
        let esc = escape(&t, false);
        let above = centre(&groups[oi]) < centre(&g);
        let cell = &mut tables[ti].rows[ri][ci];
        if above {
            cell.text = py::strip(&format!("{} {t}", cell.text)).to_string();
            cell.html = if cell.html.is_empty() { esc } else { format!("{}<br>{esc}", cell.html) };
        } else {
            cell.text = py::strip(&format!("{t} {}", cell.text)).to_string();
            cell.html = if cell.html.is_empty() { esc } else { format!("{esc}<br>{}", cell.html) };
        }
        all_text = all_text_of(tables);
        (have1, have2) = token_sets(tables);
        placed.push(t);
    }
    let _ = words_of; // table_words is computed by V0 but never read
    (placed, missing)
}

pub fn report_lines(where_: &str, res: &CheckResult) -> Vec<String> {
    let mut out = Vec::new();
    if res.unresolved.len() > 3 {
        let shown = res.unresolved.iter().take(20).map(|t| py::repr_str(t)).collect::<Vec<_>>().join(", ");
        let more =
            if res.unresolved.len() > 20 { format!(" and {} more", res.unresolved.len() - 20) } else { String::new() };
        out.push(format!(
            "{where_}: {} values surya wrote have no match in the PDF text (kept surya's values; part of the table may be an image): {shown}{more}",
            res.unresolved.len()
        ));
    } else {
        for tok in &res.unresolved {
            out.push(format!(
                "{where_}: surya wrote {}; no matching value in the PDF text (kept surya's value)",
                py::repr_str(tok)
            ));
        }
    }
    if res.ambiguous.len() > 3 {
        let shown = res
            .ambiguous
            .iter()
            .take(10)
            .map(|(t, c)| format!("{} ~ {}", py::repr_str(t), py::repr_list(c)))
            .collect::<Vec<_>>()
            .join("; ");
        let more =
            if res.ambiguous.len() > 10 { format!(" and {} more", res.ambiguous.len() - 10) } else { String::new() };
        out.push(format!(
            "{where_}: {} values surya wrote are close to several PDF values (kept surya's): {shown}{more}",
            res.ambiguous.len()
        ));
    } else {
        for (tok, cands) in &res.ambiguous {
            out.push(format!(
                "{where_}: surya wrote {}; several close PDF values {} (kept surya's value)",
                py::repr_str(tok),
                py::repr_list(cands)
            ));
        }
    }
    out
}

pub fn empty_share(table: &Table) -> f64 {
    let cells: Vec<&Cell> = table.rows.iter().flatten().collect();
    cells.iter().filter(|c| !py::has_ink(&c.text)).count() as f64 / cells.len().max(1) as f64
}

// ---- text surya read outside a table ----------------------------------------------------

/// A word of running text: the table check's edges plus the sentence marks.
const TEXT_EDGE: &str = "()[]{},;:\"'“”‘’.?!";

/// A word and its parts around `-` and `/` (`pull-up/pull-down`), each with its edges off.
fn word_and_parts(raw: &str) -> (String, Vec<String>) {
    let whole = py::strip_chars(raw, TEXT_EDGE).to_string();
    let parts = whole
        .split(['-', '/', '–', '—'])
        .map(|p| py::strip_chars(p, TEXT_EDGE).to_string())
        .filter(|p| !p.is_empty())
        .collect();
    (whole, parts)
}

/// The words of a text surya read (the `ocr` route: a text block whose PDF text is missing
/// or broken, on a page that has a text layer) that the PDF's own text of that page does not
/// have. Numbers, identifiers and plain words are looked up as the table check looks up a
/// cell's words (exact, or equal after `norm`), a compound also by its parts. In reading
/// order, each word once.
///
/// It cannot prove surya right (the block is there because the PDF gives no usable text for
/// it), but a text surya made up shows here: asked to read a blank or half-cut strip, it
/// answered "Table 2. Comparison of the P. macrophylla population in the three regions of
/// the study" on a page about strapping pins (docs/LEARNINGS.md, "A page keeps its size").
pub fn words_not_on_page(text: &str, lines: &[Line]) -> Vec<String> {
    let mut page: HashSet<String> = HashSet::new();
    for l in lines {
        for raw in py::split_ws(&l.text()) {
            let (whole, parts) = word_and_parts(raw);
            page.insert(norm(&whole));
            page.extend(parts.iter().map(|p| norm(p)));
        }
    }
    let known = |t: &str| page.contains(t) || page.contains(&norm(t));
    let mut out: Vec<String> = Vec::new();
    for raw in py::split_ws(text) {
        let (whole, parts) = word_and_parts(raw);
        if whole.is_empty() || known(&whole) {
            continue;
        }
        for p in if parts.len() > 1 { parts } else { vec![whole] } {
            if (checkable(&p) || plain_word(&p)) && !known(&p) && !out.contains(&p) {
                out.push(p);
            }
        }
    }
    out
}

/// The report line for `words_not_on_page` (None: every word is on the page).
pub fn text_report_line(where_: &str, unknown: &[String]) -> Option<String> {
    if unknown.is_empty() {
        return None;
    }
    let shown = unknown.iter().take(12).map(|t| py::repr_str(t)).collect::<Vec<_>>().join(", ");
    let more = if unknown.len() > 12 { format!(" and {} more", unknown.len() - 12) } else { String::new() };
    Some(format!(
        "{where_}: surya read {} word{} that the PDF text of this page does not have (kept surya's reading; \
         the PDF gives no usable text there): {shown}{more}",
        unknown.len(),
        if unknown.len() == 1 { "" } else { "s" }
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lev_swaps_once() {
        assert_eq!(lev("kpbs", "kbps", 3), 1);
        assert_eq!(lev("abc", "abd", 3), 1);
        assert_eq!(lev("abc", "xyzuvw", 2), 3);
    }
    #[test]
    fn simple_math() {
        assert_eq!(simple_math_html(r"V_{DD}").as_deref(), Some("V<sub>DD</sub>"));
        assert_eq!(simple_math_html(r"\frac{a}{b}"), None);
    }

    fn text_line(text: &str) -> Line {
        use crate::pdf::textlayer::Span;
        let bbox = [0.0, 0.0, 100.0, 10.0];
        let span = Span {
            text: text.to_string(),
            bbox,
            font: String::new(),
            weight: 400,
            bold: false,
            italic: false,
            superscript: false,
            subscript: false,
            chars: vec![],
            pos: 0,
        };
        Line { spans: vec![span], bbox, vocab: None }
    }

    /// ESP32-S31.pdf page 36: what surya answered for a blank strip and for a half-cut line,
    /// against the page's own text.
    #[test]
    fn text_surya_made_up_is_not_on_the_page() {
        let page: Vec<Line> = [
            "3.2 VDD_SPI Voltage Control",
            "The default values of the strapping pins, namely the logic levels, are determined by the pins’ internal weak",
            "pull-up/pull-down resistors at reset if the pins are not connected to any circuit, or connected to an external",
            "high-impedance circuit.",
            "Table 3-1. Default Configuration of Strapping Pins",
            "GPIO60 Weak pull-up 1",
        ]
        .iter()
        .map(|t| text_line(t))
        .collect();
        // read right: nothing to report, with or without the sentence marks, compounds by their parts
        assert!(words_not_on_page("Table 3-1. Default Configuration of Strapping Pins", &page).is_empty());
        assert!(
            words_not_on_page("connected to an external high-impedance circuit, pull-down (weak).", &page).is_empty()
        );
        // a blank strip
        assert_eq!(
            words_not_on_page(
                "Table 2. Comparison of the P. macrophylla population in the three regions of the study",
                &page
            ),
            ["2", "Comparison", "macrophylla", "population", "three", "regions", "study"]
        );
        // a line cut through the middle, and a compound that is on no line
        assert_eq!(words_not_on_page("3.2 VDD_ST voltage Control", &page), ["VDD_ST", "voltage"]);
        assert_eq!(words_not_on_page("24. Slip-Dot-Mach-Cont", &page), ["24", "Slip", "Dot", "Mach", "Cont"]);
        let line = text_report_line("page 36, text", &words_not_on_page("3.2 VDD_ST voltage Control", &page)).unwrap();
        assert!(line.starts_with("page 36, text: surya read 2 words that the PDF text of this page does not have"));
        assert!(line.ends_with(": 'VDD_ST', 'voltage'"));
        assert_eq!(text_report_line("page 36, text", &[]), None);
    }
}
