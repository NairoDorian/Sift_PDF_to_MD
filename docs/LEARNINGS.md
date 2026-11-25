# What Sift's development measured

The lessons that still shape the code, newest section last. Machine: i9-13900H, 32 GB RAM,
RTX 4070 Laptop 8 GB, Windows 11. Reference set: seven datasheets (515 pages) from
`PROJECTS\SLAB\DATASHEET_TRUTH_SOURCE_HARDWARE`, with the user's marker outputs as the reference
to beat.

## Process

- Don't start the next change while a long run is going: read its trace and server logs first
  (`<out>\.sift\logs\<stamp>-sift-trace.jsonl`: every model request with tokens, seconds, retries).
- Measure every change on all reference documents, per line and per table; compare tables parsed,
  never as raw strings. The golden test (`docs/GOLDEN.md`) makes A/B comparisons exact.
- When a local model's answers are unstable, check its runtime settings (sampling, thinking,
  prompt cache) against the user's tuned launchers (`PROJECTS\Llama.cpp`) before writing prompt
  workarounds.

## Text layer (pdfium + pdftext's algorithm)

- pdfium pages must be closed explicitly, under one lock: a page left to a garbage collector was
  closed at a random moment through the form environment and crashed on the 250-page LR20xx.
- Space characters get bogus boxes: span and line boxes come from non-space characters only;
  lines are matched to layout boxes by horizontal extent x vertical middle band.
- Superscripts: a smaller, raised fragment on the same row is merged back into its line; the flag
  is kept only when the span is clearly smaller and raised above its neighbours.
- A bullet glyph without a Unicode mapping comes out as a leading space: a reliable list signal.
- The document decides: line-end hyphens and wrapped names are joined from the document's own
  vocabulary (`appli-` + `cations` joins, `on-chip` stays), never from word lists.
- Tables and figures are often drawn after the body text in the character stream: they are
  placed by their position on the page.

## Layout (surya's rf-detr)

- rf-detr often gives a Text box and a SectionHeader box over the same line: lines go to the more
  specific label; an empty duplicate box is dropped.
- In Rust (candle): 0.80 s/page on one thread; candle's element-wise ops are single-threaded, so
  pages run in parallel, one thread each (0.28 s/page on the CPU). On the GPU (candle CUDA,
  `scripts/build-cuda.cmd`): 68 ms/page. The CUDA build needs the toolkit 13.4 (cudarc 0.19 doesn't
  know 13.5), MSVC with `/Zc:preprocessor`, and no `/MD` C dependency (candle's kernels use the
  static C runtime: LNK2038).
- rf-detr's scores differ in the 3rd decimal between CPU and GPU: a box at the 0.4 threshold can
  flip. The golden test replays stored layouts for that reason.

## surya 2.5 on llama.cpp

- Q8_0 = BF16 output on the test tables, 1.3x faster, 0.5 GB less. 6 slots = 0.50 s/table (4 =
  0.59, 8 = no better).
- Not byte-repeatable with several slots (batching changes floating-point results); after
  parsing, 48/48 tables identical run to run.
- No speculative decoding: the GGUF has no MTP head; n-gram drafting measured no faster.
- surya reads figures natively (`<p class="img-text">` with the labels): its labels are given to
  Gemma as "text inside the figure" (misread labels per figure 0.71 -> 0.25).
- Every table goes to surya and every value is checked against the PDF (marker's text-layer table
  rebuild let structurally wrong tables through at its 0.75 threshold). On LR20xx 99.58 % of
  surya's checked table tokens match the PDF exactly (15,579 of 15,644); the rest are corrected
  from the PDF, none left unresolved.
- The table check: the unique closest PDF candidate wins; 2-3 neighbouring PDF words are joined
  (`32.0 ± 0.30`); words stacked in one column are joined (wrapped names); a value failing alone
  is retried joined with its neighbour by `_`, nothing or `-` (`start time` -> `start_time(31:0)`).

## Gemma 4 E4B

- MTP drafter with images: 1.5x faster, 60-90 % draft acceptance, same answers.
- Thinking is what made heading levels robust: 13/23 -> 23/23 on LR20xx; the answer still follows
  the JSON schema. Cost ~10 s per request; on for heading levels and table merges.
- llama-server's prompt cache made identical requests answer differently after other requests
  warmed it: `cache_prompt: false` on structured requests.
- 8192 tokens per slot cost +34 MiB over 4096 (Gemma 4 shares and windows its KV cache).
- Descriptions: 88.9 % (LR20xx) and 68.5 % (Wio) of the numbers in them are confirmed by the
  figure's own text; the rest are read off curves (backlog: grounding).
- Table reviews accepted 0 of 5 corrections on the digital datasheets (the guards refuse anything
  not in the PDF): on verified pages they cost time and change nothing (backlog: measure on scans).

## Speed

- Two llama-servers on one 8 GB GPU time-slice: one after the other is faster (281 s vs 288 s on
  LR20xx) and needs 2.2 GB instead of 5.8 GB. Default: surya, then Gemma.
- A model with no work is never started; a server stops 1 s after its last answer (a 5 s grace
  waited for a stop signal that never came in a console-less run).
- Layout runs batch by batch so surya gets its first tables after 8 pages.
- Cold, LLM pass on: LR20xx (250 pages) 355 s (surya 184 s, Gemma 159 s), Wio (23 pages) 86 s.
- Engine b12600 -> b12728 (2026-11-25), Wio cold, same release binary (CPU layout), back to back:
  the Markdown is identical outside the image descriptions (Gemma's wording varies run to run,
  also between two runs of one build); scorecard v2 122.26 errors per 1,000 PDF words on both
  builds and on a second b12728 run. Speed not measurable that day (the machine was busy:
  103 s and 116 s on b12728, 110 s on b12600 in between); re-time Wio and LR20xx when it is idle.
  The 122.26 against the 121.52 measured before is the same on both builds, so it is not the
  engine. Traced the same day: the golden output (the reference run's stored answers and
  layout) scores 121.52, and today's code replaying them scores 121.52 too, so the code is not it
  either. The live runs differ from it in the title's split and five heading levels (`####` for
  `#####`: layout boxes, then Gemma's heading votes) and one table cell (`PO` for `P0`: surya),
  and their layout ran on the CPU (the release build) where the reference ran on the GPU: rf-detr
  differs between the two near its 0.4 threshold (AGENTS.md). Compare live scores with the
  GPU layout build (`scripts/build-cuda.cmd`), or record the CPU build's own baseline.

## Quality against marker (scorecard v2, errors per 1,000 PDF words, lower is better)

| | Sift | marker |
|---|---|---|
| LR20xx | 46.76 | 56.13 |
| Wio | 121.52 | 136.38 |

Sift is ahead on all seven reference datasheets: table values (marker's LLM rewrite invented cells
on LR20xx p43/p64), identifiers (`SetLoRaSynchTimeout`), heading outline, list items, descriptions
(marker's online model invented figure titles and values).

## The Rust port (2026-11-06/07)

- Ported against the Python version as an oracle: every stage and every model answer recorded,
  then replayed. Text layer: 0 differences in 941,838 characters; renders 515/515 pixel-identical;
  Markdown + reports 7/7 byte-identical. That recorded run is today's golden set.
- What mattered for identical output: Pillow's LANCZOS runs the vertical pass first when the
  height shrinks more than twice as much as the width (21 % of surya crops differed by one grey
  level without it); Python's `$` before a final newline, `\w` including `²`, `strip()` including
  `\x1c-\x1f`, `round()` ties to even, `max()` returning the first maximum; prompt files read with
  universal newlines (they are CRLF on disk).
- The Markdown is written with LF line endings.

## One OCR model (2026-11-15)

Other OCR models were tried as checkers of surya's reading (2026-11-07, every page read as a
scan): 1-6 % fewer errors on most documents, one document worse, for 3-5x the time; none was
better than surya alone. Decision (user): surya 2.5 + Gemma 4 E4B only.

## The engine

- Official Windows CUDA builds of llama.cpp moved from CUDA 13.4 to 13.5 around b12174; the
  NVIDIA driver here supports 13.5. GitHub's anonymous API allows 60 requests an hour: the engine
  falls back to the releases feed and the release page.

## A page keeps its size (2026-11-23/25)

- Symptom: in the Markdown of Espressif's datasheets and manuals, text that no PDF holds ("Table 2.
  Comparison of the P. macrophylla population in the three regions of the study", about 2,000
  times in 17 files of the SLAB project), scrambled tables, and whole pages lost: on the ESP32-P4
  hardware design guidelines, pages 11-17, sections 1.3.3 to 1.3.6 were missing.
- Cause: pdfium's `FPDFPage_Flatten` reads MediaBox and CropBox from the page's own dictionary
  only, and writes them back. A page that inherits its MediaBox from the page tree (every PDF made
  by LaTeX, Espressif's included) came back as US Letter, 612 x 792 pt, whatever its real size: an
  A4 page lost its top 50 pt in every render, and the layout boxes, measured on the size read
  before the flatten, were 2.7 % off. surya was sent blank strips and half-cut tables; asked to read
  a blank image, it answers with text of its own.
- Fix: the inherited box is written into the page before the flatten, compared after it and put
  back if it moved; a page whose box cannot be kept stops the conversion (the text, the renders and
  the layout must describe one page). Every box arrangement measured on pdfium 7891 is a unit test.
  An image of one colour is never sent to a model. The words surya reads in a text block that the
  page's own text does not have are listed in the report ("surya read N words that the PDF text of
  this page does not have").
- Measured: four ranges (ESP32-P4 pages 41-46, its hardware design guidelines 11-17, ESP32-S31
  36-40, the S31 errata 3-6): 11 made-up captions before, none after; the lost sections back; no
  word reported as off its page. Golden set: Markdown 14/14 byte-identical (no reference PDF has an
  inherited box); the two LR2021 reports gain the new check's lines (the "seeed studio" logo, the
  package drawing's notes, drawn as outlines), so their expected reports were updated; answers and
  layouts unchanged.
- To tell an affected PDF: a page with no MediaBox of its own and at least one annotation (a page
  without annotations is left alone by the flatten) changes size when flattened and reloaded. In
  the SLAB project, 17 of 108 conversions (all Espressif's, 6,960 of their 7,284 pages); every
  Markdown made from such a PDF before this fix is to be converted again.
