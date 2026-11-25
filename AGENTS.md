# AGENTS.md

Guidance for AI coding assistants (and people) working on **Sift**: PDFs to Markdown with their
images, image descriptions and a verification report, with two local models (surya 2.5 and
Gemma 4 E4B) on official llama.cpp. Rust core + command line, Tauri 3 + SolidJS 2 + Bun window,
Windows only for now. Structure and habits follow the maintainer's ZER0 app (`PROJECTS\Handy_V2\AGENTS.md`,
a fork of Handy); the lessons copied from it are marked "(ZER0)" below.

Repository: <https://github.com/NairoDorian/Sift_PDF_to_MD> (public, Apache-2.0). Everything lives
at the repository root (until 2026-11-19 it lived in a `V1/` folder; code comments still say V0
for the first, Python version and V1 for this Rust version).

- The plan (living, kept current): [MASTER_PLAN.md](MASTER_PLAN.md)
- How the pipeline works today: [CURRENT_ARCHITECTURE.md](CURRENT_ARCHITECTURE.md)
- What was measured: [docs/LEARNINGS.md](docs/LEARNINGS.md); the accuracy guard: [docs/GOLDEN.md](docs/GOLDEN.md)
- Reviewing or auditing the code: [docs/AUDIT.md](docs/AUDIT.md)

## Read first: the hard rules

- **Two models only: surya 2.5 and Gemma 4 E4B** (plus surya's rf-detr layout model). Don't add,
  propose or keep code for other OCR models or a model hub (decided 2026-11-15 after measuring).
- **Local only.** Converting never calls a network service; no API keys. The network is used only
  by explicit actions: `sift setup`, `sift engine check|update`, `sift models download|check`,
  their buttons, `scripts/update-deps.ts`.
- **Folders are the user's.** The llama.cpp engine and the models go only into the folders chosen
  in the window or with `sift engine folder` / `sift models folder` / `sift setup`. Never a default
  folder, never `%APPDATA%` (only the small `settings.toml` lives in the app's config folder).
- **Official llama.cpp builds only** (ggml-org/llama.cpp releases), CUDA 13.5 by default. No
  community-patched engines, overlays or forks. Sift never switches the CUDA version on its own.
- **Accuracy first, never traded for speed.** `bash bench/golden/run.sh` must stay 14/14 identical
  after any change between the text layer and the Markdown; a deliberate output change updates
  the golden set together with its measurement in `docs/LEARNINGS.md`. marker is the reference
  to beat.
- **The document decides.** No word lists, no per-document constants, no hand-picked thresholds;
  rules read the document's own words, fonts and positions. Model constants (surya's 12,288-token
  slot) live with their source.
- **No model server outlives its job**; a model with no work is never started; Cancel never starts
  a model. Servers run in a kill-on-close Job Object (`llm/jobobject.rs`).
- **Strict dependencies** (`docs/DEPENDENCIES.md`, each with its reason), always at their newest
  versions, prereleases included (`bun scripts/update-deps.ts`).
- **Commit only when the user asks.** Plain language in every message, report and UI text; never
  write "VLM".

## The routine, in order (ZER0)

The routine is what you run before a commit; it is not optional and has no shortcut.

```bash
bun scripts/update-deps.ts      # 1. every dependency to its newest version (prereleases incl., Tauri 3 and
                                #    Solid 2 included), then cargo update -> check -> unit tests -> bun install ->
                                #    tsc -> vite build -> release build -> golden test (--quick skips the last two)
bun scripts/precommit.ts --full # 2-4 in one command, stopping at the first failure: cargo fmt --check, clippy
                                #    (warnings fail), unit tests, tsc, vite build, the workflow files parse, then
                                #    the release build and the golden test (14/14 identical); without --full, the
                                #    quick part only (about 40 s)
cd app && bun run tauri dev     # 5. after a window change: open it and walk the pages you touched
git commit                      # 6. only when the user asks; the hook runs the quick gate on the working tree
```

**The pre-commit gate** (ZER0): `scripts/precommit.ts`, also `bun run precommit` /
`precommit:full` from `app/`. Git runs its quick part on every commit once the hook is on, per
clone: `bun run hooks:install` from `app/` (`git config core.hooksPath .githooks`; it is on in
the maintainer's clone, as in ZER0's); `hooks:uninstall` turns it off. The golden test stays out
of the hook (minutes, not seconds): run `--full` before a commit that touches anything between
the text layer and the Markdown, and before a release.

**Step 1 always goes to prerelease, for every dependency.** For part of the stack the newest is
only ever a prerelease: Tauri 3 only as alphas, TypeScript's newest as dev builds. A stable-only
update would silently hold them back. The few packages that can't follow "newest published" are
held on purpose and the report says why: `NPM_LINE_PINNED` (the Solid packages stay on their major,
Solid 2 and its Vite plugin 3, and take the newer of their `next` and `latest` tags: each tag has
been the stale one; following `next` alone kept Sift on 2.0.0-rc.23 after Solid 2.0 went stable),
`NPM_MAJOR_LOCKED_PREFIXES` (`@tauri-apps/*` stays on the backend's major, so a JS API never talks
to a different Tauri over IPC) and `CARGO_MAJOR_LOCKED` (the Tauri crates). Transitive
dependencies move too (`cargo update`, `bun update`); the toolchain (Rust, Bun) and GitHub Actions
are reported, never changed (`rustup update`, `bun upgrade` are the maintainer's call: they are
machine-wide). pdfium stays on `chromium/7891` on purpose (the golden test expects its output).

**A lockfile that resolves is not a lockfile that builds** (ZER0): `update-deps` always runs its
validation (check, tests, tsc, a production Vite build, the release build, the golden test).
`--quick` skips only the last two, and only for an intermediate run.

**Read a script before trusting it** (ZER0): `scripts/` and `bench/` are outside `tsconfig` and
outside every check; nothing type-checks or tests them.

### rtk (ZER0)

The maintainer's Claude Code setup proxies shell commands through
[rtk](https://github.com/rtk-ai/rtk), which condenses command output to save tokens. **`rtk` is used
for every shell command, with one exception: `bun` commands are never proxied** (`rtk bun …` is not
supported). So `rtk git …`, `rtk cargo …`, `rtk gh …` and plain `bun …`. Treat condensed output
as complete; re-run with `rtk proxy <cmd>` only when a result is clearly unusable (empty when
output was expected, contradicting its exit code, or garbled). rtk is tooling for the agent, not a
dependency of Sift: contributors without it run the same commands without the prefix.

## Development commands

```bash
cd app && bun install && bun run tauri dev       # the window in development: Vite dev server (port 1430) + the app,
                                                 # rebuilt on change; the quickest way to test the program by hand
cd app && bun run dev                            # the window alone in a browser (http://localhost:1430) on lib/mock.ts:
                                                 # ?demo=running|done|folder|setup ?page=<page> ?theme=dark|light ?motion=on
bun scripts/gen-icons.ts                         # app icons (tauri icon) + docs/brand art, from app/src/lib/brand.ts
cargo build --release -p sift-cli                # target/release/sift.exe
sift setup --engine-dir <dir> --models-dir <dir> # folders + newest engine + the three models (development machine: engine\, models\ at the repository root, git-ignored)
sift convert <pdf|folder> -o <out> [--pages 1-5,137] [--no-llm] [--describer off] [--json] [--profile]
sift <pdf>                                       # same as sift convert <pdf>
sift convert ... --answers <dir>                 # keep every model answer (a re-run reuses them)
sift convert ... --replay <store> --replay-only --layout-from <dir>   # exact A/B (the golden test)
sift convert ... --mask-text-layer               # read every page as scanned (score with bench/score.py)
sift engine status | check | update | list | use <b> | rollback | remove <b> | remove-runtime <b> | folder [<dir>]
sift models status | check | download [id] | verify [id] | delete <id> | folder [<dir>]
sift config show | path | set <section.key> <value>
sift dev textlayer <pdf>                         # the text layer as JSON (the scorers' ground truth)
sift dev crop <pdf> <page> x0 y0 x1 y1 <out>     # a crop as surya sees it
uv run --no-project python -I bench/score.py <pdf> name=out.md [name=marker.md]   # scorecard v2
```

The window, release build: `cd app && bun install && bun run build && cd .. && cargo build --release
-p sift-app` (`sift-app.exe`; with a subcommand it runs the command line without a window, printing
to the console it was started from). The installer: `cd app && bun x tauri build --config
src-tauri/tauri.release.conf.json` (`target/release/bundle/nsis/`; see the gotcha on its
resources); on GitHub, Actions > Release builds it into a draft release. Layout on the GPU:
`scripts\build-cuda.cmd` (output in `target-cuda\`).

Development prerequisites: Rust 1.99+, Bun 1.4+, uv and bash (Git Bash) for the golden test and the
scorers, and `pdfium.dll` from pdfium-binaries `chromium/7891` in `runtime/pdfium/` (git-ignored;
`SIFT_PDFIUM` overrides). The reference PDFs for the golden test live outside the repository
(`SIFT_GOLDEN_PDFS`, default `../SLAB/DATASHEET_TRUTH_SOURCE_HARDWARE`; `docs/GOLDEN.md` lists them
with their sha256).

## Architecture overview

```
Sift_PDF_to_MD/
  README.md  AGENTS.md  MASTER_PLAN.md  CURRENT_ARCHITECTURE.md  LICENSE  NOTICE
  Cargo.toml            workspace (versions in [workspace.dependencies], kept newest by update-deps)
  crates/sift-core/     everything that converts (module map below)
  crates/sift-cli/      the `sift` command line (also run by the app binary when given a subcommand)
  app/                  Tauri 3 + SolidJS 2 window: Setup (first run), Convert, Engine, Models, Settings, Logs, About
    src/lib/            api.ts (typed commands), store.ts (shared state, events), brand.ts (the mark), mock.ts (dev preview)
    src/components/     ui.tsx (ZER0's groups, rows, switches), SiftFlow.tsx (the sieve), Icon, Brand, Sidebar, Footer
    src/pages/          one file per page
    src-tauri/          main.rs (commands), tauri.conf.json, tauri.release.conf.json (installer resources), icons/
  .github/workflows/    release.yml: manual, builds the installer into a draft release
  catalog/              surya-2.5.toml, surya-layout.toml, gemma-4-e4b-qat.toml: pinned files + sha256
  prompts/              surya-2.5/ (surya's own, word for word), describe/ (figure prompts)
  tests/golden/         the golden set (docs/GOLDEN.md)
  bench/                golden/ (run.sh, compare.py), score.py + compare_marker.py (standard-library Python)
  scripts/              update-deps.ts (Bun), gen-icons.ts (app icons + docs/brand art from brand.ts), build-cuda.cmd
  docs/                 AUDIT, LEARNINGS, GOLDEN, settings, events, DEPENDENCIES, blocks.schema.json
  engine/, models/      the development machine's engine and models folders (git-ignored; chosen in the settings)
  runtime/pdfium/       development copy of pdfium.dll (git-ignored)
```

### sift-core, module by module

| Module | What it does | Ported from |
|---|---|---|
| `py` | Python-compatible text semantics the ported rules rely on: `str.isspace/isalnum/isupper`, `strip`, `split`, `round` (ties to even), `repr`, regex `\w`, `\s`, `$` before a final newline, `max/min` first-wins | (port layer) |
| `paths` | settings file, the user's engine and models folders (no defaults), prompts, catalog, `read_text` (universal newlines) | |
| `config` | the settings (`docs/settings.md`): `[folders]`, `[engine]`, `[convert]`, `[surya]`, `[gemma]`, `[llm]`, `[describe]`, `[scheduler]`; env overrides; load / save / set | |
| `imaging` | RGB buffers, crops, **Pillow's LANCZOS** (fixed point, vertical-first rule), surya's `scale_to_fit`, PNG, pixel hashes | Pillow `Resample.c`, surya `util.py` |
| `pdf::pdfium` | pdfium's C API through libloading, one global lock, explicit page lifetimes, form flattening | |
| `pdf::textlayer` | pdftext 0.7.1 (chars, word dedup, spans, lines, scripts) + Sift's rules (ink boxes, row fragments, script runs, the document vocabulary) | pdftext |
| `pdf::quality` | broken-text checks (marker's thresholds) | marker |
| `layout` | rf-detr (DINOv2-S windowed + LW-DETR) and the reading-order head on candle, page-parallel; labels, containment merge | surya's fast layout |
| `surya::{dom, entities, compact, html, client}` | surya's HTML format: an `html.parser`-compatible DOM, the HTML5 entity table, surya 2.5's compact tokens, tables / Markdown / LaTeX / figure labels, the request + retry loop | surya |
| `build::{blocks, page, text, headings, tables, cleanup}` | boxes + lines -> blocks, inline Markdown, lists, heading levels, the table checks against the PDF, margins, joins | |
| `render::markdown` | blocks -> Markdown (pipe tables, HTML only for merged cells) | |
| `describe` | figure context, kinds, description prompts | |
| `brain` (+ `passes`) | Gemma 4 E4B: server flags, descriptions, classification, JSON-schema requests; the LLM pass (table review, heading levels, table merge, formulas) with its guards | |
| `llm` (+ `jobobject`) | llama-server supervisor (free port, health, logs, stop), `ModelService` (slots, lazy start, stop when idle, gate: surya then Gemma), chat client, loop detection | ZER0 `job_object.rs` |
| `answers` | the answer store: key = sha256(role, prompt, pixel hashes, max_tokens, extra) | |
| `engine` | official llama.cpp builds in the engine folder: release list, check, download, extract, verify, switch, list, use, rollback, remove, CUDA runtime handling | ZER0 `llama_releases.rs` |
| `models` | the three pinned models: status, resumable sha256-checked downloads, verify, delete, upstream revision check | |
| `events` | JSONL / human progress events (`docs/events.md`) | |
| `meters` | the footer's CPU, RAM, GPU and VRAM: Windows' `GetSystemTimes` / `GlobalMemoryStatusEx`, NVIDIA's `nvml.dll` by its full path through libloading (no new dependency) | ZER0 `system_monitor.rs` |
| `pipeline`, `batch` | one PDF: text -> layout batches -> blocks -> queued surya jobs -> answers applied **in submission order** -> Gemma -> clean-up -> Markdown + report; folder mode | |

### Key patterns

- **Every model answer can go through the answer store**; keys use decoded pixels, never PNG bytes,
  so any encoder gives the same key.
- **Deterministic application order**: jobs return results; the file job applies them in the order
  it queued them.
- **Identity is an id**: blocks carry a unique `id`.
- **The text layer is the ground truth** where intact: table values are checked and corrected
  against it; every correction and every unresolved value is in the report.
- **Commands never block the window** (ZER0's rule): long work runs on its own thread and reports
  through events.

## Gotchas (each cost time once)

- Official llama.cpp's Windows CUDA builds moved from 13.4 to **13.5** (b12174+). GitHub's anonymous
  API allows 60 requests an hour: the engine falls back to the releases feed and the release page.
- A shell hook halves `\\` inside bash heredocs: write code that contains backslashes with the
  Write/Edit tools, never through `cat <<EOF` or `python -c`.
- Prompt files are read with universal newlines (`paths::read_text`): every answer-store key hashes
  the prompt text, so a CRLF checkout must not change it (V0's prompt files were CRLF). The
  repository stores every text file with LF (`.gitattributes`), and `tests/golden/` is never
  converted.
- Pillow runs LANCZOS vertical-first when the height shrinks more than twice as much as the width;
  without that rule 21 % of surya's crops differed by one grey level.
- rf-detr's scores differ in the 3rd decimal between CPU and CUDA: boxes at the 0.4 threshold can
  flip. Compare outputs with stored layouts (`--layout-from`).
- pdfium is not thread-safe: every call goes through `pdf::pdfium::lock()`.
- candle's element-wise ops are single-threaded: pages run in parallel, one thread each
  (`layout::init_threads` sets RAYON_NUM_THREADS=1 at start-up).
- **Layout on the GPU** (`scripts/build-cuda.cmd`, feature `sift-core/cuda`, output in
  `target-cuda/`): 68 ms/page instead of 280 on the CPU. The build needs the CUDA toolkit **13.4**
  (cudarc 0.19 panics on 13.5: the script puts 13.4 first on PATH), MSVC's `cl.exe` (vcvars64) and
  `CL=/Zc:preprocessor`; any C dependency built `/MD` breaks the link with LNK2038.
  `SIFT_LAYOUT_DEVICE=cpu` forces the CPU. The link warns LNK4098 (LIBCMT against the kernels'
  objects); the binary runs (Wio converted end to end, 2026-11-25). The first build took 79 min
  here (nvcc compiles candle's kernels; at low priority on a busy machine).
- Tauri 3 is alpha: the Tauri crates, `@tauri-apps/*` and `tauri-plugin-dialog` (plugins-workspace
  `v3` branch) must agree; update them together with `scripts/update-deps.ts`, never one by hand.
- Solid 2 batches writes: a signal read right after its setter, in the same frame, still returns
  the old value. Compute from locals (lib/store.ts does).
- WebView2 follows Windows' "Animation effects": with them off, `prefers-reduced-motion` is
  `reduce` (the maintainer's machine has them off). The sieve then moves gently; Settings >
  Appearance > Motion forces it. Headless Chrome screenshots inherit the same setting
  (`?motion=on` in the dev preview).
- The release config (`tauri.release.conf.json`) adds the installer's resources, which must exist:
  `target/release/sift.exe` (its `beforeBuildCommand` builds it) and `runtime/pdfium/` with
  `pdfium.dll`, `LICENSE` and `licenses/` from the pdfium-binaries archive. The base config has
  none, so `cargo check` and `tauri dev` work on a fresh clone. NSIS only: MSI refuses
  prerelease versions.
- Moving the repository folder moves the development engine and models folders with it: the
  settings file keeps absolute paths (`sift config path`; fix with `sift engine folder` /
  `sift models folder`), and cargo rebuilds `target/` from scratch.

## Code style (ZER0)

**Rust:**
- `cargo fmt` and `cargo clippy` before committing.
- Handle errors explicitly (`anyhow` with context); no `unwrap` on anything a user's file or a
  model's answer can make fail.
- Descriptive names; doc comments on public items that say why, and cite the rule (`D10`, `VD7`)
  or the measurement (`docs/LEARNINGS.md`) behind a number.

**TypeScript / Solid 2:**
- Strict TypeScript, no `any`.
- Never destructure props in a component body (it runs once: a destructured prop is a mount-time
  snapshot); read reactive values inside JSX bindings, not in component bodies; use the
  `createEffect(compute, apply)` form with a returned cleanup; `Dynamic` for components switched
  reactively.
- Commands never block the window: long work runs on its own Rust thread and reports through
  events (`docs/events.md`).
- **One place per setting.** A setting has one control, on the page it belongs to (conversion
  options on Convert, the engine's on Engine, the models folder on Models); other pages link to
  it. The Convert page's options are the saved defaults the command line reads too.
- The look is ZER0's (DESIGN.md there): 0 corner radius except switches, 1px hairlines, no
  shadows, tracked uppercase group titles, mono for numbers; plain CSS variables in `styles.css`,
  amber the only accent. No network call from a page opening: only from a button (or the opt-in
  start-up check).

## Commits and GitHub (ZER0)

- Commit only when the user asks. Conventional prefixes (`feat:`, `fix:`, `docs:`, `refactor:`,
  `chore:`, `perf:`, `test:`); the message says **why**, not what.
- State the cost of any new thread, poll, dependency (with its line in `docs/DEPENDENCIES.md`) or
  model call in the commit message.
- A change between the text layer and the Markdown names its golden result (14/14, or the
  measured reason the golden set changed) in the message.
- End the message with a `Co-Authored-By:` line naming the assistant when an assistant wrote it.
- Never write a bare `@name` in a commit, issue or PR unless you mean to notify that GitHub user.
