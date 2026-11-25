import type { JSX } from "@solidjs/web";
import { createEffect, createSignal, For, Show } from "solid-js";
import { Icon } from "../components/Icon";
import { SiftFlow } from "../components/SiftFlow";
import { Callout, DocGlyph, Group, PageHead, Progress, Seg, Setting, Toggle } from "../components/ui";
import { clock, dirName, fileName, joinPath, plural, seconds, size, stem } from "../lib/format";
import {
  STAGES,
  cancelConvert,
  chooseInput,
  clearJob,
  converting,
  dragOver,
  input,
  inputError,
  inputInfo,
  job,
  openPath,
  pickFolder,
  pickPdf,
  ready,
  revealPath,
  runOptions,
  saveSetting,
  setPage,
  setRunOptions,
  settings,
  startConvert,
  type Job,
} from "../lib/store";

/* The Convert page is laid out as what happens: the PDF on top, the sieve under it (the pipeline's
   stages on its left, Convert on its right), the Markdown at the bottom. The options beside it are
   the saved defaults (the command line reads the same settings file); the "this run" ones are
   not saved. */

const STAGE_LABEL: Record<string, string> = {
  text: "Text layer",
  layout: "Layout",
  build: "Pages",
  surya: "Tables, formulas, scans",
  refine: "LLM pass",
  describe: "Image descriptions",
};

type StageState = "off" | "pending" | "active" | "done";

function stageState(j: Job | null, s: (typeof STAGES)[number], llm: boolean, describe: boolean): StageState {
  if ((s === "refine" && !llm) || (s === "describe" && !describe)) return "off";
  if (!j) return "pending";
  if (j.state === "done") return "done";
  const order = STAGES.filter((x) => (x === "refine" ? j.llm : x === "describe" ? j.describe : true));
  const i = order.indexOf(s);
  const furthest = order.reduce((acc, x, k) => (j.stages[x] ? k : acc), -1);
  const st = j.stages[s];
  // a later stage has started: this one is over (layout goes on while pages are built)
  if (i < furthest && !(s === "layout" && order[furthest] === "build")) return "done";
  if (st && st.total > 0 && st.done >= st.total) return "done";
  if (st && j.state === "running") return "active";
  return "pending";
}

function Stages(): JSX.Element {
  const llm = () => job()?.llm ?? settings()?.llm.enabled ?? true;
  const describe = () => job()?.describe ?? settings()?.convert.describer !== "off";
  return (
    <div class="stages" aria-label="Pipeline">
      <For each={STAGES}>
        {(s) => {
          const state = () => stageState(job(), s, llm(), describe());
          const count = () => job()?.stages[s];
          return (
            <div class={["stage-row", state()]}>
              <span class="mark">
                <Show when={state() === "done"} fallback={<Show when={state() === "active"} fallback={<span class="dot" />}><span class="spinner" /></Show>}>
                  <Icon name="check" size={13} />
                </Show>
              </span>
              <span class="ellipsis">{STAGE_LABEL[s]}</span>
              <Show when={state() === "off"} fallback={<Show when={count()}>{(c) => <span class="n">{`${c().done}/${c().total}`}</span>}</Show>}>
                <span class="n">off</span>
              </Show>
            </div>
          );
        }}
      </For>
    </div>
  );
}

function PdfCard(): JSX.Element {
  const choosePdf = async () => {
    const p = await pickPdf();
    if (p) await chooseInput(p);
  };
  const chooseFolder = async () => {
    const p = await pickFolder("A folder of PDFs");
    if (p) await chooseInput(p);
  };
  const info = () => inputInfo();
  return (
    <div class={["card drop", { empty: !input(), over: dragOver() }]}>
      <span class="card-tag">{info()?.kind === "folder" ? "Folder of PDFs" : "PDF"}</span>
      <Show
        when={input()}
        fallback={
          <div class="drop-empty">
            <DocGlyph kind="pdf" dim />
            <div class="hint">
              <b>Drop a PDF or a folder of PDFs here</b>
              <span class="muted small">or choose one. The file never leaves this computer.</span>
            </div>
            <button class="btn primary" onClick={choosePdf}>
              <Icon name="pdf" size={15} /> Choose a PDF
            </button>
            <button class="btn" onClick={chooseFolder}>
              <Icon name="folder" size={15} /> Folder
            </button>
          </div>
        }
      >
        <div class="drop-file">
          <DocGlyph kind={info()?.kind === "folder" ? "folder" : "pdf"} />
          <div class="meta">
            <div class="name ellipsis" title={input()}>
              {info()?.name ?? fileName(input())}
            </div>
            <div class="path ellipsis" title={info()?.dir ?? dirName(input())}>
              {info()?.dir ?? dirName(input())}
            </div>
            <div class="small">
              <Show when={inputError()} fallback={<span class="muted">{describeInput()}</span>}>
                <span class="error-text">{inputError()}</span>
              </Show>
            </div>
          </div>
          <Show when={!converting()}>
            <button class="btn sm" onClick={choosePdf}>
              PDF...
            </button>
            <button class="btn sm" onClick={chooseFolder}>
              Folder...
            </button>
            <button class="btn sm icon ghost" title="Clear" onClick={() => chooseInput("")}>
              <Icon name="x" size={14} />
            </button>
          </Show>
        </div>
      </Show>
    </div>
  );
}

function describeInput(): string {
  const i = inputInfo();
  if (!i) return "reading...";
  if (i.kind === "folder") {
    const top = i.pdfs ?? 0;
    const all = i.pdfs_all ?? top;
    return `${plural(top, "PDF")}${all > top ? ` (${all} with its subfolders)` : ""}`;
  }
  return [i.pages != null ? plural(i.pages, "page") : null, i.size != null ? size(i.size) : null].filter(Boolean).join(" · ");
}

function Sifter(): JSX.Element {
  const [now, setNow] = createSignal(Date.now());
  createEffect(
    () => converting(),
    (running) => {
      if (!running) return;
      const t = setInterval(() => setNow(Date.now()), 1000);
      return () => clearInterval(t);
    },
  );
  const elapsed = () => {
    const j = job();
    return j ? clock((j.ended ?? now()) - j.started) : "";
  };
  const canStart = () => !!input() && ready() && !inputError() && !converting();
  return (
    <div class="sifter">
      <Stages />
      <SiftFlow />
      <div class="action">
        <Show
          when={converting()}
          fallback={
            <>
              <button id="start-convert" class="btn primary lg" disabled={!canStart()} onClick={() => void startConvert()}>
                <Icon name="play" size={14} /> Convert
              </button>
              <span class="small muted">{!ready() ? "Finish the setup first" : !input() ? "Choose a PDF first" : describeInput()}</span>
            </>
          }
        >
          <button id="cancel-convert" class="btn danger" disabled={job()?.cancelling} onClick={() => void cancelConvert()}>
            <Icon name="stop" size={13} /> {job()?.cancelling ? "Cancelling..." : "Cancel"}
          </button>
          <span class="clock">{elapsed()}</span>
          <span class="small muted">
            {(job()?.files ?? 1) > 1 ? `file ${(job()?.fileIndex ?? 0) + 1} of ${job()?.files} · ` : ""}
            {Math.round((job()?.progress ?? 0) * 100)}%
          </span>
        </Show>
      </div>
    </div>
  );
}

/** Where the Markdown goes: the output folder of this run, or next to the PDF. */
function outputPlace(): string {
  const o = runOptions().output;
  const i = inputInfo();
  if (o) return i?.kind === "pdf" ? joinPath(o, stem(i.name)) : o;
  if (!i) return "Next to the PDF, in a folder of its name";
  return i.kind === "pdf" ? joinPath(i.dir, stem(i.name)) : i.path;
}

function Preview(props: { text: string }): JSX.Element {
  const lines = () => props.text.split("\n").slice(0, 60);
  return (
    <div class="md-sheet">
      <For each={lines()}>
        {(l) => <div class={/^#{1,6}\s/.test(l) ? "h" : l.startsWith(">") ? "q" : undefined}>{l || " "}</div>}
      </For>
    </div>
  );
}

const GHOST = [42, 88, 74, 92, 60, 85, 30, 78, 66, 90, 55, 81];

/** What Cancel left behind, true for one PDF or a folder: the PDF in progress is never written
 * (Runtime::check_cancel), files finished before it are complete. */
function cancelNote(j: Job): string {
  const written = j.results.filter((r) => !r.error).length;
  if (j.kind === "pdf") {
    return written
      ? "Cancelled after the Markdown was written: it is complete."
      : "Cancelled: nothing was written, and an earlier Markdown of this PDF is unchanged. The model servers have stopped.";
  }
  const kept = written ? `the ${plural(written, "file")} converted before it ${written === 1 ? "is" : "are"} complete; nothing else was written` : "nothing was written";
  return `Cancelled: ${kept}. The model servers have stopped.`;
}

function MarkdownCard(): JSX.Element {
  const j = () => job();
  const single = () => {
    const r = j()?.results ?? [];
    return j()?.kind === "pdf" && r.length === 1 && !r[0].error ? r[0] : null;
  };
  const chooseOutput = async () => {
    const p = await pickFolder("Where the Markdown goes");
    if (p) setRunOptions((o) => ({ ...o, output: p }));
  };
  const ghostCount = () => Math.max(1, Math.round((j()?.progress ?? 0) * GHOST.length));
  const stat = (k: string) => single()?.stats[k] ?? 0;
  return (
    <div class={["card result", { done: j()?.state === "done", failed: j()?.state === "failed" }]}>
      <span class="card-tag">Markdown</span>
      <div class="result-head">
        <DocGlyph kind="md" dim={!j()} />
        <div class="meta">
          <div class="name ellipsis">
            <Show when={single()} fallback={inputInfo()?.kind === "folder" ? "One Markdown file per PDF" : inputInfo() ? `${stem(inputInfo()!.name)}.md` : "Markdown"}>
              {(r) => fileName(r().md ?? "")}
            </Show>
          </div>
          <div class="path ellipsis" title={outputPlace()}>
            {single() ? dirName(single()!.md ?? "") : outputPlace()}
          </div>
          <div class="small muted">
            <Show when={j()?.state === "done" && j()} fallback={`with its images at ${settings()?.convert.image_dpi ?? 300} DPI${settings()?.convert.describer !== "off" ? ", their descriptions" : ""} and a report of anything Sift could not verify`}>
              {(d) => `Done in ${clock((d().ended ?? d().started) - d().started)}${d().results.length > 1 ? ` · ${plural(d().results.filter((r) => !r.error).length, "file")} converted` : ""}${d().skipped ? ` · ${d().skipped} already up to date` : ""}`}
            </Show>
          </div>
        </div>
        <Show
          when={j() && !converting()}
          fallback={
            <Show when={!converting()}>
              <button class="btn sm" onClick={chooseOutput}>
                Output folder...
              </button>
              <Show when={runOptions().output}>
                <button class="btn sm icon ghost" title="Back to: next to the PDF" onClick={() => setRunOptions((o) => ({ ...o, output: "" }))}>
                  <Icon name="undo" size={14} />
                </button>
              </Show>
            </Show>
          }
        >
          <button class="btn sm icon ghost" title="Clear the result" onClick={clearJob}>
            <Icon name="x" size={14} />
          </button>
        </Show>
      </div>

      <Show when={j()}>
        {(cur) => (
          <>
            <Show when={cur().state === "running"}>
              <div style={{ "margin-top": "12px" }}>
                <Progress value={cur().progress} max={1} />
              </div>
              <div class="md-sheet">
                <div class="ghost-lines">
                  <For each={GHOST.slice(0, ghostCount())}>
                    {(w, i) => <i class={{ h: i() === 0 || i() === 5, caret: i() === ghostCount() - 1 }} style={{ width: `${w}%` }} />}
                  </For>
                </div>
              </div>
            </Show>

            <Show when={cur().state === "failed" || cur().state === "cancelled"}>
              <div style={{ "margin-top": "12px" }}>
                <Callout kind={cur().state === "failed" ? "error" : "warn"}>
                  {cur().state === "cancelled" ? cancelNote(cur()) : cur().error ?? "The conversion stopped."}
                </Callout>
              </div>
            </Show>

            <Show when={single()}>
              {(r) => (
                <>
                  <div class="row" style={{ "margin-top": "12px" }}>
                    <button class="btn primary" onClick={() => openPath(r().md ?? "")}>
                      <Icon name="markdown" size={15} /> Open the Markdown
                    </button>
                    <button class="btn" onClick={() => revealPath(r().md ?? "")}>
                      <Icon name="folder" size={15} /> Show in folder
                    </button>
                    <Show when={r().report}>
                      <button class="btn" onClick={() => openPath(r().report ?? "")}>
                        <Icon name="report" size={15} /> Report
                      </button>
                    </Show>
                    <span class="spacer" />
                    <span class="small muted mono">{seconds(r().seconds)}</span>
                  </div>
                  <div class="box" style={{ "margin-top": "12px" }}>
                    <div class="stats four">
                      <div>
                        <div class="stat-label">Pages</div>
                        <div class="stat-value">{stat("pages")}</div>
                      </div>
                      <div>
                        <div class="stat-label">Tables</div>
                        <div class="stat-value">{stat("tables")}</div>
                      </div>
                      <div>
                        <div class="stat-label">Images</div>
                        <div class="stat-value">{r().images}</div>
                        <Show when={stat("described")}>
                          <div class="small muted mono">{stat("described")} described</div>
                        </Show>
                      </div>
                      <div>
                        <div class="stat-label">Values fixed</div>
                        <div class="stat-value" title="table values replaced by the exact value from the PDF's text">
                          {stat("table_values_corrected")}
                        </div>
                      </div>
                    </div>
                  </div>
                  <Show when={cur().preview}>
                    <Preview text={cur().preview} />
                  </Show>
                </>
              )}
            </Show>

            {/* one PDF's error or cancellation is the callout above; a folder lists its files */}
            <Show when={cur().kind === "folder" && cur().results.length > 0}>
              <div class="file-list">
                <For each={cur().results}>
                  {(r) => (
                    <div class="item">
                      <Icon name={r.error ? "alert" : "markdown"} size={14} class={r.cancelled ? "warn-text" : r.error ? "error-text" : "accent-text"} />
                      <span class="ellipsis" style={{ flex: "1" }} title={r.error ?? r.md ?? r.file}>
                        {r.md ? fileName(r.md) : fileName(r.file)}
                        <Show when={r.error}>
                          <span class={r.cancelled ? "warn-text" : "error-text"}> · {r.error}</span>
                        </Show>
                      </span>
                      <Show when={r.md}>
                        <span class="muted small mono">{seconds(r.seconds)}</span>
                        <button class="btn sm ghost icon" title="Open" onClick={() => openPath(r.md ?? "")}>
                          <Icon name="external" size={13} />
                        </button>
                        <button class="btn sm ghost icon" title="Show in folder" onClick={() => revealPath(r.md ?? "")}>
                          <Icon name="folder" size={13} />
                        </button>
                      </Show>
                    </div>
                  )}
                </For>
              </div>
            </Show>
          </>
        )}
      </Show>
    </div>
  );
}

function Options(): JSX.Element {
  const s = () => settings();
  const off = () => !s() || converting();
  return (
    <Group title="Options" aside={<span class="small muted">saved</span>}>
      <Setting title="Image descriptions" desc="Gemma describes every figure">
        <Toggle label="Image descriptions" disabled={off()} checked={s()?.convert.describer !== "off"} onChange={(v) => saveSetting("convert", "describer", v ? "gemma" : "off")} />
      </Setting>
      <Setting title="LLM pass" desc="Gemma checks heading levels, split tables and formulas">
        <Toggle label="LLM pass" disabled={off()} checked={s()?.llm.enabled ?? true} onChange={(v) => saveSetting("llm", "enabled", v)} />
      </Setting>
      <Setting title="Scanned PDF" desc="surya reads every page, ignoring the text layer">
        <Toggle label="Scanned PDF" disabled={off()} checked={s()?.convert.mode === "full-ocr"} onChange={(v) => saveSetting("convert", "mode", v ? "full-ocr" : "hybrid")} />
      </Setting>
      <Setting title="Keep page headers" desc="Running headers and footers stay">
        <Toggle label="Keep page headers" disabled={off()} checked={s()?.convert.keep_headers ?? false} onChange={(v) => saveSetting("convert", "keep_headers", v)} />
      </Setting>
      <Setting title="Tables" desc={s()?.convert.tables === "markdown" ? "Always pipe tables" : "HTML only where cells merge"}>
        <Seg
          label="Tables"
          disabled={off()}
          value={s()?.convert.tables === "markdown" ? "markdown" : "auto"}
          options={[
            { value: "auto", label: "Auto" },
            { value: "markdown", label: "Pipe" },
          ]}
          onChange={(v) => saveSetting("convert", "tables", v)}
        />
      </Setting>
      <Setting title="Image resolution" desc="DPI of the extracted figures">
        <select class="input" disabled={off()} onChange={(e) => saveSetting("convert", "image_dpi", Number(e.currentTarget.value))}>
          <For each={[150, 200, 300, 400, 600]}>
            {(d) => (
              <option value={String(d)} selected={(s()?.convert.image_dpi ?? 300) === d}>
                {d}
              </option>
            )}
          </For>
        </select>
      </Setting>
    </Group>
  );
}

function ThisRun(): JSX.Element {
  const folder = () => inputInfo()?.kind === "folder";
  return (
    <Group title="This run">
      <Setting title="Pages" desc="e.g. 1-5, 137 (empty: all)">
        <input
          class="input mono"
          style={{ width: "96px" }}
          aria-label="Pages"
          placeholder="all"
          disabled={converting()}
          value={runOptions().pages}
          onInput={(e) => {
            const v = e.currentTarget.value;
            setRunOptions((o) => ({ ...o, pages: v }));
          }}
        />
      </Setting>
      <Setting title="Subfolders" desc="Mirrored in the output" disabled={!folder()}>
        <Toggle label="Subfolders" disabled={!folder() || converting()} checked={runOptions().recursive} onChange={(v) => setRunOptions((o) => ({ ...o, recursive: v }))} />
      </Setting>
      <Setting title="Convert again" desc="Even Markdown that is up to date" disabled={!folder()}>
        <Toggle label="Convert again" disabled={!folder() || converting()} checked={runOptions().force} onChange={(v) => setRunOptions((o) => ({ ...o, force: v }))} />
      </Setting>
    </Group>
  );
}

export default function Convert(): JSX.Element {
  return (
    <div class="page">
      <PageHead title="Convert" desc="PDFs to Markdown, with their images, descriptions and a report. Everything runs on this computer." />
      <Show when={!ready()}>
        <Callout
          kind="warn"
          action={
            <button class="btn sm primary" onClick={() => setPage("setup")}>
              Finish the setup
            </button>
          }
        >
          Sift needs llama.cpp and its three models before it can convert.
        </Callout>
      </Show>
      <div class="convert">
        <div class={["flow", { running: converting(), "done-flow": job()?.state === "done" }]}>
          <PdfCard />
          <Sifter />
          <MarkdownCard />
        </div>
        <aside class="stack" style={{ gap: "22px" }}>
          <Options />
          <ThisRun />
        </aside>
      </div>
    </div>
  );
}
