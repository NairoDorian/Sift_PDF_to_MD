# The golden test

Sift's accuracy guard: seven reference datasheets (515 pages) are converted with **stored model
answers and stored layout**, so no model runs and llama.cpp's run-to-run variation plays no part.
The Markdown and the reports must equal the expected outputs **byte for byte**. Any change
between the text layer and the Markdown is checked with it before anything else.

```bash
bash bench/golden/run.sh            # both variants: with the LLM pass and without (about 1 minute)
bash bench/golden/run.sh full       # only with the LLM pass
```

It needs the release build (`cargo build --release -p sift-cli`), the reference PDFs
(`SIFT_GOLDEN_PDFS`, default `../SLAB/DATASHEET_TRUTH_SOURCE_HARDWARE` next to the repository, the
maintainer's layout), pdfium-binaries `chromium/7891` and uv (the comparison is a standard-library
Python script). The saved settings are ignored (the script points `SIFT_SETTINGS` at nothing), so
only the defaults apply.

## The reference PDFs

The PDFs are the makers' public datasheets and are **not in the repository**; get them from their
makers and check them against these hashes. Another revision of a datasheet will not match: the
golden set expects these exact files, at these paths under `SIFT_GOLDEN_PDFS`
(`tests/golden/pdfs.txt`).

| Stem | Bytes | sha256 |
|---|---|---|
| `infineon-im73a135-IM73A135V01XTSA1` | 2,526,561 | `c45dffc1cd10abcca82df0fd62f502564e5c5b67e69324756e06286228e0993a` |
| `bst-mis-hs000` | 1,512,529 | `28aee724804cf23302b067f7bdad451f9fbe9db22e2dc6f5ce9d265a7f659982` |
| `bst-bmm350-hs000` | 2,083,440 | `6d6aa0c7c8153ccf7003b6f8d39e63ef06e77c7d7e2a00bd039e74ddb4a1783f` |
| `bst-bmm350-ds001` | 1,411,269 | `a79338230dca5dade0233e1cc975f5fcfed4f149610f56c0a37cef14fcd59100` |
| `cc1101` | 2,758,709 | `daa14cd9b83adf92545ab072939f85f6a8fef9ce654e3f37366087db714ea639` |
| `Wio-LR2021 Module Datasheet` | 3,212,978 | `1ff7d592ebeafcf3ca2a9f13340185cc7d8a9cc2206e1d3d4a803ace59739bea` |
| `63293790.LR20xxDatasheet_V2_2` | 5,980,032 | `7e55a15dcdbe044dd615eb1f1ea21c26bf8e28758d94face240998e5fbad25c2` |

The expected outputs in `tests/golden/expected/` are conversions of these datasheets; they are
kept only as the test's reference.

## What `tests/golden/` holds

| Path | Content |
|---|---|
| `pdfs.txt` | `<stem>|<path under SIFT_GOLDEN_PDFS>` for each reference PDF |
| `answers/` | every model answer the reference run made (surya's tables, formulas, scanned pages and figure labels; Gemma's descriptions, table reviews, heading votes, table merges), keyed by `sha256(role, prompt, decoded pixels, max tokens, extra)` |
| `layout/<stem>/layout.json` | the layout boxes of every page (`--layout-from`), so CPU/GPU rounding at rf-detr's 0.4 threshold can't move a box |
| `expected/full/<stem>/` | `<stem>.md`, `<stem>.sift-report.txt`, `images.txt` (the image file names) with the LLM pass |
| `expected/nollm/<stem>/` | the same with `--no-llm` |

The run is `sift convert <pdf> --replay tests/golden/answers --replay-only --layout-from
tests/golden/layout [--no-llm]`: a request Sift makes differently than the reference (other
pixels, another prompt) is a miss and shows up in the report and the comparison.

## Where the baseline comes from

The answers, layouts and expected outputs are the reference run of 2026-11-06 (the Python
version, then the Rust port reproduced it byte for byte with the same inputs: text layer
941,838 characters identical, 515/515 page renders pixel-identical, 578/578 surya crops
identical). Since 2026-11-16 this folder is the only baseline.

## Changing the baseline

Only on purpose, with the measurement in `docs/LEARNINGS.md`: a change that improves the output
changes the expected files. Run the change live on the reference PDFs (`--answers <dir>` keeps the
new answers), score it (`bench/score.py` against the text layer and marker's outputs), then copy
the new answers, layouts and outputs into `tests/golden/` in the same commit as the change.
