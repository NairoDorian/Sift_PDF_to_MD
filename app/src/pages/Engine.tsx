import type { JSX } from "@solidjs/web";
import { createEffect, createSignal, For, Show } from "solid-js";
import { Icon } from "../components/Icon";
import { Callout, ConfirmButton, FolderSetting, Group, PageHead, Progress, ServerDot, Setting, Toggle } from "../components/ui";
import type { ServerCommand } from "../lib/api";
import { mb } from "../lib/format";
import {
  check,
  checkEngine,
  checking,
  download,
  engine,
  engineAction,
  engineBusy,
  engineLog,
  installEngine,
  lastOutput,
  openPath,
  saveSetting,
  serverLines,
  serverLog,
  servers,
  settings,
} from "../lib/store";

/* The llama.cpp engine (ZER0's LlamaSettings page): official ggml-org builds only. GitHub's
   release list is read when the user presses Check (or at start-up, when that is turned on);
   nothing downloads before Install or Update. */

const SERVER_INFO: [string, string][] = [
  ["surya", "surya 2.5: starts with the first table, formula or scanned page"],
  ["gemma", "Gemma 4 E4B: starts with the first description or LLM-pass request"],
];

/** One model server: its state, and on demand the command Sift starts it with (read-only) and the
 * end of its log (refreshed every 2 s while it runs and the panel is open). */
function ServerRow(props: { id: string; desc: string; command?: ServerCommand; commandError: string }): JSX.Element {
  const [open, setOpen] = createSignal<"" | "command" | "log">("");
  const [log, setLog] = createSignal<{ file: string | null; lines: string[] } | null>(null);
  const [copied, setCopied] = createSignal(false);
  const state = () => servers()[props.id];
  const live = () => state() === "starting" || state() === "ready";
  const load = async () => {
    try {
      setLog(await serverLog(props.id));
    } catch {
      setLog({ file: null, lines: [] });
    }
  };
  createEffect(
    () => [open(), live()] as const,
    ([panel, running]) => {
      if (panel !== "log") return;
      void load();
      if (!running) return;
      const t = setInterval(() => void load(), 2000);
      return () => clearInterval(t);
    },
  );
  const commandText = () =>
    (props.command?.args ?? [])
      .map((a, i, all) => (i === all.length - 1 && a === "0" ? "<free port>" : /\s/.test(a) ? `"${a}"` : a))
      .join(" ");
  const copy = async () => {
    await navigator.clipboard.writeText(commandText());
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };
  const toggle = (p: "command" | "log") => setOpen(open() === p ? "" : p);
  // a log reads from the bottom: keep its newest lines in view after each refresh
  let logBox: HTMLPreElement | undefined;
  createEffect(
    () => log(),
    () => {
      if (logBox) logBox.scrollTop = logBox.scrollHeight;
    },
  );
  return (
    <div>
      <Setting
        title={
          <span class="row" style={{ gap: "8px" }}>
            <ServerDot state={state()} />
            {props.id}
          </span>
        }
        desc={props.desc}
      >
        <button class="btn sm ghost" aria-pressed={open() === "command" ? "true" : "false"} onClick={() => toggle("command")}>
          Command
        </button>
        <button class="btn sm ghost" aria-pressed={open() === "log" ? "true" : "false"} onClick={() => toggle("log")}>
          Log
        </button>
        <span class="badge">{state() ?? "stopped"}</span>
      </Setting>
      <Show when={open() === "command"}>
        <div class="server-panel">
          <Show when={props.command} fallback={<p class="small muted">Not available: {props.commandError || "the engine or the models are not installed"}.</p>}>
            <pre class="console">{commandText()}</pre>
            <div class="row">
              <span class="small muted">How Sift starts it, with the settings and the build in use; read-only.</span>
              <span class="spacer" />
              <button class="btn sm" onClick={() => void copy()}>
                <Icon name={copied() ? "check" : "copy"} size={13} /> {copied() ? "Copied" : "Copy"}
              </button>
            </div>
          </Show>
        </div>
      </Show>
      <Show when={open() === "log"}>
        <div class="server-panel">
          <pre class="console" ref={(el) => (logBox = el)}>
            {log()?.lines.length ? log()!.lines.join("\n") : lastOutput() ? "No log for this server in the latest run." : "No server has run in this session yet."}
          </pre>
          <Show when={log()?.file}>
            <div class="row">
              <span class="path ellipsis" style={{ flex: "1" }} title={log()?.file ?? ""}>
                {log()?.file}
              </span>
              <button class="btn sm ghost icon" title="Open the log" onClick={() => openPath(log()?.file ?? "")}>
                <Icon name="external" size={13} />
              </button>
            </div>
          </Show>
        </div>
      </Show>
    </div>
  );
}

export default function Engine(): JSX.Element {
  const e = () => engine();
  const backends = () => {
    const b = ["cuda-13.5", "vulkan", "cpu", "auto"];
    const newer = check()?.newer_cuda;
    const cur = e()?.backend;
    return [...new Set([...(newer ? [newer] : []), ...b, ...(cur ? [cur] : [])])];
  };
  return (
    <div class="page">
      <PageHead title="Engine" desc="Official llama.cpp builds from ggml-org, never a patched one. Sift reads GitHub's release list only when you press Check (or at start-up if you turn that on); nothing downloads before you press Install or Update." />

      <div class="box">
        <div class="stats">
          <div>
            <div class="stat-label">Active build</div>
            <div class="stat-value">{e()?.active ?? "none"}</div>
          </div>
          <div>
            <div class="stat-label">Version</div>
            <div class="stat-value ellipsis" title={e()?.version ?? ""}>
              {e()?.version?.replace(/^version:\s*/, "") ?? "-"}
            </div>
          </div>
          <div>
            <div class="stat-label">Backend</div>
            <div class="stat-value">{e()?.backend ?? "-"}</div>
          </div>
          <div>
            <div class="stat-label">GPU (detected)</div>
            <div class="stat-value">
              {e()?.detected ?? "-"}
              <Show when={e()?.driver_cuda}>
                <span class="muted small"> · CUDA {e()?.driver_cuda}</span>
              </Show>
            </div>
          </div>
        </div>
      </div>

      <Group
        title="Updates"
        aside={
          <button class="btn sm" disabled={checking()} onClick={() => void checkEngine()}>
            <Show when={checking()} fallback={<Icon name="refresh" size={13} />}>
              <span class="spinner" />
            </Show>
            Check now
          </button>
        }
      >
        <Setting
          title="Newest official build"
          desc={
            <Show when={check()} fallback={checking() ? "Reading the release list..." : "Not checked yet."}>
              {(c) => (
                <span>
                  <span class="mono">{c().newest}</span> · {c().published} · {mb(c().size)} ·{" "}
                  <span class={c().update_available ? "accent-text" : "ok-text"}>{c().update_available ? "update available" : "up to date"}</span>
                </span>
              )}
            </Show>
          }
        >
          <Show when={!e()?.active || check()?.update_available}>
            <button class="btn primary" disabled={engineBusy() || !e() || !settings()?.folders.engine_dir} onClick={() => void installEngine()}>
              <Icon name="download" size={14} /> {e()?.active ? "Update" : "Install"}
            </button>
          </Show>
        </Setting>
        <Show when={check()?.newer_cuda}>
          <div class="setting compact">
            <span class="small warn-text">Official builds now also use {check()?.newer_cuda}: choose it as the backend below to move to it (Sift never switches on its own).</span>
          </div>
        </Show>
        <Show when={download()}>
          {(d) => (
            <div class="setting stacked" style={{ display: "flex", "flex-direction": "column", "align-items": "stretch", gap: "6px" }}>
              <div class="row small">
                <span>Downloading</span>
                <span class="spacer" />
                <span class="mono muted">
                  {mb(d().done)} / {mb(d().total)}
                </span>
              </div>
              <Progress value={d().done} max={d().total} />
            </div>
          )}
        </Show>
        <Setting title="Check when Sift starts" desc="Shows a dot on the Engine menu when a newer build exists">
          <Toggle label="Check when Sift starts" checked={settings()?.engine.check_at_startup ?? false} onChange={(v) => saveSetting("engine", "check_at_startup", v)} />
        </Setting>
      </Group>

      <div class="two-col">
        <Group title="Build">
          <Setting title="Backend" desc={`Detected: ${e()?.detected ?? "-"}${e()?.driver_cuda ? `; the NVIDIA driver supports CUDA ${e()?.driver_cuda}` : ""}`}>
            <select class="input" onChange={(x) => saveSetting("engine", "backend", x.currentTarget.value)}>
              <For each={backends()}>
                {(b) => (
                  <option value={b} selected={b === (e()?.backend ?? "cuda-13.5")}>
                    {b}
                  </option>
                )}
              </For>
            </select>
          </Setting>
          <Setting
            title="CUDA runtime"
            desc={e()?.toolkit ? `From the installed toolkit (${e()?.toolkit})` : "No CUDA toolkit found: the runtime package comes with the build"}
          >
            <span class="small muted">always download</span>
            <Toggle label="Always download the CUDA runtime" checked={e()?.include_cudart === "always"} onChange={(v) => saveSetting("engine", "include_cudart", v ? "always" : "auto")} />
          </Setting>
          <FolderSetting kind="engine" title="Engine folder" />
        </Group>

        <Group title="Model servers">
          <For each={SERVER_INFO}>
            {([id, desc]) => <ServerRow id={id} desc={desc} command={serverLines().list.find((c) => c.model === id)} commandError={serverLines().error} />}
          </For>
          <div class="setting compact">
            <span class="small muted">A server stops as soon as it has no work, and every server stops with Sift.</span>
          </div>
        </Group>
      </div>

      <Group title="Installed builds" aside={<span class="small muted">{e()?.builds.length ?? 0} in the engine folder</span>}>
        <Show when={e()?.builds.length} fallback={<div class="setting"><span class="muted">No build installed yet.</span></div>}>
          <table class="list">
            <thead>
              <tr>
                <th>Build</th>
                <th>Size</th>
                <th>CUDA runtime inside</th>
                <th />
                <th />
              </tr>
            </thead>
            <tbody>
              <For each={e()?.builds ?? []}>
                {(b) => (
                  <tr>
                    <td class="mono">{b.name}</td>
                    <td class="mono">{b.size_mb} MB</td>
                    <td class="mono">
                      {b.cuda_runtime_mb ? `${b.cuda_runtime_mb} MB` : "-"}
                      <Show when={b.cuda_runtime_mb && e()?.toolkit}>
                        <button class="btn sm ghost" style={{ "margin-left": "6px" }} disabled={engineBusy()} onClick={() => void engineAction("remove_runtime", b.name)}>
                          Remove
                        </button>
                      </Show>
                    </td>
                    <td>
                      <Show when={b.active} fallback={<Show when={b.previous}><span class="badge">previous</span></Show>}>
                        <span class="badge ok">active</span>
                      </Show>
                    </td>
                    <td class="actions">
                      <Show when={!b.active}>
                        <button class="btn sm" disabled={engineBusy()} onClick={() => void engineAction("use", b.name)}>
                          Use
                        </button>
                      </Show>
                      <Show when={b.previous}>
                        <button class="btn sm" disabled={engineBusy()} onClick={() => void engineAction("rollback")}>
                          Roll back
                        </button>
                      </Show>
                      <Show when={!b.active && !b.previous}>
                        <ConfirmButton label="Delete" confirm="Delete this build?" disabled={engineBusy()} onConfirm={() => void engineAction("remove", b.name)} />
                      </Show>
                    </td>
                  </tr>
                )}
              </For>
            </tbody>
          </table>
        </Show>
      </Group>

      <Show when={engineLog().length}>
        <Group title="Activity">
          <pre class="console">{engineLog().join("\n")}</pre>
        </Group>
      </Show>
      <Show when={!e()?.active && !settings()?.folders.engine_dir}>
        <Callout kind="warn">Choose an engine folder first: Sift installs llama.cpp only where you say.</Callout>
      </Show>
    </div>
  );
}
