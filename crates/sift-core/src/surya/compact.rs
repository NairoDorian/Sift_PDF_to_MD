//! surya 2.5's compact output format -> canonical HTML (surya/inference/compact.py, decode
//! and canonical_tag, Apache-2.0, see NOTICE). A block header is one label token plus four
//! coordinate tokens; `</td>`, `</th>`, `</tr>` and block `</div>` are left out.

use crate::py::Re;

pub const BLOCK_LABELS: [&str; 20] = [
    "Caption",
    "Footnote",
    "Equation-Block",
    "List-Group",
    "Page-Header",
    "Page-Footer",
    "Image",
    "Section-Header",
    "Table",
    "Text",
    "Complex-Block",
    "Code-Block",
    "Form",
    "Table-Of-Contents",
    "Figure",
    "Chemical-Block",
    "Diagram",
    "Bibliography",
    "Blank-Page",
    "Picture",
];

const ATTR_ORDER: [&str; 16] = [
    "type",
    "checked",
    "value",
    "colspan",
    "rowspan",
    "data-cs",
    "data-rs",
    "data-kind",
    "display",
    "class",
    "align",
    "style",
    "border",
    "href",
    "alt",
    "text",
];
const VOID: [&str; 4] = ["br", "hr", "img", "input"];
const OMIT_END: [&str; 3] = ["td", "th", "tr"];

crate::pyre_static!(ATTR_RE, Re, r#"([a-zA-Z_:][-a-zA-Z0-9_:.]*)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+)))?"#);

static DECODE_RE: std::sync::LazyLock<Re> = std::sync::LazyLock::new(|| {
    let labels = BLOCK_LABELS.iter().map(|l| regex_escape(l)).collect::<Vec<_>>().join("|");
    let coords = |p: &str| (0..4).map(|k| format!(r"<\|(?P<{p}{k}>\d{{1,4}})\|>")).collect::<String>();
    Re::new(&format!(
        r"(?P<hdr><\|(?P<label>{labels})\|>{})|(?P<box><\|box\|>{})|(?P<tag><(?P<tclose>/?)(?P<tname>[a-zA-Z][a-zA-Z0-9-]*)(?P<tattrs>(?:\s+[^<>]*?)?)\s*(?P<tself>/?)>)",
        coords("h"),
        coords("b")
    ))
});

fn regex_escape(s: &str) -> String {
    let mut o = String::new();
    for c in s.chars() {
        if "\\.+*?()|[]{}^$#&-~".contains(c) {
            o.push('\\');
        }
        o.push(c);
    }
    o
}

fn parse_attrs(s: &str) -> Vec<(String, Option<String>)> {
    ATTR_RE
        .0
        .captures_iter(s)
        .filter_map(|c| c.ok())
        .map(|m| {
            let name = m.get(1).unwrap().as_str().to_lowercase();
            let val = [2, 3, 4].iter().find_map(|&g| m.get(g).map(|x| x.as_str().to_string()));
            (name, val)
        })
        .collect()
}

fn attr_key(name: &str) -> (usize, String) {
    match ATTR_ORDER.iter().position(|a| *a == name) {
        Some(i) => (i, String::new()),
        None => (ATTR_ORDER.len(), name.to_string()),
    }
}

pub fn canonical_tag(close: bool, name: &str, attrs: &str, selfclose: bool) -> String {
    let name = name.to_lowercase();
    if close {
        return format!("</{name}>");
    }
    let mut pairs = parse_attrs(attrs);
    pairs.sort_by_key(|(k, _)| attr_key(k)); // stable, as Python's sorted
    let parts: Vec<String> = pairs
        .iter()
        .map(|(k, v)| {
            if k == "checked" {
                k.clone()
            } else {
                match v {
                    None => format!("{k}=\"\""),
                    Some(v) => format!("{k}=\"{}\"", v.replace('"', "&quot;")),
                }
            }
        })
        .collect();
    let inner = if parts.is_empty() { name.clone() } else { format!("{name} {}", parts.join(" ")) };
    if VOID.contains(&name.as_str()) || selfclose { format!("<{inner}/>") } else { format!("<{inner}>") }
}

pub fn decode(text: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut stack: Vec<String> = Vec::new();
    let mut in_block = false;

    fn close_cells(out: &mut Vec<String>, stack: &mut Vec<String>, upto: &[&str]) {
        while let Some(top) = stack.last() {
            if OMIT_END.contains(&top.as_str()) && !upto.contains(&top.as_str()) {
                let t = stack.pop().unwrap();
                out.push(format!("</{t}>"));
            } else {
                break;
            }
        }
    }
    fn close_block(out: &mut Vec<String>, stack: &mut Vec<String>, in_block: &mut bool) {
        if *in_block {
            close_cells(out, stack, &[]);
            while stack.last().is_some_and(|t| OMIT_END.contains(&t.as_str())) {
                let t = stack.pop().unwrap();
                out.push(format!("</{t}>"));
            }
            if stack.last().is_some_and(|t| t == "div") {
                stack.pop();
            }
            out.push("</div>".into());
            *in_block = false;
        }
    }

    let mut pos = 0;
    let mut pending_box: Option<String> = None;
    for m in DECODE_RE.0.captures_iter(text).filter_map(|c| c.ok()) {
        let whole = m.get(0).unwrap();
        let seg = &text[pos..whole.start()];
        if !seg.is_empty() {
            out.push(seg.to_string());
        }
        pos = whole.end();
        let mut g = whole.as_str().to_string();
        if m.name("box").is_some() {
            pending_box = Some(
                (0..4)
                    .map(|k| m.name(&format!("b{k}")).unwrap().as_str().parse::<u32>().unwrap().to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
            );
            continue;
        }
        if m.name("hdr").is_some() {
            pending_box = None;
            let label = m.name("label").unwrap().as_str();
            let bx: Vec<String> = (0..4)
                .map(|k| m.name(&format!("h{k}")).unwrap().as_str().parse::<u32>().unwrap().to_string())
                .collect();
            close_block(&mut out, &mut stack, &mut in_block);
            if out.last().is_some_and(|l| !l.ends_with('\n')) {
                out.push("\n".into());
            }
            out.push(format!("<div data-bbox=\"{}\" data-label=\"{label}\">", bx.join(" ")));
            stack.clear();
            stack.push("div".into());
            in_block = true;
            continue;
        }
        let close = !m.name("tclose").unwrap().as_str().is_empty();
        let name = m.name("tname").unwrap().as_str().to_lowercase();
        let selfclose = !m.name("tself").unwrap().as_str().is_empty();
        if let Some(pb) = &pending_box
            && !close
        {
            g = canonical_tag(
                false,
                &name,
                &format!("{} data-bbox=\"{pb}\"", m.name("tattrs").unwrap().as_str()),
                selfclose,
            );
        }
        pending_box = None;
        if !close {
            if name == "td" || name == "th" {
                if let Some(t) = stack.pop_if(|t| *t == "td" || *t == "th") {
                    out.push(format!("</{t}>"));
                }
            } else if name == "tr" {
                close_cells(&mut out, &mut stack, &["table", "thead", "tbody"]);
            } else if name == "thead" || name == "tbody" {
                close_cells(&mut out, &mut stack, &["table"]);
            }
            out.push(g);
            if !VOID.contains(&name.as_str()) && !selfclose {
                stack.push(name);
            }
            continue;
        }
        if name == "table" || name == "thead" || name == "tbody" {
            if name != "table" {
                close_cells(&mut out, &mut stack, &["table", "thead", "tbody"]);
            } else {
                close_cells(&mut out, &mut stack, &["table"]);
            }
        }
        if name == "div" && in_block && stack.len() == 1 && stack[0] == "div" {
            close_block(&mut out, &mut stack, &mut in_block);
            continue;
        }
        if stack.contains(&name) {
            while stack.last().is_some_and(|t| *t != name) {
                let top = stack.pop().unwrap();
                if OMIT_END.contains(&top.as_str()) {
                    out.push(format!("</{top}>"));
                }
            }
            stack.pop();
        }
        out.push(g);
    }
    if pos < text.len() {
        out.push(text[pos..].to_string());
    }
    close_block(&mut out, &mut stack, &mut in_block);
    out.concat()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decodes_headers_and_cells() {
        let s = decode("<|Table|><|1|><|2|><|3|><|4|><table><tr><td>a<td>b<tr><td>c</table>");
        assert_eq!(
            s,
            "<div data-bbox=\"1 2 3 4\" data-label=\"Table\"><table><tr><td>a</td><td>b</td></tr><tr><td>c</td></tr></table></div>"
        );
    }
}
