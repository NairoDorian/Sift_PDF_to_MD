//! Sift's core (MASTER_PLAN section 4): everything that turns a PDF into Markdown, without the
//! window. The app binary and the `sift` command line both call into it.
//!
//! Module map (AGENTS.md has the full table):
//! - `pdf`        pdfium (renders, characters), the text layer (pdftext's algorithm + V0's rules), quality checks
//! - `layout`     surya's rf-detr layout model and reading order, on candle (GPU or CPU)
//! - `build`      blocks, routing, table checks against the PDF, headings, clean-up
//! - `surya`      surya 2.5 requests and its HTML format
//! - `brain`      Gemma 4 E4B: descriptions and the LLM pass; `describe`: figure prompts
//! - `llm`        llama-server supervisor, model services, the Job Object
//! - `engine`     official llama.cpp builds in the user's engine folder
//! - `models`     the three pinned models in the user's models folder
//! - `pipeline`, `batch`  one PDF, a folder of PDFs; `render`: Markdown
//! - `answers`    the answer store (golden test, re-runs); `config`, `paths`, `events`
//! - `py`, `imaging`, `util`  Python-compatible text semantics, images, helpers

pub mod answers;
pub mod batch;
pub mod brain;
pub mod build;
pub mod config;
pub mod describe;
pub mod engine;
pub mod events;
pub mod imaging;
pub mod layout;
pub mod llm;
pub mod models;
pub mod paths;
pub mod pdf;
pub mod pipeline;
pub mod py;
pub mod render;
pub mod surya;
pub mod util;
