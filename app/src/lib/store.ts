import { createSignal } from "solid-js";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import {
  call,
  inTauri,
  on,
  type DoneEvent,
  type EngineCheck,
  type EngineStatus,
  type InputInfo,
  type ModelProgress,
  type ModelStatus,
  type Overview,
  type Section,
  type Settings,
  type SiftEvent,
} from "./api";
import { fileName } from "./format";

/* The window's state, shared by every page: what the Rust side reports (overview, engine, models,
   settings), the running conversion and the session log. Module-level signals, one listener per
   event (ZER0's stores: the footer and the pages read the same subscription). Solid 2 batches
   writes, so nothing here reads a signal back in the frame that wrote it. */

export type Page = "setup" | "convert" | "engine" | "models" | "settings" | "logs" | "about";
export const [page, setPage] = createSignal<Page>("convert");

export const [overview, setOverview] = createSignal<Overview | null>(null);
export const [engine, setEngine] = createSignal<EngineStatus | null>(null);
export const [settings, setSettings] = createSignal<Settings | null>(null);
export const models = (): ModelStatus[] => overview()?.models ?? [];

/** The four setup steps (the Setup page, the menu count). */
export const setupSteps = () => {
  const o = overview();
  return [
    !!o?.engine_dir,
    !!o?.models_dir,
    !!o?.engine_active,
    !!o && o.models.length > 0 && o.models.every((m) => m.installed),
  ];
};
export const ready = () => setupSteps().every(Boolean);

export async function refresh(): Promise<void> {
  try {
    const [o, e, s] = await Promise.all([
      call<Overview>("overview"),
      call<EngineStatus>("engine_status"),
      call<Settings>("settings_get"),
    ]);
    setOverview(o);
    setEngine(e);
    setSettings(s);
  } catch (err) {
    log("error", "app", `refresh: ${err}`);
  }
  await loadServerLines();
}

/** Save one key of the settings file (the window and the command line share it). */
export async function saveSetting<S extends Section, K extends keyof Settings[S]>(section: S, key: K, value: Settings[S][K]) {
  const cur = settings();
  if (!cur) return;
  const next = { ...cur, [section]: { ...cur[section], [key]: value } } as Settings;
  setSettings(next);
  try {
    await call("settings_save", { settings: next });
  } catch (err) {
    log("error", "settings", `${String(section)}.${String(key)}: ${err}`);
  }
  void refresh();
}

// ---- the session log (the Logs page) --------------------------------------------------------

export type Level = "info" | "ok" | "warn" | "error" | "server" | "progress";
export type LogEntry = { id: number; time: string; level: Level; kind: string; text: string };
export const [logs, setLogs] = createSignal<LogEntry[]>([]);
let logId = 0;
export function log(level: Level, kind: string, text: string) {
  const d = new Date();
  const time = [d.getHours(), d.getMinutes(), d.getSeconds()].map((n) => String(n).padStart(2, "0")).join(":");
  setLogs((l) => {
    const last = l.at(-1);
    // progress lines of one stage replace each other: the log keeps the newest count
    if (level === "progress" && last?.level === "progress" && last.kind === kind) {
      return [...l.slice(0, -1), { ...last, time, text }];
    }
    return [...l.slice(-1999), { id: ++logId, time, level, kind, text }];
  });
}

// ---- model servers (the footer, the Engine page) ----------------------------------------------

export const [servers, setServers] = createSignal<Record<string, string>>({});
/** The output folder of this session's latest job: where its servers' logs are (`.sift\logs`). */
export const [lastOutput, setLastOutput] = createSignal<string | null>(null);

export type ServerCommand = { model: string; args: string[] };
/** How Sift would start each model server now (read-only; nothing is started), or why it can't
 * say (no engine, no models). Refreshed with the rest by `refresh()`: a local call, no network. */
export const [serverLines, setServerLines] = createSignal<{ list: ServerCommand[]; error: string }>({ list: [], error: "" });
async function loadServerLines() {
  try {
    setServerLines({ list: await call<ServerCommand[]>("server_commands"), error: "" });
  } catch (err) {
    setServerLines({ list: [], error: String(err) });
  }
}
/** The end of a server's newest log in the latest job's output folder. */
export async function serverLog(model: string): Promise<{ file: string | null; lines: string[] } | null> {
  const out = lastOutput();
  if (!out) return null;
  return call("server_log", { model, output: out });
}

// ---- the conversion --------------------------------------------------------------------------

export type RunOptions = { pages: string; recursive: boolean; force: boolean; output: string };
export const [input, setInput] = createSignal("");
export const [inputInfo, setInputInfo] = createSignal<InputInfo | null>(null);
export const [inputError, setInputError] = createSignal("");
export const [runOptions, setRunOptions] = createSignal<RunOptions>({ pages: "", recursive: false, force: false, output: "" });
export const [dragOver, setDragOver] = createSignal(false);

export async function chooseInput(path: string) {
  setInput(path);
  setInputInfo(null);
  setInputError("");
  if (!path) return;
  try {
    setInputInfo(await call<InputInfo>("input_info", { path }));
  } catch (err) {
    setInputError(String(err));
  }
}

export type FileResult = {
  file: string;
  md: string | null;
  report: string | null;
  images: number;
  seconds: number;
  stats: Record<string, number>;
  error?: string;
  /** stopped on Cancel: nothing was written for it */
  cancelled?: boolean;
};
export type StageCount = { done: number; total: number };
export type Job = {
  kind: "pdf" | "folder";
  input: string;
  started: number;
  ended: number | null;
  state: "running" | "done" | "failed" | "cancelled";
  error: string | null;
  describe: boolean;
  llm: boolean;
  files: number;
  skipped: number;
  fileIndex: number;
  file: string;
  pages: number;
  stages: Record<string, StageCount>;
  stage: string;
  /** the whole job, 0..1, never going back */
  progress: number;
  results: FileResult[];
  preview: string;
  cancelling: boolean;
};
export const [job, setJob] = createSignal<Job | null>(null);
export const converting = () => job()?.state === "running";

/** The pipeline's stages in the order they finish (docs/events.md). */
export const STAGES = ["text", "layout", "build", "surya", "refine", "describe"] as const;
/** Rough share of a file's time per stage, for the progress bar only. From one Wio run from the
 * window (2026-11-25, server-log timestamps): surya's phase about 40 % of the file, Gemma's (the
 * LLM pass, then 15 descriptions) about 53 %, text and layout the rest, smaller still in a release
 * build. An estimate: the bar never goes back and the file's end sets it to its share. */
const WEIGHT: Record<string, number> = { text: 0.02, layout: 0.08, build: 0.05, surya: 0.3, refine: 0.1, describe: 0.45 };

function fileFraction(j: Job): number {
  const expected = STAGES.filter((s) => (s === "refine" ? j.llm : s === "describe" ? j.describe : true));
  let furthest = -1;
  expected.forEach((s, i) => {
    if (j.stages[s]) furthest = i;
  });
  let sum = 0;
  let total = 0;
  expected.forEach((s, i) => {
    const w = WEIGHT[s];
    total += w;
    const st = j.stages[s];
    // a later stage has started, so this one is over; except layout while pages are being
    // built: the two run batch by batch, side by side
    const over = i < furthest && !(s === "layout" && expected[furthest] === "build");
    if (over) sum += w;
    else if (st) sum += w * (st.total > 0 ? Math.min(1, st.done / st.total) : 1);
  });
  return total ? sum / total : 0;
}

function patchJob(f: (j: Job) => Job) {
  setJob((j) => (j ? f(j) : j));
}

export async function startConvert(): Promise<void> {
  const s = settings();
  const path = input().trim();
  if (!path || !s || converting()) return;
  const info = inputInfo();
  const ro = runOptions();
  const describe = s.convert.describer !== "off";
  setJob({
    kind: info?.kind ?? "pdf",
    input: path,
    started: Date.now(),
    ended: null,
    state: "running",
    error: null,
    describe,
    llm: s.llm.enabled,
    files: info?.kind === "folder" ? 0 : 1,
    skipped: 0,
    fileIndex: 0,
    file: info?.kind === "pdf" ? path : "",
    pages: info?.pages ?? 0,
    stages: {},
    stage: "",
    progress: 0,
    results: [],
    preview: "",
    cancelling: false,
  });
  log("info", "convert", `Converting ${path}`);
  try {
    await call("start_convert", {
      req: {
        input: path,
        output: ro.output || null,
        pages: ro.pages.trim() || null,
        recursive: ro.recursive,
        force: ro.force,
        describer: describe ? "gemma" : "off",
        llm: s.llm.enabled,
        mode: s.convert.mode,
        tables: s.convert.tables,
        keep_headers: s.convert.keep_headers,
        image_dpi: s.convert.image_dpi,
      },
    });
  } catch (err) {
    patchJob((j) => ({ ...j, state: "failed", error: String(err), ended: Date.now() }));
    log("error", "convert", String(err));
  }
}

export async function cancelConvert(): Promise<void> {
  if (await call<boolean>("cancel_convert")) {
    patchJob((j) => ({ ...j, cancelling: true }));
    log("warn", "convert", "Cancelling: queued work stops, then the model servers.");
  }
}

export function clearJob() {
  if (!converting()) setJob(null);
}

async function loadPreview(md: string) {
  try {
    const text = await call<string>("md_preview", { path: md, maxBytes: 6000 });
    patchJob((j) => ({ ...j, preview: text }));
  } catch {
    /* the result card shows the file without its first lines */
  }
}

function onSiftEvent(d: SiftEvent) {
  switch (d.type) {
    case "job_start":
      patchJob((j) => ({ ...j, files: d.files ?? j.files, skipped: d.skipped ?? 0 }));
      if (d.output) setLastOutput(d.output);
      log("info", "job", `${d.files ?? 0} PDF(s) to convert${d.skipped ? `, ${d.skipped} up to date` : ""} -> ${d.output ?? ""}${d.engine ? ` (llama.cpp ${d.engine})` : ""}`);
      break;
    case "file_start":
      patchJob((j) => ({ ...j, fileIndex: d.index ?? 0, files: d.total ?? j.files, file: d.file ?? "", pages: d.pages ?? 0, stages: {}, stage: "" }));
      log("info", "file", `${(d.index ?? 0) + 1}/${d.total ?? 1}: ${fileName(d.file ?? "")} (${d.pages ?? "?"} pages)`);
      break;
    case "stage":
      patchJob((j) => {
        // after Cancel the remaining requests fail at once: the counts stay where the work was
        if (j.cancelling) return j;
        const stages = { ...j.stages, [d.name ?? "?"]: { done: d.done ?? 0, total: d.total ?? 0 } };
        const next = { ...j, stages, stage: d.name ?? j.stage };
        const p = (j.fileIndex + fileFraction(next)) / Math.max(1, j.files);
        return { ...next, progress: Math.max(j.progress, Math.min(0.995, p)) };
      });
      log("progress", `stage:${d.name}`, `${d.name} ${d.done}/${d.total}  ${fileName(d.file ?? "")}`);
      break;
    case "server":
      setServers((s) => ({ ...s, [d.model ?? "?"]: d.state ?? "" }));
      log("server", d.model ?? "server", `${d.state}${d.seconds ? ` (${d.seconds.toFixed(1)} s)` : ""}`);
      break;
    case "file_done": {
      const r: FileResult = { file: d.file ?? "", md: d.md ?? null, report: d.report ?? null, images: d.images ?? 0, seconds: d.seconds ?? 0, stats: d.stats ?? {} };
      patchJob((j) => ({ ...j, results: [...j.results, r], progress: Math.max(j.progress, (j.fileIndex + 1) / Math.max(1, j.files)) }));
      log("ok", "done", `${fileName(r.md ?? r.file)} in ${r.seconds.toFixed(1)} s${r.images ? `, ${r.images} images` : ""}${r.report ? ", report written" : ""}`);
      if (r.md) void loadPreview(r.md);
      break;
    }
    case "file_error": {
      const error = d.cancelled ? "Cancelled: not written" : (d.error ?? "failed");
      patchJob((j) => ({ ...j, results: [...j.results, { file: d.file ?? "", md: null, report: null, images: 0, seconds: 0, stats: {}, error, cancelled: !!d.cancelled }] }));
      if (d.cancelled) log("warn", "cancelled", `${fileName(d.file ?? "")}: nothing written; an earlier Markdown of it is unchanged`);
      else log("error", "failed", `${fileName(d.file ?? "")}: ${d.error ?? ""}`);
      break;
    }
    case "job_done":
      log("info", "job", `finished: ${d.ok ?? 0} converted, ${d.failed ?? 0} failed, ${d.skipped ?? 0} up to date (${(d.seconds ?? 0).toFixed(1)} s)`);
      break;
    case "warning":
      log("warn", "warning", d.message ?? "");
      break;
    case "log":
      log("info", "log", d.message ?? "");
      break;
    default:
      log("info", d.type, JSON.stringify(d));
  }
}

function onSiftDone(e: DoneEvent) {
  patchJob((j) => ({
    ...j,
    state: e.cancelled ? "cancelled" : e.ok ? "done" : "failed",
    error: e.error ?? (e.ok || e.cancelled ? null : j.results.find((r) => r.error)?.error ?? "a file failed (see the log)"),
    ended: Date.now(),
    progress: e.ok ? 1 : j.progress,
    cancelling: false,
  }));
  log(e.cancelled ? "warn" : e.ok ? "ok" : "error", "convert", e.cancelled ? "Cancelled." : e.ok ? "Finished." : `Stopped: ${e.error ?? "a file failed"}`);
}

// ---- the engine (llama.cpp) ---------------------------------------------------------------------

export const [check, setCheck] = createSignal<EngineCheck | null>(null);
export const [checking, setChecking] = createSignal(false);
export const [engineBusy, setEngineBusy] = createSignal(false);
export const [engineLog, setEngineLog] = createSignal<string[]>([]);
export const [download, setDownload] = createSignal<{ done: number; total: number } | null>(null);
export const updateAvailable = () => !!check()?.update_available;

function engineLine(s: string) {
  setEngineLog((l) => [...l.slice(-400), s]);
  log("info", "engine", s);
}

/** Read GitHub's release list (no download): only from a button, or at start-up when the user
 * turned that on (the Rust side does it then). */
export async function checkEngine() {
  setChecking(true);
  try {
    setCheck(await call<EngineCheck>("engine_check"));
  } catch (err) {
    engineLine(`Check failed: ${err}`);
  }
  setChecking(false);
}

export async function installEngine(tag?: string) {
  setEngineBusy(true);
  setEngineLog([]);
  try {
    await call("engine_install", { tag: tag ?? null });
  } catch (err) {
    setEngineBusy(false);
    engineLine(String(err));
  }
}

export async function engineAction(action: "use" | "rollback" | "remove" | "remove_runtime", build?: string) {
  setEngineBusy(true);
  try {
    await call("engine_action", { action, build: build ?? null });
  } catch (err) {
    setEngineBusy(false);
    engineLine(String(err));
  }
}

// ---- the models ---------------------------------------------------------------------------------

export const [modelProgress, setModelProgress] = createSignal<Record<string, ModelProgress>>({});
export const [modelNote, setModelNote] = createSignal<Record<string, { text: string; level: Level }>>({});
const note = (id: string, text: string, level: Level = "info") => setModelNote((n) => ({ ...n, [id]: { text, level } }));

export async function downloadModel(id: string) {
  const ids = id === "all" ? models().filter((m) => !m.installed).map((m) => m.id) : [id];
  for (const m of ids) note(m, "waiting for the download to start");
  try {
    await call("model_download", { id });
  } catch (err) {
    for (const m of ids) note(m, String(err), "error");
  }
}

export async function verifyModel(id: string) {
  note(id, "checking every file's sha256...");
  try {
    const r = await call<{ file: string; ok: boolean }[]>("model_verify", { id });
    const bad = r.filter((x) => !x.ok).map((x) => x.file);
    note(id, bad.length ? `missing or damaged: ${bad.join(", ")}` : "every file verified (sha256)", bad.length ? "error" : "ok");
  } catch (err) {
    note(id, String(err), "error");
  }
}

export async function deleteModel(id: string) {
  try {
    await call("model_delete", { id });
    note(id, "deleted");
  } catch (err) {
    note(id, String(err), "error");
  }
  void refresh();
}

export const [revisions, setRevisions] = createSignal<Record<string, string>>({});
export const [checkingRevisions, setCheckingRevisions] = createSignal(false);
export async function checkRevisions() {
  setCheckingRevisions(true);
  try {
    const r = await call<{ id: string; pinned: string; upstream: string | null }[]>("models_check");
    setRevisions(
      Object.fromEntries(
        r.map((x) => [
          x.id,
          x.upstream == null ? "could not check" : x.upstream === x.pinned ? "the pinned revision is the newest" : `newer upstream: ${x.upstream.slice(0, 8)} (adopting it needs the golden test)`,
        ]),
      ),
    );
  } catch (err) {
    log("error", "models", `revision check: ${err}`);
  }
  setCheckingRevisions(false);
}

// ---- folders, files ----------------------------------------------------------------------------

export async function pickFolder(title: string): Promise<string | null> {
  return call<string | null>("pick_folder", { title });
}
export async function pickPdf(): Promise<string | null> {
  return call<string | null>("pick_pdf");
}
export function openPath(path: string) {
  call("open_path", { path }).catch((err) => log("error", "open", String(err)));
}
export function revealPath(path: string) {
  call("reveal_path", { path }).catch((err) => log("error", "open", String(err)));
}
export function openUrl(url: string) {
  call("open_url", { url }).catch((err) => log("error", "open", String(err)));
}

// ---- appearance ----------------------------------------------------------------------------------

export type Theme = "system" | "dark" | "light";
const readPref = (k: string) => {
  try {
    return localStorage.getItem(`sift.${k}`);
  } catch {
    return null;
  }
};
const writePref = (k: string, v: string) => {
  try {
    localStorage.setItem(`sift.${k}`, v);
  } catch {
    /* a window without storage keeps the choice for this session only */
  }
};
const savedTheme = readPref("theme");
export const [theme, setThemeSignal] = createSignal<Theme>(savedTheme === "dark" || savedTheme === "light" ? savedTheme : "system");
function applyTheme(t: Theme) {
  if (t === "system") delete document.documentElement.dataset.theme;
  else document.documentElement.dataset.theme = t;
}
export function setTheme(t: Theme) {
  setThemeSignal(t);
  writePref("theme", t);
  applyTheme(t);
}
applyTheme(theme());

/** Motion: `system` follows Windows' "Animation effects" (reduced, not removed, when they are
 * off), `on` plays every animation, `off` none. */
export type Motion = "system" | "on" | "off";
const savedMotion = readPref("motion");
export const [motion, setMotionSignal] = createSignal<Motion>(savedMotion === "on" || savedMotion === "off" ? savedMotion : "system");
function applyMotion(m: Motion) {
  if (m === "system") delete document.documentElement.dataset.motion;
  else document.documentElement.dataset.motion = m;
}
export function setMotion(m: Motion) {
  setMotionSignal(m);
  writePref("motion", m);
  applyMotion(m);
}
applyMotion(motion());
/** What Windows asks for right now (shown next to the Motion setting). */
export const systemReducesMotion = () => typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;

export const [collapsed, setCollapsedSignal] = createSignal(readPref("sidebar") === "collapsed");
export function setSidebarCollapsed(v: boolean) {
  setCollapsedSignal(v);
  writePref("sidebar", v ? "collapsed" : "open");
}
export const toggleSidebar = () => setSidebarCollapsed(!collapsed());

// ---- start-up --------------------------------------------------------------------------------------

export function start() {
  on<SiftEvent>("sift-event", onSiftEvent);
  on<DoneEvent>("sift-done", onSiftDone);
  on<{ message: string }>("engine-log", (e) => engineLine(e.message));
  on<{ done: number; total: number }>("engine-progress", (e) => setDownload(e));
  on<{ ok: boolean; error?: string; message?: string }>("engine-done", (e) => {
    setEngineBusy(false);
    setDownload(null);
    engineLine(e.ok ? e.message ?? "The engine is ready." : `Engine not changed: ${e.error}`);
    void refresh();
  });
  on<EngineCheck>("engine-check", (c) => setCheck(c));
  on<ModelProgress>("model-progress", (p) => {
    setModelProgress((m) => ({ ...m, [p.id]: p }));
  });
  on<{ id: string; ok: boolean; error?: string }>("model-done", (e) => {
    setModelProgress((m) => {
      const { [e.id]: _gone, ...rest } = m;
      return rest;
    });
    note(e.id, e.ok ? "downloaded and verified" : `download failed: ${e.error}`, e.ok ? "ok" : "error");
    log(e.ok ? "ok" : "error", "models", `${e.id}: ${e.ok ? "ready" : e.error}`);
    void refresh();
  });

  // a PDF or a folder dropped anywhere on the window goes to the Convert page
  if (inTauri) {
    void getCurrentWebview().onDragDropEvent((e) => {
      const p = e.payload;
      if (p.type === "enter" || p.type === "over") setDragOver(true);
      else setDragOver(false);
      if (p.type === "drop" && p.paths.length && !converting()) {
        void chooseInput(p.paths[0]);
        setPage("convert");
      }
    });
  }

  void refresh().then(() => {
    if (!ready()) setPage("setup");
  });
  if (import.meta.env.DEV && !inTauri) {
    void import("./mock").then((m) => m.runDemo({ chooseInput, startConvert, setPage, setSidebarCollapsed }));
  }
}
