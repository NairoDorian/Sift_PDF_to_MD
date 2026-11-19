"""Sift scorecard v2: one or more Markdown outputs of the same PDF, scored against the PDF's
own text layer (the ground truth where it exists, D10), plus a grade per model from the run's
own checks. Replaces the headline numbers of bench/compare_marker.py (whose detailed
report stays useful), fixing what that report got wrong:

1. "PDF lines found" (3-gram cover) counted a correct join as a lost line: "appli-" + "cations"
   written as "applications", a name wrapped in a narrow cell joined back, a space restored after
   a section number. Here a PDF line is found when its words are found, where the joins the PDF
   itself licenses count (line-end hyphen + next line's first word, adjacent pieces of a name).
2. "Words not in the PDF" counted those same joins as invented words. Here a word is invented
   only when no PDF token, no licensed join and no concatenation of two adjacent PDF tokens
   gives it. Invented NUMBERS are counted apart: a wrong number is the costliest error.
3. Reading order was not measured (word coverage is a bag of words). Here: for consecutive PDF
   lines of a page that both appear in the output, the share that keep their order.
4. One headline figure for comparing runs: errors per 1,000 PDF words =
   (missed words + invented words + 5 x invented numbers) / PDF words x 1000. Lower is better.
5. Model grades, from the trace of a Sift run:
   - surya, tables: the share of checked table tokens it wrote exactly as the PDF has them
     (verified / (verified + corrected + unresolved)): free ground truth on digital pages;
   - Gemma, descriptions: the share of numbers in its descriptions that the figure's own text
     (PDF or OCR) confirms; LLM table reviews accepted / refused.

    uv run --no-project python -I bench/score.py <pdf> name=out.md [name=out.md ...] [--trace run.jsonl] [--json]

Descriptions of images are left out of every text check (they are not in the PDF).
"""
from __future__ import annotations

import json
import re
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from compare_marker import Output, PdfTruth, tokens  # noqa: E402

NUM = re.compile(r"^[-+]?\d+(?:[.,]\d+)*$")


def licensed_joins(truth: PdfTruth) -> Counter:
    """Tokens the PDF licenses by joining: a line-end hyphenated word with the next line's first
    word (both "appli-cations"-less forms), and two adjacent tokens of one line (a name the
    text layer splits: "start" "time(31:0)" -> no; "PSRRInBand_" "NP" -> yes)."""
    j = Counter()
    by_page = {}
    for page, l, tk in truth.lines:
        by_page.setdefault(page, []).append((l.text, tk))
    for page, ls in by_page.items():
        for (t1, a), (t2, b) in zip(ls, ls[1:]):
            if a and b and t1.rstrip().endswith("-"):
                j[a[-1] + b[0]] += 1
                j[a[-1] + "-" + b[0]] += 1
            if a and b:  # a name wrapped over two lines inside a cell
                j[a[-1] + b[0]] += 1
        for _, tk in ls:
            for x, y in zip(tk, tk[1:]):
                j[x + y] += 1
                j[x + "_" + y] += 1
    return j


class Scored:
    def __init__(self, name: str, md: Path, truth: PdfTruth, joins: Counter):
        self.name = name
        self.out = Output("sift" if name != "marker" else "marker", md)
        o = self.out
        lower_pdf = {w.lower() for w in truth.all_tokens}
        lower_joins = {w.lower() for w in joins}
        # words found (multiset, joins accepted: a joined output token covers its pieces)
        have = Counter(o.counts)
        for w, c in o.counts.items():
            if truth.all_tokens[w] == 0 and w.lower() in lower_joins:
                for piece in re.findall(r"\w+", w):
                    have[piece] += c
        total = sum(truth.all_tokens.values())
        found = sum(min(have[w], c) for w, c in truth.all_tokens.items())
        self.words_total, self.words_found = total, found
        # lines found: >= 80 % of a line's words present in the output (with joins)
        self.lines_total = len(truth.lines)
        self.lines_found = 0
        pos = {}
        for i, w in enumerate(o.tk):
            pos.setdefault(w, i)
        firsts = []
        for page, l, tk in truth.lines:
            ok = sum(have[w] > 0 for w in tk) / len(tk) >= 0.8
            self.lines_found += ok
            firsts.append((page, ok, self._line_pos(tk) if ok else None))
        # reading order: consecutive found lines of a page keep their order
        pairs = kept = 0
        for (p1, ok1, x1), (p2, ok2, x2) in zip(firsts, firsts[1:]):
            if p1 == p2 and ok1 and ok2 and x1 is not None and x2 is not None:
                pairs += 1
                kept += x2 >= x1
        self.order = kept / pairs if pairs else 1.0
        # invented words / numbers
        inv = Counter()
        for w, c in o.counts.items():
            if truth.all_tokens[w] or w in joins:
                continue
            if w.lower() in lower_pdf or w.lower() in lower_joins:
                continue  # casing only: reported by compare_marker, not an invention
            inv[w] += c
        self.invented = inv
        self.invented_numbers = Counter({w: c for w, c in inv.items() if NUM.match(w)})
        missed = total - found
        n_inv = sum(inv.values()) - sum(self.invented_numbers.values())
        self.errors_per_k = (missed + n_inv + 5 * sum(self.invented_numbers.values())) / max(1, total) * 1000

    def _line_pos(self, tk):
        """Output position of the line: where its rarest 3-gram (or word) first appears."""
        o = self.out
        if not hasattr(self, "_g3pos"):
            self._g3pos = {}
            for i in range(len(o.tk) - 2):
                self._g3pos.setdefault(tuple(o.tk[i:i + 3]), i)
            self._wpos = {}
            for i, w in enumerate(o.tk):
                self._wpos.setdefault(w, i)
        cands = [self._g3pos[g] for g in (tuple(tk[i:i + 3]) for i in range(len(tk) - 2)) if g in self._g3pos]
        if cands:
            return min(cands)
        cands = [self._wpos[w] for w in tk if w in self._wpos and self.out.counts[w] == 1]
        return min(cands) if cands else None


def model_grades(trace: Path) -> dict:
    v = c = u = 0
    reviews = fixed = 0
    desc_nums = desc_unconfirmed = 0
    for line in open(trace, encoding="utf-8"):
        r = json.loads(line)
        if r.get("event") == "file":
            s = r.get("stats") or {}
            v += s.get("table_values_verified", 0)
            c += s.get("table_values_corrected", 0)
            u += s.get("table_values_unresolved", 0)
            reviews += s.get("llm_table_reviews", 0)
            fixed += s.get("llm_table_corrections", 0)
            desc_nums += s.get("description_numbers", 0)
            desc_unconfirmed += s.get("description_numbers_unconfirmed", 0)
    out = {}
    if v + c + u:
        out["surya tables: tokens exactly as the PDF"] = f"{v / (v + c + u):.2%} ({v} of {v + c + u}; {c} corrected from the PDF, {u} unresolved)"
    if desc_nums:
        out["brain descriptions: numbers confirmed by the figure's text"] = f"{1 - desc_unconfirmed / desc_nums:.1%} ({desc_nums - desc_unconfirmed} of {desc_nums})"
    if reviews:
        out["brain table reviews: corrections accepted"] = f"{fixed} of {reviews} reviews"
    return out


def main() -> int:
    args = sys.argv[1:]
    as_json = "--json" in args
    trace = None
    if "--trace" in args:
        trace = Path(args[args.index("--trace") + 1])
    pdf = Path(args[0])
    outs = [a.split("=", 1) for a in args[1:] if "=" in a and not a.startswith("--")]
    truth = PdfTruth(pdf)
    if "--pages" in args:
        # only these pages (1-based, "1-30,45"): outputs converted with the same --pages
        keep = set()
        for part in args[args.index("--pages") + 1].split(","):
            a, _, b = part.partition("-")
            keep.update(range(int(a), int(b or a) + 1))
        truth.lines = [x for x in truth.lines if x[0] + 1 in keep]
        truth.page_tokens = [pc if i + 1 in keep else Counter() for i, pc in enumerate(truth.page_tokens)]
        truth.all_tokens = Counter()
        for pc in truth.page_tokens:
            truth.all_tokens.update(pc)
    joins = licensed_joins(truth)
    rows = [Scored(n, Path(p), truth, joins) for n, p in outs]
    res = {r.name: {
        "PDF words found": round(r.words_found / max(1, r.words_total), 4),
        "PDF lines found": round(r.lines_found / max(1, r.lines_total), 4),
        "reading order kept": round(r.order, 4),
        "invented words": sum(r.invented.values()) - sum(r.invented_numbers.values()),
        "invented numbers": sum(r.invented_numbers.values()),
        "errors per 1000 PDF words": round(r.errors_per_k, 2),
    } for r in rows}
    if trace:
        res["models"] = model_grades(trace)
    if as_json:
        print(json.dumps(res, ensure_ascii=False, indent=1))
        return 0
    names = [r.name for r in rows]
    print(f"Scorecard v2: {pdf.name}  ({truth.lines.__len__()} PDF lines, {sum(truth.all_tokens.values())} words)")
    print("| measure | " + " | ".join(names) + " |")
    print("|---|" + "---|" * len(names))
    for k in next(iter(res.values())):
        vals = [res[n][k] for n in names]
        fmt = [f"{x:.2%}" if isinstance(x, float) and x <= 1 and "per" not in k else str(x) for x in vals]
        print(f"| {k} | " + " | ".join(fmt) + " |")
    for r in rows:
        top = ", ".join(f"`{w}` x{c}" for w, c in r.invented_numbers.most_common(8))
        if top:
            print(f"  {r.name} invented numbers: {top}")
    for k, v in res.get("models", {}).items():
        print(f"  model grade: {k}: {v}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
