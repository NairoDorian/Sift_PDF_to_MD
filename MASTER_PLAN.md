# Sift: Master Plan

The living plan of Sift: updated as development goes (last update 2026-11-19). How the code works
today is in `CURRENT_ARCHITECTURE.md`; what was measured is in `docs/LEARNINGS.md`.

**What Sift is.** A local Windows app with a command line that turns PDFs (one file or a folder)
into Markdown: text, headings, lists, tables, formulas, every image kept at 300 DPI with a written
description, and a short report of anything it could not verify. Two models do the work, both on
official llama.cpp (CUDA 13.5):

- **surya 2.5** reads what the PDF's text layer can't give: tables, formulas, scanned or broken
  text, labels inside images. surya's own layout model (rf-detr) finds the blocks on each page.
- **Gemma 4 E4B** describes the images and runs the LLM pass: heading levels, tables split across
  pages, missing formulas, tables the checks couldn't verify.

Nothing else: no second OCR model, no cloud, no account.

Built with Rust (the converter) and Tauri 3 + SolidJS 2 + Bun + TypeScript (the window), on the
skeleton of the user's ZER0 app (`PROJECTS\Handy_V2`). ZER0 also gives Sift its llama.cpp
manager: detect new builds, install, update, roll back, from the Engine page or one command.

---

## Start here (handover for the next AI agent)

**Status (2026-11-25).** The final architecture is in place, the code is public, and releases ship
as Windows installers from the Release workflow (1.0.0-alpha.2 was the first; 1.0.0-alpha.3
carries the audit's fixes, the server panels, the meters and the strainer-only mark):
- **Repository:** <https://github.com/NairoDorian/Sift_PDF_to_MD>, first commit 2026-11-19. Everything
  is at the repository root (the `V1/` folder was flattened that day). The first, Python version
  and the multi-model experiments are gone; the Python version's recorded run lives on as the
  golden set.
- **Next step: a full audit and code review** (`docs/AUDIT.md`), before section 11.
- **Two models:** surya 2.5 and Gemma 4 E4B only.
- **Folders:** the engine and models folders are chosen by the user and stored in the settings.
  Nothing is downloaded into `%APPDATA%`.
- **Command line:** `sift setup`, the full `sift engine` set (status, check, update, list, use,
  rollback, remove, remove-runtime, folder), `sift models` (status, check, download, verify,
  delete, folder) and `sift config`.
- **The window (redesigned 2026-11-25, ZER0's look in Sift's palette):** Setup (first run),
  Convert (the PDF, the sieve, the Markdown; the saved options), Engine, Models, Settings (theme,
  motion, GPU sharing), Logs, About. One place per setting (section 9).
- **Release:** NSIS installer with `sift.exe`, pdfium and its licenses, prompts, catalog
  (`app/src-tauri/tauri.release.conf.json`); `.github/workflows/release.yml` builds it into a draft.
- **Golden test:** 14/14 byte-identical.
- **Dependencies:** at their newest prereleases.

Next: the audit (`docs/AUDIT.md`), then section 11.

**Read in this order:** this section; 1 (the idea); 2 (decisions); 5 (folders); 6 (engine);
11 (what's next). Before touching the app, read `AGENTS.md`, ZER0's `AGENTS.md` and the files in 4.4.

**Hard rules (never break):**
- **Local only.** Converting never touches the network. The network is used only when the user
  asks: checking for or installing a llama.cpp build, downloading a model, `scripts/update-deps.ts`.
- **Two models only:** surya 2.5 and Gemma 4 E4B QAT, plus surya's layout model. Don't add,
  propose or keep code for other OCR models (2.1).
- **Nothing in `%APPDATA%` except the small settings file** (portable mode keeps even that next to
  `sift.exe`). The engine and the models go only into the folders the user chose; nothing is
  downloaded before a folder is chosen.
- **Official llama.cpp builds only** (ggml-org/llama.cpp releases): no community patches, overlays
  or forks. Sift never switches the CUDA version on its own.
- **Accuracy first, never traded for speed.** The golden test (`docs/GOLDEN.md`) must pass after
  every change between the text layer and the Markdown; a deliberate output change updates the
  golden set with its measured reason in `docs/LEARNINGS.md`. marker is the reference to beat.
- **The PDF's text layer is the ground truth; the document decides** (no word lists, no
  per-document constants).
- **No model server outlives its job**; a model with no work is never started; Cancel never starts
  a model (Job Object).
- **Strict dependencies**, kept at their newest versions (prereleases included).
- Never kill processes Sift didn't start. Commit only when the user asks. Plain language
  everywhere; never write "VLM".

### Where everything is

| What | Path |
|---|---|
| Repository | `PROJECTS\Sift-PDF-to-MD\` = <https://github.com/NairoDorian/Sift_PDF_to_MD> (`main`) |
| This plan, the architecture, the guide | `MASTER_PLAN.md`, `CURRENT_ARCHITECTURE.md`, `AGENTS.md` |
| Engine and models on the development machine | `engine\`, `models\` at the repository root (git-ignored; set in the settings file, changeable in the app) |
| Settings file | `%APPDATA%\com.nairodorian.sift\settings.toml` (`sift config path`) |
| Golden set | `tests\golden\` (`docs/GOLDEN.md`) |
| ZER0 (skeleton, engine manager) | `PROJECTS\Handy_V2\` (MIT fork of Handy), reference only |
| User's llama.cpp scripts | `PROJECTS\Llama.cpp\` (`download-llama_v2.ps1`, tuned launchers), reference only |
| Reference documents | `PROJECTS\SLAB\DATASHEET_TRUTH_SOURCE_HARDWARE\` (7 PDFs + marker outputs); new Sift outputs go into a `SIFT_V1\` folder there, never into marker's folders |

**Machine:** Windows 11 Pro, i9-13900H, 32 GB RAM, RTX 4070 Laptop 8 GB (~6.9 GB free with the
desktop), NVIDIA driver supports CUDA 13.5; CUDA toolkit 13.4 (needed only to build the layout's
GPU version, 4.3); Rust 1.99; Bun 1.4.

---

## 1. The idea in one page

```
 PDF
  |
  text layer (pdfium + pdftext's algorithm) ............ exact characters, fonts, positions: the ground truth
  |
  layout: surya's rf-detr (inside Sift, candle; GPU 68 ms/page) .... blocks, labels, reading order
  |
  routing: the text layer is used as is, except
  |     tables, formulas, scanned or broken text, labels inside images  ->  surya 2.5 (llama.cpp, 6 slots)
  |
  checks: every number and identifier surya writes in a table is looked up in the PDF's text
  |       (found: kept; one close PDF value: replaced by it; otherwise kept and reported)
  |
  Gemma 4 E4B (llama.cpp, 2 slots, MTP drafter), after surya has stopped:
  |       image descriptions (with the figure's own text as context), heading levels,
  |       tables split across pages, missing formulas, review of tables the checks couldn't verify
  |
  <name>.md + images/ + <name>.sift-report.txt   (+ trace and server logs in .sift/logs/)
```

Measured (live, cold, LLM pass on): the 250-page LR20xx datasheet in 355 s (surya 184 s,
Gemma 159 s), the 23-page Wio datasheet in 86 s. Errors per 1,000 PDF words (scorecard v2):
LR20xx 46.8 (marker 56.1), Wio 121.5 (marker 136.4). surya's table values: 99.58 % exactly as in
the PDF, the rest corrected from it, none left unresolved.

---

## 2. Decisions

*user* = decided by the user; *measured* = decided by numbers; *proposed* = open to the user.

| # | Topic | Decision | Status |
|---|---|---|---|
| VD1 | Stack | Rust (edition 2024) for everything that converts; Tauri 3 + SolidJS 2 + Bun + TypeScript for the window; **Windows only** | user |
| VD2 | Local only | No online model API, no keys, no telemetry; network only on the user's action | user |
| VD3 | Binaries | One Rust core (`sift-core`), two front ends: `sift.exe` (command line) and `sift-app.exe` (window; runs the command line when given a subcommand). Models run only in `llama-server` child processes inside a kill-on-close Job Object | user |
| VD4 | **Models** | **surya 2.5** (OCR, Q8_0), **Gemma 4 E4B QAT** (Unsloth UD-Q4_K_XL + mmproj-F16 + MTP drafter), **surya's layout model** (rf-detr, `datalab-to/surya_layout2`). No other model, no model hub | user (2026-11-15) |
| VD5 | What surya reads | Only what the text layer can't give (hybrid). `--mode full-ocr`: every page. A scanned page: surya's whole-page read | user |
| VD6 | Trust | The text layer is the ground truth; table values and words checked both ways (D10, D27); the document decides (D28); every unresolved or model-changed value goes to the report | user |
| VD7 | **Folders** | The user chooses the engine folder and the models folder (window: folder pickers; command line: `sift setup --engine-dir --models-dir`, `sift engine folder`, `sift models folder`). Nothing is downloaded before a folder is chosen; nothing goes into `%APPDATA%`. The settings file lives in the app's config folder, or next to `sift.exe` in portable mode | user (2026-11-15/16) |
| VD8 | **Engine** | Official llama.cpp only, **CUDA 13.5 Windows build** by default. The Engine page and `sift engine ...`: check for new builds, install or update with progress, verify before switching, keep the previous build, use any installed build, roll back, delete. When official builds move to a newer CUDA version, the check says so and the user chooses | user |
| VD9 | Update detection | A check reads GitHub's release list (no download). It runs when the Engine page opens, on "Check for updates", and at start-up only if the user turned that on (asked in onboarding). A newer build puts a dot on the Engine menu and a line in the footer | user + proposed (when) |
| VD10 | Model files | Pinned repository revision + sha256 per file (`catalog/*.toml`); resumable downloads; Verify re-checks sha256. A newer upstream revision is shown, never installed silently: adopting it is a development change (golden test + scorecard) | proposed |
| VD11 | Speculative decoding | Gemma: MTP drafter on (1.5x faster with images, same answers). surya: none (n-gram drafting measured no faster) | measured |
| VD12 | Scheduling | One model at a time on the GPU: surya (6 slots), then Gemma (2 slots); each server starts with its first job and stops when idle (1 s grace) | measured |
| VD13 | Acceptance | Golden test: 7 PDFs, stored answers and layout, Markdown and reports byte-identical. Live runs: scorecard and time at least as good as the last measured ones (12) | user |
| VD14 | Dependencies | Each crate and package has a stated reason (`docs/DEPENDENCIES.md`); ZER0's set first; newest versions, prereleases included (Tauri 3, Solid 2) | user |
| VD15 | Command line | Every option of the window is a flag; flags never change saved settings; `--json` prints the progress events (`docs/events.md`) | user |
| VD16 | ZER0 as the skeleton | App shell, settings, Job Object, server supervision, engine manager and its page follow ZER0 (4.4); copied code keeps Handy's MIT notice | user |

### 2.1 Why one OCR model

Other OCR models were tried as checkers of surya's reading (2026-11-07, every page read as a scan,
the most favourable case for them): 1-6 % fewer errors on most documents, one document worse, for
3-5x the time; none was better than surya alone, on clean pages or poor scans. On digital
datasheets 99.58 % of surya's table values already match the PDF and the rest are corrected from
it. Decision (user, 2026-11-15): surya + Gemma only. Not measured: real scans, other languages,
handwriting; reopen only with evidence of that kind.

---

## 3. Rules from the first version (still in force; the code cites their numbers)

| # | Rule |
|---|---|
| D1 | Hybrid by default: text, headings and lists from the text layer; tables, formulas, scanned or broken areas to surya; `--mode full-ocr` for useless text layers |
| D2, D23 | A model server starts with its first job and stops after its last; a model with no work is never started |
| D3, D4, D11 | Input: a PDF or a folder (optionally with subfolders). Output: Markdown + images + descriptions (+ a report when needed). Folder mode mirrors subfolders: `OUT\<subfolders>\<name>\<name>.md` |
| D9 | Pipe tables; an HTML `<table>` only for merged cells (`--tables markdown` forces pipes); must display in VS Code, Obsidian and GitHub |
| D10, D27 | Table values and words checked against the PDF, both ways; corrected to the exact PDF value; unresolved -> report |
| D12 | Running headers and footers removed (`--keep-headers` keeps them) |
| D13, D26 | Gemma describes every figure; surya first reads the labels of raster figures and Gemma gets them as "text inside the figure" |
| D18 | Gemma's settings are Sift's own measured ones (`[gemma]`); thinking on for heading levels and table merges |
| D21 | Known-working first: surya's own prompts, flags and parsers; pdftext's algorithm; rf-detr with surya's weights |
| D24 | Accuracy first; marker is the reference to beat |
| D25 | LLM pass on by default (`--no-llm`): table review (kept only if every value is in the PDF and nothing verified is lost), heading levels (3 votes), split tables, missing formulas |
| D28 | The document decides: hyphens, wrapped names and the like come from the document's own vocabulary |

---

## 4. Architecture

### 4.1 Processes

```
sift-app.exe (Tauri 3 window)            sift.exe (command line)
        \                                    /
         +------- sift-core (Rust) ---------+      same code, same settings file
                   |  pdfium.dll (in process): renders, characters
                   |  rf-detr layout (in process, candle, GPU or CPU)
                   |  Job Object: every child process dies with Sift
                   +-- llama-server  surya 2.5     (engine from the engine folder, model from the models folder)
                   +-- llama-server  Gemma 4 E4B   (starts after surya has stopped)
```

The window's Rust backend (`app/src-tauri/src/main.rs`) exposes commands for conversion (start,
cancel), settings and folders (pick, set with move or use-as-is, open), the engine (status,
check, install, use / roll back / delete / remove runtime) and the models (status, download,
verify, delete, revision check). Long work runs on its own thread and reports through events
(`sift-event`, `engine-log`, `engine-progress`, `engine-done`, `engine-check`, `model-progress`,
`model-done`, `sift-done`); commands never block the window.

### 4.2 Repository

```
Sift_PDF_to_MD/
  README.md  AGENTS.md  MASTER_PLAN.md  CURRENT_ARCHITECTURE.md  LICENSE  NOTICE
  .gitignore  .gitattributes
  Cargo.toml  Cargo.lock                   workspace
  crates/sift-core/                        the converter (module map: AGENTS.md)
  crates/sift-cli/                         sift.exe
  app/                                     Tauri 3 + SolidJS 2 + Bun (src/, src-tauri/)
  catalog/                                 surya-2.5.toml, surya-layout.toml, gemma-4-e4b-qat.toml
  prompts/surya-2.5/  prompts/describe/    surya's prompts word for word; description prompts
  tests/golden/                            the golden set
  bench/                                   golden/ (run.sh, compare.py), score.py + compare_marker.py
  scripts/                                 update-deps.ts, build-cuda.cmd
  docs/                                    AUDIT, LEARNINGS, GOLDEN, settings, events, DEPENDENCIES, blocks.schema.json
  engine/  models/                         this machine's folders (git-ignored)
  runtime/pdfium/                          pdfium.dll for development (git-ignored); shipped next to the executables
```

### 4.3 Layout on the GPU

The candle CUDA build of the layout model needs the **CUDA toolkit 13.4** at build time (cudarc
0.19 doesn't know 13.5; `scripts/build-cuda.cmd` puts 13.4 first on PATH), MSVC with
`/Zc:preprocessor`, and no `/MD` C dependency. This is separate from the llama.cpp engine, which
ships its own CUDA 13.5 build. The CPU build works everywhere (280 ms/page instead of 68).

### 4.4 What comes from ZER0

| ZER0 (`PROJECTS\Handy_V2`) | In Sift |
|---|---|
| `src-tauri/src/llama_releases.rs`: release list, asset per backend, CUDA toolkit detection, streamed download with progress, flat unzip, installed list with sizes, bundled CUDA runtime removal | `crates/sift-core/src/engine.rs` (plus verify-before-switch, `current.txt` / `previous.txt`, release-feed fallback) |
| `src/components/settings/llama/LlamaSettings.tsx` + `src/stores/llamaStore.ts` | the Engine page in `app/src/App.tsx` |
| `src-tauri/src/llama_server.rs`: supervisor, `/health`, state events, 400-line log ring | `llm/mod.rs` (state events done; the live log ring: 11) |
| `src-tauri/src/job_object.rs` | `llm/jobobject.rs` |
| `src-tauri/src/portable.rs` | `paths::portable_dir` (settings next to the exe) |
| `src/components/update-checker/UpdateChecker.tsx` | the Engine menu dot and footer note |
| `file_transcription.rs` + Transcribe Files page | the Convert page |
| `system_monitor.rs`, `footer/SystemMeters.tsx` | not yet (11) |
| `scripts/update-deps.ts`, pre-commit gate | `scripts/update-deps.ts` (whole lockfile, held conflicts retried, golden test); the gate: 11 |

### 4.5 Development routine

```bash
bun scripts/update-deps.ts      # newest dependencies (prereleases incl.), then cargo update -> check -> unit
                                # tests -> bun install -> tsc -> vite build -> release build -> golden test
cargo test -p sift-core --lib   # unit tests
cargo build --release -p sift-cli && bash bench/golden/run.sh
```

After any change that could move quality or speed: cold runs of LR20xx and Wio, scored with
`bench/score.py` against the text layer and marker's outputs; read the trace and the server logs
before continuing.

---

## 5. Folders and settings

| Folder | Holds | Set by |
|---|---|---|
| **Engine folder** | one subfolder per installed build (`b12368-cuda-13.5\`), `current.txt`, `previous.txt` | Start / Settings / Engine page, `sift engine folder <path>`, `sift setup --engine-dir` |
| **Models folder** | `surya-ocr-2.5-gguf\`, `surya_layout2\`, `gemma-4-E4B-it-qat-GGUF\`, `.part` files while downloading | Start / Settings / Models page, `sift models folder <path>`, `sift setup --models-dir` |
| Output | `<name>\<name>.md`, `images\`, the report, `.sift\logs\` | each conversion (`-o`, the Convert page); default next to the input |
| Settings | `settings.toml` (`docs/settings.md`) | the app's config folder `%APPDATA%\com.nairodorian.sift\`, or `Data\` next to `sift.exe` when a `portable` file is there |

Rules:
- Nothing is created or downloaded before the user chose the folder; the command line stops with
  a clear message (`No models folder set. Choose one in the app (Settings), or run: sift models
  folder <path>`).
- A chosen folder must be writable. **Changing a folder** in the window asks "Move the files there"
  (a rename; across drives the user moves them) or "Use it as it is" (when it already holds them).
  The old folder is never deleted by Sift.
- Development overrides for one process: `SIFT_ENGINE_DIR`, `SIFT_MODELS_DIR`, `SIFT_SETTINGS`.
- The answer store exists only when `--answers <dir>` is given.

---

## 6. The engine: official llama.cpp, CUDA 13.5

1. **Check** (no download): the `ggml-org/llama.cpp` release list (GitHub API; the releases feed
   and release page when the anonymous limit is hit). For the chosen backend, the newest build with
   a Windows x64 asset is compared with the active one: up to date / update available (build, date,
   size) / "official builds now also use cuda-X.Y: switch the backend to use it".
2. **Install or update**: download with progress into the engine folder, unzip (entries escaping
   the folder are refused); the CUDA runtime comes from an installed CUDA 13 toolkit when one is
   found, otherwise from the official `cudart` package of the same CUDA version.
3. **Verify** before switching: `llama-server --version` reports the expected build and
   `--list-devices` shows the GPU.
4. **Switch**: `current.txt` names the new build; the old one becomes `previous.txt`. A failure at
   any step leaves the active build untouched and says why. Older builds stay until the user
   deletes them.
5. **Never during a conversion**: installing or switching waits for the running job.
6. **Manage builds**: Use (verify, then activate), Roll back, Delete (never the active or previous
   build), Remove bundled CUDA runtime (when a toolkit provides it).

The Engine page: active build and version; newest build with "Check for updates" and "Update";
a download progress bar; backend selector (cuda-13.5, a newer CUDA when offered, vulkan, cpu, auto)
with the detected backend and the driver's CUDA version; CUDA runtime source and "always download
the runtime package"; "check at start-up"; the engine folder (Change, Open); installed builds
(size, bundled runtime, active / previous, Use, Roll back, Delete, Remove runtime); running
servers; the log of the last action. A dot on the Engine menu item and a footer note when an
update was found.

Command line: `sift engine status | check | update [--tag] [--backend] [--with-cudart] | install |
list | use <build> | rollback | remove <build> | remove-runtime <build> | folder [<path>]`;
`check` exits with 10 when an update is available.

---

## 7. The models

| Entry | Repository @ revision | Files | Size |
|---|---|---|---|
| `surya-2.5` | `datalab-to/surya-ocr-2.5-gguf` @ `eb78671a` | `surya-2.5-Q8_0.gguf`, `surya-2.5-mmproj.gguf` (optional BF16 `surya-2.5.gguf`, +1.18 GB) | 0.84 GB |
| `surya-layout` | `datalab-to/surya_layout2` @ `0aee81d5` | `config.json`, `rfdetr_layout.pth`, `order/config.json`, `order/order_ar.pt`, `LICENSE` | 0.14 GB |
| `gemma-4-e4b-qat` | `unsloth/gemma-4-E4B-it-qat-GGUF` @ `8c5a9e4f` | `gemma-4-E4B-it-qat-UD-Q4_K_XL.gguf`, `mmproj-F16.gguf`, `mtp-gemma-4-E4B-it.gguf` | 5.27 GB |

Downloads: Hugging Face `resolve/<revision>/<file>`, resumable (`.part` + range requests), sha256
checked, then renamed; no Hugging Face library. The Models page: one card per model (size,
revision, license, installed or missing, Download, Verify, Delete), the models folder, "Check for
new revisions" (shown only). Command line: `sift models status | check | download [all|id]
[--quant BF16] | verify [id] | delete <id> | folder [<path>]`. `sift setup` sets both folders,
installs the newest engine build and downloads and verifies the three models.

---

## 8. Gemma's jobs and what they are worth

| Gemma's job | Measured | Keep? |
|---|---|---|
| figure descriptions | 88.9 % (LR20xx) and 68.5 % (Wio) of the numbers confirmed by the figure's text | yes; grounding is backlog B2 |
| heading levels (thinking on, 3 votes) | 13/23 -> 23/23 with thinking | yes |
| tables split across pages (thinking on) | | yes |
| missing formulas | not measured separately | yes |
| table review (guarded) | 0 of 5 corrections accepted on digital datasheets | yes for now; measure on scans (B3) |

---

## 9. The window

ZER0's design language (sharp corners, hairlines, tracked group titles, mono numbers) in Sift's
palette (warm ink and paper, one amber accent), dark, light or the system's. One place per setting:
a setting has one control, on the page it belongs to; other pages link to it.

| Page | Content |
|---|---|
| Setup | first run, in four steps: engine folder, models folder, install llama.cpp (+ the start-up check), download the models; in the menu until everything is ready |
| Convert | laid out as the job: the PDF card (drop or choose a PDF or a folder), the sieve (the stages with their counts, Convert / Cancel, elapsed time), the Markdown card (output folder; then the result: open, show in folder, report, numbers, first lines; one row per file for a folder). Beside it: the saved options (descriptions, LLM pass, scanned PDF, page headers, tables, image DPI) and this run's (pages, subfolders, convert again) |
| Engine | section 6: status strip, Check now / Install / Update with progress, the start-up check, backend, CUDA runtime, engine folder, model servers, installed builds |
| Models | section 7: models folder, one card per model with progress, verify, delete (asks once), revision check |
| Settings | theme, motion, how the two models share the GPU, where the folders and the settings file are |
| Logs | the session's events, readable; filters (progress, servers, problems), copy, clear |
| About | the mark, version, GPU, CUDA, CPU, engine, the command line, credits and licenses |

Footer: the two model servers (dots), the current stage and progress, the active llama.cpp build
(+ "update available"), the version. The sieve's motion follows Windows' "Animation effects"
(reduced, not removed, when they are off) unless Settings forces it on or off.

---

## 10. Command line

```
sift convert <pdf|folder> [-o <out>] [flags]     (also sift <pdf|folder> [flags])
sift setup [--engine-dir <path>] [--models-dir <path>] [--with-cudart]
sift engine status | check | update | install | list | use | rollback | remove | remove-runtime | folder
sift models status | check | download | verify | delete | folder
sift config show | path | set <section.key> <value>
sift dev textlayer <pdf> | crop <pdf> <page> x0 y0 x1 y1 <out>
```

Convert flags: `-o/--output`, `--mode hybrid|full-ocr`, `-r/--recursive`, `--force`, `--pages`,
`--image-dpi`, `--tables auto|markdown`, `--keep-headers`, `--describer gemma|off`,
`--no-describe`, `--no-surya`, `--no-llm`, `--json`, `--profile`, `--config` (an extra settings
file on top of the saved one). Development: `--answers`, `--replay`, `--replay-only`,
`--layout-from` (the golden test), `--mask-text-layer` (read every page as scanned, score against
the text layer).

---

## 11. What's next

0. **Audit and code review** (`docs/AUDIT.md`): rounds 1 to 3 done on 2026-11-25 (clippy, untrusted
   input, Cancel, the first run, dead code), each with the golden test; the fixes are in the
   changelog's Unreleased section. Next round when the code changes enough to need one.
1. **Live check** on this machine: cold runs of LR20xx and Wio with the new folders (engine and
   models in `engine\`, `models\` at the repository root). Exit: LR20xx at most 355 s, Wio at most 86 s; errors per
   1,000 PDF words at most 46.8 and 121.5; report reviewed; logs read. Engine updated to
   b12728-cuda-13.5 on 2026-11-25 with the user's OK (b12600 kept as previous): Wio's Markdown is
   identical to b12600's outside the image descriptions, scorecard 122.26 on both builds
   (docs/LEARNINGS.md: the 0.7 over the bar is not the code, which replays the reference to
   121.52, nor the layout device (the GPU layout build scores 122.26 too), but the live answers).
   Timed on an idle machine the same night: LR20xx 367.0 s / 359.8 s and Wio 87.1 s / 89.3 s
   (layout on the CPU / on the GPU): within run-to-run noise of the bars on the GPU build, about
   3 % slower on LR20xx with the CPU layout (surya waits for pages); LR20xx scores 46.58 / 46.59
   live (bar 46.8). Done.
   The window was redesigned and walked page by page on 2026-11-25 (dev preview and the real
   app); a live conversion from the window is part of this check.
2. **Window, ZER0 parity**: done 2026-11-25: each llama-server's log, followed live, and its
   command line (read-only, Copy) on the Engine page; the pre-commit gate (`scripts/precommit.ts`,
   `.githooks/`: format, clippy with warnings as errors, unit tests, typecheck, production build,
   workflow files; `--full` adds the release build and the golden test); the footer's meters
   (CPU, RAM, GPU, VRAM) without ZER0's two dependencies: Windows' own calls and the driver's
   `nvml.dll` through libloading, one thread that samples once a second only while the window
   shows. Typed commands and events (2026-11-26): tauri-specta publishes for Tauri 2 only, so
   Sift uses the maintainer's Tauri 3 fork (as ZER0), brought up to upstream `main` the same
   day; `app/src/lib/bindings.ts` is generated from `main.rs` and `ipc.rs`, the browser
   preview's mock is typed against the same commands, and the gate checks the file.
3. **Window check** (done 2026-11-25): the first-run flow on fresh settings files
   (`SIFT_SETTINGS`): Setup 0/4, both folders, llama.cpp installed into an empty folder (226 MB),
   the three models downloaded into an empty folder (6 GB in 147 s with their checks), then Wio
   converted in that profile (scorecard 122.26, as every live run); `sift setup` on another fresh
   settings file found both folders filled, checked the ten model files and was ready in 17 s.
4. **Installer** (done 2026-11-25): NSIS, per user: `sift-app.exe` + `sift.exe` + `pdfium.dll`
   (+ its licenses) + `prompts\` + `catalog\`; first run = the Setup page. v1.0.0-alpha.2 was
   published from the Release workflow the same day; the next release takes the changelog's
   Unreleased section.
5. Then the backlog (13).

---

## 12. Quality bar

1. **Golden:** byte-identical on the 7 PDFs (with and without the LLM pass) after every change in
   the conversion path.
2. **Live:** scorecard v2 not worse than now, in errors per 1,000 PDF words:

   | | Sift | marker |
   |---|---|---|
   | LR20xx | 46.76 | 56.13 |
   | Wio | 121.52 | 136.38 |

   These are the reference runs' answers (the golden test replays them). Live runs on today's
   engines answer slightly differently: Wio scores 122.26 live (b12600 and b12728, layout on
   the CPU or the GPU; docs/LEARNINGS.md), which is the bar for a live run.
   Also: surya's table values verified at 99.5 % or more, and no new unresolved table values.
3. **Speed:** not slower than 355 s (LR20xx) and 86 s (Wio), cold, on this machine, with the GPU
   layout build; Gemma alone varies about 7 s from run to run, so judge with two runs. The
   installer's build (layout on the CPU) measured 367 s and 87 s (2026-11-25).
4. **Engine:** every build verified before it becomes active; rollback always possible; nothing
   downloaded without the user's action.
5. **Viewer check** (VS Code, Obsidian, GitHub) during development.

---

## 13. Backlog (surya + Gemma quality)

| # | Item |
|---|---|
| B1 | Measure structure, not only words: table cells and heading levels in the scorecard |
| B2 | Ground the descriptions: a number in a description must be in the figure's text; otherwise describe the trend |
| B3 | Table review: measure it on scanned pages; skip it where the PDF already verified the table if it never changes anything there |
| B4 | Text inside vector figures: offer it from the text layer as a "text in this figure" block |
| B5 | Reading order on multi-column pages (BMI423: 71 % kept) |
| B6 | A real scanned test set (printed and scanned datasheets) for surya alone |
| B7 | Port the scorecard to Rust (`sift dev score`) once it gives the same numbers as the Python one; then no Python at all |
| B8 | Heading pass speed (about 57 s on LR20xx) |
| B9 | Known output issues: schematic labels as loose text; LR20xx p249 table structure; surya table errors on LR20xx p96 and p228 (reported) |

---

## 14. Risks

| Risk | Mitigation |
|---|---|
| Tauri 3 alpha / Solid 2 release candidate APIs move | ZER0 runs the same stack; `scripts/update-deps.ts` moves them together and validates build, tests and golden |
| A new llama.cpp build breaks surya or Gemma | verify before switching; previous build kept; rollback; after an update, a live Wio run compared with the last one |
| llama.cpp moves past CUDA 13.5 | the check reports it; the user switches the backend; the last 13.5 build stays usable |
| GitHub's anonymous limit (60 requests an hour) | release-feed fallback; optional `GITHUB_TOKEN` |
| A folder on a slow or removable disk | writable check; model load times in the logs |
| The candle GPU build needs the 13.4 toolkit | documented; the CPU build is the fallback |
| The golden set lives only in this repository | it is small (9 MB) and committed with the code; the reference PDFs are not (sha256 in `docs/GOLDEN.md`) |

---

## 15. Questions and answers

**Answered by the user:**
- **2026-11-19:** flatten `V1/` into the repository root; open source on GitHub
  (`NairoDorian/Sift_PDF_to_MD`); update every dependency one last time, then a full audit and
  code review; carry ZER0's lessons (rtk, `bun run tauri dev`, the routine) into `AGENTS.md`; the
  project's origin stays a short note in `README.md` (no separate history file).
- **2026-11-16:** the engine and models folders on this machine are `V1\engine` and `V1\models`
  (since 2026-11-19: `engine\` and `models\` at the repository root)
  (git-ignored); the settings file lives in the app's config folder; overwrite and remove
  everything of the old directions; all dependencies at their newest prereleases.
- **2026-11-15:** two models only (surya 2.5 + Gemma 4 E4B); engine and models folders chosen by
  the user, never `%APPDATA%`; one `V1` folder, the first version removed; the Engine page detects
  and installs llama.cpp updates like ZER0; CUDA 13.5.
- **2026-11-07:** no Review or History page; Windows only; Gemma is Unsloth's Gemma 4 E4B QAT.
- **2026-11-06:** the engine is installed and updated on request; ZER0 is the skeleton.
- **2026-11-05:** local only; official llama.cpp builds only; licenses accepted by use; a command
  line ships.

**Open:** none.

---

## 16. Stack and versions

The exact versions are in `Cargo.toml`, `app/src-tauri/Cargo.toml` and `app/package.json`, kept at
their newest (prereleases included) by `scripts/update-deps.ts`: Rust 1.99 (edition 2024), Tauri 3
(alpha), `tauri-plugin-dialog` (plugins-workspace `v3`), SolidJS 2 (stable since 2.0; the
newer of npm `next` and `latest`), Vite, TypeScript (dev builds), Bun 1.4, candle (layout), pdfium-binaries
chromium/7891, llama.cpp official CUDA 13.5 builds.
