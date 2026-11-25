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
status. The first entries are the warnings the baseline showed; none was a known bug.

**Round 1 (2026-11-25): clippy is clean (0 warnings, Rust 1.102 nightly), golden 14/14 after the
round, 14 unit tests pass.** Code the golden test does not run (the rf-detr layout, replayed from
stored layouts) only got rewrites that keep the same operations in the same order.

- `pdf/pdfium.rs:33,41,55` — `SizeF` and `FormFillInfo` more private than the `Api` fields that
  use them — cleanup — none — fixed: the `Api` function pointers are private to the module (no
  other module called them; the compiler now keeps it that way)
- `render/markdown.rs:239-241` — caption: two `if` branches return the same `t` — cleanup — none
  — fixed: one condition with `||`
- `answers.rs:101` — test string `"…\064…"` reads as octal to clippy (it is `\0` + `"64"`) —
  cleanup (test only) — none — fixed: written `\x00` + `"64"`, same bytes
- `py.rs:426,439` — negated comparisons on `PartialOrd` keys — risk — none — fixed: the same rule
  through `partial_cmp`, documented: CPython's `max`/`min` replace the best only on a strictly
  greater/smaller key, so an equal or incomparable (NaN) key keeps it, exactly as before
- `py.rs:463` — items after the test module — cleanup — none — fixed: the module moved last
- `pdf/pdfium.rs:138`, `pipeline.rs:999` — collapsible `if` — cleanup — none — fixed: a
  let-chain; the pipeline one only called `table_to_figure` for its effect, now a plain call
- `build/tables.rs:539`, `events.rs:16` — very complex types — cleanup — none — fixed: `GridMap`
  and `Sink` type aliases
- `build/tables.rs:694` — `sort_by` that could be `sort_by_key` — cleanup — none — fixed:
  `sort_by_key(Reverse(len))`, still a stable sort, ties keep their order
- `layout/rfdetr.rs:90` — 8 arguments — cleanup — none — kept: the 8 plain values of the resize
  it ports, with a documented `allow`
- `layout/rfdetr.rs:323`, `pdf/quality.rs:89` — `chunks_exact` with a constant size — cleanup —
  none — fixed: `as_chunks::<3>()` (same elements, same order, remainder ignored as before)
- `imaging.rs:137`, `layout/rfdetr.rs:553` — index loops — cleanup — none — fixed: iterators over
  the same indices in the same order (Pillow's sums keep their order)
- `brain/passes.rs:448`, `py.rs:421` — needless `.into_iter()` — cleanup — none — fixed
- `surya/compact.rs:199` — manual `Vec::pop_if` — cleanup — none — fixed: `pop_if`
- `engine.rs:use_build` — a build name could be a path: `engine_dir.join(name)` accepted an
  absolute path or `..`, and the verify step runs that folder's `llama-server` (window: Use;
  command line: `sift engine use`) — risk — none (not the conversion path) — fixed: the name must
  be in the engine folder's own list, as `remove` and `remove_cuda_runtime` already did; checked
  with `..\engine`, `C:\Windows` and an unknown build (all refused, nothing switched)
- `app/src-tauri/src/main.rs:open_path` — the window's open command handed any path to Explorer,
  which runs a program it is given — risk — none — fixed: folders, `.md` and `.txt` only (what
  Sift writes); `reveal_path` only selects, `open_url` only opens `https://`
- `.github/workflows/release.yml` — GitHub could not parse it (a plain YAML value holding
  `": "`), so it had no trigger and every push showed a failed run — bug — none — fixed in
  `bafaee5`; check workflows with a YAML parser before pushing
- `Cargo.toml` — `[workspace.package] rust-version` declared but inherited by no crate, so the
  minimum Rust (1.99) was never enforced (a nightly cargo lint) — cleanup — none — fixed in
  `b685ab9`: every crate inherits it

**Round 2 (2026-11-25): untrusted input, robustness, the hard rules; golden 14/14 after the round,
20 unit tests (6 new), clippy clean.**

- `surya/dom.rs:parse` — no limit on nesting: the walks over the tree (`html.rs`) recurse once per
  level, so a model that nests a few thousand tags (a loop gone wrong) overflowed the thread's
  stack and aborted the whole process (reproduced: 100,000 nested `<b>` gave
  `STATUS_STACK_OVERFLOW`) — bug — none — fixed: past `MAX_DEPTH` (256; real output nests about
  10 deep) a tag is kept but not entered; test `deep_nesting_is_capped`
- `surya/html.rs:415` — `colspan`/`rowspan` from the model had no upper bound and the grids loop
  `rowspan x colspan` times: a made-up `colspan="99999999"` hung the run or ran out of memory —
  risk — none — fixed: capped at the HTML standard's limits (1000, 65534; browsers do the same),
  and the three grid loops (`build/tables.rs`, `render/markdown.rs`) visit only the table's own
  rows (rows past its end were never read); tests for spans and for expanding a capped cell
- `engine.rs:install`, `current_build` — the release tag, the backend, the asset names (from the
  release list) and the build in `current.txt` became paths unchecked: a name with `..` or a
  separator would reach outside the engine folder (zip entries were already safe:
  `enclosed_name`) — risk — none — fixed: they must be plain names; test
  `names_from_a_release_list_stay_plain`
- `batch.rs:is_done` — folder mode counted a Markdown as up to date by its date alone, so a quick
  `--pages 1-2 --no-llm` run stood in for the real conversion on the next run — bug — none
  (the golden test converts single files) — fixed: the options of each conversion are kept in
  `.sift-options` next to its Markdown and must match (outputs from before have none and are
  trusted as before); checked: same options skipped, other pages reconverted
- `pdf/pdfium.rs:216` — an unreadable PDF said "error 3" — cleanup — none — fixed: pdfium's codes
  in words ("it is not a PDF, or it is damaged", "it is protected by a password"...)
- `models.rs:download_file` — a `.part` already complete (a run stopped while checking it) was
  downloaded again in full — cleanup — none — fixed: straight to the size and sha256 check

Checked and sound, no change:
- Model output: every string slice in `surya/`, `brain/`, `build/`, `render/` cuts at a position
  from `find`, a regex or a char index (no cut inside a UTF-8 character); `describe::clip` is
  only called with 200 or more; surya's box coordinates are `\d{1,4}` before `parse::<u32>()`.
- PDFs: empty, plain text, truncated and garbage files each fail cleanly ("cannot open ...",
  exit 1, the job goes on); nothing panics.
- Downloads: zip entries through `enclosed_name`; a `.part` resumes with a Range request, a
  server that ignores it restarts the file, the size and the pinned sha256 decide before the
  rename.
- Hard rules: HTTP only in `engine.rs`, `models.rs` and `llm/` (servers on `127.0.0.1` only);
  `%APPDATA%` only for `settings.toml`; every llama-server is in the kill-on-close Job Object as
  soon as it is spawned, before any request; the start-up check only when turned on.
- Folder mode: subfolders mirrored, a broken PDF reported while the others go on, `--force`.
- The remaining `unwrap`s on input-driven paths all follow a check that makes them safe.

**Round 3 (2026-11-25): Cancel, the first run, dead code; golden 14/14, 20 unit tests, clippy
clean (warnings fail).** Environment overrides and portable mode were checked in `6dc8c9a`.

- `batch.rs`, `pipeline.rs:finalize_inner` — after Cancel the PDF in progress carried on with
  every model request failing (the services shut down on purpose), then wrote its Markdown and
  report over the earlier good ones and stamped `.sift-options`, so a later folder run took the
  degraded copy as up to date; its report blamed "surya server unavailable" — bug — none (the
  command line has no Cancel) — fixed: `Runtime::check_cancel` between layout batches, at the
  start of finalize and before the write; the file ends as `file_error` with `cancelled: true`,
  nothing written or stamped. Checked in the window: the earlier `.md`, report and stamp keep
  their hashes and times
- the same path — Cancel did not stop CPU work: a long PDF ran its whole layout after it —
  risk — none — fixed by the same check per layout batch
- window, Convert — after Cancel the stage list ticked stages whose requests had failed at once
  ("LLM pass 7/7") and offered "Open the Markdown" — cleanup — none — fixed: counts freeze at
  Cancel; the callout says what was kept
- `pipeline::server_command_lines` (new, never released) — built `LlamaServer`s whose blocking
  HTTP client panics when dropped on Tauri's async runtime: the Engine page's Command panel
  never loaded — bug — none — fixed: `llm::command_line`, shared with the server, no server
  object; no other async command builds one
- `pdf/pdfium.rs` — the not-found error sent users to `sift setup pdfium`, which does not exist,
  with an empty "Tried:" list — bug — none — fixed: says where pdfium belongs, lists every place
  looked
- dead code — `set_library_path`, `Runtime::wait_brain_gate` and its field, `Gate::is_open`,
  `ModelService::wait_idle`, `AnswerStore::disabled`, `Rgb::from_png`, `Dom::has_attr`,
  `Re::match_start` (whose comment promised a retry it never made), `str_isspace`,
  `last_dim_max`, `words_of`, `queue_full_page`'s unused argument — cleanup — none — removed;
  `LlamaServer::stop`'s comment said "1 s grace" for a 50 ms wait — fixed

Checked and sound, no change:
- Cancel by hand in the window (UI Automation): before any model, no llama-server in the 25 s
  after; during surya, servers gone 2.6 to 3.5 s after it, none in the 25 s after.
- The first run on an empty profile (`SIFT_SETTINGS` on a new file): Setup 0/4, both folders,
  llama.cpp installed into an empty folder (226 MB, CUDA runtime from the toolkit, verified
  before switching), models found in the chosen folder, then "Everything is in place".
- The remaining V0 mentions name where code was ported from (kept on purpose, AGENTS.md).
