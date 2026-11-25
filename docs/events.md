# Progress events (JSONL)

`sift convert <input> --json` prints one JSON object per line on **stdout**; the window gets the
same events (`sift-event`). Everything else goes to stderr.

Every event has:

| Field | Type | Meaning |
|---|---|---|
| `type` | string | One of the types below |
| `t` | number | Seconds since the command started |
| `file` | string | Absolute PDF path (file-level events only) |

## Types

| `type` | Extra fields | When |
|---|---|---|
| `job_start` | `files` (int), `output` (folder), `mode` (`hybrid` \| `full-ocr`), `describer` (`gemma` \| `off`), `engine` (llama.cpp build), `skipped` (int: up-to-date files in folder mode: newer than their PDF and converted with the same options, recorded in `.sift-options` next to each Markdown) | The job starts |
| `file_start` | `index` (0-based), `total`, `pages` | A PDF is opened |
| `stage` | `name`, `done`, `total` | Progress inside a file. Names: `text` (text layer), `layout`, `build` (pages), `surya` (tables, formulas, broken text, scanned pages, figure labels), `refine` (the LLM pass: table reviews and formulas), `describe` (figures). In a file, `refine` comes before `describe`; heading levels and table merges (also the LLM pass) run after both, without a count |
| `server` | `model` (`surya` \| `gemma`), `state` (`starting` \| `ready` \| `stopped`), `seconds` (load time, on `ready`) | Model server lifecycle: a server starts with its first job and stops when idle |
| `warning` | `message` | Something worth showing (e.g. `cancelled`) |
| `file_done` | `md` (path), `images` (int), `report` (path or null), `seconds`, `stats` (counts: pages, tables, figures, described, corrected table values, ...) | The `.md` is written |
| `file_error` | `error` (one line) | The file failed; the job goes on |
| `job_done` | `files`, `ok`, `failed`, `skipped`, `seconds` | End of the command |
| `log` | `message` | Free text (profile lines) |

## Exit codes

`0` all files converted (or nothing to do), `1` at least one file failed, `2` the command could
not start (bad input, no engine or models folder, missing files), `130` cancelled.
`sift engine check`: `0` up to date, `10` an update is available.

## Example

```json
{"type": "job_start", "t": 0.41, "files": 1, "output": "C:\\out", "mode": "hybrid", "describer": "gemma", "engine": "b12368-cuda-13.5", "skipped": 0}
{"type": "server", "t": 0.42, "model": "surya", "state": "starting", "seconds": null}
{"type": "file_start", "t": 0.43, "file": "C:\\pdf\\a.pdf", "index": 0, "total": 1, "pages": 23}
{"type": "stage", "t": 1.5, "file": "C:\\pdf\\a.pdf", "name": "layout", "done": 8, "total": 23}
{"type": "server", "t": 3.6, "model": "surya", "state": "ready", "seconds": 3.1}
{"type": "file_done", "t": 18.0, "file": "C:\\pdf\\a.pdf", "md": "C:\\out\\a\\a.md", "images": 16, "report": null, "seconds": 17.6, "stats": {"pages": 23}}
{"type": "job_done", "t": 18.1, "files": 1, "ok": 1, "failed": 0, "skipped": 0, "seconds": 18.1}
```
