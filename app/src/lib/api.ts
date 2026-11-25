import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

/* The Rust commands (app/src-tauri/src/main.rs) and events (docs/events.md), typed. In a plain
   browser during development (`bun run dev`, no Tauri) the calls go to lib/mock.ts instead, so
   the window can be looked at and tested there; a production build never contains the mock. */

export const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

// `import.meta.env.DEV` stays written out at each use: Vite replaces it with `false` in a
// production build, so these branches and the mock's chunk are dropped from it.
export async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (import.meta.env.DEV && !inTauri) return (await import("./mock")).mockCall<T>(cmd, args ?? {});
  return invoke<T>(cmd, args);
}

export function on<T>(event: string, fn: (payload: T) => void): void {
  if (import.meta.env.DEV && !inTauri) {
    void import("./mock").then((m) => m.mockListen<T>(event, fn));
    return;
  }
  void listen<T>(event, (e) => fn(e.payload));
}

// ---- types ---------------------------------------------------------------------------------

export type SiftEvent = {
  type: string;
  t?: number;
  file?: string;
  name?: string;
  done?: number;
  total?: number;
  index?: number;
  pages?: number;
  files?: number;
  output?: string;
  mode?: string;
  describer?: string;
  engine?: string | null;
  skipped?: number;
  message?: string;
  md?: string | null;
  images?: number;
  report?: string | null;
  seconds?: number;
  stats?: Record<string, number>;
  model?: string;
  state?: string;
  error?: string;
  /** file_error: the file stopped on Cancel, nothing was written for it */
  cancelled?: boolean;
  ok?: number;
  failed?: number;
};
export type DoneEvent = { ok: boolean; cancelled?: boolean; error?: string };

export type Build = { name: string; dir: string; size_mb: number; cuda_runtime_mb: number; active: boolean; previous: boolean };
export type EngineStatus = {
  active: string | null;
  previous: string | null;
  version: string | null;
  dir: string | null;
  backend: string;
  detected: string;
  driver_cuda: string | null;
  toolkit: string | null;
  include_cudart: string;
  check_at_startup: boolean;
  builds: Build[];
};
export type EngineCheck = {
  active: string | null;
  newest: string;
  backend: string;
  size: number;
  published: string;
  update_available: boolean;
  newer_cuda: string | null;
};
export type ModelStatus = {
  id: string;
  name: string;
  role: string;
  license: string;
  revision: string;
  folder: string | null;
  size: number;
  missing: string[];
  installed: boolean;
};
export type Overview = {
  version: string;
  settings_file: string;
  portable: boolean;
  engine_dir: string;
  models_dir: string;
  engine_active: string | null;
  models: ModelStatus[];
  check_at_startup: boolean;
};
export type InputInfo = {
  kind: "pdf" | "folder";
  name: string;
  dir: string;
  path: string;
  size?: number;
  pages?: number | null;
  pdfs?: number;
  pdfs_all?: number;
};
export type Gpu = { name: string; total_mib: number; used_mib: number; driver: string };
export type AboutInfo = { version: string; settings: string; exe_dir: string; gpus: Gpu[]; driver_cuda: string | null; cpu_threads: number };
export type ModelProgress = { id: string; file: string; phase: string; done: number; total: number };

/** The settings file (docs/settings.md), as `settings_get` returns it. Only the keys the window
 * edits are typed; every other key travels through untouched when it is saved back. */
export type Settings = {
  folders: { engine_dir: string; models_dir: string };
  engine: { backend: string; check_at_startup: boolean; include_cudart: string };
  convert: { mode: string; image_dpi: number; tables: string; keep_headers: boolean; describer: string; pages: string };
  llm: { enabled: boolean };
  scheduler: { models: string };
};
export type Section = keyof Settings;
