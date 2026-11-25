/* Development only: the Rust commands and events, faked, so the window runs in a plain browser
   (`bun run dev`, then http://localhost:1430). Never part of a production build: lib/api.ts loads
   this file only when `import.meta.env.DEV` is true and Tauri is absent.

   URL parameters drive what it shows:
     ?page=engine          open a page first (setup, convert, engine, models, settings, logs, about)
     ?demo=setup           nothing installed, no folders chosen
     ?demo=running         a conversion in progress (it stays in the surya stage)
     ?demo=done            a finished conversion with its Markdown
     ?demo=folder          a finished folder of PDFs
     ?theme=dark|light     force a theme
     ?sidebar=collapsed    the menu as icons */

import type { EngineCheck, EngineStatus, InputInfo, ModelStatus, Overview, SiftEvent } from "./api";

type Listener = (payload: unknown) => void;
const listeners = new Map<string, Listener[]>();
const params = new URLSearchParams(location.search);
const demo = params.get("demo") ?? "";

export function mockListen<T>(event: string, fn: (payload: T) => void): void {
  listeners.set(event, [...(listeners.get(event) ?? []), fn as Listener]);
}
function emit(event: string, payload: unknown) {
  for (const fn of listeners.get(event) ?? []) fn(payload);
}
const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));

const fresh = demo === "setup";
const state = {
  engineDir: fresh ? "" : "D:\\Sift\\engine",
  modelsDir: fresh ? "" : "D:\\Sift\\models",
  engineActive: fresh ? null : ("b12368-cuda-13.5" as string | null),
  installed: { "surya-2.5": !fresh, "surya-layout": !fresh, "gemma-4-e4b-qat": !fresh } as Record<string, boolean>,
  settings: {
    folders: { engine_dir: fresh ? "" : "D:\\Sift\\engine", models_dir: fresh ? "" : "D:\\Sift\\models" },
    engine: { backend: "cuda-13.5", check_at_startup: false, include_cudart: "auto" },
    convert: { mode: "hybrid", image_dpi: 300, tables: "auto", keep_headers: false, describer: "gemma", layout_batch: 8, pages: "" },
    surya: { model: "q8_0", parallel: 6, ctx_per_slot: 12288 },
    llm: { enabled: true, tables: true, headings: true, table_merge: true, equations: true, reasoning: true },
    scheduler: { models: "sequential", together_min_free_mib: 6300, layout_workers: 0 },
  },
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

export async function mockCall<T>(cmd: string, args: Record<string, unknown>): Promise<T> {
  await wait(40);
  const r = (v: unknown) => v as T;
  switch (cmd) {
    case "overview": {
      const o: Overview = {
        version: "1.0.0-alpha.2",
        settings_file: "C:\\Users\\you\\AppData\\Roaming\\com.nairodorian.sift\\settings.toml",
        portable: false,
        engine_dir: state.engineDir,
        models_dir: state.modelsDir,
        engine_active: state.engineActive,
        models: models(),
        check_at_startup: state.settings.engine.check_at_startup,
      };
      return r(o);
    }
    case "engine_status": {
      const e: EngineStatus = {
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
      };
      return r(e);
    }
    case "settings_get":
      return r(structuredClone(state.settings));
    case "settings_save":
      state.settings = structuredClone(args.settings as typeof state.settings);
      return r(null);
    case "engine_check": {
      await wait(500);
      const c: EngineCheck = { active: state.engineActive, newest: "b12428-cuda-13.5", backend: "cuda-13.5", size: 618_000_000, published: "2026-11-20", update_available: true, newer_cuda: null };
      return r(c);
    }
    case "engine_install":
      void (async () => {
        emit("engine-log", { message: "Downloading llama-b12428-bin-win-cuda-13.5-x64.zip" });
        for (let i = 1; i <= 10; i++) {
          await wait(160);
          emit("engine-progress", { done: i * 61_800_000, total: 618_000_000 });
        }
        state.engineActive = "b12428-cuda-13.5";
        emit("engine-done", { ok: true, message: "b12428-cuda-13.5 is active (verified: llama-server --version)" });
      })();
      return r(null);
    case "engine_action":
      void wait(300).then(() => emit("engine-done", { ok: true, message: `${String(args.action)} done` }));
      return r(null);
    case "model_download":
      void (async () => {
        const ids = args.id === "all" ? MODELS.filter((m) => !state.installed[m.id]).map((m) => m.id) : [String(args.id)];
        for (const id of ids) {
          const total = MODELS.find((m) => m.id === id)?.size ?? 1;
          for (let i = 1; i <= 8; i++) {
            await wait(120);
            emit("model-progress", { id, file: `${id}.gguf`, phase: "downloading", done: (total * i) / 8, total });
          }
          state.installed[id] = true;
          emit("model-done", { id, ok: true });
        }
      })();
      return r(null);
    case "model_verify":
      await wait(600);
      return r([{ file: "model.gguf", ok: true }]);
    case "model_delete":
      state.installed[String(args.id)] = false;
      return r(null);
    case "models_check":
      await wait(400);
      return r(MODELS.map((m) => ({ id: m.id, pinned: m.revision, upstream: m.revision })));
    case "pick_folder":
      return r("D:\\Sift\\chosen");
    case "pick_pdf":
      return r("C:\\Docs\\Datasheets\\LR2021 datasheet.pdf");
    case "set_folder":
      if (args.kind === "engine") state.engineDir = state.settings.folders.engine_dir = String(args.path);
      else state.modelsDir = state.settings.folders.models_dir = String(args.path);
      return r({ moved: [], old: "" });
    case "input_info": {
      const path = String(args.path);
      const isPdf = path.toLowerCase().endsWith(".pdf");
      const name = path.split("\\").pop() ?? path;
      const info: InputInfo = isPdf
        ? { kind: "pdf", name, dir: path.slice(0, path.lastIndexOf("\\")), path, size: 4_812_330, pages: 23 }
        : { kind: "folder", name, dir: path.slice(0, path.lastIndexOf("\\")), path, pdfs: 7, pdfs_all: 12 };
      return r(info);
    }
    case "md_preview":
      return r(SAMPLE_MD);
    case "server_commands": {
      const exe = `${state.engineDir}\\${state.engineActive ?? "b12368-cuda-13.5"}\\llama-server.exe`;
      const host = ["--host", "127.0.0.1", "--port", "0"];
      return r([
        { model: "surya", args: [exe, "-m", `${state.modelsDir}\\surya-ocr-2.5-gguf\\surya-2.5-Q8_0.gguf`, "--mmproj", `${state.modelsDir}\\surya-ocr-2.5-gguf\\surya-2.5-mmproj.gguf`, "-ngl", "99", "--parallel", "6", "--ctx-size", "73728", "--alias", "surya", "--jinja", "--special", ...host] },
        { model: "gemma", args: [exe, "-m", `${state.modelsDir}\\gemma-4-E4B-it-qat-GGUF\\gemma-4-E4B-it-qat-UD-Q4_K_XL.gguf`, "-ngl", "-1", "--flash-attn", "on", "--parallel", "2", ...host] },
      ]);
    }
    case "server_log":
      return r({ file: `${String(args.output)}\\.sift\\logs\\${String(args.model)}-20261125-174331-59911.log`, lines: ["main: server is listening on http://127.0.0.1:59911", "srv  update_slots: all slots are idle", "slot launch_slot_: id  0 | task 12 | processing task", "slot print_timing: id  0 | prompt eval time = 41.20 ms / 812 tokens"] });
    case "start_convert":
      state.cancel = false;
      void runJob(String((args.req as { input: string }).input));
      return r(null);
    case "cancel_convert":
      state.cancel = true;
      return r(true);
    case "about_info":
      return r({ version: "1.0.0-alpha.2", settings: "C:\\Users\\you\\AppData\\Roaming\\com.nairodorian.sift\\settings.toml", gpus: [{ name: "NVIDIA GeForce RTX 4070 Laptop GPU", total_mib: 8188, used_mib: 1210, driver: "591.44" }], driver_cuda: "13.5", cpu_threads: 20 });
    default:
      return r(null);
  }
}

async function runJob(input: string) {
  const ev = (e: SiftEvent) => emit("sift-event", e);
  const folder = demo === "folder";
  const files = folder ? ["Wio-E5 module.pdf", "LR2021 datasheet.pdf", "STM32WL55 reference.pdf"] : [input.split("\\").pop() ?? input];
  const holdInSurya = demo === "running";
  ev({ type: "job_start", files: files.length, output: "C:\\Docs\\Datasheets", mode: "hybrid", describer: "gemma", engine: state.engineActive, skipped: folder ? 4 : 0 });
  for (let f = 0; f < files.length; f++) {
    const file = `C:\\Docs\\Datasheets\\${files[f]}`;
    ev({ type: "file_start", file, index: f, total: files.length, pages: 23 });
    const step = async (name: string, total: number, ms: number) => {
      const by = Math.max(1, Math.round(total / 6));
      for (let i = 0; i < total + by; i += by) {
        if (state.cancel) return false;
        ev({ type: "stage", file, name, done: Math.min(i, total), total });
        await wait(ms);
      }
      return true;
    };
    if (!(await step("text", 23, 30))) break;
    if (!(await step("layout", 23, 60))) break;
    if (!(await step("build", 23, 40))) break;
    ev({ type: "server", model: "surya", state: "starting" });
    await wait(150);
    ev({ type: "server", model: "surya", state: "ready", seconds: 3.1 });
    if (holdInSurya) {
      ev({ type: "stage", file, name: "surya", done: 12, total: 40 });
      return;
    }
    if (!(await step("surya", 40, 40))) break;
    ev({ type: "server", model: "surya", state: "stopped" });
    ev({ type: "server", model: "gemma", state: "ready", seconds: 4.2 });
    if (!(await step("refine", 4, 60))) break;
    if (!(await step("describe", 16, 50))) break;
    ev({ type: "server", model: "gemma", state: "stopped" });
    const stem = files[f].replace(/\.pdf$/i, "");
    ev({
      type: "file_done",
      file,
      md: `C:\\Docs\\Datasheets\\${stem}\\${stem}.md`,
      images: 16,
      report: f === 0 ? `C:\\Docs\\Datasheets\\${stem}\\${stem}.report.md` : null,
      seconds: 74.6 + f * 12,
      stats: { pages: 23, tables: 11, equations: 2, figures: 16, described: 16, table_values_corrected: 3, table_values_unresolved: 0 },
    });
  }
  emit("sift-done", state.cancel ? { ok: false, cancelled: true } : { ok: true });
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
    emit("meters", {
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
