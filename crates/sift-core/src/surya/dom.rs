//! A tiny HTML DOM with Python `html.parser` semantics (V0 built its DOM on it): the
//! parser of surya's HTML answers uses it.
//!
//! Tokenizer: CPython 3.14 HTMLParser (convert_charrefs=True): lowercase tag and attribute
//! names, attribute values and text unescaped with `html.unescape`, comments / declarations /
//! processing instructions dropped, `<script>` / `<style>` raw. Tree builder: V0's `_Builder`
//! (implied closes for cells, rows, sections, list items and paragraphs).

use super::entities::{HTML5, INVALID_CHARREFS, INVALID_CODEPOINTS};

pub type NodeId = usize;

#[derive(Clone, Debug)]
pub struct Node {
    pub tag: String, // "" root, "#text"
    pub attrs: Vec<(String, Option<String>)>,
    pub children: Vec<NodeId>,
    pub text: String,
    pub parent: Option<NodeId>,
}

#[derive(Clone, Debug)]
pub struct Dom {
    pub nodes: Vec<Node>,
}

pub const VOID: [&str; 11] = ["br", "hr", "img", "input", "meta", "link", "col", "area", "base", "wbr", "source"];

fn implied_close(tag: &str) -> Option<&'static [&'static str]> {
    Some(match tag {
        "td" | "th" => &["td", "th"],
        "tr" => &["tr", "td", "th"],
        "thead" => &["tr", "td", "th", "tbody"],
        "tbody" => &["tr", "td", "th", "thead"],
        "li" => &["li"],
        "p" => &["p"],
        _ => return None,
    })
}
const SCOPE: [&str; 4] = ["table", "ul", "ol", "div"];

impl Dom {
    pub fn new_root() -> Dom {
        Dom {
            nodes: vec![Node {
                tag: String::new(),
                attrs: vec![],
                children: vec![],
                text: String::new(),
                parent: None,
            }],
        }
    }
    pub const ROOT: NodeId = 0;

    pub fn n(&self, id: NodeId) -> &Node {
        &self.nodes[id]
    }

    pub fn attr(&self, id: NodeId, k: &str) -> Option<&str> {
        self.nodes[id].attrs.iter().rev().find(|(a, _)| a == k).and_then(|(_, v)| v.as_deref())
    }
    pub fn set_attr(&mut self, id: NodeId, k: &str, v: &str) {
        let attrs = &mut self.nodes[id].attrs;
        if let Some(p) = attrs.iter_mut().find(|(a, _)| a == k) {
            p.1 = Some(v.to_string());
        } else {
            attrs.push((k.to_string(), Some(v.to_string())));
        }
    }
    pub fn pop_attr(&mut self, id: NodeId, k: &str) {
        self.nodes[id].attrs.retain(|(a, _)| a != k);
    }

    pub fn add(&mut self, parent: NodeId, tag: &str, attrs: Vec<(String, Option<String>)>, text: String) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(Node { tag: tag.to_string(), attrs, children: vec![], text, parent: Some(parent) });
        self.nodes[parent].children.push(id);
        id
    }

    /// A new detached element (Python `Node("table", children=...)`); children keep their
    /// own parent pointers, as in V0.
    pub fn detached(&mut self, tag: &str, children: Vec<NodeId>) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(Node { tag: tag.to_string(), attrs: vec![], children, text: String::new(), parent: None });
        id
    }

    pub fn find_all(&self, id: NodeId, tags: &[&str], recursive: bool) -> Vec<NodeId> {
        let mut out = Vec::new();
        for &c in &self.nodes[id].children {
            if tags.contains(&self.nodes[c].tag.as_str()) {
                out.push(c);
            }
            if recursive && self.nodes[c].tag != "#text" {
                out.extend(self.find_all(c, tags, true));
            }
        }
        out
    }

    pub fn find(&self, id: NodeId, tag: &str) -> Option<NodeId> {
        self.find_all(id, &[tag], true).into_iter().next()
    }

    pub fn get_text(&self, id: NodeId) -> String {
        let n = &self.nodes[id];
        if n.tag == "#text" {
            return n.text.clone();
        }
        if n.tag == "br" {
            return "\n".into();
        }
        n.children.iter().map(|&c| self.get_text(c)).collect()
    }

    pub fn remove(&mut self, id: NodeId) {
        if let Some(p) = self.nodes[id].parent.take() {
            self.nodes[p].children.retain(|&c| c != id);
        }
    }

    pub fn to_html(&self, id: NodeId) -> String {
        let n = &self.nodes[id];
        if n.tag == "#text" {
            return escape(&n.text, false);
        }
        let inner: String = n.children.iter().map(|&c| self.to_html(c)).collect();
        if n.tag.is_empty() {
            return inner;
        }
        let attrs: String = n
            .attrs
            .iter()
            .map(|(k, v)| match v {
                Some(v) => format!(" {k}=\"{}\"", escape(v, true)),
                None => format!(" {k}"),
            })
            .collect();
        if VOID.contains(&n.tag.as_str()) {
            return format!("<{}{attrs}>", n.tag);
        }
        format!("<{}{attrs}>{inner}</{}>", n.tag, n.tag)
    }
}

/// Python html.escape.
pub fn escape(s: &str, quote: bool) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' if quote => o.push_str("&quot;"),
            '\'' if quote => o.push_str("&#x27;"),
            c => o.push(c),
        }
    }
    o
}

fn lookup(name: &str) -> Option<&'static str> {
    HTML5.binary_search_by(|(k, _)| (*k).cmp(name)).ok().map(|i| HTML5[i].1)
}

/// Python html.unescape.
pub fn unescape(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let b: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] != '&' {
            out.push(b[i]);
            i += 1;
            continue;
        }
        // &(#[0-9]+;?|#[xX][0-9a-fA-F]+;?|[^\t\n\f <&#;]{1,32};?)
        let j = i + 1;
        if j < b.len() && b[j] == '#' {
            let hex = j + 1 < b.len() && (b[j + 1] == 'x' || b[j + 1] == 'X');
            let start = if hex { j + 2 } else { j + 1 };
            let mut k = start;
            while k < b.len() && (if hex { b[k].is_ascii_hexdigit() } else { b[k].is_ascii_digit() }) {
                k += 1;
            }
            if k > start {
                let digits: String = b[start..k].iter().collect();
                let end = if k < b.len() && b[k] == ';' { k + 1 } else { k };
                let num =
                    if hex { u64::from_str_radix(&digits, 16) } else { digits.parse::<u64>() }.unwrap_or(u64::MAX);
                if let Some((_, r)) = INVALID_CHARREFS.iter().find(|(c, _)| *c as u64 == num) {
                    out.push_str(r);
                } else if (0xD800..=0xDFFF).contains(&num) || num > 0x10FFFF {
                    out.push('\u{FFFD}');
                } else if INVALID_CODEPOINTS.contains(&(num as u32)) {
                } else if let Some(c) = char::from_u32(num as u32) {
                    out.push(c);
                }
                i = end;
                continue;
            }
            out.push('&');
            i += 1;
            continue;
        }
        let mut k = j;
        while k < b.len() && k - j < 32 && !matches!(b[k], '\t' | '\n' | '\x0c' | ' ' | '<' | '&' | '#' | ';') {
            k += 1;
        }
        if k == j {
            out.push('&');
            i += 1;
            continue;
        }
        let end = if k < b.len() && b[k] == ';' { k + 1 } else { k };
        let name: String = b[j..end].iter().collect();
        if let Some(r) = lookup(&name) {
            out.push_str(r);
            i = end;
            continue;
        }
        // longest known prefix (entities that work without ';')
        let chars: Vec<char> = name.chars().collect();
        let mut done = false;
        for x in (2..chars.len()).rev() {
            let pre: String = chars[..x].iter().collect();
            if let Some(r) = lookup(&pre) {
                out.push_str(r);
                out.extend(&chars[x..]);
                done = true;
                break;
            }
        }
        if !done {
            out.push('&');
            out.push_str(&name);
        }
        i = end;
    }
    out
}

fn is_ws(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r' | '\x0c' | ' ')
}

/// Parse with V0's _Builder rules.
/// The deepest an element can sit in a parsed tree. The text comes from a model, and the
/// walks over the tree (`html.rs`: inline HTML, Markdown, cell text) recurse once per level:
/// a few thousand nested tags would overflow a thread's stack and abort the whole run. surya's
/// real output nests about 10 deep (table, row, cell, bold, superscript...); Python's version
/// stopped at its recursion limit of 1,000.
pub const MAX_DEPTH: usize = 256;

pub fn parse(html: &str) -> Dom {
    let mut dom = Dom::new_root();
    let mut stack: Vec<NodeId> = vec![Dom::ROOT];
    let s: Vec<char> = html.chars().collect();
    let n = s.len();
    let mut text = String::new();
    let mut i = 0;

    fn flush(dom: &mut Dom, stack: &[NodeId], text: &mut String) {
        if !text.is_empty() {
            let t = unescape(text);
            if !t.is_empty() {
                let top = *stack.last().unwrap();
                dom.add(top, "#text", vec![], t);
            }
            text.clear();
        }
    }

    fn close_implied(dom: &Dom, stack: &mut Vec<NodeId>, tag: &str) {
        let Some(closes) = implied_close(tag) else { return };
        for i in (1..stack.len()).rev() {
            let t = dom.nodes[stack[i]].tag.as_str();
            if SCOPE.contains(&t) && !closes.contains(&t) {
                return;
            }
            if closes.contains(&t) {
                stack.truncate(i);
                return;
            }
        }
    }

    while i < n {
        let c = s[i];
        if c != '<' {
            text.push(c);
            i += 1;
            continue;
        }
        let nx = if i + 1 < n { s[i + 1] } else { '\0' };
        if nx.is_ascii_alphabetic() {
            // start tag: find its end '>' respecting quoted attribute values
            let mut k = i + 1;
            while k < n && !is_ws(s[k]) && s[k] != '/' && s[k] != '>' {
                k += 1;
            }
            let name: String = s[i + 1..k].iter().collect::<String>().to_lowercase();
            let mut attrs: Vec<(String, Option<String>)> = Vec::new();
            let mut self_close = false;
            let mut closed = false;
            loop {
                while k < n && (is_ws(s[k]) || (s[k] == '/' && !(k + 1 < n && s[k + 1] == '>'))) {
                    k += 1;
                }
                if k >= n {
                    break;
                }
                if s[k] == '>' {
                    k += 1;
                    closed = true;
                    break;
                }
                if s[k] == '/' && k + 1 < n && s[k + 1] == '>' {
                    self_close = true;
                    k += 2;
                    closed = true;
                    break;
                }
                let a0 = k;
                k += 1; // first char of a name may be anything but ws / >
                while k < n && !is_ws(s[k]) && s[k] != '/' && s[k] != '=' && s[k] != '>' {
                    k += 1;
                }
                let aname: String = s[a0..k].iter().collect::<String>().to_lowercase();
                let mut j = k;
                while j < n && is_ws(s[j]) {
                    j += 1;
                }
                if j < n && s[j] == '=' {
                    j += 1;
                    while j < n && is_ws(s[j]) {
                        j += 1;
                    }
                    if j < n && (s[j] == '"' || s[j] == '\'') {
                        let q = s[j];
                        let v0 = j + 1;
                        let mut e = v0;
                        while e < n && s[e] != q {
                            e += 1;
                        }
                        let v: String = s[v0..e.min(n)].iter().collect();
                        attrs.push((aname, Some(unescape(&v))));
                        k = (e + 1).min(n);
                    } else {
                        let v0 = j;
                        let mut e = v0;
                        while e < n && !is_ws(s[e]) && s[e] != '>' {
                            e += 1;
                        }
                        let v: String = s[v0..e].iter().collect();
                        attrs.push((aname, Some(unescape(&v))));
                        k = e;
                    }
                } else {
                    attrs.push((aname, None));
                }
            }
            if !closed {
                // an unfinished tag at the end of the input is dropped (HTML5 EOF-in-tag)
                break;
            }
            flush(&mut dom, &stack, &mut text);
            close_implied(&dom, &mut stack, &name);
            let parent = *stack.last().unwrap();
            let id = dom.add(parent, &name, attrs, String::new());
            // past MAX_DEPTH a tag is kept but not entered: its content stays at that level, so
            // the recursive walks over this tree stay shallow whatever the model wrote
            if !self_close && !VOID.contains(&name.as_str()) && stack.len() < MAX_DEPTH {
                stack.push(id);
            }
            i = k;
            if (name == "script" || name == "style") && !self_close {
                // raw text up to the matching end tag
                let rest: String = s[i..].iter().collect();
                let low = rest.to_lowercase();
                let endt = format!("</{name}");
                let (body, adv) = match low.find(&endt) {
                    Some(p) => (rest[..p].to_string(), p),
                    None => (rest.clone(), rest.len()),
                };
                if !body.is_empty() {
                    dom.add(id, "#text", vec![], body.clone());
                }
                i += rest[..adv].chars().count();
            }
            continue;
        }
        if nx == '/' {
            let n2 = if i + 2 < n { s[i + 2] } else { '\0' };
            let mut k = i + 2;
            while k < n && s[k] != '>' {
                k += 1;
            }
            if k >= n {
                break; // unfinished end tag at EOF: dropped
            }
            if n2.is_ascii_alphabetic() {
                let mut e = i + 2;
                while e < k && !is_ws(s[e]) && s[e] != '/' && s[e] != '>' {
                    e += 1;
                }
                let name: String = s[i + 2..e].iter().collect::<String>().to_lowercase();
                flush(&mut dom, &stack, &mut text);
                for j in (1..stack.len()).rev() {
                    if dom.nodes[stack[j]].tag == name {
                        stack.truncate(j);
                        break;
                    }
                }
            }
            // "</>" and "</ ..." are dropped
            i = k + 1;
            continue;
        }
        if nx == '!' || nx == '?' {
            let end = if s[i..].starts_with(&['<', '!', '-', '-']) {
                let rest: String = s[i + 4..].iter().collect();
                match rest.find("-->").or_else(|| rest.find("--!>")) {
                    Some(p) => i + 4 + rest[..p].chars().count() + if rest[p..].starts_with("--!>") { 4 } else { 3 },
                    None => n,
                }
            } else {
                let mut k = i + 2;
                while k < n && s[k] != '>' {
                    k += 1;
                }
                (k + 1).min(n)
            };
            flush(&mut dom, &stack, &mut text);
            i = end;
            continue;
        }
        text.push('<');
        i += 1;
    }
    flush(&mut dom, &stack, &mut text);
    dom
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unescape_like_python() {
        assert_eq!(
            unescape("a &amp; b &lt;c&gt; &#65;&#x42; &nbsp;x &copy2025 &bogus;"),
            "a & b <c> AB \u{a0}x ©2025 &bogus;"
        );
        assert_eq!(unescape("&#128;"), "€");
    }

    #[test]
    fn implied_closes() {
        let d = parse("<table><tr><td>a<td>b<tr><td>c</table>");
        let t = d.find(Dom::ROOT, "table").unwrap();
        assert_eq!(d.find_all(t, &["td"], true).len(), 3);
        // V0's _Builder: a <tr> closes the open cell only, so the new row nests in the old
        // one (checked against V0 2026-11-06; surya's compact decoder adds the </tr> first)
        assert_eq!(d.to_html(Dom::ROOT), "<table><tr><td>a</td><td>b</td><tr><td>c</td></tr></tr></table>");
    }

    /// A model that nests 100,000 tags (a loop gone wrong) gets a tree no deeper than MAX_DEPTH,
    /// so the recursive walks over it finish instead of overflowing the stack.
    #[test]
    fn deep_nesting_is_capped() {
        let html = format!("{}x{}", "<b>".repeat(100_000), "</b>".repeat(100_000));
        let d = parse(&html);
        let (mut depth, mut id) = (0, Dom::ROOT);
        while let Some(&c) = d.n(id).children.iter().find(|&&c| d.n(c).tag == "b") {
            id = c;
            depth += 1;
        }
        assert!(depth <= MAX_DEPTH, "depth {depth}");
        assert!(d.to_html(Dom::ROOT).contains('x'));
        assert!(crate::surya::html::block_to_markdown(&html, None).contains('x'));
    }
}
