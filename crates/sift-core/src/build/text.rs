//! Text-layer lines -> inline Markdown: paragraphs, lists, tables of contents, code
//! (V0 `sift/build/text.py`, ideas from marker).

use crate::build::blocks::ListItem;
use crate::pdf::textlayer::{BBox, Char, Line, Span, Vocabulary};
use crate::py::{self};

crate::pyre_static!(HTML_LIKE, Re, r"<(?=[A-Za-z/!?])");
crate::pyre_static!(PUB_BULLET, ReStart, r"\s*(?:[•●▪■◦○·‣⁃∙►▸➢✓]\s*|[\-–—*]\s+)");
crate::pyre_static!(PUB_NUMBERED_LINE, ReStart, r"\s*\(?\d{1,3}[.)]\s+\S");
crate::pyre_static!(ORDERED, ReStart, r"\s*(\(?(?:\d{1,3}|[a-zA-Z]|[ivxIVX]{1,4})[.)])(?:\s+|$)");
crate::pyre_static!(LEADERS, Re, r"\s*(?:\.\s*){4,}|\s*(?:…\s*){2,}|\s*_{4,}");

pub const TERMINAL: &str = ".:;!?)]\"”’";

/// BULLET.match(text)
pub fn bullet_match(t: &str) -> Option<usize> {
    PUB_BULLET.m(t).map(|m| m.get(0).unwrap().end())
}
pub fn is_bullet(t: &str) -> bool {
    PUB_BULLET.is(t)
}
pub fn is_numbered_line(t: &str) -> bool {
    PUB_NUMBERED_LINE.is(t)
}

pub fn escape_md(text: &str) -> String {
    crate::pyre_static!(UNDERSCORE, Re, r"(?<![A-Za-z0-9])_|_(?![A-Za-z0-9])");
    crate::pyre_static!(LINK, Re, r"\[(?=[^\]]*\]\()");
    let t = text.replace('\\', "\\\\").replace('*', "\\*").replace('`', "\\`");
    let t = UNDERSCORE.replace_all(&t, r"\_");
    let t = HTML_LIKE.replace_all(&t, "&lt;");
    let t = LINK.replace_all(&t, r"\[");
    t.replace("~~", "\\~\\~").replace('$', "\\$")
}

pub fn escape_line_start(text: &str) -> String {
    crate::pyre_static!(BLOCK, ReStart, r"(#{1,6}\s|>|[-+]\s|\d{1,9}[.)]\s|={3,}|-{3,}|\|)");
    crate::pyre_static!(NUM, ReStart, r"(\d{1,9})([.)])");
    if BLOCK.is(text) {
        if let Some(m) = NUM.m(text) {
            let end = m.get(0).unwrap().end();
            return format!("{}\\{}{}", &m[1], &m[2], &text[end..]);
        }
        return format!("\\{text}");
    }
    text.to_string()
}

fn span_md(span: &Span, block_bold: bool, block_italic: bool) -> String {
    let t = escape_md(&span.text);
    if !py::has_ink(&t) {
        return t;
    }
    if span.superscript {
        return format!("<sup>{}</sup>", py::strip(&t));
    }
    if span.subscript {
        return format!("<sub>{}</sub>", py::strip(&t));
    }
    let core0 = py::strip(&t);
    let lead = &t[..t.len() - py::lstrip(&t).len()];
    let trail = &t[py::rstrip(&t).len()..];
    let mut core = core0.to_string();
    if span.bold && !block_bold {
        core = format!("**{core}**");
    }
    if span.italic && !block_italic {
        core = format!("*{core}*");
    }
    format!("{lead}{core}{trail}")
}

pub fn line_markdown(line: &Line, block_bold: bool, block_italic: bool) -> String {
    crate::pyre_static!(BOLD_JOIN, Re, r"\*\*(\s*)\*\*");
    crate::pyre_static!(SPACES, Re, r"[ \t]+");
    let mut runs: Vec<Span> = Vec::new();
    for s in &line.spans {
        if let Some(p) = runs.last_mut()
            && py::has_ink(&s.text)
            && py::has_ink(&p.text)
            && (p.bold, p.italic, p.superscript, p.subscript) == (s.bold, s.italic, s.superscript, s.subscript)
        {
            p.text.push_str(&s.text);
            p.chars.extend(s.chars.iter().cloned());
            continue;
        }
        runs.push(s.clone());
    }
    let out: String = runs.iter().map(|s| span_md(s, block_bold, block_italic)).collect();
    let out = BOLD_JOIN.replace_all(&out, "${1}");
    py::strip(&SPACES.replace_all(&out, " ")).to_string()
}

fn style(lines: &[Line]) -> (bool, bool) {
    let spans: Vec<&Span> = lines.iter().flat_map(|l| l.spans.iter()).filter(|s| py::has_ink(&s.text)).collect();
    if spans.is_empty() {
        return (false, false);
    }
    let chars: usize = spans.iter().map(|s| py::char_len(&s.text)).sum();
    let bold: usize = spans.iter().filter(|s| s.bold).map(|s| py::char_len(&s.text)).sum();
    let italic: usize = spans.iter().filter(|s| s.italic).map(|s| py::char_len(&s.text)).sum();
    (bold as f64 / chars as f64 > 0.9, italic as f64 / chars as f64 > 0.9)
}

/// Join wrapped lines; a line-end hyphen before a lowercase letter goes unless the document's
/// own words say it is a compound (D28).
pub fn join_lines<S: AsRef<str>>(texts: &[S], vocab: Option<&Vocabulary>) -> String {
    crate::pyre_static!(HYPHEN_END, Re, r"[A-Za-z]-$");
    crate::pyre_static!(WORD, ReStart, r"[A-Za-z]+");
    let mut out = String::new();
    for t in texts {
        let t = py::strip(t.as_ref());
        if t.is_empty() {
            continue;
        }
        if out.is_empty() {
            out = t.to_string();
            continue;
        }
        let first: String = t.chars().take(1).collect();
        if HYPHEN_END.is_match(&out) && py::str_islower(&first) {
            let head = &out[..out.len() - 1]; // the '-' is ASCII
            let before = if py::has_ink(head) { py::last_token(head) } else { "" };
            let keep = match (vocab, WORD.m(t)) {
                (Some(v), Some(m)) => v.keep_hyphen(before, m.get(0).unwrap().as_str()),
                _ => false,
            };
            if !keep {
                out.pop();
            }
            out.push_str(t);
        } else {
            out.push(' ');
            out.push_str(t);
        }
    }
    out
}

pub fn vocab_of(lines: &[Line]) -> Option<&Vocabulary> {
    lines.iter().find_map(|l| l.vocab.as_deref())
}

fn ink(line: &Line) -> Vec<&Char> {
    line.spans.iter().flat_map(|s| s.chars.iter()).filter(|c| !py::is_space(c.c)).collect()
}

pub fn forced_breaks(lines: &[Line]) -> Vec<bool> {
    let n = lines.len();
    let mut out = vec![false; n];
    if n < 2 {
        return out;
    }
    let mut rights: Vec<f64> = lines.iter().map(|l| l.bbox[2]).collect();
    rights.sort_by(|a, b| py::fcmp(*b, *a));
    for i in 0..n - 1 {
        let (cur, nxt) = (&lines[i], &lines[i + 1]);
        let (ic, inx) = (ink(cur), ink(nxt));
        if ic.is_empty() || inx.is_empty() {
            continue;
        }
        let cw = ic.iter().map(|c| c.bbox[2] - c.bbox[0]).sum::<f64>() / ic.len() as f64;
        let right = if n >= 3 && rights[0] - rights[1] > 3.0 * cw { rights[1] } else { rights[0] };
        let mut word: Vec<&Char> = Vec::new();
        'spans: for s in &nxt.spans {
            for c in &s.chars {
                if py::is_space(c.c) {
                    if !word.is_empty() {
                        break 'spans;
                    }
                    continue;
                }
                word.push(c);
            }
        }
        if word.is_empty() {
            continue;
        }
        let first_w = word.iter().map(|c| c.bbox[2]).fold(f64::NEG_INFINITY, f64::max)
            - word.iter().map(|c| c.bbox[0]).fold(f64::INFINITY, f64::min);
        let ct = cur.text();
        let nt = nxt.text();
        let (t_cur, t_nxt) = (py::rstrip(&ct), py::lstrip(&nt));
        let f: String = t_nxt.chars().take(1).collect();
        if py::str_islower(&f) || t_cur.ends_with('-') {
            continue;
        }
        if nxt.bbox[1] < (cur.bbox[1] + cur.bbox[3]) / 2.0 {
            continue;
        }
        out[i] = right - cur.bbox[2] > first_w + 2.0 * cw;
    }
    out
}

pub fn paragraph(lines: &[Line]) -> String {
    let (bold, italic) = style(lines);
    let vocab = vocab_of(lines);
    let mds: Vec<String> = lines.iter().map(|l| line_markdown(l, bold, italic)).collect();
    let mut groups: Vec<Vec<String>> = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    for (md, brk) in mds.into_iter().zip(forced_breaks(lines)) {
        cur.push(md);
        if brk {
            groups.push(std::mem::take(&mut cur));
        }
    }
    groups.push(cur);
    let mut texts: Vec<String> = groups.iter().map(|g| join_lines(g, vocab)).filter(|t| !t.is_empty()).collect();
    if texts.is_empty() {
        return String::new();
    }
    let total: usize = texts.iter().map(|t| py::char_len(t)).sum();
    if bold && total < 300 {
        texts = texts.into_iter().map(|t| format!("**{t}**")).collect();
    } else if italic {
        texts = texts.into_iter().map(|t| format!("*{t}*")).collect();
    }
    texts.iter().map(|t| escape_line_start(t)).collect::<Vec<_>>().join("  \n")
}

pub fn plain(lines: &[Line]) -> String {
    let texts: Vec<String> = lines.iter().map(|l| l.text()).collect();
    join_lines(&texts, vocab_of(lines))
}

pub fn heading_text(lines: &[Line]) -> String {
    crate::pyre_static!(NUM, Re, r"^(\d{1,2}(?:\.\d{1,2}){1,5})(?=[A-Z][a-z])");
    let texts: Vec<String> = lines.iter().map(|l| escape_md(&l.text())).collect();
    let j = join_lines(&texts, vocab_of(lines));
    let t = py::strip(py::strip_chars(&j, "#")).to_string();
    NUM.0.replacen(&t, 1, "${1} ").into_owned()
}

fn graphic_bullet(line: &Line) -> bool {
    let t = line.text();
    let first: String = py::strip(&t).chars().take(1).collect();
    let starts_space = t.chars().next().is_some_and(py::is_space);
    (starts_space && !first.is_empty() && !py::str_islower(&first))
        || t.chars().next().is_some_and(|c| (0xE000..=0xF8FF).contains(&(c as u32)))
}

fn starts_item(lines: &[Line], k: usize, x_min: f64, max_w: f64, gap0: f64) -> bool {
    let (cur, prev) = (&lines[k], &lines[k - 1]);
    let ct = cur.text();
    let first: String = py::strip(&ct).chars().take(1).collect();
    if first.is_empty() || py::str_islower(&first) {
        return false;
    }
    if (cur.bbox[0] - x_min).abs() > 6.0 {
        return false;
    }
    let prev_short = (prev.bbox[2] - x_min) < 0.8 * max_w;
    let big_gap = (cur.bbox[1] - prev.bbox[3]) > gap0 + 0.35 * cur.height().max(1.0);
    let pt = prev.text();
    let pr = py::rstrip(&pt);
    prev_short || big_gap || pr.ends_with('.') || pr.ends_with(';') || pr.ends_with(':')
}

pub fn list_items(lines: &[Line]) -> Vec<ListItem> {
    if lines.is_empty() {
        return vec![];
    }
    let marked: Vec<bool> = lines
        .iter()
        .map(|l| {
            let t = l.text();
            is_bullet(&t) || ORDERED.is(&t) || graphic_bullet(l)
        })
        .collect();
    let max_w = lines.iter().map(|l| l.bbox[2]).fold(f64::NEG_INFINITY, f64::max)
        - lines.iter().map(|l| l.bbox[0]).fold(f64::INFINITY, f64::min);
    let mut gaps: Vec<f64> = lines.windows(2).map(|w| w[1].bbox[1] - w[0].bbox[3]).collect();
    gaps.sort_by(|a, b| py::fcmp(*a, *b));
    let gap0 = if gaps.is_empty() { 0.0 } else { gaps[gaps.len() / 2] };
    let x_min = lines.iter().map(|l| l.bbox[0]).fold(f64::INFINITY, f64::min);
    let marker_x: Vec<f64> = lines.iter().zip(&marked).filter(|(_, m)| **m).map(|(l, _)| l.bbox[0]).collect();
    let base_x = if marker_x.is_empty() { x_min } else { marker_x.iter().cloned().fold(f64::INFINITY, f64::min) };
    let mut items: Vec<ListItem> = Vec::new();
    let (bold, italic) = style(lines);
    let mut text_x: Option<f64> = None;
    for (k, (line, &is_marked)) in lines.iter().zip(&marked).enumerate() {
        let raw = line.text();
        if is_marked {
            let (mut ordered, mut marker) = (false, String::new());
            let cut: usize; // in characters
            if let Some(end) = bullet_match(&raw) {
                cut = raw[..end].chars().count();
            } else if let Some(m) = ORDERED.m(&raw) {
                ordered = true;
                marker = m[1].to_string();
                cut = raw[..m.get(0).unwrap().end()].chars().count();
            } else {
                let lead = py::char_len(&raw) - py::char_len(py::lstrip(&raw));
                let mut c = lead;
                if let Some(ch) = raw.chars().nth(lead)
                    && (0xE000..=0xF8FF).contains(&(ch as u32))
                {
                    c += 1;
                }
                cut = c;
            }
            let body = drop_chars(line, cut);
            text_x = Some(ink_x0(&body, line.bbox[0]));
            let level = py::round_half_even((line.bbox[0] - base_x) / 14.0).max(0.0) as i64;
            items.push(ListItem {
                text: py::strip(&line_markdown(&body, bold, italic)).to_string(),
                level: level.min(4),
                ordered,
                marker,
            });
            continue;
        }
        let md = line_markdown(line, bold, italic);
        if marker_x.is_empty() && (items.is_empty() || starts_item(lines, k, x_min, max_w, gap0)) {
            items.push(ListItem::plain(py::strip(&md).to_string()));
            text_x = Some(line.bbox[0]);
        } else if let Some(last) = items.last_mut() {
            let stacked = text_x.is_some_and(|tx| line.bbox[0] > tx + 6f64.max(0.5 * line.height()));
            last.text = if stacked {
                format!("{}\n{}", last.text, py::strip(&md))
            } else {
                join_lines(&[last.text.clone(), md], line.vocab.as_deref())
            };
        } else {
            items.push(ListItem::plain(py::strip(&md).to_string()));
            text_x = Some(line.bbox[0]);
        }
    }
    items.into_iter().filter(|i| !i.text.is_empty()).collect()
}

/// The line without its first n characters (the list marker), span formatting kept.
pub fn drop_chars(line: &Line, n: usize) -> Line {
    let mut n = n;
    let mut spans: Vec<Span> = Vec::new();
    for s in &line.spans {
        if n == 0 {
            spans.push(s.clone());
            continue;
        }
        let len = py::char_len(&s.text);
        if len <= n {
            n -= len;
            continue;
        }
        let chars = if s.chars.len() == len { s.chars[n..].to_vec() } else { s.chars.clone() };
        let mut ns = s.clone();
        ns.text = s.text.chars().skip(n).collect();
        ns.chars = chars;
        spans.push(ns);
        n = 0;
    }
    while spans.first().is_some_and(|s| !py::has_ink(&s.text)) {
        spans.remove(0);
    }
    if let Some(f) = spans.first_mut() {
        f.text = py::lstrip(&f.text).to_string();
    }
    Line { spans, bbox: line.bbox, vocab: line.vocab.clone() }
}

fn bbox_of(boxes: &[BBox]) -> BBox {
    crate::pdf::textlayer::union(boxes)
}

/// (left, right) halves of a line cut at x, or None.
pub fn split_line_at(line: &Line, x: f64) -> Option<(Line, Line)> {
    let mut left: Vec<Span> = Vec::new();
    let mut right: Vec<Span> = Vec::new();
    for s in &line.spans {
        if !py::has_ink(&s.text) {
            if s.bbox[0] < x {
                left.push(s.clone())
            } else {
                right.push(s.clone())
            }
            continue;
        }
        if s.bbox[2] <= x {
            left.push(s.clone());
            continue;
        }
        if s.bbox[0] >= x {
            right.push(s.clone());
            continue;
        }
        if s.chars.len() != py::char_len(&s.text) {
            return None;
        }
        let k = s
            .chars
            .iter()
            .position(|c| !py::is_space(c.c) && (c.bbox[0] + c.bbox[2]) / 2.0 >= x)
            .unwrap_or(s.chars.len());
        let tchars: Vec<char> = s.text.chars().collect();
        for (side, a, b) in [(0, 0, k), (1, k, s.chars.len())] {
            if a < b {
                let inked: Vec<BBox> = s.chars[a..b].iter().filter(|c| !py::is_space(c.c)).map(|c| c.bbox).collect();
                let mut ns = s.clone();
                ns.text = tchars[a..b].iter().collect();
                ns.chars = s.chars[a..b].to_vec();
                ns.bbox = if inked.is_empty() { s.bbox } else { bbox_of(&inked) };
                ns.pos = s.pos + a as i64;
                if side == 0 { left.push(ns) } else { right.push(ns) }
            }
        }
    }
    let mut halves = Vec::new();
    for mut spans in [left, right] {
        while spans.last().is_some_and(|s| !py::has_ink(&s.text)) {
            spans.pop();
        }
        while spans.first().is_some_and(|s| !py::has_ink(&s.text)) {
            spans.remove(0);
        }
        let inked: Vec<BBox> = spans.iter().filter(|s| py::has_ink(&s.text)).map(|s| s.bbox).collect();
        if inked.is_empty() {
            return None;
        }
        let last = spans.len() - 1;
        for (i, is_first) in [(0usize, true), (last, false)] {
            let s = &spans[i];
            let t = if is_first { py::lstrip(&s.text) } else { py::rstrip(&s.text) }.to_string();
            let len = py::char_len(&s.text);
            let chars = if t != s.text && s.chars.len() == len {
                let a = if is_first { len - py::char_len(py::lstrip(&s.text)) } else { 0 };
                s.chars[a..a + py::char_len(&t)].to_vec()
            } else {
                s.chars.clone()
            };
            let s = &mut spans[i];
            s.text = t;
            s.chars = chars;
        }
        halves.push(Line { spans, bbox: bbox_of(&inked), vocab: line.vocab.clone() });
    }
    let r = halves.pop().unwrap();
    let l = halves.pop().unwrap();
    Some((l, r))
}

fn ink_x0(line: &Line, default: f64) -> f64 {
    for s in &line.spans {
        let ink: Vec<f64> = s.chars.iter().filter(|c| !py::is_space(c.c)).map(|c| c.bbox[0]).collect();
        if !ink.is_empty() {
            return ink.iter().cloned().fold(f64::INFINITY, f64::min);
        }
        if py::has_ink(&s.text) {
            return s.bbox[0];
        }
    }
    default
}

pub fn toc_lines(lines: &[Line]) -> String {
    crate::pyre_static!(WS, Re, r"\s+");
    let mut out = Vec::new();
    for l in lines {
        let t = py::strip(&LEADERS.replace_all(&escape_md(&l.text()), " … ")).to_string();
        let t = WS.replace_all(&t, " ");
        if !t.is_empty() {
            out.push(t);
        }
    }
    out.join("  \n")
}

pub fn code_text(lines: &[Line]) -> String {
    if lines.is_empty() {
        return String::new();
    }
    let left = lines.iter().map(|l| l.bbox[0]).fold(f64::INFINITY, f64::min);
    let width: f64 = lines.iter().map(|l| (l.bbox[2] - l.bbox[0]).max(0.0)).sum();
    let chars: usize = lines.iter().map(|l| py::char_len(py::strip(&l.text()))).sum();
    let cw = if chars > 0 { width / chars as f64 } else { 0.0 };
    lines
        .iter()
        .map(|l| {
            let pad = if cw > 0.0 { ((l.bbox[0] - left) / cw) as i64 } else { 0 };
            format!("{}{}", " ".repeat(pad.max(0) as usize), py::strip(&l.text()))
        })
        .collect::<Vec<_>>()
        .join("\n")
}
