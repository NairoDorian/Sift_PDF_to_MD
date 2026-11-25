import type { JSX } from "@solidjs/web";
import { Show } from "solid-js";
import { fileName } from "../lib/format";
import { converting, engine, job, overview, ready, servers, setPage, updateAvailable } from "../lib/store";
import { ServerDot } from "./ui";

/* ZER0's footer: what runs now. The two model servers, the conversion's stage, the active
   llama.cpp build (and its update), the version. */

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
      return `${fileName(j.file || j.input)} · ${what}${st ? ` ${st.done}/${st.total}` : ""} · ${Math.round(j.progress * 100)}%`;
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
      <Show when={updateAvailable()}>
        <button class="link" onClick={() => setPage("engine")}>
          update available
        </button>
        <span class="sep">|</span>
      </Show>
      <span class="mono">llama.cpp {engine()?.active ?? "not installed"}</span>
      <span class="sep">|</span>
      <span class="mono">v{overview()?.version ?? ""}</span>
    </footer>
  );
}
