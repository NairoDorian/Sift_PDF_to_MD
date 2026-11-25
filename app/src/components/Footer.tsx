import type { JSX } from "@solidjs/web";
import { Show } from "solid-js";
import { fileName } from "../lib/format";
import { converting, engine, job, meters, overview, ready, servers, setPage, updateAvailable } from "../lib/store";
import { ServerDot } from "./ui";

/* ZER0's footer: what runs now. The two model servers, the conversion's stage, the machine's
   load (CPU, RAM, GPU, VRAM), the active llama.cpp build (and its update), the version. */

const gb = (mb: number) => `${(mb / 1024).toFixed(1)} GB`;
const share = (used: number | null, total: number | null) => (used != null && total ? (used / total) * 100 : null);

/** One meter (ZER0's): a label, a short bar (amber, then warning from 75 %, red from 90 %), the
 * percent. Props are read inside the bindings: the bar follows every sample. */
function Meter(props: { label: string; value: number; title: string }): JSX.Element {
  const pct = () => Math.max(0, Math.min(100, props.value));
  return (
    <span class="meter" title={props.title}>
      <span class="meter-label">{props.label}</span>
      <span class="meter-bar">
        <i class={{ warn: pct() >= 75 && pct() < 90, hot: pct() >= 90 }} style={{ width: `${pct()}%` }} />
      </span>
      <span class="meter-value">{Math.round(pct())}%</span>
    </span>
  );
}

function Meters(): JSX.Element {
  return (
    <Show when={meters()}>
      {(m) => (
        <span class="meters">
          <Show when={m().cpu != null}>
            <Meter label="CPU" value={m().cpu ?? 0} title={`Processor: ${Math.round(m().cpu ?? 0)} % (all cores)`} />
          </Show>
          <Show when={share(m().ram_used_mb, m().ram_total_mb) != null}>
            <Meter label="RAM" value={share(m().ram_used_mb, m().ram_total_mb) ?? 0} title={`Memory: ${gb(m().ram_used_mb ?? 0)} of ${gb(m().ram_total_mb ?? 0)}`} />
          </Show>
          <Show when={m().gpu != null}>
            <Meter label="GPU" value={m().gpu ?? 0} title={`${m().gpu_name ?? "GPU"}: ${Math.round(m().gpu ?? 0)} % busy`} />
          </Show>
          <Show when={share(m().vram_used_mb, m().vram_total_mb) != null}>
            <Meter
              label="VRAM"
              value={share(m().vram_used_mb, m().vram_total_mb) ?? 0}
              title={`Video memory: ${gb(m().vram_used_mb ?? 0)} of ${gb(m().vram_total_mb ?? 0)} (surya and Gemma load here)`}
            />
          </Show>
        </span>
      )}
    </Show>
  );
}

const STAGE_WORD: Record<string, string> = {
  text: "reading the text layer",
  layout: "finding the layout",
  build: "building pages",
  surya: "surya: tables, formulas, scans",
  refine: "Gemma: the LLM pass",
  describe: "Gemma: describing images",
};

export function Footer(): JSX.Element {
  const status = () => {
    const j = job();
    if (converting() && j) {
      const st = j.stages[j.stage];
      const what = STAGE_WORD[j.stage] ?? "starting";
      // the stage first: when the footer is short of room, the file's name is what gets cut
      return `${what}${st ? ` ${st.done}/${st.total}` : ""} · ${Math.round(j.progress * 100)}% · ${fileName(j.file || j.input)}`;
    }
    if (!overview()) return "";
    return ready() ? "Ready" : "Setup needed";
  };
  return (
    <footer class="status">
      <span class="row" style={{ gap: "6px" }} title="model servers: each starts with its first job and stops when idle">
        <ServerDot state={servers().surya} />
        surya
      </span>
      <span class="row" style={{ gap: "6px" }}>
        <ServerDot state={servers().gemma} />
        gemma
      </span>
      <span class="sep">|</span>
      <span class="grow">{status()}</span>
      <Meters />
      <Show when={updateAvailable()}>
        <span class="sep">|</span>
        <button class="link" onClick={() => setPage("engine")}>
          update available
        </button>
      </Show>
      {/* short of room, these go first (styles.css): the version, then the build (both on About
          and Engine), then the meters; the servers and the stage always stay */}
      <span class="sep build">|</span>
      <span class="mono build">llama.cpp {engine()?.active ?? "not installed"}</span>
      <span class="sep version">|</span>
      <span class="mono version">v{overview()?.version ?? ""}</span>
    </footer>
  );
}
