# Settings

One file, `settings.toml`, shared by the window and the command line:
`%APPDATA%\com.nairodorian.sift\settings.toml`, or `Data\settings.toml` next to `sift.exe` when a
file named `portable` sits beside it. `sift config path` prints where it is, `sift config show`
prints it, `sift config set <section.key> <value>` changes one key. The window writes the same
file, each key from one page: Convert (`[convert]` `mode`, `image_dpi`, `tables`, `keep_headers`,
`describer`, `[llm] enabled`), Engine (`[folders] engine_dir`, `[engine]`), Models (`[folders]
models_dir`), Settings (`[scheduler] models`). The theme and motion are the window's own (kept in
its web storage, not in this file).

**Priority:** command-line flags > environment (`SIFT_<SECTION>_<KEY>`, e.g. `SIFT_SURYA_PARALLEL=6`)
> a `--config <file>` given to `sift convert` (any older `sift.toml` works as is) > the settings
file > the defaults below. Flags apply to one run only and never change the file.

**Local only:** converting never touches the network. Only `sift setup`, `sift engine
check|update` and `sift models download|check` (and their buttons) do.

## [folders]

| Key | Default | Meaning |
|---|---|---|
| `engine_dir` | empty | Where the llama.cpp builds go (`b12368-cuda-13.5\`, `current.txt`, `previous.txt`). Chosen by the user: onboarding, Settings, the Engine page, `sift engine folder <path>`, `sift setup --engine-dir`. Nothing is installed while it is empty. |
| `models_dir` | empty | Where the models go (`surya-ocr-2.5-gguf\`, `surya_layout2\`, `gemma-4-E4B-it-qat-GGUF\`). Chosen the same way (`sift models folder <path>`, `sift setup --models-dir`). |

## [engine]

| Key | Default | Meaning |
|---|---|---|
| `backend` | `"cuda-13.5"` | Which official llama.cpp build to install: `cuda-13.5`, a newer `cuda-X.Y` when official builds move to it (the update check says so), `vulkan`, `cpu`, or `auto` (the newest CUDA build the NVIDIA driver supports, else Vulkan, else CPU). Sift never changes it on its own. |
| `check_at_startup` | `false` | Read GitHub's release list when the window opens and show a dot on the Engine menu when a newer build exists. Never downloads. Asked in onboarding. |
| `include_cudart` | `"auto"` | `auto`: the CUDA runtime comes from an installed CUDA 13 toolkit when one is found, otherwise llama.cpp's `cudart` package is downloaded with the build. `always`: always download it. |

## [convert]

| Key | Default | Flag | Meaning |
|---|---|---|---|
| `mode` | `"hybrid"` | `--mode` | `hybrid`: paragraphs, headings and lists from the PDF's text layer; tables, formulas and broken or scanned text from surya. `full-ocr`: surya reads every page. |
| `image_dpi` | `300` | `--image-dpi` | Resolution of the extracted figure PNGs (and of the image Gemma sees). |
| `tables` | `"auto"` | `--tables` | `auto`: pipe tables, an HTML `<table>` only for merged cells. `markdown`: always pipe tables. |
| `keep_headers` | `false` | `--keep-headers` | Keep running page headers and footers. |
| `describer` | `"gemma"` | `--describer`, `--no-describe` | Image descriptions: `gemma` or `off`. |
| `layout_batch` | `8` | | Pages per layout batch (surya starts on the first tables while layout goes on). |
| `pages` | `""` | `--pages` | Only these pages, 1-based: `"1-5,137"`, `"249-"`. Empty = every page. |

## [surya]

| Key | Default | Meaning |
|---|---|---|
| `model` | `"q8_0"` | `q8_0` or `bf16` (`sift models download surya-2.5 --quant BF16`). Measured: identical tables, Q8_0 1.3x faster and 0.5 GB smaller. |
| `parallel` | `6` | llama-server slots. Measured: 6 = 0.50 s per table, 4 = 0.59, 8 = no better. |
| `ctx_per_slot` | `12288` | Context per slot (surya's own). |

surya's sampling (temperature 0, top-p 0.1, warmer retries when it loops) is how the model is meant
to be used, so it is not a setting.

## [gemma]

| Key | Default | Meaning |
|---|---|---|
| `parallel` | `2` | Slots (figures described at once). |
| `ctx_per_slot` | `8192` | Context per slot (room for an image, the prompt and thinking). |
| `mtp` | `true` | Speculative decoding with Gemma's MTP drafter: 1.5x faster with images, same answers. |
| `spec_draft_n_max` | `4` | Tokens drafted per step. |
| `kv_type` | `"f16"` | KV cache type. |
| `batch`, `ubatch` | `2048`, `512` | `-b` / `-ub`. |
| `temperature`, `top_p`, `top_k`, `min_p` | `0.05`, `0.35`, `64`, `0.0` | Sampling for descriptions (the structured LLM-pass requests decode greedily). |
| `max_tokens` | `900` | Longest description; an answer that hits it or loops is retried once, then trimmed and marked. |
| `reasoning` | `"off"` | The server's default thinking mode (the decision tasks turn thinking on per request, `[llm] reasoning`). |
| `image_min_tokens`, `image_max_tokens` | unset | Image detail (both must be raised together to matter). |
| `repeat_penalty` | unset | Sampling repeat penalty. |

## [describe]

| Key | Default | Meaning |
|---|---|---|
| `figure_ocr` | `true` | A figure with no PDF text inside it is first read by surya; its labels go to Gemma as "text inside the figure" (misread labels per figure 0.71 -> 0.25). |

## [llm]

The LLM pass beyond descriptions (`--no-llm` turns it off).

| Key | Default | Meaning |
|---|---|---|
| `enabled` | `true` | Master switch. |
| `tables` | `true` | Review tables Sift couldn't fully verify; a correction is kept only if every value in it is in the PDF text and nothing verified is lost. |
| `headings` | `true` | Levels of unnumbered headings (numbered headings keep their exact level); asked 3 times, majority wins. |
| `table_merge` | `true` | Tables split across pages that the geometric rule left apart. |
| `equations` | `true` | Formulas surya returned no LaTeX for. |
| `reasoning` | `true` | Gemma thinks before the decision tasks (heading levels: 13/23 -> 23/23 with thinking). |

## [scheduler] (also read as `[budget]`)

| Key | Default | Meaning |
|---|---|---|
| `models` | `"sequential"` | `sequential`: surya first, Gemma after surya stopped (measured faster on one 8 GB GPU: 281 s vs 288 s, 2.2 GB vs 5.8 GB). `together`: both at once. |
| `together_min_free_mib` | `6300` | With `together`: the free VRAM needed to start both. |
| `layout_workers` | `0` | Page-parallel layout workers (0 = automatic). |

## Environment only (development)

| Variable | Meaning |
|---|---|
| `SIFT_SETTINGS` | Use this settings file instead (the golden test points it at nothing). |
| `SIFT_ENGINE_DIR`, `SIFT_MODELS_DIR` | Override the two folders for one process. |
| `SIFT_ANSWERS` | Same as `--answers`: keep every model answer in this folder; a re-run reuses them. |
| `SIFT_PDFIUM` | Path of `pdfium.dll` (default: next to the executable, or `runtime\pdfium\` for development builds). |
| `SIFT_PROMPTS_DIR`, `SIFT_CATALOG_DIR` | Where the prompts and the model catalog are (default: next to the executable or in an ancestor folder). |
| `SIFT_LAYOUT_DEVICE=cpu` | Run the layout model on the CPU even when the GPU build finds a GPU. |
| `GITHUB_TOKEN` | Used for GitHub's release list when set (raises the anonymous limit of 60 requests an hour). |
