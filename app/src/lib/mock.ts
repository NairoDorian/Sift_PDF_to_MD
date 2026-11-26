/* Development only: the Rust commands and events, faked, so the window runs in a plain browser
   (`bun run dev`, then http://localhost:1430). Never part of a production build: lib/store.ts
   loads this file only when `import.meta.env.DEV` is true and Tauri is absent.

   The handlers are typed by the generated bindings (`MockCommandHandlers`): each command's
   arguments and result here are the Rust side's own, so the preview cannot drift from the app.
   `mockCommands` installs them through Tauri's own IPC mock, which also delivers the events this
   file sends (`events.*.emit`) to the pages' listeners.

   URL parameters drive what it shows:
     ?page=engine          open a page first (setup, convert, engine, models, settings, logs, about)
     ?demo=setup           nothing installed, no folders chosen
     ?demo=running         a conversion in progress (it stays in the surya stage)
     ?demo=done            a finished conversion with its Markdown
     ?demo=folder          a finished folder of PDFs
     ?theme=dark|light     force a theme
     ?sidebar=collapsed    the menu as icons */

import type { SiftEvent } from "./api";
import {
  events,
  mockCommands,
  type MockCommandHandlers,
  type ModelStatus_Serialize as ModelStatus,
  type Settings_Serialize as Settings,
  type Value,
} from "./bindings";

const params = new URLSearchParams(location.search);
const demo = params.get("demo") ?? "";
const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));

const fresh = demo === "setup";
/** The settings file as the app sends it: every section, the defaults of sift_core::config. */
const settings: Settings = {
  folders: { engine_dir: fresh ? "" : "D:\\Sift\\engine", models_dir: fresh ? "" : "D:\\Sift\\models" },
  engine: { backend: "cuda-13.5", check_at_startup: false, include_cudart: "auto" },
  convert: { mode: "hybrid", image_dpi: 300, tables: "auto", keep_headers: false, describer: "gemma", layout_batch: 8, pages: "" },
  surya: { model: "q8_0", parallel: 6, ctx_per_slot: 12288 },
  gemma: {
    model: "e4b",
    tuned: "not yet",
    parallel: 2,
    ctx_per_slot: 8192,
    mtp: true,
    draft: null,
    spec_draft_n_max: 4,
    kv_type: "f16",
    batch: 2048,
    ubatch: 512,
    temperature: 0.05,
    top_p: 0.35,
    top_k: 64,
    min_p: 0,
    repeat_penalty: null,
    max_tokens: 900,
    image_min_tokens: null,
    image_max_tokens: null,
    reasoning: "off",
  },
  llm: { enabled: true, tables: true, headings: true, table_merge: true, equations: true, reasoning: true },
  describe: { figure_ocr: true },
  scheduler: { models: "sequential", together_min_free_mib: 6300, layout_workers: 0 },
};
const state = {
  engineDir: settings.folders.engine_dir,
  modelsDir: settings.folders.models_dir,
  engineActive: fresh ? null : ("b12368-cuda-13.5" as string | null),
  installed: { "surya-2.5": !fresh, "surya-layout": !fresh, "gemma-4-e4b-qat": !fresh } as Record<string, boolean>,
  settings,
  cancel: false,
};

const MODELS: Omit<ModelStatus, "installed" | "missing" | "folder">[] = [
  { id: "surya-2.5", name: "surya 2.5", role: "OCR: tables, formulas, scanned or broken text, labels in figures", license: "modified OpenRAIL-M (Datalab)", revision: "eb78671ae9af9bdddfac2e25507c55337b6098e2", size: 836026016 },
  { id: "surya-layout", name: "surya layout (rf-detr)", role: "layout: blocks, labels and reading order of each page", license: "modified OpenRAIL-M (Datalab)", revision: "0aee81d5fd9275c0582e545bf3a56944b1e75679", size: 142004665 },
  { id: "gemma-4-e4b-qat", name: "Gemma 4 E4B (QAT, Unsloth UD-Q4_K_XL)", role: "image descriptions and the LLM pass", license: "Gemma terms", revision: "8c5a9e4fd5482e2be20fe0bf013b4c262a8f4265", size: 5265746464 },
];

function models(): ModelStatus[] {
  return MODELS.map((m) => ({
    ...m,
    installed: state.installed[m.id],
    missing: state.installed[m.id] ? [] : ["(every file)"],
    folder: state.modelsDir ? `${state.modelsDir}\\${m.id}` : null,
  }));
}

const SAMPLE_MD = `# LR2021 Transceiver

## 1 Overview

The LR2021 is a long-range, low-power transceiver for the sub-GHz and 2.4 GHz bands. It supports
LoRa, FLRC, (G)FSK and the Sidewalk physical layers.

| Parameter | Min | Typ | Max | Unit |
|---|---|---|---|---|
| Supply voltage | 1.8 | 3.3 | 3.7 | V |
| Sleep current | | 0.6 | | uA |

![Figure 1: Block diagram](images/page-3-figure-1.png)

> **Image description:** A block diagram: the RF front end on the left feeds two receive paths
> (sub-GHz and 2.4 GHz) that meet in the modem, connected to the SPI interface on the right.

### 1.1 Features
`;

const SETTINGS_FILE = "C:\\Users\\you\\AppData\\Roaming\\com.nairodorian.sift\\settings.toml";

const handlers: MockCommandHandlers = {
  overview: () => ({
    version: "1.0.0-alpha.3",
    settings_file: SETTINGS_FILE,
    portable: false,
    engine_dir: state.engineDir,
    models_dir: state.modelsDir,
    engine_active: state.engineActive,
    models: models(),
    check_at_startup: state.settings.engine.check_at_startup,
  }),
  engineStatus: () => ({
    active: state.engineActive,
    previous: state.engineActive ? "b12201-cuda-13.5" : null,
    version: state.engineActive ? "version: 7168 (b12368)" : null,
    dir: state.engineActive ? `${state.engineDir}\\${state.engineActive}` : null,
    backend: state.settings.engine.backend,
    detected: "cuda",
    driver_cuda: "13.5",
    toolkit: null,
    include_cudart: state.settings.engine.include_cudart,
    check_at_startup: state.settings.engine.check_at_startup,
    builds: state.engineActive
      ? [
          { name: "b12368-cuda-13.5", dir: "", size_mb: 612, cuda_runtime_mb: 403, active: true, previous: false },
          { name: "b12201-cuda-13.5", dir: "", size_mb: 598, cuda_runtime_mb: 403, active: false, previous: true },
        ]
      : [],
  }),
  settingsGet: () => structuredClone(state.settings),
  // the page sends back what settingsGet gave it, one key changed
  settingsSave: ({ settings }) => {
    state.settings = structuredClone(settings) as Settings;
    return null;
  },
  engineCheck: async () => {
    await wait(500);
    return { active: state.engineActive, newest: "b12428-cuda-13.5", backend: "cuda-13.5", size: 618_000_000, published: "2026-11-20", update_available: true, newer_cuda: null };
  },
  engineInstall: () => {
    void (async () => {
      await events.engineLog.emit({ message: "Downloading llama-b12428-bin-win-cuda-13.5-x64.zip" });
      for (let i = 1; i <= 10; i++) {
        await wait(160);
        await events.engineProgress.emit({ done: i * 61_800_000, total: 618_000_000 });
      }
      state.engineActive = "b12428-cuda-13.5";
      await events.engineDone.emit({ ok: true, message: "b12428-cuda-13.5 is active (verified: llama-server --version)" });
    })();
    return null;
  },
  engineAction: ({ action }) => {
    void wait(300).then(() => events.engineDone.emit({ ok: true, message: `${action} done` }));
    return null;
  },
  modelDownload: ({ id }) => {
    void (async () => {
      const ids = id === "all" ? MODELS.filter((m) => !state.installed[m.id]).map((m) => m.id) : [id];
      for (const m of ids) {
        const total = MODELS.find((x) => x.id === m)?.size ?? 1;
        for (let i = 1; i <= 8; i++) {
          await wait(120);
          await events.modelProgress.emit({ id: m, file: `${m}.gguf`, phase: "downloading", done: (total * i) / 8, total });
        }
        state.installed[m] = true;
        await events.modelDone.emit({ id: m, ok: true });
      }
    })();
    return null;
  },
  modelVerify: async () => {
    await wait(600);
    return [{ file: "model.gguf", ok: true }];
  },
  modelDelete: ({ id }) => {
    state.installed[id] = false;
    return null;
  },
  modelsCheck: async () => {
    await wait(400);
    return MODELS.map((m) => ({ id: m.id, pinned: m.revision, upstream: m.revision }));
  },
  pickFolder: () => "D:\\Sift\\chosen",
  pickPdf: () => "C:\\Docs\\Datasheets\\LR2021 datasheet.pdf",
  setFolder: ({ kind, path }) => {
    if (kind === "engine") state.engineDir = state.settings.folders.engine_dir = path;
    else state.modelsDir = state.settings.folders.models_dir = path;
    return { moved: [], old: "" };
  },
  inputInfo: ({ path }) => {
    const name = path.split("\\").pop() ?? path;
    const dir = path.slice(0, path.lastIndexOf("\\"));
    return path.toLowerCase().endsWith(".pdf")
      ? { kind: "pdf", name, dir, path, size: 4_812_330, pages: 23 }
      : { kind: "folder", name, dir, path, pdfs: 7, pdfs_all: 12 };
  },
  mdPreview: () => SAMPLE_MD,
  serverCommands: () => {
    const exe = `${state.engineDir}\\${state.engineActive ?? "b12368-cuda-13.5"}\\llama-server.exe`;
    const host = ["--host", "127.0.0.1", "--port", "0"];
    return [
      { model: "surya", args: [exe, "-m", `${state.modelsDir}\\surya-ocr-2.5-gguf\\surya-2.5-Q8_0.gguf`, "--mmproj", `${state.modelsDir}\\surya-ocr-2.5-gguf\\surya-2.5-mmproj.gguf`, "-ngl", "99", "--parallel", "6", "--ctx-size", "73728", "--alias", "surya", "--jinja", "--special", ...host] },
      { model: "gemma", args: [exe, "-m", `${state.modelsDir}\\gemma-4-E4B-it-qat-GGUF\\gemma-4-E4B-it-qat-UD-Q4_K_XL.gguf`, "-ngl", "-1", "--flash-attn", "on", "--parallel", "2", ...host] },
    ];
  },
  serverLog: ({ model, output }) => ({
    file: `${output}\\.sift\\logs\\${model}-20261125-174331-59911.log`,
    lines: ["main: server is listening on http://127.0.0.1:59911", "srv  update_slots: all slots are idle", "slot launch_slot_: id  0 | task 12 | processing task", "slot print_timing: id  0 | prompt eval time = 41.20 ms / 812 tokens"],
  }),
  startConvert: ({ req }) => {
    state.cancel = false;
    void runJob(req.input);
    return null;
  },
  cancelConvert: () => {
    state.cancel = true;
    return true;
  },
  aboutInfo: () => ({
    version: "1.0.0-alpha.3",
    settings: SETTINGS_FILE,
    exe_dir: "C:\\Users\\you\\AppData\\Local\\Sift",
    gpus: [{ name: "NVIDIA GeForce RTX 4070 Laptop GPU", total_mib: 8188, used_mib: 1210, driver: "591.44" }],
    driver_cuda: "13.5",
    cpu_threads: 20,
  }),
  openPath: () => null,
  revealPath: () => null,
  openUrl: () => null,
};

/** Every command answers after a short delay, as over IPC. */
function delayed(h: MockCommandHandlers): MockCommandHandlers {
  const out: Record<string, (args: never) => Promise<unknown>> = {};
  for (const [name, fn] of Object.entries(h) as [string, (args: never) => unknown][]) {
    out[name] = async (args: never) => {
      await wait(40);
      return fn(args);
    };
  }
  return out as unknown as MockCommandHandlers;
}

/** Install the handlers (and Tauri's event mock) before the pages call anything. */
export function installMocks(): void {
  mockCommands(delayed(handlers));
}

// `sift-event` is JSON on the Rust side (`ConvertEvent`); SiftEvent is its documented shape
const siftEvent = (e: SiftEvent) => events.siftEvent.emit(e as unknown as Value);

async function runJob(input: string) {
  const folder = demo === "folder";
  const files = folder ? ["Wio-E5 module.pdf", "LR2021 datasheet.pdf", "STM32WL55 reference.pdf"] : [input.split("\\").pop() ?? input];
  const holdInSurya = demo === "running";
  await siftEvent({ type: "job_start", files: files.length, output: "C:\\Docs\\Datasheets", mode: "hybrid", describer: "gemma", engine: state.engineActive, skipped: folder ? 4 : 0 });
  for (let f = 0; f < files.length; f++) {
    const file = `C:\\Docs\\Datasheets\\${files[f]}`;
    await siftEvent({ type: "file_start", file, index: f, total: files.length, pages: 23 });
    const step = async (name: string, total: number, ms: number) => {
      const by = Math.max(1, Math.round(total / 6));
      for (let i = 0; i < total + by; i += by) {
        if (state.cancel) return false;
        await siftEvent({ type: "stage", file, name, done: Math.min(i, total), total });
        await wait(ms);
      }
      return true;
    };
    if (!(await step("text", 23, 30))) break;
    if (!(await step("layout", 23, 60))) break;
    if (!(await step("build", 23, 40))) break;
    await siftEvent({ type: "server", model: "surya", state: "starting" });
    await wait(150);
    await siftEvent({ type: "server", model: "surya", state: "ready", seconds: 3.1 });
    if (holdInSurya) {
      await siftEvent({ type: "stage", file, name: "surya", done: 12, total: 40 });
      return;
    }
    if (!(await step("surya", 40, 40))) break;
    await siftEvent({ type: "server", model: "surya", state: "stopped" });
    await siftEvent({ type: "server", model: "gemma", state: "ready", seconds: 4.2 });
    if (!(await step("refine", 4, 60))) break;
    if (!(await step("describe", 16, 50))) break;
    await siftEvent({ type: "server", model: "gemma", state: "stopped" });
    const stem = files[f].replace(/\.pdf$/i, "");
    await siftEvent({
      type: "file_done",
      file,
      md: `C:\\Docs\\Datasheets\\${stem}\\${stem}.md`,
      images: 16,
      report: f === 0 ? `C:\\Docs\\Datasheets\\${stem}\\${stem}.report.md` : null,
      seconds: 74.6 + f * 12,
      stats: { pages: 23, tables: 11, equations: 2, figures: 16, described: 16, table_values_corrected: 3, table_values_unresolved: 0 },
    });
  }
  await events.siftDone.emit({ ok: !state.cancel, cancelled: state.cancel });
}

type DemoHooks = {
  chooseInput: (p: string) => Promise<void>;
  startConvert: () => Promise<void>;
  setPage: (p: "setup" | "convert" | "engine" | "models" | "settings" | "logs" | "about") => void;
  setSidebarCollapsed: (v: boolean) => void;
};

export async function runDemo(h: DemoHooks) {
  const theme = params.get("theme");
  if (theme === "dark" || theme === "light") document.documentElement.dataset.theme = theme;
  const motion = params.get("motion");
  if (motion === "on" || motion === "off") document.documentElement.dataset.motion = motion;
  if (params.has("sidebar")) h.setSidebarCollapsed(params.get("sidebar") === "collapsed");
  // the footer's meters, as the app pushes them once a second (a running job loads the GPU)
  const meters = () => {
    const busy = demo === "running";
    void events.meters.emit({
      cpu: 8 + Math.random() * (busy ? 40 : 10),
      ram_used_mb: 15100 + Math.random() * 300,
      ram_total_mb: 32394,
      gpu_name: "NVIDIA GeForce RTX 4070 Laptop GPU",
      gpu: busy ? 70 + Math.random() * 28 : Math.random() * 6,
      vram_used_mb: busy ? 6900 + Math.random() * 300 : 1650,
      vram_total_mb: 8188,
    });
  };
  meters();
  setInterval(meters, 1000);
  await wait(200);
  const page = params.get("page");
  if (page) h.setPage(page as Parameters<DemoHooks["setPage"]>[0]);
  if (demo === "running" || demo === "done" || demo === "folder") {
    await h.chooseInput(demo === "folder" ? "C:\\Docs\\Datasheets" : "C:\\Docs\\Datasheets\\LR2021 datasheet.pdf");
    await wait(100);
    await h.startConvert();
  }
}
