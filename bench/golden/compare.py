"""The golden comparison (docs/GOLDEN.md): for every <stem> folder of the expected root, the
Markdown must be byte-identical (line endings ignored), the report must have the same lines
(its 3-line header ignored) and the images the same file names (expected: images.txt).

    uv run --no-project python -I compare.py <expected root> <output root> [stem filter]

Standard library only. Exit code 1 when any document differs or is missing.
"""
import difflib
import os
import sys


def read(p):
    try:
        with open(p, encoding="utf-8") as f:
            return f.read().replace("\r\n", "\n")
    except FileNotFoundError:
        return None


def image_names(root, stem):
    listed = read(os.path.join(root, stem, "images.txt"))
    if listed is not None:
        return sorted(l for l in listed.splitlines() if l.strip())
    d = os.path.join(root, stem, "images")
    return sorted(os.listdir(d)) if os.path.isdir(d) else []


def main():
    exp_root, out_root = sys.argv[1], sys.argv[2]
    only = sys.argv[3] if len(sys.argv) > 3 else ""
    bad = 0
    stems = [s for s in sorted(os.listdir(exp_root)) if not s.startswith(".") and (not only or only in s)]
    for stem in stems:
        exp = read(os.path.join(exp_root, stem, stem + ".md"))
        out = read(os.path.join(out_root, stem, stem + ".md"))
        if out is None:
            print(f"MISSING    {stem}: no output")
            bad += 1
            continue
        rep_e = (read(os.path.join(exp_root, stem, stem + ".sift-report.txt")) or "").splitlines()[3:]
        rep_o = (read(os.path.join(out_root, stem, stem + ".sift-report.txt")) or "").splitlines()[3:]
        md_diff = list(difflib.unified_diff(exp.splitlines(), out.splitlines(), "expected", "output", n=0, lineterm=""))
        only_e = [l for l in rep_e if l not in set(rep_o)]
        only_o = [l for l in rep_o if l not in set(rep_e)]
        im_e, im_o = image_names(exp_root, stem), image_names(out_root, stem)
        ok = not md_diff and not only_e and not only_o and im_e == im_o
        bad += not ok
        changed = sum(1 for l in md_diff if l.startswith(("+", "-")) and not l.startswith(("+++", "---")))
        print(f"{'IDENTICAL' if ok else 'DIFFERENT'}  {stem}: md {len(exp)} vs {len(out)} chars, {changed} changed lines; "
              f"report {len(rep_e)} vs {len(rep_o)} lines; images {len(im_e)} vs {len(im_o)}")
        for l in md_diff[:30]:
            print("    " + l[:220])
        for l in only_e[:6]:
            print("    expected report: " + l[:220])
        for l in only_o[:6]:
            print("    output report:   " + l[:220])
        if im_e != im_o:
            print(f"    images only expected: {sorted(set(im_e) - set(im_o))[:6]}, only output: {sorted(set(im_o) - set(im_e))[:6]}")
    print(f"{len(stems) - bad}/{len(stems)} identical")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
