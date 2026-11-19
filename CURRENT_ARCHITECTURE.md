# Sift: the architecture as built

How one PDF becomes `<name>.md` + `images/` + `<name>.sift-report.txt`, step by step: what each
step reads, what it decides, and where surya and Gemma are called. This file describes the code as
it is; the plan for what comes next is `MASTER_PLAN.md`, the measurements behind each choice are
in `docs/LEARNINGS.md`.

The one idea behind everything: **the PDF's own text layer is the ground truth.** Text comes from
the text layer whenever it is intact (exact, free, instant). surya reads only what the text layer
cannot give (tables, formulas, broken or scanned text, labels inside images), and whatever it
writes is checked against the PDF's text again wherever that text exists.

---

## 0. The pieces

```
sift.exe / sift-app.exe              one Rust core (crates/sift-core), two front ends
 |                                   (command line, Tauri 3 window); same options, same settings
 +-- pdfium.dll (in process)         page renders, characters with their boxes and fonts
 +-- rf-detr layout (in process)     surya's layout model on candle (GPU or CPU)
 +-- llama-server children           official llama.cpp from the user's engine folder; started on
     |                               first use, stopped when idle, killed with Sift (Job Object)
     +-- surya 2.5        tables, formulas, broken or scanned text, figure labels
     +-- Gemma 4 E4B      descriptions, the LLM pass (starts after surya stopped)
```

The models come from the user's models folder (`surya-ocr-2.5-gguf\`, `surya_layout2\`,
`gemma-4-E4B-it-qat-GGUF\`); the folders are in the settings file (`docs/settings.md`). Every model
answer can go through the **answer store** (`answers.rs`, `--answers`): key = sha256(model role,
prompt, decoded pixels, max tokens, task); a stored answer is reused. The golden test replays one.

## 1. The run, end to end

```
 PDF
  |
  1  text layer ............ pdfium chars -> pdftext's algorithm -> lines, spans, fonts
  2  page check ............ is the text layer intact? (per page)  -> digital | scanned
  3  layout ................ render 96 DPI -> rf-detr boxes + labels + order
  4  build blocks .......... boxes x lines -> blocks; decide what surya must read
  5  queue surya's work .... crops at 192 DPI -> surya ; figures cropped at 300 DPI
  |        (layout of the next pages goes on while surya already reads)
  6  apply answers ......... in queue order: parse, check against the PDF, place in blocks
  7  LLM pass .............. Gemma: table reviews, formulas
  8  descriptions .......... Gemma: one description per figure
  9  assemble .............. margins, headings (+ Gemma vote), split tables (+ Gemma),
  |                          paragraph joins across pages
 10  write ................. Markdown, images, report, trace
```

A folder is processed file by file; the next file is prepared (steps 1-5) while the previous one
finishes (steps 6-10). Cancel stops queued work, then the servers; no model starts after it.

---

## 2. The steps in detail

### 1. Text layer (`pdf/textlayer.rs`, `pdf/pdfium.rs`)
pdfium gives every character with its box, font, size and rotation. pdftext's algorithm groups
them into words, spans and lines in the PDF's order. Sift's rules on top: ink boxes,
super/subscripts kept inside their span, row fragments, and the **document vocabulary** (which
hyphenated words exist in the document, so `appli-` + `cations` joins but `on-chip` stays).

### 2. Page check (`pdf/quality.rs`)
A page whose text is missing or garbled (too few letters, private-use characters, broken
encodings) is **scanned**: its text layer is not trusted. `--mode full-ocr` treats every page as
scanned; `--mask-text-layer` hides the text layer completely (a measurement mode).

### 3. Layout (`layout/`)
Each page is rendered at 96 DPI and given to rf-detr: boxes with labels (Text, SectionHeader,
Table, Figure, Picture, Equation, Caption, ListGroup, PageHeader, PageFooter, Form, ...) and, on
scanned pages, a reading order. Pages run in parallel, one thread each (GPU: 68 ms/page).

### 4. Build blocks (`build/page.rs`)
Boxes and text lines are married:
- a line goes to the box that holds at least 40 % of it (ties: the box holding most of it, then
  the more specific label, then the smaller box); a line straddling two boxes is split;
- lines no box claims become their own text blocks (except page furniture in the margins);
- order: on digital pages the **PDF's own order** of the lines; figures and tables are placed
  after the text above them; on scanned pages the layout's order;
- each box becomes a block: paragraph, heading, list (items, levels, markers), caption, footnote,
  code, table, equation, figure. Bold/italic and sub/superscripts come from the fonts.

Then the **routing**:

| block | goes to | why |
|---|---|---|
| Table, Form | surya (`table`) | the text layer has the cells but not the structure |
| Equation | surya (`equation`) | LaTeX is not in the text layer |
| Figure, Picture | image file + description (Gemma); surya reads its labels if the PDF has none | raster images have no text layer |
| text whose PDF text is bad or missing | surya (`ocr`) | broken encoding, text drawn as graphics |
| everything else | **nothing**: the PDF's text is used as is | exact and free |
| a scanned page | surya reads the whole page (`high_accuracy_bbox` prompt), box by box as a fallback | no usable text layer |

### 5. Queue surya's work (`pipeline.rs`, `surya/client.rs`)
Each region is cropped from a 192 DPI render (1.5 pt padding), resized the way surya expects (its
`scale_to_fit`, 28-pixel grid) and queued on surya's service (6 parallel requests) with surya's
own prompt and sampling (temperature 0, top-p 0.1, warmer retries when it loops; an answer cut at
its token limit is asked again with 8,192). Figures are cropped at 300 DPI and saved as PNG.

### 6. Apply answers (`pipeline.rs::apply_block`, `surya/html.rs`, `build/tables.rs`)
Answers are applied **in the order they were queued** (runs are reproducible):
- **tables** -> a grid with merged cells, then the **checks against the PDF**: every number and
  identifier in a cell must exist in the PDF text inside the table's box. Found -> kept; exactly
  one close PDF value -> replaced by it; otherwise kept and listed in the report. Also: centred
  cells merged, math cells taken from the PDF, PDF text missing from the table put back,
  structure compared with the page geometry;
- **equations** -> LaTeX; **text** -> Markdown paragraph or heading;
- a failed or empty answer -> the PDF text is used and the report says so.

### 7. LLM pass (`brain/passes.rs`), Gemma, on by default (`--no-llm`)
- **table review**: tables the checks could not fully verify; Gemma's corrected table is kept only
  if every value is in the PDF, no verified value is lost and the structure still fits the page
  (on scanned pages: only with a self-score of 4-5/5);
- **formulas**: equation blocks surya gave no LaTeX for.

### 8. Descriptions (`describe/`, `brain/mod.rs`), Gemma
Every figure larger than an icon gets a description. Its kind (graph, timing diagram, pinout, block
diagram, photo, ...) comes from its caption and inner text, or Gemma classifies the image. The
prompt for that kind is filled with the caption, the section, the text inside the figure (from the
PDF, or read by surya) and the sentences that mention the figure. Numbers in the description are
checked against the figure's own text (the trace counts confirmed and unconfirmed numbers).

### 9. Assemble (`pipeline.rs::assemble_and_write`, `build/cleanup.rs`, `build/headings.rs`)
- running headers/footers repeated in the margins are dropped (unless `--keep-headers`);
- tables and equations without an answer fall back to the PDF text;
- **heading levels**: numbered headings get their level from the numbering (`3.2` -> level 3);
  font sizes cluster the rest; Gemma assigns levels to the unnumbered ones (asked 3 times, majority
  wins, thinking on), and the nesting is repaired;
- **tables split across pages** are joined: by rule when the columns match, otherwise Gemma
  decides from both table images;
- paragraphs split across pages or columns are joined.

### 10. Write (`render/markdown.rs`)
- Markdown: headings, paragraphs, lists, **pipe tables** (HTML tables only when cells are merged;
  `--tables markdown` forces pipes), LaTeX in `$$ ... $$`, images as
  `![Figure on page N](images/pNNN-figure-NN.png)` with the description quoted below;
- `<name>.sift-report.txt`: only what could not be resolved or was changed by a model (values not
  in the PDF, surya answers that failed, LLM corrections), sorted by page;
- `.sift/logs/`: the trace (every stage, every model request, timings) and each server's log.

---

## 3. When Gemma is called, and with what

Gemma 4 E4B (Unsloth's QAT GGUF) runs on its own llama-server, after surya has stopped.

| when | input | output | settings |
|---|---|---|---|
| figure description (step 8) | **image** + kind prompt (`prompts/describe/<kind>.txt` + `_context.txt` + `_rules.txt`) | plain text | temperature 0.05, top-p 0.35, MTP drafter |
| figure kind (step 8), when caption/text don't tell | **image** + classification prompt | one word | temperature 0 |
| table review (step 7) | **table image** + table HTML | corrected HTML (JSON) | temperature 0, no thinking |
| formula (step 7) | **formula image** + PDF text of the block | LaTeX (JSON) | temperature 0 |
| split table (step 9) | **both table images** + both HTML tables | merge yes/no, direction (JSON) | thinking on |
| heading levels (step 9) | **text only**: headings, pages, font sizes, chapter outline | level per unnumbered heading (JSON), x3, majority | thinking on |

All structured answers use a JSON schema (llama-server turns it into a grammar), temperature 0 and
no prompt cache.

## 4. Why the output can be trusted

- text from the text layer is exact; surya reads only where the text layer fails;
- every value surya writes in a table is checked against the PDF text in that area;
- Gemma's corrections are kept only if they pass the same checks and lose nothing verified;
- anything unresolved, corrected or changed by a model is in the report with its page;
- the same inputs give the same output (answers applied in queue order, fixed sampling); the golden
  test proves it on every change.

## 5. Around the conversion

| what | module |
|---|---|
| orchestration, steps 5-10 | `crates/sift-core/src/pipeline.rs` (`FileJob::prepare`, `finalize`) |
| folder mode, Cancel | `batch.rs` |
| text layer, page check | `pdf/textlayer.rs`, `pdf/quality.rs`, `pdf/pdfium.rs` |
| layout | `layout/` |
| blocks and routing | `build/page.rs`, `build/text.rs` |
| table checks | `build/tables.rs` |
| clean-up, headings | `build/cleanup.rs`, `build/headings.rs` |
| surya | `surya/client.rs`, `surya/html.rs`, `surya/compact.rs` |
| Gemma | `brain/mod.rs`, `brain/passes.rs`, `describe/`, `prompts/describe/` |
| model servers | `llm/mod.rs`, `llm/jobobject.rs` |
| the engine (llama.cpp builds: check, install, verify, use, roll back, remove) | `engine.rs` |
| the three models (catalog, download, verify, delete) | `models.rs`, `catalog/*.toml` |
| settings, folders | `config.rs`, `paths.rs` |
| answer store | `answers.rs` |
| Markdown | `render/markdown.rs` |
| command line | `crates/sift-cli/src/lib.rs` |
| window | `app/src-tauri/src/main.rs` (commands, events), `app/src/App.tsx` (pages) |
