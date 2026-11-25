//! V0's equivalents of marker's `use_llm` processors (D25, `sift/refine/passes.py`), each with
//! its accuracy guard. Prompts adapted from marker 2.1.0 (Apache-2.0, see NOTICE), word for
//! word as V0 sends them (answer-store keys depend on it).

use std::collections::HashMap;

use serde_json::{Value, json};

use crate::answers::AnswerStore;
use crate::build::blocks::Block;
use crate::build::headings::numbering_depth;
use crate::build::tables as xcheck;
use crate::imaging::Rgb;
use crate::llm::LlamaServer;
use crate::pdf::textlayer::Line;
use crate::py::{self};
use crate::render::markdown::html_table;
use crate::surya::html::{Table, math_latex, parse_table};

pub const MAX_REVIEW_ROWS: usize = 60;

pub const TABLE_PROMPT: &str = "You are a text correction expert specializing in accurately reproducing text from images.
You will receive an image of a table and an html representation of that table.
Your task is to correct any errors in the html representation. The html representation should be as faithful to the original table image as possible. The table image may be rotated, but the html representation must not be rotated. Include the full table, with the opening and closing table tags.

Guidelines:
- Reproduce the original values from the image exactly, character for character. Never guess a value.
- Fix stray characters in the html that don't match the image.
- Make sure column headers sit above the correct column values, and merged cells use colspan/rowspan.
- Inline math goes in <math> tags.
- Only use the tags table, tr, th, td, br, span, sup, sub, i, b and math, and only the attributes colspan and rowspan.

Steps:
1. Compare the image with the html, paying special attention to the column headers matching the right values. Write this comparison in \"comparison\".
2. If the html is completely correct, or you cannot read the image properly, set \"needs_corrections\" to false and leave \"corrected_html\" empty.
3. Otherwise set \"needs_corrections\" to true and write the full corrected table in \"corrected_html\".
4. Give a score from 1 to 5 for how well the final html matches the image (5 = perfect).

html representation:
```html
{html}
```";

fn table_schema() -> Value {
    json!({"type": "object", "properties": {"comparison": {"type": "string"}, "needs_corrections": {"type": "boolean"},
        "corrected_html": {"type": "string"}, "score": {"type": "integer"}},
        "required": ["comparison", "needs_corrections", "corrected_html", "score"]})
}

pub struct Ctx<'a> {
    pub server: Option<&'a LlamaServer>,
    pub store: &'a AnswerStore,
    pub think: bool,
    /// Gemma's parallel slots (independent requests run side by side)
    pub slots: usize,
}

fn checkable_tokens(t: &Table) -> HashMap<String, i64> {
    let mut m = HashMap::new();
    for c in t.rows.iter().flatten() {
        for w in py::split_ws(&c.text) {
            let tok = xcheck::strip_edges(w);
            if !tok.is_empty() && is_checkable(tok) {
                *m.entry(tok.to_string()).or_insert(0) += 1;
            }
        }
    }
    m
}

fn is_checkable(tok: &str) -> bool {
    // tables::checkable is private there: same rule
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

fn same_table(a: &Table, b: &Table) -> bool {
    let shape = |t: &Table| -> Vec<Vec<(String, usize, usize)>> {
        t.rows.iter().map(|r| r.iter().map(|c| (py::squash_ws(&c.text), c.colspan, c.rowspan)).collect()).collect()
    };
    shape(a) == shape(b)
}

/// review_table: a report line when the table changed (or a correction was refused).
pub fn review_table(ctx: &Ctx, block: &mut Block, page_lines: &[Line], scanned: bool, where_: &str) -> Option<String> {
    let png = block.meta.png.clone()?;
    if let Some(t) = &block.table
        && t.rows.len() > MAX_REVIEW_ROWS
    {
        return None;
    }
    let mut html = block.table.as_ref().map(html_table).unwrap_or_else(|| "<table></table>".into());
    if block.table.is_none() && !page_lines.is_empty() {
        let txt: String = block
            .lines_or_empty()
            .iter()
            .map(|l| l.text())
            .collect::<Vec<_>>()
            .join(" | ")
            .chars()
            .take(3000)
            .collect();
        html = format!("<table></table>\n(The table could not be read; its text, line by line: {txt})");
    }
    let images = vec![((*png).clone(), png.png(true))];
    let (ans, _) = crate::brain::ask_json(
        ctx.server,
        ctx.store,
        &TABLE_PROMPT.replace("{html}", &html),
        &table_schema(),
        &images,
        4096,
        "table",
        false,
    );
    let ans = ans?;
    if !ans["needs_corrections"].as_bool().unwrap_or(false) {
        return None;
    }
    let new = parse_table(ans["corrected_html"].as_str().unwrap_or(""))?;
    if new.rows.iter().map(|r| r.len()).sum::<usize>() <= 1 {
        return None;
    }
    let score = ans["score"].as_i64().unwrap_or(0);
    if xcheck::empty_share(&new) > 0.4 {
        return Some(format!("{where_}: LLM review produced a mostly empty table; kept the checked version"));
    }
    if scanned || page_lines.is_empty() {
        if score < 4 {
            return None;
        }
        block.table = Some(new);
        block.kind = "table".into();
        return Some(format!(
            "{where_}: corrected by the LLM review (scanned page, values could not be checked against a text layer; score {score}/5)"
        ));
    }
    let mut trial = new.clone();
    let res = xcheck::cross_check(&mut trial, page_lines, &block.bbox);
    if !res.unresolved.is_empty() || !res.ambiguous.is_empty() {
        return Some(format!(
            "{where_}: LLM review suggested changes that are not in the PDF text; kept the checked version"
        ));
    }
    let pdf = xcheck::pdf_tokens(page_lines, &block.bbox);
    let before: HashMap<String, i64> = block
        .table
        .as_ref()
        .map(|t| checkable_tokens(t).into_iter().filter(|(k, _)| pdf.contains(k)).collect())
        .unwrap_or_default();
    let after = checkable_tokens(&trial);
    if before.iter().any(|(k, v)| after.get(k).copied().unwrap_or(0) < *v) {
        return Some(format!("{where_}: LLM review would drop verified values; kept the checked version"));
    }
    let g_new = xcheck::geometry_score(&trial, page_lines, &block.bbox);
    let g_old = block.table.as_ref().and_then(|t| xcheck::geometry_score(t, page_lines, &block.bbox));
    if g_new.is_none_or(|g| g < 0.9 || g_old.is_some_and(|o| g < o)) {
        return Some(format!(
            "{where_}: LLM review changed the structure away from the PDF layout; kept the checked version"
        ));
    }
    if let Some(t) = &block.table
        && same_table(&trial, t)
    {
        return None;
    }
    let (_, still) = xcheck::fill_missing(&mut [trial.clone()], page_lines, &block.bbox, &[]);
    let before_missing = match &block.table {
        Some(t) => xcheck::fill_missing(&mut [t.clone()], page_lines, &block.bbox, &[]).1,
        None => vec![],
    };
    if !before_missing.is_empty() && before_missing.iter().all(|m| still.contains(m)) {
        return Some(format!("{where_}: LLM review did not restore the missing PDF text; kept the checked version"));
    }
    block.table = Some(trial);
    block.kind = "table".into();
    Some(format!("{where_}: corrected by the LLM review (values and layout checked against the PDF)"))
}

pub const HEADING_PROMPT: &str = "You are an expert at document structure. Below are section headings of one technical document, in reading order, with their page and font height (points).
Numbered headings (like \"3.2 Power\") already have the correct level: level = number of numbering parts + 1 (so \"3\" is 2, \"3.2\" is 3).
Font height of the numbered headings, by level: {ref}
The document's numbered chapters (level 2), with their page: {outline}
Decide the correct level of each UNNUMBERED heading (marked \"?\"), so that it nests correctly under the heading it belongs to. Use the font height, the headings around it and what the heading means:
- The main headings of the front matter before the first numbered chapter (features, description, ordering information, contents, lists of figures or tables, revision history) and of the back matter after the chapters (glossary, appendix, index, legal or important notices) are level 2, like the chapters, even when their font is smaller. Headings that belong to one of these parts (the sheets or sub-sections of a glossary or appendix) nest one level below it.
- A heading inside a numbered section is one level deeper than that section, or deeper.
{title_rule}
Return only the unnumbered headings, by their index \"i\".

Headings:
{rows}";
const TITLE_RULE: &str = "- The document title (on page 1) is level 1; nothing else is level 1.";
const NO_TITLE_RULE: &str = "- Level 1 is not used (the document title is already set).";

fn heading_schema() -> Value {
    json!({"type": "object", "properties": {"levels": {"type": "array", "items": {"type": "object",
        "properties": {"i": {"type": "integer"}, "level": {"type": "integer"}}, "required": ["i", "level"]}}}, "required": ["levels"]})
}

fn runs(unnumbered: &[usize], is_numbered: &dyn Fn(usize) -> bool) -> Vec<Vec<usize>> {
    let mut out: Vec<Vec<usize>> = Vec::new();
    for &i in unnumbered {
        if let Some(last) = out.last_mut()
            && !((last[last.len() - 1] + 1)..i).any(is_numbered)
        {
            last.push(i);
            continue;
        }
        out.push(vec![i]);
    }
    out
}

fn heading_chunks(unnumbered: &[usize], is_numbered: &dyn Fn(usize) -> bool, chunk: usize) -> Vec<Vec<usize>> {
    let mut rs = runs(unnumbered, is_numbered);
    if rs.len() > 1 {
        let end = unnumbered[unnumbered.len() - 1] + 1;
        let first_num = (0..end).find(|&j| is_numbered(j));
        let last_num = (0..end).filter(|&j| is_numbered(j)).max();
        if let (Some(f), Some(l)) = (first_num, last_num) {
            let outside: Vec<Vec<usize>> = rs.iter().filter(|r| r[r.len() - 1] < f || r[0] > l).cloned().collect();
            if !outside.is_empty() {
                let mut v = vec![outside.iter().flatten().cloned().collect::<Vec<_>>()];
                v.extend(rs.into_iter().filter(|r| !outside.contains(r)));
                rs = v;
            }
        }
    }
    let mut chunks: Vec<Vec<usize>> = Vec::new();
    for run in rs {
        for piece in run.chunks(chunk) {
            if let Some(last) = chunks.last_mut()
                && last.len() + piece.len() <= chunk
            {
                last.extend_from_slice(piece);
                continue;
            }
            chunks.push(piece.to_vec());
        }
    }
    chunks
}

pub const VOTE_CHUNKS: [usize; 3] = [12, 16, 24];

/// Re-level unnumbered headings (V0 heading_levels): one run per group size, a level kept when
/// most runs agree. Returns (index, old, new).
pub fn heading_levels(ctx: &Ctx, heads: &mut [&mut Block]) -> Vec<(usize, i64, i64)> {
    crate::pyre_static!(MD, Re, r"[*_]");
    let n = heads.len();
    let numbered: Vec<bool> = heads.iter().map(|h| numbering_depth(&h.text).is_some()).collect();
    let is_numbered = |j: usize| numbered[j];
    let mut by_level: Vec<(i64, Vec<f64>)> = Vec::new();
    for h in heads.iter() {
        if numbering_depth(&h.text).is_some() {
            match by_level.iter_mut().find(|(l, _)| *l == h.level) {
                Some((_, v)) => v.push(h.size),
                None => by_level.push((h.level, vec![h.size])),
            }
        }
    }
    by_level.sort_by_key(|(l, _)| *l);
    let r = by_level
        .iter()
        .map(|(lv, v)| {
            let mut s = v.clone();
            s.sort_by(|a, b| py::fcmp(*a, *b));
            format!("{lv}: {}", py::fmt_fixed(s[s.len() / 2], 1))
        })
        .collect::<Vec<_>>()
        .join(", ");
    let reff = if r.is_empty() { "none".to_string() } else { r };
    let mut chapters: Vec<usize> = (0..n).filter(|&i| numbering_depth(&heads[i].text) == Some(1)).collect();
    if chapters.len() > 40 {
        let mut c = chapters[..20].to_vec();
        c.extend_from_slice(&chapters[chapters.len() - 20..]);
        chapters = c;
    }
    let ol = chapters
        .iter()
        .map(|&i| {
            format!(
                "p{} {}",
                heads[i].page + 1,
                MD.replace_all(&heads[i].text, "").chars().take(40).collect::<String>()
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    let outline = if ol.is_empty() { "none".to_string() } else { ol };
    let first_page = heads.iter().map(|h| h.page).min().unwrap_or(0);
    let title_free = !heads.iter().any(|h| h.level == 1);
    let idx: Vec<usize> = (0..n).filter(|&i| !is_numbered(i) && heads[i].level != 1).collect();
    let base: Vec<i64> = heads.iter().map(|h| h.level).collect();
    let pages: Vec<usize> = heads.iter().map(|h| h.page).collect();
    let row = |i: usize, mark: &str| -> String {
        let h = &heads[i];
        let t: String = MD.replace_all(&h.text, "").chars().take(70).collect();
        format!(
            "{{\"i\": {i}, \"page\": {}, \"height\": {}, \"level\": \"{mark}\", \"text\": {}}}",
            h.page + 1,
            py::fmt_fixed(h.size, 1),
            json_dumps_str(&t)
        )
    };
    let one_run = |parts: &[Vec<usize>]| -> HashMap<usize, i64> {
        let mut levels = base.clone();
        let mut decided = HashMap::new();
        let mut free = title_free;
        for part in parts {
            let mut ctxset: std::collections::BTreeSet<usize> = part.iter().cloned().collect();
            for &i in part {
                if i > 0 {
                    ctxset.insert(i - 1);
                }
                if i + 1 < n {
                    ctxset.insert(i + 1);
                }
                let mut j = i as isize - 1;
                while j >= 0 && !is_numbered(j as usize) {
                    j -= 1;
                }
                if j >= 0 {
                    ctxset.insert(j as usize);
                }
            }
            let rows: Vec<String> = ctxset
                .iter()
                .map(|&i| row(i, &if part.contains(&i) { "?".to_string() } else { levels[i].to_string() }))
                .collect();
            let title_rule =
                if free && part.iter().any(|&i| pages[i] == first_page) { TITLE_RULE } else { NO_TITLE_RULE };
            let prompt = HEADING_PROMPT
                .replace("{ref}", &reff)
                .replace("{outline}", &outline)
                .replace("{title_rule}", title_rule)
                .replace("{rows}", &rows.join("\n"));
            if py::char_len(&prompt) > 10000 {
                continue;
            }
            let (ans, _) = crate::brain::ask_json(
                ctx.server,
                ctx.store,
                &prompt,
                &heading_schema(),
                &[],
                100 + 20 * part.len() as u32,
                "headings",
                ctx.think,
            );
            for item in ans.as_ref().and_then(|a| a["levels"].as_array()).into_iter().flatten() {
                let (Some(i), Some(lvl)) = (item["i"].as_i64(), item["level"].as_i64()) else { continue };
                let i = i as usize;
                if !part.contains(&i) || !(1..=6).contains(&lvl) {
                    continue;
                }
                if lvl == 1 && !(free && pages[i] == first_page) {
                    continue;
                }
                levels[i] = lvl;
                decided.insert(i, lvl);
                free = free && lvl != 1;
            }
        }
        decided
    };
    // identical groupings give identical answers: each distinct grouping runs once, and the
    // distinct runs go in parallel on Gemma's slots (each run's requests are unchanged, so
    // the answers are the same as one after the other; V1 speed-up, 2026-11-07)
    let groupings: Vec<Vec<Vec<usize>>> =
        VOTE_CHUNKS.iter().map(|&ch| heading_chunks(&idx, &is_numbered, ch)).collect();
    let mut distinct: Vec<Vec<Vec<usize>>> = Vec::new();
    for g in &groupings {
        if !distinct.contains(g) {
            distinct.push(g.clone());
        }
    }
    let run_results: Vec<HashMap<usize, i64>> =
        crate::pipeline::par_map(distinct.clone(), ctx.slots.max(1), |p| one_run(&p));
    let votes: Vec<HashMap<usize, i64>> =
        groupings.iter().map(|g| run_results[distinct.iter().position(|d| d == g).unwrap()].clone()).collect();
    let need = votes.len() / 2 + 1;
    let mut fin: HashMap<usize, i64> = HashMap::new();
    for &i in &idx {
        if let Some((v, c)) = most_common(votes.iter().filter_map(|v| v.get(&i).copied()))
            && c >= need
        {
            fin.insert(i, v);
        }
    }
    for run in runs(&idx, &is_numbered) {
        let mut groups: Vec<(i64, Vec<usize>)> = Vec::new();
        for &i in &run {
            if *fin.get(&i).unwrap_or(&base[i]) != 1 {
                let k = py::round_half_even(heads[i].size * 2.0) as i64;
                match groups.iter_mut().find(|(g, _)| *g == k) {
                    Some((_, v)) => v.push(i),
                    None => groups.push((k, vec![i])),
                }
            }
        }
        for (_, members) in groups {
            let (top, cnt) = most_common(members.iter().map(|i| *fin.get(i).unwrap_or(&base[*i]))).unwrap();
            if members.len() >= 3 && cnt * 2 > members.len() {
                for i in members {
                    fin.insert(i, top);
                }
            }
        }
    }
    let mut changes = Vec::new();
    let mut titled = !title_free;
    for &i in &idx {
        let lvl = *fin.get(&i).unwrap_or(&base[i]);
        if lvl == base[i] {
            continue;
        }
        if lvl == 1 {
            if titled {
                continue;
            }
            titled = true;
        }
        changes.push((i, base[i], lvl));
        heads[i].level = lvl;
    }
    changes
}

/// Counter(...).most_common(1): the highest count, ties to the first seen.
fn most_common(it: impl Iterator<Item = i64>) -> Option<(i64, usize)> {
    let mut v: Vec<(i64, usize)> = Vec::new();
    for x in it {
        match v.iter_mut().find(|(k, _)| *k == x) {
            Some(e) => e.1 += 1,
            None => v.push((x, 1)),
        }
    }
    py::first_max_by(v, |e| e.1)
}

/// Python json.dumps(str) (ensure_ascii=True).
pub fn json_dumps_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            '\x08' => o.push_str("\\b"),
            '\x0c' => o.push_str("\\f"),
            c if (c as u32) < 0x20 || (c as u32) > 0x7e => {
                let mut buf = [0u16; 2];
                for u in c.encode_utf16(&mut buf) {
                    o.push_str(&format!("\\u{:04x}", u));
                }
            }
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

pub const MERGE_PROMPT: &str = "You're a text correction expert specializing in accurately reproducing tables from PDFs.
You'll receive two images of tables from successive pages of a PDF: Table 1 (first page) and Table 2 (next page). Decide whether Table 2 continues Table 1 (the same larger table) and must be joined at the bottom of Table 1 (Table 2 has no new header, rows continue) or on the right (each row of Table 2 extends a row of Table 1).
Only merge if Table 2 cannot be interpreted on its own and you can read both images properly.

Table 1 html:
{t1}

Table 2 html:
{t2}";

fn merge_schema() -> Value {
    json!({"type": "object", "properties": {"analysis": {"type": "string"}, "merge": {"type": "boolean"}, "direction": {"type": "string", "enum": ["bottom", "right"]}},
        "required": ["analysis", "merge", "direction"]})
}

pub fn merge_tables(ctx: &Ctx, blocks: Vec<Block>) -> Vec<Block> {
    let mut out: Vec<Block> = Vec::new();
    for b in blocks {
        if let Some(prev) = out.last_mut()
            && b.kind == "table"
            && prev.kind == "table"
            && b.table.is_some()
            && prev.table.is_some()
            && b.page == prev.page + 1
            && b.table.as_ref().unwrap().n_cols().abs_diff(prev.table.as_ref().unwrap().n_cols()) <= 1
            && prev.meta.png.is_some()
            && b.meta.png.is_some()
            && !py::lstrip_chars(b.caption.as_deref().unwrap_or(""), "*_ ").to_lowercase().starts_with("tab")
        {
            let t1: String = html_table(prev.table.as_ref().unwrap()).chars().take(4000).collect();
            let t2: String = html_table(b.table.as_ref().unwrap()).chars().take(4000).collect();
            let p1 = prev.meta.png.clone().unwrap();
            let p2 = b.meta.png.clone().unwrap();
            let imgs: Vec<(Rgb, Vec<u8>)> = vec![((*p1).clone(), p1.png(true)), ((*p2).clone(), p2.png(true))];
            let (ans, _) = crate::brain::ask_json(
                ctx.server,
                ctx.store,
                &MERGE_PROMPT.replace("{t1}", &t1).replace("{t2}", &t2),
                &merge_schema(),
                &imgs,
                700,
                "table_merge",
                ctx.think,
            );
            if let Some(a) = ans
                && a["merge"].as_bool() == Some(true)
                && a["direction"].as_str() == Some("bottom")
                && b.table.as_ref().unwrap().n_cols() == prev.table.as_ref().unwrap().n_cols()
            {
                let bt = b.table.unwrap();
                let pt = prev.table.as_mut().unwrap();
                let mut rows = bt.rows;
                let ph = pt.header_rows();
                if ph > 0 && rows[0].iter().map(|c| &c.text).eq(pt.rows[0].iter().map(|c| &c.text)) {
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

pub const EQUATION_PROMPT: &str = "You're an expert mathematician who is good at writing LaTeX for equations.
You'll receive an image of a math block. Write its content: equations in LaTeX inside <math display=\"block\"> (block) or <math> (inline) tags, other text around them in place. Keep the LaTeX simple and KaTeX compatible. Only use the html tags math, i, b, p and br.
Text extracted from the block (may be incomplete): {text}";

pub fn equation(ctx: &Ctx, block: &mut Block) -> bool {
    let Some(png) = block.meta.png.clone() else { return false };
    if block.latex.is_some() {
        return false;
    }
    let text: String =
        block.lines_or_empty().iter().map(|l| l.text()).collect::<Vec<_>>().join(" ").chars().take(1500).collect();
    let imgs = vec![((*png).clone(), png.png(true))];
    let schema = json!({"type": "object", "properties": {"html": {"type": "string"}}, "required": ["html"]});
    let (ans, _) = crate::brain::ask_json(
        ctx.server,
        ctx.store,
        &EQUATION_PROMPT.replace("{text}", if text.is_empty() { "none" } else { &text }),
        &schema,
        &imgs,
        1500,
        "equation",
        false,
    );
    let html = ans.as_ref().and_then(|a| a["html"].as_str()).unwrap_or("").to_string();
    if let Some(l) = math_latex(&html) {
        block.kind = "equation".into();
        block.latex = Some(l);
        return true;
    }
    false
}
