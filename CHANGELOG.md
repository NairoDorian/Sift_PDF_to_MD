# Changelog

Sift's releases. The release workflow (`.github/workflows/release.yml`) takes a release's notes
from its section here.

## Unreleased

Fixes from the audit of alpha.2 (docs/AUDIT.md, rounds 1 to 3), and the Engine page's server
panels.

**Conversion**
- Cancel no longer writes anything for the PDF in progress. Before, it carried on with every
  model request failing, wrote a degraded Markdown and report over the earlier good ones, and
  marked them up to date, so a later folder run skipped the PDF. Cancel now also stops the
  layout at its next batch instead of the last page.
- A model answer can no longer crash or hang a run: surya's HTML is cut off past 256 nested
  tags (a looping answer overflowed the stack), and merged-cell spans are capped at the HTML
  standard's limits (a made-up `colspan="99999999"` hung the run).
- Folder mode keeps the options each Markdown was made with (`.sift-options`): a quick test run
  (`--pages 1-2 --no-llm`) no longer counts as the real conversion on the next run.
- An unreadable PDF says why in words; a missing pdfium says where it belongs and where Sift
  looked (it pointed to a command that does not exist).

**The window**
- Engine page: for each model server, the command Sift starts it with (read-only, with Copy)
  and the end of its log, followed live while it runs.
- The footer shows the machine's load: CPU, RAM, and with an NVIDIA driver the GPU and its
  memory (where surya and Gemma load), once a second while the window shows.
- After Cancel the stages stay where the work was (they used to tick the ones whose requests
  failed at once), and the result card says what was kept.
- The window opens only what Sift writes (folders, `.md`, `.txt`).
- The mark and the app icon are the strainer alone: the two amber lines under it are gone.

**Engine and models**
- Names from the llama.cpp release list and a build name given to Use must be plain names
  inside the engine folder, never a path.
- A download already complete (stopped while being checked) is checked instead of downloaded
  again.
- `sift config show` lists the `SIFT_` environment variables that override the file shown.
- `scripts\build-cuda.cmd` (the layout on the GPU) finds Visual Studio again.

**Development**
- `bun scripts/precommit.ts` runs the routine's checks in one command (`--full` adds the release
  build and the golden test); git runs it on commit once `bun run hooks:install` turned it on.
- Unused code removed; clippy runs with warnings as errors.

The conversion is unchanged where nothing was cancelled: the golden test is 14/14 identical.

## 1.0.0-alpha.2 (2026-11-25)

The first release with an installer, and a new window.

**The window**
- Redesigned on ZER0's design language (sharp corners, hairlines, mono numbers) in Sift's own
  palette, with a dark and a light theme (or the system's).
- The Convert page reads like the job: the PDF on top, the sieve under it with the pipeline's
  stages and Convert, the Markdown at the bottom. While Sift works the sieve shakes and Markdown
  pours out of its net into the document; at the end the card shows the file, its numbers and its
  first lines, with Open, Show in folder and Report. Drop a PDF or a folder anywhere on the
  window, or choose one (a PDF picker is new).
- Every setting lives in one place. Before, the engine and models folders were on three pages
  each, the start-up check on three, and image descriptions, the LLM pass and page headers on
  two (where the Convert page's switches ignored the saved defaults, and an unchecked "Keep page
  headers" could not override a saved one). Now the conversion options are on Convert and saved,
  the engine's settings on Engine, the models' folder on Models; Settings keeps the theme, motion
  and how the two models share the GPU.
- Engine page: the release list is read only when you press Check (or at start-up, when turned
  on); opening the page no longer contacts GitHub.
- Deleting a model or an engine build asks once, inline.
- Progress for image descriptions and the LLM pass (the `describe` and `refine` stages of
  docs/events.md were documented but never sent).
- Motion follows Windows' "Animation effects": with them off the sieve moves gently; Settings can
  force it on or off.
- A new mark and app icon: a strainer over two lines of text.

**Release**
- A Windows installer (NSIS, per user, no administrator rights) with the window, `sift.exe`,
  pdfium (chromium/7891) and its licenses, the prompts and the model catalog.
- The window binary's command-line mode prints to the console it was started from.

**Dependencies** (all at their newest, `bun scripts/update-deps.ts`)
- Solid 2.0.4 (stable; was 2.0.0-rc.23), Vite 8.4.0-beta.2, TypeScript 7.1 dev build of
  2026-11-25, anyhow 1.0.105, clap 4.6.10, zip 9.8.2, toml 1.2.3, tauri-plugin-dialog at its `v3`
  branch tip, and the whole Cargo.lock and bun.lock. Validated on Rust 1.102 nightly and Bun
  1.4.6; the crates now inherit the workspace's `rust-version` (1.99), so an older Rust stops
  with a clear message.
- The updater follows Solid's `latest` tag as well as `next` (it had stopped at the release
  candidate), refreshes the npm lockfile's sub-dependencies (`bun update`), and reports the
  toolchain (Rust, Bun) and GitHub Actions with a newer release.

The conversion itself is unchanged: the golden test is 14/14 byte-identical.

## 1.0.0-alpha.1 (2026-11-19)

The first public commit: the Rust converter, the command line and the first window. No binaries.
