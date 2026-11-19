"""Side-by-side of Sift's Markdown and marker's for the same PDF, with the PDF's own text
layer as ground truth (the text layer comes from `sift dev textlayer`; standard library only).

    uv run --no-project python -I bench/compare_marker.py <pdf> <sift.md> <marker.md> [--out report.md] [--examples 12]

What it checks (descriptions of images are left out of every text check):
 1. Text coverage. Every text-layer line of the PDF (repeated page headers/footers removed)
    is looked up in each output by its word 3-grams. Lines one tool has and the other lacks
    are listed: that's text a tool dropped or garbled.
 2. Words that are not in the PDF. Words and numbers each tool wrote that the text layer
    doesn't contain (OCR errors, rewritten identifiers, lost minus signs). Words from the 8-ish
    scanned pages show up for both tools.
 3. Tables. Each table is placed on the PDF page that shares most of its words; cell words that
    are not on that page are errors. Tables both tools have are compared word by word.
 4. Headings. Counts per level, numbered-heading depth vs level, headings only one tool has
    (with the PDF line's height and weight, the evidence for "is this a heading").
 5. Figures and descriptions per page.
 6. Markdown hygiene: emphasis markers left unbalanced, bullet glyphs left inside list items.
"""

from __future__ import annotations

import argparse
import html
import re
import sys
import unicodedata
from collections import Counter, defaultdict
from pathlib import Path

DASHES = str.maketrans({"−": "-", "–": "-", "—": "-", "‐": "-", "‑": "-", "\u00ad": ""})
TOKEN = re.compile(r"(?:(?<![\w.])[-+])?\d+(?:[.,]\d+)*\w*|\w+")


# -- normalising ------------------------------------------------------------------------------
def norm(text: str) -> str:
    return unicodedata.normalize("NFKC", text).translate(DASHES)


def tokens(text: str):
    # "3.1Push-button contacts": the page shows a space after the section number that the text
    # layer lacks (BMM350 handling guide); "3.1Push" and "3.1 Push" count the same, for every side
    return TOKEN.findall(re.sub(r"(\d(?:\.\d+)+)(?=[A-Z][a-z])", r"\1 ", norm(text)))


def strip_md(md: str) -> str:
    """Markdown (with inline HTML) -> plain text, for word matching."""
    t = re.sub(r"!\[[^\]]*\]\([^)]*\)", " ", md)  # image links (Sift's alt text repeats the caption)
    t = re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", t)  # links: keep the text
    t = re.sub(r"(?i)<br\s*/?>|</?(?:td|th|tr|p|li|div|table|thead|tbody)(?:\s[^<>\n]*)?>", " ", t)
    t = re.sub(r"</?[A-Za-z][A-Za-z0-9]*(?:\s[^<>\n]*)?/?>", "", t)  # sup/sub/b/i/u/span: no space (the PDF writes "PHY1, 2")
    t = re.sub(r"\\(?:text|mathrm|mathbf|frac|left|right|times|cdot|pm|mu|Omega|circ|leq|geq|le|ge|sim|approx|quad|,)\b", " ", t)
    t = re.sub(r"\\([\\`*_{}\[\]()#+\-.!|~<>$&%])", r"\1", t)  # Markdown escapes
    return html.unescape(t)


def split_descriptions(md: str, tool: str):
    """(text without image descriptions, list of (image, description))."""
    descs = []
    out_lines = []
    lines = md.splitlines()
    i = 0
    last_img = None
    while i < len(lines):
        l = lines[i]
        m = re.search(r"!\[[^\]]*\]\(([^)]*)\)", l)
        if m:
            last_img = m.group(1)
        if tool == "sift" and l.startswith("> **Image description:**"):
            block = []
            while i < len(lines) and lines[i].startswith(">"):
                block.append(lines[i][1:].strip())
                i += 1
            descs.append((last_img, "\n".join(block).replace("**Image description:**", "").strip()))
            continue
        mm = re.match(r"Image /page/(\d+)/(\w+)/(\d+) description:\s*(.*)", l)
        if tool == "marker" and mm:
            block = [mm.group(4)]
            i += 1
            while i < len(lines) and lines[i].strip():
                block.append(lines[i])
                i += 1
            descs.append((f"_page_{mm.group(1)}_{mm.group(2)}_{mm.group(3)}.png", "\n".join(block).strip()))
            continue
        out_lines.append(l)
        i += 1
    return "\n".join(out_lines), descs


# -- the PDF --------------------------------------------------------------------------------------
def extract(pdf: Path):
    """The PDF's text layer from Sift itself (`sift dev textlayer`: pdftext's algorithm + Sift's
    rules), as pages with `.index`, `.height` and `.lines` (each with `.text` and `.bbox`).
    SIFT_BIN picks the binary (default: target/release/sift.exe next to this folder)."""
    import json
    import os
    import subprocess
    from types import SimpleNamespace

    exe = os.environ.get("SIFT_BIN") or str(Path(__file__).resolve().parents[1] / "target" / "release" / "sift.exe")
    out = subprocess.run([exe, "dev", "textlayer", str(pdf)], capture_output=True, check=True).stdout
    pages = []
    for p in json.loads(out.decode("utf-8"))["pages"]:
        lines = [SimpleNamespace(text=l["text"], bbox=l["bbox"]) for l in p["lines"]]
        pages.append(SimpleNamespace(index=p["index"], width=p["width"], height=p["height"], lines=lines))
    return pages


class PdfTruth:
    def __init__(self, pdf: Path):
        self.pages = extract(pdf)
        n = len(self.pages)
        keys = Counter()
        for p in self.pages:
            seen = set()
            for l in p.lines:
                if self._margin(l, p):
                    seen.add(self._key(l.text))
            keys.update(seen)
        need = max(3, int(0.2 * n))
        self.furniture = {k for k, c in keys.items() if c >= need}
        self.lines = []  # (page, line, tokens)
        self.page_tokens = []
        for p in self.pages:
            pc = Counter()
            for l in p.lines:
                tk = tokens(l.text)
                if not tk:
                    continue
                if self._margin(l, p) and (self._key(l.text) in self.furniture or re.fullmatch(r"[\d\s/|.-]*", l.text)):
                    continue
                self.lines.append((p.index, l, tk))
                pc.update(tk)
            self.page_tokens.append(pc)
        self.all_tokens = Counter()
        for pc in self.page_tokens:
            self.all_tokens.update(pc)
        self.scanned = {p.index for p in self.pages if len(p.lines) < 3}

    @staticmethod
    def _key(text: str) -> str:
        return re.sub(r"\d+", "#", re.sub(r"\s+", " ", text.lower())).strip()

    @staticmethod
    def _margin(l, p) -> bool:
        return l.bbox[3] < p.height * 0.08 or l.bbox[1] > p.height * 0.92

    def find_line(self, text: str):
        """The PDF line whose text matches a heading (exact words first, then contained)."""
        want = tokens(strip_md(text))
        if not want:
            return None
        best = None
        for page, l, tk in self.lines:
            if tk == want:
                return page, l
            if best is None and len(want) >= 2 and _contains(tk, want):
                best = (page, l)
        return best


def _contains(hay, needle) -> bool:
    n = len(needle)
    return any(hay[i:i + n] == needle for i in range(len(hay) - n + 1))


def ngrams(tk, n):
    return [tuple(tk[i:i + n]) for i in range(len(tk) - n + 1)]


class Output:
    def __init__(self, name: str, path: Path):
        self.name = name
        raw = path.read_text(encoding="utf-8")
        self.raw = raw
        self.body, self.descs = split_descriptions(raw, name)
        self.plain = strip_md(self.body)
        self.tk = tokens(self.plain)
        self.counts = Counter(self.tk)
        self.g2 = set(ngrams(self.tk, 2))
        self.g3 = set(ngrams(self.tk, 3))
        self.tables = parse_tables(self.body)
        self.headings = parse_headings(self.body)

    def coverage(self, tk) -> float:
        if len(tk) >= 3:
            g = ngrams(tk, 3)
            return sum(x in self.g3 for x in g) / len(g)
        if len(tk) == 2:
            return float(tuple(tk) in self.g2)
        return float(self.counts[tk[0]] > 0)

    def has_whole_name(self, tk) -> bool:
        """A short PDF line that is a piece of a name wrapped inside a narrow table cell
        ("cfg_sen" / "s_tim_a" / "on" for cfg_sens_tim_aon): found when the output has a
        longer snake_case name containing every piece of it."""
        if not hasattr(self, "_names"):
            self._names = [w for w in self.counts if "_" in w and len(w) > 3]
        pieces = [t for t in tk if not re.fullmatch(r"0x[0-9A-Fa-f]+|\d+|reserved", t)]
        return bool(pieces) and len(tk) <= 3 and all(any(p in w and p != w for w in self._names) for p in pieces)


# -- tables -----------------------------------------------------------------------------------------
def _pipe_cells(line: str):
    s = line.strip()
    if s.startswith("|"):
        s = s[1:]
    if s.endswith("|") and not s.endswith("\\|"):
        s = s[:-1]
    return [c.strip() for c in re.split(r"(?<!\\)\|", s)]


def parse_tables(md: str):
    """Pipe tables and HTML tables -> list of {"rows": [[cell text]], "line": n}."""
    out = []
    lines = md.splitlines()
    i = 0
    while i < len(lines):
        l = lines[i]
        if l.lstrip().startswith("|"):
            rows, start = [], i
            while i < len(lines) and lines[i].lstrip().startswith("|"):
                cells = _pipe_cells(lines[i])
                if not all(re.fullmatch(r":?-{3,}:?", c) for c in cells if c):
                    rows.append([strip_md(c).strip() for c in cells])
                i += 1
            out.append({"rows": rows, "line": start + 1})
            continue
        if l.lstrip().lower().startswith("<table"):
            start = i
            buf = []
            while i < len(lines):
                buf.append(lines[i])
                if "</table>" in lines[i].lower():
                    break
                i += 1
            i += 1
            h = "\n".join(buf)
            rows = []
            for tr in re.findall(r"(?is)<tr[^>]*>(.*?)</tr>", h):
                rows.append([strip_md(c).strip() for c in re.findall(r"(?is)<t[hd][^>]*>(.*?)</t[hd]>", tr)])
            out.append({"rows": rows, "line": start + 1})
            continue
        i += 1
    for t in out:
        t["tokens"] = Counter(x for r in t["rows"] for c in r for x in tokens(c))
        t["ncols"] = max((len(r) for r in t["rows"]), default=0)
    return out


def place_tables(tables, truth: PdfTruth):
    for t in tables:
        best, score = None, 0
        for pi, pc in enumerate(truth.page_tokens):
            s = sum((t["tokens"] & pc).values())
            if s > score:
                best, score = pi, s
        t["page"] = best
        if best is None:
            t["bad"] = Counter(t["tokens"])
        else:
            near = Counter()
            for q in (best - 1, best, best + 1):  # a merged table spans the next page
                if 0 <= q < len(truth.page_tokens):
                    near |= truth.page_tokens[q]
            t["bad"] = Counter({k: v for k, v in t["tokens"].items() if near[k] == 0})


# -- headings -------------------------------------------------------------------------------------------
def parse_headings(md: str):
    out = []
    fence = False
    for n, l in enumerate(md.splitlines(), 1):
        if l.startswith("```"):
            fence = not fence
        if fence:
            continue
        m = re.match(r"^(#{1,6})\s+(.*\S)\s*$", l)
        if m:
            # marker wraps heading text in ** and <span id> anchors: "**6.4.2 CalibFE**"
            out.append({"level": len(m.group(1)), "text": re.sub(r"(?<!\\)\*+", "", strip_md(m.group(2))).strip(), "line": n})
    return out


def numbering_depth(text: str):
    m = re.match(r"^((?:\d{1,2}|[A-Z])(?:\.\d{1,2}){0,5})\.?\s+\S", text)
    if not m or re.fullmatch(r"[A-Z]", m.group(1)):
        return None
    return m.group(1).count(".") + 1


def hkey(text: str) -> str:
    return " ".join(tokens(text)).lower()


# -- hygiene ---------------------------------------------------------------------------------------------
def hygiene(out: Output, minus_lines=frozenset()):
    """minus_lines: PDF lines starting with a minus sign and a digit, without the minus."""
    issues = defaultdict(list)
    fence = False
    for n, l in enumerate(out.body.splitlines(), 1):
        if l.startswith("```"):
            fence = not fence
        if fence or l.lstrip().startswith(("|", "<")):
            continue
        if re.match(r"^\s*(?:[-*+]|\d+[.)])\s+(?:\*+|_+)?\s*[•●▪■◦○·‣∙►▸➢*]+(?:\*+)?\s", l):
            issues["bullet glyph left in a list item"].append((n, l))
        bare = re.sub(r"\\\*", "", l)
        if bare.count("**") % 2:
            issues["unbalanced **"].append((n, l))
        if re.search(r"(?<![\\*])\*(?!\*)", re.sub(r"\*\*", "", bare)) and re.sub(r"\*\*", "", bare).count("*") % 2:
            issues["unbalanced *"].append((n, l))
        m = re.match(r"^\s*[-*+]\s+(\d.*)$", l)
        if m and minus_lines and norm(strip_md(m.group(1))).strip() in minus_lines:
            issues["minus sign lost (PDF line '-N ...' became a list item)"].append((n, l))
    return issues


# -- report ----------------------------------------------------------------------------------------------
def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("pdf", type=Path)
    ap.add_argument("sift", type=Path)
    ap.add_argument("marker", type=Path)
    ap.add_argument("--out", type=Path, help="write the full report here (Markdown)")
    ap.add_argument("--examples", type=int, default=12)
    a = ap.parse_args(argv)

    truth = PdfTruth(a.pdf)
    tools = [Output("sift", a.sift), Output("marker", a.marker)]
    S, M = tools
    k = a.examples
    rep = []

    def say(s=""):
        rep.append(s)

    say(f"# Sift vs marker on {a.pdf.name}")
    say()
    say(f"PDF: {len(truth.pages)} pages, {len(truth.lines)} text-layer lines after removing {len(truth.furniture)} "
        f"repeated header/footer texts, {sum(truth.all_tokens.values())} words; pages without a text layer: "
        f"{sorted(p + 1 for p in truth.scanned) or 'none'}")
    say()
    say("| | Sift | marker |")
    say("|---|---|---|")
    say(f"| characters (without descriptions) | {len(S.body):,} | {len(M.body):,} |")
    say(f"| words | {len(S.tk):,} | {len(M.tk):,} |")

    # 1. text coverage
    cov = {t.name: [] for t in tools}
    for page, l, tk in truth.lines:
        for t in tools:
            cov[t.name].append(t.coverage(tk))
    n = len(truth.lines)
    for t in tools:
        c = cov[t.name]
        t.found = sum(x >= 0.5 for x in c)
    words_total = sum(len(tk) for _, _, tk in truth.lines)
    for t in tools:
        t.word_cov = sum(min(t.counts[w], c) for w, c in truth.all_tokens.items()) / max(1, sum(truth.all_tokens.values()))
    say(f"| PDF lines found (3-gram cover >= 50%) | {S.found:,} / {n:,} ({S.found / n:.1%}) | {M.found:,} / {n:,} ({M.found / n:.1%}) |")
    for t in tools:  # pieces of names wrapped in narrow cells count when the whole name is there
        t.found_names = t.found + sum(1 for (page, l, tk), c in zip(truth.lines, cov[t.name]) if c < 0.5 and t.has_whole_name(tk))
    say(f"| PDF lines found, wrapped names joined | {S.found_names:,} ({S.found_names / n:.1%}) | {M.found_names:,} ({M.found_names / n:.1%}) |")
    say(f"| PDF words found (multiset) | {S.word_cov:.1%} | {M.word_cov:.1%} |")
    not_in_pdf = {}
    for t in tools:
        lower = Counter()
        for w, c in truth.all_tokens.items():
            lower[w.lower()] += c
        absent = Counter({w: c for w, c in t.counts.items() if truth.all_tokens[w] == 0})
        case = Counter({w: c for w, c in absent.items() if lower[w.lower()] > 0})
        not_in_pdf[t.name] = (absent, case)
    say(f"| words not in the PDF text (all / casing changed only) | {sum(not_in_pdf['sift'][0].values()):,} / {sum(not_in_pdf['sift'][1].values()):,} "
        f"| {sum(not_in_pdf['marker'][0].values()):,} / {sum(not_in_pdf['marker'][1].values()):,} |")
    minus = frozenset(norm(l.text).strip()[1:].strip() for _, l, _ in truth.lines if re.match(r"^-\d", norm(l.text).strip()))
    for t in tools:
        t.hyg = hygiene(t, minus)
    say(f"| tables | {len(S.tables)} | {len(M.tables)} |")
    say(f"| headings (by level 1-6) | {len(S.headings)} {[sum(h['level'] == i for h in S.headings) for i in range(1, 7)]} "
        f"| {len(M.headings)} {[sum(h['level'] == i for h in M.headings) for i in range(1, 7)]} |")
    say(f"| images / descriptions | {len(re.findall(r'!\[', S.raw))} / {len(S.descs)} | {len(re.findall(r'!\[', M.raw))} / {len(M.descs)} |")
    say(f"| `<sup>` / `<sub>` | {S.raw.count('<sup>')} / {S.raw.count('<sub>')} | {M.raw.count('<sup>')} / {M.raw.count('<sub>')} |")
    for key in sorted(set(S.hyg) | set(M.hyg)):
        say(f"| hygiene: {key} | {len(S.hyg.get(key, []))} | {len(M.hyg.get(key, []))} |")
    say()

    say("## 1. Text one tool has and the other lacks")
    say()
    for a_, b_ in ((S, M), (M, S)):
        lost = [(page, l.text) for (page, l, tk), ca, cb in zip(truth.lines, cov[a_.name], cov[b_.name]) if ca < 0.5 and cb >= 0.8 and len(tk) >= 2]
        say(f"### Lines {b_.name} has and {a_.name} lacks: {len(lost)}")
        say()
        by_page = Counter(p for p, _ in lost)
        say("Pages with the most: " + ", ".join(f"p{p + 1} ({c})" for p, c in by_page.most_common(15)))
        say()
        for p, txt in lost[: k * 3]:
            say(f"- p{p + 1}: `{txt.strip()[:140]}`")
        say()
    both = [(page, l.text) for (page, l, tk), ca, cb in zip(truth.lines, cov["sift"], cov["marker"]) if ca < 0.5 and cb < 0.5 and len(tk) >= 2]
    say(f"### Lines neither tool has: {len(both)} (mostly text inside vector figures, which both turn into images)")
    say()
    for p, txt in both[:k]:
        say(f"- p{p + 1}: `{txt.strip()[:140]}`")
    say()

    say("## 2. Words not in the PDF text layer (outside image descriptions)")
    say()
    for t in tools:
        absent, case = not_in_pdf[t.name]
        say(f"### {t.name}: {sum(absent.values())} words, {len(absent)} distinct")
        say()
        say("Casing changed (the PDF has the word with other capitals): " + ", ".join(f"`{w}` x{c}" for w, c in case.most_common(k * 2)))
        say()
        rest = Counter({w: c for w, c in absent.items() if w not in case})
        say("Other: " + ", ".join(f"`{w}` x{c}" for w, c in rest.most_common(k * 4)))
        say()
    # identifier counts that differ from the PDF (e.g. SetLoRaSynchTimeout x4 / SetLoraSynchTimeout x2)
    say("### Identifier counts that differ from the PDF (CamelCase or snake_case words)")
    say()
    ident = [w for w in truth.all_tokens if re.search(r"[a-z][A-Z]|_", w) and len(w) > 5]
    rows = []
    for w in ident:
        p, s, m = truth.all_tokens[w], S.counts[w], M.counts[w]
        if s != p or m != p:
            rows.append((abs(s - p) + abs(m - p), w, p, s, m))
    say("| identifier | PDF | Sift | marker |")
    say("|---|---|---|---|")
    for _, w, p, s, m in sorted(rows, reverse=True)[: k * 3]:
        say(f"| `{w}` | {p} | {s} | {m} |")
    say()

    say("## 3. Tables")
    say()
    for t in tools:
        place_tables(t.tables, truth)
        bad = sum(sum(x["bad"].values()) for x in t.tables)
        words = sum(sum(x["tokens"].values()) for x in t.tables)
        say(f"- {t.name}: {len(t.tables)} tables, {words:,} cell words, **{bad} cell words not on the table's page** "
            f"(pages without a text layer excluded: {sum(sum(x['bad'].values()) for x in t.tables if x['page'] in truth.scanned)})")
    say()
    for t in tools:
        worst = sorted((x for x in t.tables if x["bad"] and x["page"] not in truth.scanned), key=lambda x: -sum(x["bad"].values()))
        say(f"### {t.name}: tables with words not on their page ({len(worst)})")
        say()
        for x in worst[:k]:
            say(f"- p{(x['page'] or 0) + 1} (md line {x['line']}): " + ", ".join(f"`{w}`" + (f" x{c}" if c > 1 else "") for w, c in x["bad"].most_common(10)))
        say()
    # pairs
    pairs, used = [], set()
    for st in S.tables:
        best, bj = None, 0.0
        for j, mt in enumerate(M.tables):
            if j in used or mt["page"] is None or st["page"] is None or abs(mt["page"] - st["page"]) > 1:
                continue
            inter = sum((st["tokens"] & mt["tokens"]).values())
            union = sum((st["tokens"] | mt["tokens"]).values())
            jac = inter / union if union else 0
            if jac > bj:
                best, bj = j, jac
        if best is not None and bj >= 0.3:
            used.add(best)
            pairs.append((st, M.tables[best], bj))
    only_s = [x for x in S.tables if not any(x is p[0] for p in pairs)]
    only_m = [x for j, x in enumerate(M.tables) if j not in used]
    same = sum(1 for s_, m_, _ in pairs if s_["tokens"] == m_["tokens"])
    say(f"Matched {len(pairs)} table pairs; {same} have exactly the same cell words. Only in Sift: {len(only_s)}; only in marker: {len(only_m)}.")
    say()
    say("### Matched tables whose words differ (which side matches the PDF page)")
    say()
    diffs = []
    for s_, m_, j in pairs:
        if s_["tokens"] == m_["tokens"]:
            continue
        page = s_["page"]
        pc = truth.page_tokens[page] if page is not None else Counter()
        s_only = s_["tokens"] - m_["tokens"]
        m_only = m_["tokens"] - s_["tokens"]
        s_bad = sum(c for w, c in s_only.items() if pc[w] == 0)
        m_bad = sum(c for w, c in m_only.items() if pc[w] == 0)
        diffs.append((page, s_, m_, s_only, m_only, s_bad, m_bad))
    say(f"Tables where Sift has words that aren't on the page: {sum(1 for d in diffs if d[5])}; where marker does: {sum(1 for d in diffs if d[6])}.")
    say()
    for page, s_, m_, s_only, m_only, s_bad, m_bad in sorted(diffs, key=lambda d: -(d[5] + d[6]))[: k * 2]:
        def fmt(cnt, pc):
            return ", ".join(("~~" if pc[w] == 0 else "") + f"`{w}`" + ("~~" if pc[w] == 0 else "") + (f" x{c}" if c > 1 else "") for w, c in cnt.most_common(8))
        pc = truth.page_tokens[page] if page is not None else Counter()
        say(f"- p{(page or 0) + 1} (Sift line {s_['line']}, marker line {m_['line']}; {s_['ncols']} vs {m_['ncols']} columns, "
            f"{len(s_['rows'])} vs {len(m_['rows'])} rows): Sift only: {fmt(s_only, pc) or '-'}; marker only: {fmt(m_only, pc) or '-'}")
    say()
    say("(~~struck~~ words are not on the PDF page.)")
    say()
    for name, lst in (("Sift", only_s), ("marker", only_m)):
        say(f"### Tables only {name} has: {len(lst)}")
        say()
        for x in lst[:k]:
            first = " / ".join(c for c in (x["rows"][0] if x["rows"] else [])[:5])
            say(f"- p{(x['page'] or 0) + 1} (md line {x['line']}, {len(x['rows'])}x{x['ncols']}): `{first[:120]}`")
        say()

    say("## 4. Headings")
    say()
    for t in tools:
        by_depth = defaultdict(Counter)
        for h in t.headings:
            d = numbering_depth(h["text"])
            by_depth[d][h["level"]] += 1
        say(f"- {t.name}: numbered depth -> levels used: " + "; ".join(
            f"depth {d}: " + ", ".join(f"#{lv} x{c}" for lv, c in sorted(cnt.items())) for d, cnt in sorted(by_depth.items(), key=lambda x: (x[0] is None, x[0] or 0))))
    say()
    sk = Counter(hkey(h["text"]) for h in S.headings)
    mk = Counter(hkey(h["text"]) for h in M.headings)
    for a_, ak, bk, b_ in ((M, mk, sk, S), (S, sk, mk, M)):
        only = [h for h in a_.headings if hkey(h["text"]) and bk[hkey(h["text"])] == 0]
        say(f"### Headings only {a_.name} has: {len(only)} ({sum(1 for h in only if numbering_depth(h['text']))} numbered)")
        say()
        say("| level | text | PDF page | line height | bold | in the other output as |")
        say("|---|---|---|---|---|---|")
        for h in only[: k * 3]:
            f = truth.find_line(h["text"])
            where = ""
            tk = tokens(h["text"])
            if tk and b_.coverage(tk) >= 0.5:
                where = "text"
            if f:
                page, l = f
                say(f"| {h['level']} | {h['text'][:70]} | {page + 1} | {l.height:.1f} | {'yes' if l.bold else 'no'} | {where or 'missing'} |")
            else:
                say(f"| {h['level']} | {h['text'][:70]} | ? | | | {where or 'missing'} |")
        say()

    say("## 5. Figures and descriptions")
    say()
    pages_img = {}
    for t in tools:
        per = Counter()
        for m in re.finditer(r"!\[[^\]]*\]\(([^)]*)\)", t.raw):
            src = m.group(1)
            mm = re.search(r"_page_(\d+)_", src)
            ms = re.search(r"p(\d{3})-", src)
            if mm:
                per[int(mm.group(1)) + 1] += 1
            elif ms:
                per[int(ms.group(1))] += 1
        pages_img[t.name] = per
        lens = [len(d) for _, d in t.descs]
        say(f"- {t.name}: {sum(per.values())} images on {len(per)} pages; {len(t.descs)} descriptions, "
            f"median {sorted(lens)[len(lens) // 2] if lens else 0} characters")
    diff_pages = sorted(p for p in set(pages_img["sift"]) | set(pages_img["marker"]) if pages_img["sift"][p] != pages_img["marker"][p])
    say()
    say("Pages where the image counts differ (page: Sift / marker): " + ", ".join(f"p{p}: {pages_img['sift'][p]}/{pages_img['marker'][p]}" for p in diff_pages))
    say()

    say("## 6. Markdown hygiene examples")
    say()
    for t in tools:
        for key, lst in sorted(t.hyg.items()):
            say(f"### {t.name}: {key} ({len(lst)})")
            say()
            for n_, l in lst[:k // 2 or 1]:
                say(f"- line {n_}: `{l.strip()[:140]}`")
            say()

    text = "\n".join(rep) + "\n"
    if a.out:
        a.out.write_text(text, encoding="utf-8")
    summary = text.split("## 1.")[0]
    sys.stdout.buffer.write(summary.encode("utf-8"))
    if a.out:
        print(f"full report: {a.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
