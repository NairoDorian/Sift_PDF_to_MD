# Dependencies (strict necessities, each with its reason)

Versions are always the newest published, prereleases included: `bun scripts/update-deps.ts`
(or `bun run update` in `app/`), validated by the build, the tests and the golden test.

## Rust

| Crate | Why it is needed | Alternative considered |
|---|---|---|
| anyhow | error context across the pipeline | (ZER0's set) |
| serde, serde_json | settings, events, the answer store, the golden files | (ZER0's set) |
| toml | the settings file, the model catalog | |
| sha2 | answer-store keys; sha256 checks of model downloads | (ZER0's set) |
| libloading | pdfium's C API at run time | pdfium-render: a far larger API than the 30 functions Sift calls |
| png | PNG encode and decode for crops and figures | image: decoders Sift never uses |
| regex, fancy-regex | the text-layer rules are regexes with lookarounds; fancy-regex adds them on top of regex | hand-written matchers: error-prone for ~100 patterns |
| unicode-general-category | Python-exact character classes (`isalnum`, `isprintable`) for the ported rules | char methods: differ on several classes |
| unicode-normalization | NFKC for the table checks | |
| candle-core, candle-nn | surya's rf-detr layout model and its reading-order head in Rust, on the GPU or the CPU | ONNX Runtime: excluded (no second inference runtime); hand-written matrix code: slower |
| reqwest (blocking, rustls) | llama-server requests, engine and model downloads | ureq: not in ZER0's set |
| zip | extracting official llama.cpp release zips | shelling out to tar or PowerShell |
| base64 | images in chat requests | |
| clap | the command line | (ZER0's set) |
| log | library logging | (ZER0's set) |
| windows-sys | the Job Object that kills model servers with Sift | windows: larger; Sift needs 4 functions |
| tauri, tauri-runtime-wry, tauri-build | the window (Tauri 3) | (ZER0's set) |
| tauri-plugin-dialog | folder pickers for the engine and models folders (called from Rust; Tauri's own plugins-workspace, `v3` branch, as ZER0) | typing paths only |

## The window (npm, `app/package.json`)

| Package | Why |
|---|---|
| solid-js, @solidjs/web (2.x, the `next` line) | the UI (ZER0's stack) |
| @tauri-apps/api (3.x) | commands and events between the page and Rust |
| @tauri-apps/cli, vite, @solidjs/vite-plugin, typescript | build tools |

## Not added, on purpose

- **A database** (SQLite): nothing in Sift needs one; its bundled C library is built against the
  dynamic C runtime, which can't link with candle's CUDA kernels (static runtime, LNK2038).
- **sysinfo / nvml-wrapper** (ZER0's live meters): the About page reads `nvidia-smi`, which every
  NVIDIA driver installs; live meters can come later if they earn a dependency.
- **mimalloc**: measured no faster, and its dynamic C runtime breaks the CUDA layout build.
- **A Hugging Face library**: model downloads are plain HTTP range requests + sha256.
