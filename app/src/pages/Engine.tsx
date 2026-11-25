import type { JSX } from "@solidjs/web";
import { For, Show } from "solid-js";
import { Icon } from "../components/Icon";
import { Callout, ConfirmButton, FolderSetting, Group, PageHead, Progress, ServerDot, Setting, Toggle } from "../components/ui";
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
  saveSetting,
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
            {([id, desc]) => (
              <Setting title={<span class="row" style={{ gap: "8px" }}><ServerDot state={servers()[id]} />{id}</span>} desc={desc}>
                <span class="badge">{servers()[id] ?? "stopped"}</span>
              </Setting>
            )}
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
