import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

/* Sift's window (MASTER_PLAN section 9), on ZER0's layout: a sidebar of pages, the page in the
   middle, a status footer. Pages: Start (first run), Convert, Engine, Models, Settings, Logs,
   About. Every option is a flag of `sift convert`; the work runs in the Rust core and the window
   shows its events (docs/events.md). The engine and the models live only in the folders the user
   picks here; nothing is downloaded before that. */

type SiftEvent = { type: string; file?: string; name?: string; done?: number; total?: number; message?: string; md?: string; report?: string; model?: string; state?: string; seconds?: number; error?: string };
type Build = { name: string; dir: string; size_mb: number; cuda_runtime_mb: number; active: boolean; previous: boolean };
type EngineStatus = {
  active: string | null; previous: string | null; version: string | null; dir: string | null; backend: string; detected: string;
  driver_cuda: string | null; toolkit: string | null; include_cudart: string; check_at_startup: boolean; builds: Build[];
};
type Check = { active: string | null; newest: string; backend: string; size: number; published: string; update_available: boolean; newer_cuda: string | null };
type ModelStatus = { id: string; name: string; role: string; license: string; revision: string; folder: string | null; size: number; missing: string[]; installed: boolean };
type Overview = { settings_file: string; portable: boolean; engine_dir: string; models_dir: string; engine_active: string | null; models: ModelStatus[]; check_at_startup: boolean };
type Settings = Record<string, any>;
type Page = "start" | "convert" | "engine" | "models" | "settings" | "logs" | "about";

const [page, setPage] = createSignal<Page>("convert");
const [log, setLog] = createSignal<string[]>([]);
const [events, setEvents] = createSignal<string[]>([]);
const [stage, setStage] = createSignal("");
const [busy, setBusy] = createSignal(false);
const [servers, setServers] = createSignal<Record<string, string>>({});
const [ov, setOv] = createSignal<Overview | null>(null);
const [engine, setEngine] = createSignal<EngineStatus | null>(null);
const [check, setCheck] = createSignal<Check | null>(null);
const [checking, setChecking] = createSignal(false);
const [engineLog, setEngineLog] = createSignal<string[]>([]);
const [dlBytes, setDlBytes] = createSignal<{ done: number; total: number } | null>(null);
const [models, setModels] = createSignal<ModelStatus[]>([]);
const [modelMsg, setModelMsg] = createSignal<Record<string, string>>({});
const [settings, setSettings] = createSignal<Settings | null>(null);

const push = (s: string) => setLog((l) => [...l.slice(-300), s]);
const ev = (s: string) => setEvents((l) => [...l.slice(-2000), `${new Date().toLocaleTimeString()}  ${s}`]);
const gb = (b: number) => `${(b / 1e9).toFixed(2)} GB`;
const mb = (b: number) => `${(b / 1e6).toFixed(0)} MB`;
const updateDot = () => !!check()?.update_available;
const ready = () => {
  const o = ov();
  return !!o && !!o.engine_dir && !!o.models_dir && !!o.engine_active && o.models.every((m) => m.installed);
};

listen<SiftEvent>("sift-event", (e) => {
  const d = e.payload;
  ev(JSON.stringify(d));
  if (d.type === "stage") setStage(`${d.name} ${d.done}/${d.total}`);
  else if (d.type === "server") setServers((s) => ({ ...s, [d.model ?? "?"]: d.state ?? "" }));
  else if (d.type === "file_start") push(`Converting ${d.file ?? ""}`);
  else if (d.type === "file_done") push(`Done: ${d.md} (${(d.seconds ?? 0).toFixed(1)} s)${d.report ? `\n  report: ${d.report}` : ""}`);
  else if (d.type === "file_error") push(`FAILED ${d.file}: ${d.error}`);
  else if (d.type === "warning" || d.type === "log") push(d.message ?? "");
});
listen<{ ok: boolean; cancelled?: boolean; error?: string }>("sift-done", (e) => {
  setBusy(false);
  setStage("");
  push(e.payload.cancelled ? "Cancelled." : e.payload.ok ? "Finished." : `Stopped: ${e.payload.error ?? "a file failed (see above)"}`);
});
listen<{ message: string }>("engine-log", (e) => setEngineLog((l) => [...l, e.payload.message]));
listen<{ done: number; total: number }>("engine-progress", (e) => setDlBytes(e.payload));
listen<{ ok: boolean; error?: string; message?: string }>("engine-done", (e) => {
  setBusy(false);
  setDlBytes(null);
  setEngineLog((l) => [...l, e.payload.ok ? e.payload.message ?? "Engine ready." : `Engine not changed: ${e.payload.error}`]);
  refresh();
});
listen<Check>("engine-check", (e) => setCheck(e.payload));
listen<{ id: string; file: string; phase: string; done: number; total: number }>("model-progress", (e) =>
  setModelMsg((d) => ({ ...d, [e.payload.id]: `${e.payload.phase} ${e.payload.file.split("/").pop()} ${gb(e.payload.done)} / ${gb(e.payload.total)}` })),
);
listen<{ id: string; ok: boolean; error?: string }>("model-done", (e) => {
  setModelMsg((d) => ({ ...d, [e.payload.id]: e.payload.ok ? "ready" : `failed: ${e.payload.error}` }));
  refresh();
});

async function refresh() {
  try {
    const o = await invoke<Overview>("overview");
    setOv(o);
    setModels(o.models);
    setEngine(await invoke<EngineStatus>("engine_status"));
    setSettings(await invoke<Settings>("settings_get"));
  } catch (err) {
    ev(`refresh: ${err}`);
  }
}
async function saveSettings(s: Settings) {
  setSettings(s);
  try {
    await invoke("settings_save", { settings: s });
  } catch (err) {
    ev(`settings_save: ${err}`);
  }
  refresh();
}
const setKey = (section: string, key: string, value: unknown) => {
  const s = { ...(settings() ?? {}) };
  s[section] = { ...(s[section] ?? {}), [key]: value };
  saveSettings(s);
};
async function checkEngine() {
  setChecking(true);
  try {
    setCheck(await invoke<Check>("engine_check"));
  } catch (err) {
    setEngineLog((l) => [...l, `Check failed: ${err}`]);
  }
  setChecking(false);
}
async function installEngine(tag?: string) {
  setBusy(true);
  setEngineLog([]);
  try {
    await invoke("engine_install", { tag: tag ?? null });
  } catch (err) {
    setBusy(false);
    setEngineLog([String(err)]);
  }
}
async function engineAction(action: string, build?: string) {
  setBusy(true);
  try {
    await invoke("engine_action", { action, build: build ?? null });
  } catch (err) {
    setBusy(false);
    setEngineLog((l) => [...l, String(err)]);
  }
}
async function downloadModel(id: string) {
  setModelMsg((d) => ({ ...d, [id]: "starting" }));
  try {
    await invoke("model_download", { id });
  } catch (err) {
    setModelMsg((d) => ({ ...d, [id]: String(err) }));
  }
}
refresh().then(() => {
  if (!ready()) setPage("start");
});

/** A folder setting with Choose / Open, and Move or Use-as-is when it changes. */
function FolderPicker(props: { kind: "engine" | "models"; label: string }) {
  const current = () => (props.kind === "engine" ? ov()?.engine_dir : ov()?.models_dir) ?? "";
  const [pending, setPending] = createSignal<string | null>(null);
  const [msg, setMsg] = createSignal("");
  const choose = async () => {
    const p = await invoke<string | null>("pick_folder", { title: props.label });
    if (!p) return;
    if (current()) setPending(p);
    else apply(p, false);
  };
  const apply = async (path: string, moveFiles: boolean) => {
    setPending(null);
    try {
      const r = await invoke<{ moved: string[]; old: string }>("set_folder", { kind: props.kind, path, moveFiles });
      setMsg(r.moved.length ? `Moved ${r.moved.length} item(s) from ${r.old}. The old folder was kept.` : "");
    } catch (err) {
      setMsg(String(err));
    }
    refresh();
  };
  return (
    <div class="folder">
      <div class="row">
        <b>{props.label}</b>
        <code>{current() || "not chosen yet"}</code>
        <button onClick={choose}>{current() ? "Change..." : "Choose..."}</button>
        {current() ? <button onClick={() => invoke("open_folder", { path: current() })}>Open</button> : ""}
      </div>
      {pending() ? (
        <div class="row">
          <span>New folder: <code>{pending()}</code></span>
          <button onClick={() => apply(pending()!, true)}>Move the files there</button>
          <button onClick={() => apply(pending()!, false)}>Use it as it is</button>
          <button onClick={() => setPending(null)}>Cancel</button>
        </div>
      ) : ""}
      <p class="muted">{msg()}</p>
    </div>
  );
}

function Start() {
  const total = () => models().reduce((a, m) => a + m.size, 0);
  const step = (done: boolean, text: string) => <span class={done ? "ok" : "warn"}>{done ? "done" : "to do"}: {text}</span>;
  return (
    <section>
      <h1>Set up Sift</h1>
      <p class="muted">Sift runs two models on your GPU through official llama.cpp: surya 2.5 (tables, formulas, scanned text) and Gemma 4 E4B (image descriptions, heading levels). Choose where they go; nothing is downloaded before you do.</p>
      <h2>1. Engine folder</h2>
      <FolderPicker kind="engine" label="llama.cpp engine folder" />
      <h2>2. Models folder</h2>
      <p class="muted">About {gb(total())} for surya 2.5, its layout model and Gemma 4 E4B.</p>
      <FolderPicker kind="models" label="Models folder" />
      <h2>3. Updates</h2>
      <label class="check"><input type="checkbox" checked={settings()?.engine?.check_at_startup ?? false} onChange={(e) => setKey("engine", "check_at_startup", e.currentTarget.checked)} />Check for new llama.cpp builds when Sift starts (reads GitHub's release list; never downloads on its own)</label>
      <h2>4. Install llama.cpp</h2>
      <div class="row">
        {step(!!ov()?.engine_active, ov()?.engine_active ?? "official CUDA 13.5 build")}
        <button disabled={busy() || !ov()?.engine_dir} onClick={() => installEngine()}>Install llama.cpp</button>
        {dlBytes() ? <span>{mb(dlBytes()!.done)} / {mb(dlBytes()!.total)}</span> : ""}
      </div>
      <h2>5. Download the models</h2>
      <div class="row">
        {step(models().every((m) => m.installed), `${models().filter((m) => m.installed).length} of ${models().length} ready`)}
        <button disabled={!ov()?.models_dir || models().every((m) => m.installed)} onClick={() => downloadModel("all")}>Download all ({gb(total())})</button>
      </div>
      {models().map((m) => <div class="muted">{m.name}: {m.installed ? "ready" : modelMsg()[m.id] ?? "missing"}</div>)}
      <pre class="log">{engineLog().join("\n")}</pre>
      <div class="row"><button disabled={!ready()} onClick={() => setPage("convert")}>Start converting</button></div>
    </section>
  );
}

function Convert() {
  const [input, setInput] = createSignal("");
  const [output, setOutput] = createSignal("");
  const [pages, setPages] = createSignal("");
  const [noLlm, setNoLlm] = createSignal(false);
  const [describe, setDescribe] = createSignal(true);
  const [recursive, setRecursive] = createSignal(false);
  const [force, setForce] = createSignal(false);
  const [fullOcr, setFullOcr] = createSignal(false);
  const [pipeTables, setPipeTables] = createSignal(false);
  const [keepHeaders, setKeepHeaders] = createSignal(false);
  const start = async () => {
    if (!input().trim()) return;
    setBusy(true);
    setLog([]);
    try {
      await invoke("start_convert", {
        req: {
          input: input(), output: output() || null, pages: pages() || null, no_llm: noLlm(), describer: describe() ? "gemma" : "off",
          recursive: recursive(), force: force(), mode: fullOcr() ? "full-ocr" : null, tables: pipeTables() ? "markdown" : null, keep_headers: keepHeaders(),
        },
      });
    } catch (err) {
      setBusy(false);
      push(String(err));
    }
  };
  const cancel = async () => {
    if (await invoke<boolean>("cancel_convert")) push("Cancelling: queued work stops, then the models...");
  };
  const browse = async (set: (s: string) => void, title: string) => {
    const p = await invoke<string | null>("pick_folder", { title });
    if (p) set(p);
  };
  return (
    <section>
      <h1>Convert</h1>
      {ready() ? "" : <p class="warn">Not set up yet: <a href="#" onClick={(e) => { e.preventDefault(); setPage("start"); }}>finish the setup</a>.</p>}
      <label>PDF or folder<span class="row"><input class="grow" value={input()} onInput={(e) => setInput(e.currentTarget.value)} placeholder="C:\\docs\\datasheet.pdf" /><button onClick={() => browse(setInput, "Folder of PDFs")}>Folder...</button></span></label>
      <label>Output folder<span class="row"><input class="grow" value={output()} onInput={(e) => setOutput(e.currentTarget.value)} placeholder="next to the input" /><button onClick={() => browse(setOutput, "Output folder")}>Choose...</button></span></label>
      <label>Pages<input value={pages()} onInput={(e) => setPages(e.currentTarget.value)} placeholder="all (e.g. 1-5,137)" /></label>
      <div class="row">
        <label class="check"><input type="checkbox" checked={describe()} onChange={(e) => setDescribe(e.currentTarget.checked)} />Image descriptions</label>
        <label class="check"><input type="checkbox" checked={!noLlm()} onChange={(e) => setNoLlm(!e.currentTarget.checked)} />LLM pass</label>
        <label class="check"><input type="checkbox" checked={recursive()} onChange={(e) => setRecursive(e.currentTarget.checked)} />Subfolders</label>
        <label class="check"><input type="checkbox" checked={force()} onChange={(e) => setForce(e.currentTarget.checked)} />Convert again</label>
      </div>
      <div class="row">
        <label class="check"><input type="checkbox" checked={fullOcr()} onChange={(e) => setFullOcr(e.currentTarget.checked)} />surya reads every page (ignore the text layer)</label>
        <label class="check"><input type="checkbox" checked={pipeTables()} onChange={(e) => setPipeTables(e.currentTarget.checked)} />Always pipe tables</label>
        <label class="check"><input type="checkbox" checked={keepHeaders()} onChange={(e) => setKeepHeaders(e.currentTarget.checked)} />Keep page headers</label>
      </div>
      <div class="row">
        <button disabled={busy()} onClick={start}>{busy() ? "Converting..." : "Convert"}</button>
        <button disabled={!busy()} onClick={cancel}>Cancel</button>
      </div>
      <pre class="log">{log().join("\n")}</pre>
    </section>
  );
}

function Engine() {
  checkEngine();
  const e = () => engine();
  return (
    <section>
      <h1>Engine (llama.cpp)</h1>
      <p class="muted">Official ggml-org/llama.cpp builds only. A check reads GitHub's release list; nothing is downloaded until you press Update.</p>
      <table class="kv">
        <tbody>
          <tr><th>Active build</th><td>{e()?.active ?? "none"} {e()?.version ? <span class="muted">({e()!.version})</span> : ""}</td></tr>
          <tr>
            <th>Newest build</th>
            <td>
              {checking() ? "checking..." : check() ? `${check()!.newest} · ${check()!.published} · ${mb(check()!.size)} · ${check()!.update_available ? "update available" : "up to date"}` : "-"}
              <button class="mini" disabled={checking()} onClick={checkEngine}>Check for updates</button>
              {check()?.update_available ? <button class="mini" disabled={busy() || !e()?.dir} onClick={() => installEngine()}>{e()?.active ? "Update" : "Install"}</button> : ""}
            </td>
          </tr>
          {check()?.newer_cuda ? <tr><th></th><td class="warn">Official builds now also use {check()!.newer_cuda}: switch the backend below to move to it.</td></tr> : ""}
          {dlBytes() ? <tr><th>Downloading</th><td><progress max={dlBytes()!.total || 1} value={dlBytes()!.done} /> {mb(dlBytes()!.done)} / {mb(dlBytes()!.total)}</td></tr> : ""}
          <tr>
            <th>Backend</th>
            <td>
              <select value={e()?.backend ?? "cuda-13.5"} onChange={(x) => setKey("engine", "backend", x.currentTarget.value)}>
                {["cuda-13.5", ...(check()?.newer_cuda ? [check()!.newer_cuda!] : []), "vulkan", "cpu", "auto"].map((b) => <option value={b}>{b}</option>)}
              </select>
              <span class="muted"> detected: {e()?.detected}{e()?.driver_cuda ? `, the NVIDIA driver supports CUDA ${e()!.driver_cuda}` : ""}</span>
            </td>
          </tr>
          <tr>
            <th>CUDA runtime</th>
            <td>
              {e()?.toolkit ? `from the installed toolkit (${e()!.toolkit})` : "no CUDA toolkit found: the runtime package is downloaded with the build"}
              <label class="check"><input type="checkbox" checked={e()?.include_cudart === "always"} onChange={(x) => setKey("engine", "include_cudart", x.currentTarget.checked ? "always" : "auto")} />always download the runtime package</label>
            </td>
          </tr>
          <tr><th>At start-up</th><td><label class="check"><input type="checkbox" checked={e()?.check_at_startup ?? false} onChange={(x) => setKey("engine", "check_at_startup", x.currentTarget.checked)} />check for new builds when Sift starts</label></td></tr>
        </tbody>
      </table>
      <FolderPicker kind="engine" label="Engine folder" />
      <h2>Installed builds</h2>
      <table class="grid">
        <thead><tr><th>Build</th><th>Size</th><th>CUDA runtime inside</th><th></th><th></th></tr></thead>
        <tbody>
          {(e()?.builds ?? []).map((b) => (
            <tr>
              <td>{b.name}</td>
              <td>{b.size_mb} MB</td>
              <td>{b.cuda_runtime_mb ? `${b.cuda_runtime_mb} MB` : "-"}{b.cuda_runtime_mb && e()?.toolkit ? <button class="mini" disabled={busy()} onClick={() => engineAction("remove_runtime", b.name)}>Remove</button> : ""}</td>
              <td>{b.active ? <span class="ok">active</span> : b.previous ? <span class="muted">previous</span> : ""}</td>
              <td>
                {b.active ? "" : <button class="mini" disabled={busy()} onClick={() => engineAction("use", b.name)}>Use</button>}
                {b.previous ? <button class="mini" disabled={busy()} onClick={() => engineAction("rollback")}>Roll back</button> : ""}
                {b.active || b.previous ? "" : <button class="mini" disabled={busy()} onClick={() => engineAction("remove", b.name)}>Delete</button>}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      {(e()?.builds ?? []).length ? "" : <p class="muted">No build installed yet.</p>}
      <h2>Servers</h2>
      <p>{Object.entries(servers()).map(([m, s]) => `${m}: ${s}`).join(", ") || "none running (each model starts with its first job and stops when idle)"}</p>
      <pre class="log">{engineLog().join("\n")}</pre>
    </section>
  );
}

function Models() {
  const [checks, setChecks] = createSignal<Record<string, string>>({});
  const checkRevisions = async () => {
    try {
      const r = await invoke<{ id: string; pinned: string; upstream: string | null }[]>("models_check");
      setChecks(Object.fromEntries(r.map((x) => [x.id, x.upstream == null ? "could not check" : x.upstream === x.pinned ? "pinned revision is the newest" : `newer upstream: ${x.upstream.slice(0, 8)} (adopting it needs the golden test)`])));
    } catch (err) {
      ev(`models_check: ${err}`);
    }
  };
  const verify = async (id: string) => {
    setModelMsg((d) => ({ ...d, [id]: "verifying..." }));
    try {
      const r = await invoke<{ file: string; ok: boolean }[]>("model_verify", { id });
      const bad = r.filter((x) => !x.ok).map((x) => x.file);
      setModelMsg((d) => ({ ...d, [id]: bad.length ? `missing or damaged: ${bad.join(", ")}` : "every file verified (sha256)" }));
    } catch (err) {
      setModelMsg((d) => ({ ...d, [id]: String(err) }));
    }
  };
  return (
    <section>
      <h1>Models</h1>
      <p class="muted">Each model comes from its own Hugging Face repository at a pinned, tested revision and is checked against its sha256. Using a model implies accepting its license.</p>
      <FolderPicker kind="models" label="Models folder" />
      <div class="row"><button onClick={checkRevisions}>Check for new revisions</button></div>
      <div class="cards">
        {models().map((m) => (
          <div class="card">
            <div class="card-head"><b>{m.name}</b><span class="muted">{gb(m.size)}</span></div>
            <div class="muted">{m.role}</div>
            <div class="muted">revision {m.revision.slice(0, 8)} | {m.license}</div>
            <div class="row">
              {m.installed ? <span class="ok">installed</span> : <button disabled={!ov()?.models_dir} onClick={() => downloadModel(m.id)}>Download</button>}
              {m.installed ? <button onClick={() => verify(m.id)}>Verify</button> : ""}
              {m.installed ? <button onClick={async () => { await invoke("model_delete", { id: m.id }); refresh(); }}>Delete</button> : ""}
            </div>
            <div class="muted">{modelMsg()[m.id] ?? (m.installed ? "" : `missing: ${m.missing.join(", ")}`)}</div>
            <div class="muted">{checks()[m.id] ?? ""}</div>
          </div>
        ))}
      </div>
    </section>
  );
}

function SettingsPage() {
  const s = () => settings() ?? {};
  return (
    <section>
      <h1>Settings</h1>
      <p class="muted">Saved in {ov()?.settings_file}{ov()?.portable ? " (portable mode)" : ""}; the command line uses the same file.</p>
      <FolderPicker kind="engine" label="Engine folder" />
      <FolderPicker kind="models" label="Models folder" />
      <h2>Conversion defaults</h2>
      <label>Image DPI<input type="number" value={s().convert?.image_dpi ?? 300} onChange={(e) => setKey("convert", "image_dpi", Number(e.currentTarget.value))} /></label>
      <div class="row">
        <label class="check"><input type="checkbox" checked={s().convert?.describer !== "off"} onChange={(e) => setKey("convert", "describer", e.currentTarget.checked ? "gemma" : "off")} />Image descriptions</label>
        <label class="check"><input type="checkbox" checked={s().llm?.enabled ?? true} onChange={(e) => setKey("llm", "enabled", e.currentTarget.checked)} />LLM pass</label>
        <label class="check"><input type="checkbox" checked={s().convert?.keep_headers ?? false} onChange={(e) => setKey("convert", "keep_headers", e.currentTarget.checked)} />Keep page headers</label>
      </div>
      <h2>Updates</h2>
      <label class="check"><input type="checkbox" checked={s().engine?.check_at_startup ?? false} onChange={(e) => setKey("engine", "check_at_startup", e.currentTarget.checked)} />Check for new llama.cpp builds when Sift starts</label>
    </section>
  );
}

function Logs() {
  return (
    <section>
      <h1>Logs</h1>
      <p class="muted">Every event of this session (docs/events.md). Each run's trace and the model servers' logs are in the output folder's .sift/logs.</p>
      <div class="row"><button onClick={() => setEvents([])}>Clear</button></div>
      <pre class="log tall">{events().join("\n")}</pre>
    </section>
  );
}

function About() {
  const [a, setA] = createSignal<any>(null);
  invoke("about_info").then(setA);
  return (
    <section>
      <h1>About Sift</h1>
      <p>Sift {a()?.version}: PDFs to Markdown with their images, image descriptions and a report, every value checked against the PDF itself. surya 2.5 + Gemma 4 E4B on official llama.cpp. Local only: no account, no API key, no telemetry.</p>
      <table class="kv">
        <tbody>
          {(a()?.gpus ?? []).map((g: any) => <tr><th>GPU</th><td>{g.name}: {g.total_mib} MiB, {g.total_mib - g.used_mib} MiB free (driver {g.driver})</td></tr>)}
          <tr><th>CUDA (driver)</th><td>{a()?.driver_cuda ?? "none"}</td></tr>
          <tr><th>CPU threads</th><td>{a()?.cpu_threads ?? "-"}</td></tr>
          <tr><th>Engine</th><td>{engine()?.active ?? "none"}</td></tr>
          <tr><th>Settings</th><td>{a()?.settings}</td></tr>
        </tbody>
      </table>
      <h2>Command line</h2>
      <p class="muted">The same binary runs without its window: <code>sift-app convert file.pdf -o out</code>, <code>sift-app engine check</code>, <code>sift-app models status</code>.</p>
      <h2>Licenses</h2>
      <p class="muted">Sift: Apache-2.0. Ported code keeps its origin's terms (surya, marker, rf-detr, pdftext: Apache-2.0; Pillow's resampling: MIT-CMU; ZER0 / Handy: MIT). Model weights come from their own repositories under their own licenses (surya 2.5 and its layout model: Datalab's modified OpenRAIL-M; Gemma: Gemma terms).</p>
    </section>
  );
}

const PAGES: [Page, string][] = [["start", "Start"], ["convert", "Convert"], ["engine", "Engine"], ["models", "Models"], ["settings", "Settings"], ["logs", "Logs"], ["about", "About"]];

export default function App() {
  const body = () => {
    switch (page()) {
      case "start": return <Start />;
      case "engine": return <Engine />;
      case "models": return <Models />;
      case "settings": return <SettingsPage />;
      case "logs": return <Logs />;
      case "about": return <About />;
      default: return <Convert />;
    }
  };
  return (
    <div class="shell">
      <nav>
        <div class="brand">Sift</div>
        {PAGES.map(([p, label]) => (
          <button class={page() === p ? "active" : ""} onClick={() => { setPage(p); refresh(); }}>
            {label}{p === "engine" && updateDot() ? <span class="dot" title="a newer llama.cpp build is available" /> : ""}
          </button>
        ))}
      </nav>
      <main>{body()}</main>
      <footer>
        <span>{Object.entries(servers()).map(([m, s]) => `${m}: ${s}`).join("   ") || "no model loaded"}</span>
        <span>{stage()}</span>
        <span>llama.cpp {engine()?.active ?? "not installed"}{updateDot() ? " · update available" : ""}</span>
      </footer>
    </div>
  );
}
