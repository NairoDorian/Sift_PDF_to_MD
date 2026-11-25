import type { JSX } from "@solidjs/web";
import { createSignal, For, Show } from "solid-js";
import { Icon } from "../components/Icon";
import { PageHead, Seg } from "../components/ui";
import { logs, setLogs, type LogEntry } from "../lib/store";

/* This session's events (docs/events.md), readable: time, kind, message. Progress counts of one
   stage replace each other, so a long run stays short here. */

type Filter = "all" | "progress" | "servers" | "problems";
const keep: Record<Filter, (l: LogEntry) => boolean> = {
  all: () => true,
  progress: (l) => l.level === "progress" || l.kind === "file" || l.kind === "done" || l.kind === "job",
  servers: (l) => l.level === "server" || l.kind === "engine",
  problems: (l) => l.level === "warn" || l.level === "error",
};

export default function Logs(): JSX.Element {
  const [filter, setFilter] = createSignal<Filter>("all");
  const [copied, setCopied] = createSignal(false);
  const shown = () => logs().filter(keep[filter()]);
  const copy = async () => {
    await navigator.clipboard.writeText(shown().map((l) => `${l.time}  ${l.kind}  ${l.text}`).join("\n"));
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };
  return (
    <div class="page">
      <PageHead title="Logs" desc="This session's events. Each run also keeps its trace and the model servers' logs in its output folder, under .sift\logs." />
      <section class="group">
        <div class="group-head">
          <Seg<Filter>
            label="Show"
            value={filter()}
            options={[
              { value: "all", label: "All" },
              { value: "progress", label: "Progress" },
              { value: "servers", label: "Servers" },
              { value: "problems", label: "Problems" },
            ]}
            onChange={setFilter}
          />
          <div class="aside">
            <span class="small muted mono">{shown().length} lines</span>
            <button class="btn sm" disabled={!shown().length} onClick={() => void copy()}>
              <Icon name={copied() ? "check" : "copy"} size={13} /> {copied() ? "Copied" : "Copy"}
            </button>
            <button class="btn sm ghost" disabled={!logs().length} onClick={() => setLogs([])}>
              <Icon name="trash" size={13} /> Clear
            </button>
          </div>
        </div>
        <div class="box log-box">
          <Show when={shown().length} fallback={<div class="log-line"><span /><span /><span class="m muted">Nothing yet.</span></div>}>
            <For each={shown()}>
              {(l) => (
                <div class={["log-line", l.level === "progress" ? "info" : l.level]}>
                  <span class="t">{l.time}</span>
                  <span class="k ellipsis">{l.kind.replace(/^stage:/, "")}</span>
                  <span class="m">{l.text}</span>
                </div>
              )}
            </For>
          </Show>
        </div>
      </section>
    </div>
  );
}
