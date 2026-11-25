import type { JSX } from "@solidjs/web";
import { createSignal, For, onSettled, Show } from "solid-js";
import { BrandMark } from "../components/Brand";
import { Icon } from "../components/Icon";
import { Group, PageHead, Setting } from "../components/ui";
import { call, type AboutInfo } from "../lib/api";
import { engine, openUrl } from "../lib/store";

const REPO = "https://github.com/NairoDorian/Sift_PDF_to_MD";

const CREDITS: [string, string, string][] = [
  ["surya 2.5 and its layout model", "Datalab", "code ported under Apache-2.0; weights under Datalab's modified OpenRAIL-M"],
  ["Gemma 4 E4B", "Google (Unsloth's QAT GGUF)", "Gemma terms of use"],
  ["llama.cpp", "ggml-org", "MIT; official release builds, downloaded, never bundled"],
  ["marker, pdftext, RF-DETR", "Datalab, Roboflow", "Apache-2.0 (rules and code ported to Rust)"],
  ["pdfium", "The PDFium Authors; built by pdfium-binaries (Benoit Blanchon)", "BSD-3-Clause and MIT; its third-party parts keep their own licenses"],
  ["ZER0 / Handy", "CJ Pais", "MIT (the app's structure, the Job Object, the llama.cpp manager)"],
  ["Pillow's LANCZOS resampling", "Secret Labs AB, F. Lundh, J. A. Clark", "MIT-CMU"],
];

export default function About(): JSX.Element {
  const [info, setInfo] = createSignal<AboutInfo | null>(null);
  onSettled(() => {
    call<AboutInfo>("about_info").then(setInfo, () => setInfo(null));
  });
  return (
    <div class="page">
      <PageHead title="About" />
      <div class="panel hero">
        <BrandMark size={64} />
        <div class="stack" style={{ gap: "4px" }}>
          <div class="row" style={{ gap: "12px", "align-items": "baseline" }}>
            <span class="wordmark">Sift</span>
            <span class="badge accent">{info()?.version ?? ""}</span>
          </div>
          <span class="muted">PDFs to Markdown you can trust: every value checked against the PDF itself, every image kept and described, everything local. No account, no API key, no telemetry.</span>
        </div>
        <span class="spacer" />
        <button class="btn" onClick={() => openUrl(REPO)}>
          <Icon name="external" size={14} /> Repository
        </button>
      </div>

      <div class="box">
        <div class="stats">
          <For each={info()?.gpus ?? []} fallback={<div><div class="stat-label">GPU</div><div class="stat-value">none found</div></div>}>
            {(g) => (
              <div>
                <div class="stat-label">GPU</div>
                <div class="stat-value ellipsis" title={`${g.name} · driver ${g.driver}`}>
                  {g.name.replace(/^NVIDIA (GeForce )?/, "")}
                </div>
                <div class="small muted mono">
                  {Math.round((g.total_mib - g.used_mib) / 102.4) / 10} of {Math.round(g.total_mib / 102.4) / 10} GB free
                </div>
              </div>
            )}
          </For>
          <div>
            <div class="stat-label">CUDA (driver)</div>
            <div class="stat-value">{info()?.driver_cuda ?? "none"}</div>
          </div>
          <div>
            <div class="stat-label">CPU threads</div>
            <div class="stat-value">{info()?.cpu_threads ?? "-"}</div>
          </div>
          <div>
            <div class="stat-label">llama.cpp</div>
            <div class="stat-value">{engine()?.active ?? "not installed"}</div>
          </div>
        </div>
      </div>

      <Group title="Command line" aside={<Show when={info()?.exe_dir}><span class="small muted">sift.exe is in <span class="path">{info()?.exe_dir}</span></span></Show>}>
        <pre class="kbd-block">
          {`sift convert file.pdf -o out       the same converter, without the window
sift convert folder -o out -r      a folder, its subfolders mirrored
sift engine check | update         llama.cpp builds
sift models status                 the three models
sift config show                   the settings this window writes`}
        </pre>
      </Group>

      <Group title="Credits and licenses" aside={<span class="small muted">Sift itself: Apache-2.0</span>}>
        <For each={CREDITS}>
          {([what, who, terms]) => (
            <Setting compact title={what} desc={`${who} · ${terms}`} />
          )}
        </For>
      </Group>
      <Show when={info()?.settings}>
        <p class="small muted">
          Settings: <span class="path">{info()?.settings}</span>
        </p>
      </Show>
    </div>
  );
}
