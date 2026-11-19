# How Sift came to be

## It started with one small job

The job was as simple as it gets: **turn a PDF into Markdown so an AI can read it.**

Mostly datasheets: the register maps, pin tables, timing diagrams and formulas behind
electronics projects. An AI assistant reads Markdown well and PDFs badly. Hand it a 250-page
datasheet as a PDF and it skims, guesses at tables and never sees the figures. Hand it clean
Markdown and it can work with the document. So the task was a converter. Nothing more.

## marker, and why it stopped being enough

For a while the converter was [marker](https://github.com/datalab-to/marker) (marker-pdf):
`marker_single` with its LLM option, plus a small `convert.py` wrapper that re-cut every figure at
400 DPI as PNG, because marker's 192 DPI crops were too soft for schematics.

It worked, and marker is still the reference Sift measures itself against. But it kept getting
heavier: Docker, a large Python and PyTorch stack, several models to manage, and, for the best
output, an online LLM with an API key. On the reference datasheets that online step also made
things up. It invented table cells (LR20xx datasheet, pages 43 and 64) and figure titles and values
that are not in the PDF. That is the wrong way round for a tool whose whole point is to give an AI
the truth of a document.

## Starting over: Sift

So the work started again from scratch, under a new name and with a short list of goals:

- **Simple to use.** One window, or one command. Pick a PDF or a folder and get Markdown, with
  the images and a description of each one.
- **Simple to install.** One program. No Docker, no Python environment, no account, no API key.
  The models run on official [llama.cpp](https://github.com/ggml-org/llama.cpp) builds that Sift
  installs and updates itself, into folders you choose.
- **Accurate.** The PDF's own text layer is the ground truth. Models read only what the text
  layer can't give (tables, formulas, scans, labels inside images), and every number they write
  in a table is checked against the PDF. Anything Sift can't verify goes into a report rather
  than being quietly guessed.
- **Fast.** Local models on your own GPU. Today a 250-page datasheet takes about 6 minutes on a
  laptop RTX 4070.
- **Any OS.** That's the aim. The core is plain Rust and the window is Tauri, both
  cross-platform, but this first version is built and tested on Windows only (the engine manager
  downloads Windows CUDA builds of llama.cpp). Linux and macOS come next.
- **Latest everything.** Rust 2024, Tauri 3, SolidJS 2, Bun, the newest official llama.cpp, and
  one script (`scripts/update-deps.ts`) that keeps every dependency at its newest version,
  prereleases included.
- **Open source.** Apache-2.0, with every ported piece credited in `NOTICE`.

## Timeline

Code comments still say **V0** for the first, Python version and **V1** for this Rust version.

| Date (2026) | What happened |
|---|---|
| Nov 5 | Ground rules set: local only, official llama.cpp builds only, model licenses accepted by use, a command line ships next to the window. |
| Nov 5-6 | **V0**, the first version, in Python: pdfium plus pdftext's algorithm for the text layer, surya for layout, tables and formulas, Gemma for figure descriptions. Its reference run on seven datasheets (515 pages) was scored against the marker outputs. |
| Nov 6 | ZER0 becomes the skeleton. ZER0 is the maintainer's own Tauri app, a fork of [Handy](https://github.com/cjpais/Handy), and it lends Sift its kill-on-close Job Object, its llama.cpp release manager and its app structure. |
| Nov 6-7 | **The Rust port (V1)**, checked against V0 as an oracle. Every stage and every model answer of V0 was recorded and replayed: 0 differences in 941,838 text-layer characters, 515/515 page renders pixel-identical, 7/7 Markdown files and reports byte-identical. That recorded run is today's golden test. |
| Nov 7 | Other OCR models tried as second readers next to surya: 1-6 % fewer errors on most documents, one document worse, for 3-5 times the time. None beat surya alone. Windows only for the first version. |
| Nov 15 | Decision: **two models only**, surya 2.5 and Gemma 4 E4B. The engine and the models go only into folders the user chooses, never into a hidden app-data folder. llama.cpp's CUDA 13.5 builds. |
| Nov 16 | V0 and the multi-model experiments removed. The window gets its pages (Start, Convert, Engine, Models, Settings, Logs, About). Golden test 14/14 byte-identical. |
| Nov 19 | V1 becomes the repository root. Dependencies updated one last time, then the **first public commit**, ahead of a full audit and code review. |

## Where it stands

Measured on the seven reference datasheets, live and cold, on an RTX 4070 Laptop (8 GB):

| | Sift | marker |
|---|---|---|
| Errors per 1,000 PDF words, LR20xx (250 pages) | **46.8** | 56.1 |
| Errors per 1,000 PDF words, Wio-LR2021 (23 pages) | **121.5** | 136.4 |
| Time, LR20xx / Wio, with descriptions and the LLM pass | 355 s / 86 s | |

surya's table values match the PDF exactly 99.58 % of the time. The rest are corrected from the
PDF, and none are left unresolved.

What's next: the audit and code review ([docs/AUDIT.md](docs/AUDIT.md)), then the plan's
section 11 ([MASTER_PLAN.md](MASTER_PLAN.md)): a live check with the newest engine, ZER0 parity
for the window, a first-run test on a fresh profile and an installer.
