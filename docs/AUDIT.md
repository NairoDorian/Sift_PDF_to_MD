# Audit and code review guide

A map for a full audit and code review of Sift, starting from the first public commit
(2026-11-19). It says what to verify, where the risk is, and the state the code was in when the
review started. Findings go at the bottom, one line each, and are fixed one at a time.

## Before you start

```bash
bun scripts/update-deps.ts                      # newest dependencies + full validation (or skip: the first commit was updated)
cargo fmt --check
cargo clippy --workspace --all-targets
cargo test -p sift-core --lib
cargo build --release -p sift-cli && bash bench/golden/run.sh   # 14/14 identical (docs/GOLDEN.md)
cd app && bun install && bun x tsc -b && bun run tauri dev      # the window, by hand
```

Read first: `AGENTS.md` (the hard rules and the module map), `CURRENT_ARCHITECTURE.md` (the run
step by step), `docs/LEARNINGS.md` (why the odd-looking code is the way it is: most of it matches
Python's or Pillow's exact behaviour on purpose).

**The rule for every fix:** a change between the text layer and the Markdown must keep the golden
test at 14/14, or change the golden set on purpose with its measurement in `docs/LEARNINGS.md`.
A "cleanup" that changes one byte of output is a behaviour change.

## Size and shape (lines, 2026-11-19)

| Area | Lines | What to look for |
|---|---:|---|
| `crates/sift-core/src/surya/` | 3,721 | an `html.parser`-compatible DOM and entity table, surya's compact tokens, table/Markdown/LaTeX parsing of **model output** (untrusted input), the request/retry loop |
| `crates/sift-core/src/build/` | 2,646 | blocks, headings, the table checks against the PDF (the accuracy core), clean-up |
| `crates/sift-core/src/pdf/` | 1,406 | pdfium through `unsafe` FFI, the text layer (pdftext port), quality checks |
| `crates/sift-core/src/pipeline.rs` | 1,356 | one PDF end to end; job ordering; answer application in submission order |
| `crates/sift-core/src/layout/` | 1,116 | rf-detr + reading order on candle; thread setup |
| `crates/sift-core/src/brain/` | 756 | Gemma requests, the LLM pass and its guards |
| `crates/sift-core/src/engine.rs` | 606 | llama.cpp releases: network, download, zip extraction, switching builds |
| `crates/sift-core/src/py.rs` | 596 | Python-compatible string semantics (must match CPython exactly) |
| `crates/sift-core/src/llm/` | 520 | llama-server supervisor, Job Object (`unsafe`), slots, idle stop |
| `crates/sift-cli/src/` | 514 | the command line |
| `app/src/` + `app/src-tauri/src/` | 488 + 371 | the window: 18 Tauri commands, events |
| everything else in `sift-core` | ~1,900 | imaging (Pillow LANCZOS port), config, models, batch, answers, events, paths |
| `bench/`, `scripts/` | 894 + 276 | scorers, golden comparison, dependency updater: **nothing checks these** |

Counts in the Rust code at the start: 27 `unsafe`, 123 `.unwrap()`, 8 `.expect(`, 11 `#[test]`.

## What to verify

### 1. The hard rules (AGENTS.md), as checkable statements

- **Local only.** Network code lives in `engine.rs`, `models.rs` and `llm/mod.rs` only. `llm/`
  must talk to `127.0.0.1` only; `engine.rs` and `models.rs` must run only from an explicit user
  action (a command, a button, `sift setup`), never during a conversion or at start-up unless the
  user turned on the start-up check (`[engine] check_at_startup`).
- **Folders are the user's.** Nothing is written under `%APPDATA%` except `settings.toml`
  (`paths::config_dir`); nothing is downloaded before a folder is chosen; no default engine or
  models folder anywhere.
- **No model server outlives its job.** Every `llama-server` is assigned to the kill-on-close Job
  Object before it can run work; servers stop when idle; Cancel never starts one; a model with no
  work is never started.
- **Official llama.cpp only.** Release assets come from `ggml-org/llama.cpp`; a build is verified
  before it becomes active; the previous build is kept; the CUDA version never changes on its own.
- **Two models only.** No code path for another OCR model or a model hub.
- **The document decides.** No word lists or per-document constants in `build/` and `pdf/`.

### 2. Untrusted input

- **PDFs** go to pdfium (C code). Check every FFI call holds `pdf::pdfium::lock()`, pages and text
  pages are closed exactly once and in order, buffers are sized from pdfium's own return values,
  and an encrypted, broken or zero-page PDF fails cleanly.
- **Model output** (surya's HTML, Gemma's JSON) is parsed by `surya/dom.rs`, `surya/html.rs` and
  `brain/`. Look for panics on malformed or adversarial text (slicing on byte indices inside
  UTF-8, unbounded recursion on deep nesting, huge tables), and for model text that could escape
  into the Markdown as raw HTML.
- **Release metadata and downloads** (GitHub API, release feed, Hugging Face): zip extraction
  uses `enclosed_name` (no path traversal); check asset names never become paths unchecked,
  sha256 is verified before a model or build is used, partial downloads (`*.part`) resume
  safely, and a failed switch leaves the previous build active.
- **The window's commands** (`app/src-tauri/src/main.rs`): every command taking a path or a
  name from the page (open folder in Explorer, set folder, use/remove build, delete model)
  should accept only what the app itself offered (an installed build, a catalog model) and never
  run arbitrary programs. Check the CSP and `capabilities/default.json` stay minimal.

### 3. Correctness that the golden test can't see

The golden test replays stored answers and layouts, so it does **not** exercise: the live
llama-server path (start, health, retry, loop detection, idle stop), the GPU layout build,
cancellation, folder mode with subfolders, the engine and model managers, the settings file
(load, save, env overrides, portable mode), and the window. Review these by reading and by hand.

### 4. Robustness

- `unwrap`/`expect` on anything a user's file, a model's answer, the network or the settings file
  can make fail (regex compiles of constant patterns are fine).
- Threads: pdfium's global lock, `RAYON_NUM_THREADS=1` set before rayon starts, the model gate
  (surya then Gemma), no deadlock between the file job and the model service on Cancel.
- Long work never runs on the window's command thread (ZER0's rule).

### 5. Maintainability

- Clippy and fmt clean; dead code from the removed multi-model and Python versions; doc comments
  that still describe V0 behaviour; duplicated helpers between `sift-cli` and the app.
- Unit tests: 11 today. The golden test covers the conversion path end to end; the parsers
  (`surya/`, `py.rs`, `build/tables.rs`) deserve targeted tests with small inputs.

## Known gaps at the start (not findings: already planned)

- Windows only; no installer; no CI and no pre-commit gate yet (MASTER_PLAN section 11).
- The window has not been walked page by page since its rewrite (section 11, item 1).
- No live check with the newest official llama.cpp build since the folders moved (section 11).
- `bench/score.py` is Python; porting it to Rust is backlog B7.

## Baseline at the first commit

From the dependency update and checks run on 2026-11-19, before the first commit.

| Check | Result |
|---|---|
| `bun scripts/update-deps.ts` | all steps ok: regex 1.13.2, clap 4.6.8, zip 9.7.3, tauri-plugin-dialog `v3` tip, solid-js + @solidjs/web 2.0.0-rc.23, @solidjs/vite-plugin 3.0.1, typescript 7.1.0-dev.20261119.1, vite 8.4.0-beta.1 |
| `cargo fmt --check` | clean, after one `cargo fmt` pass with the new `rustfmt.toml` (width 120, `use_small_heuristics = "Max"`); formatting only |
| `cargo clippy --workspace --all-targets` | exit 0, 17 warnings (listed below) |
| `cargo test -p sift-core --lib` | 11 passed |
| golden test | 14/14 identical (7 full + 7 no-LLM), after the update and again after formatting |
| `bun x tsc -b`, `vite build` | ok |
| `bun run tauri dev` | the window starts (smoke test only; the pages were not walked) |

## Findings

One line each: `file:line` — what is wrong — severity (bug / risk / cleanup) — golden impact —
status. The first entries are the warnings the baseline already shows; none is a known bug.

- `pdf/pdfium.rs:33,41,55` — `SizeF` and `FormFillInfo` are more private than the `Api` fields
  that use them (also a rustc warning in every build) — cleanup — none — open
- `render/markdown.rs:239-241` — caption: two `if` branches return the same `t` (merge with `||`)
  — cleanup — none expected — open
- `answers.rs:101` — test string `"…\064…"` reads as octal to clippy; Rust has no octal escapes, so
  it is `\0` + `"64"` exactly as the Python formula means (write `\x0064` or split the literal) —
  cleanup (test only) — none — open
- `py.rs:426,439` — negated comparisons on floats (`!(a < b)`): check NaN handling matches Python's
  — risk — possible — open
- `py.rs:463` — items after the test module — cleanup — none — open
- `pdf/pdfium.rs:138`, `pipeline.rs:943` — collapsible `if` — cleanup — none — open
- `build/tables.rs:539`, `events.rs:16` — very complex types — cleanup — none — open
- `build/tables.rs:694` — `sort_by` that could be `sort_by_key` (keep the sort stable and the
  first-wins order) — cleanup — possible — open
- `layout/rfdetr.rs:90` — 8 arguments — cleanup — none — open
- `layout/rfdetr.rs:323`, `pdf/quality.rs:89` — `chunks_exact` with a constant size (`as_chunks`)
  — cleanup — none — open
- `imaging.rs:137`, `layout/rfdetr.rs:553` — index loops — cleanup — none (keep the arithmetic
  order: Pillow's results depend on it) — open
- `brain/passes.rs:448`, `py.rs:421` — needless `.into_iter()` — cleanup — none — open
- `surya/compact.rs:199` — manual `Vec::pop_if` — cleanup — none — open
