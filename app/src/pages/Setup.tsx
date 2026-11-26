import type { JSX } from "@solidjs/web";
import { For, Show } from "solid-js";
import { Icon } from "../components/Icon";
import { Callout, FolderSetting, PageHead, Progress, Setting, Toggle } from "../components/ui";
import { gb, mb } from "../lib/format";
import {
  download,
  downloadModel,
  engineBusy,
  engineLog,
  installEngine,
  modelNote,
  modelProgress,
  models,
  overview,
  ready,
  saveSetting,
  setPage,
  settings,
  setupSteps,
} from "../lib/store";

/* First run (MASTER_PLAN section 9): the two folders, llama.cpp, the models. Nothing is downloaded
   before the user chooses where it goes and presses the button. Shown in the menu until every
   step is done; afterwards each step lives on its own page (Engine, Models). */

function Step(props: { n: number; title: string; done: boolean; current: boolean; children: JSX.Element }): JSX.Element {
  return (
    <div class={["step", { done: props.done, current: props.current }]}>
      <span class="num">{props.done ? <Icon name="check" size={14} /> : props.n}</span>
      <div class="what">
        <div class="title">{props.title}</div>
        {props.children}
      </div>
    </div>
  );
}

export default function Setup(): JSX.Element {
  const steps = () => setupSteps();
  const current = () => steps().findIndex((d) => !d);
  const total = () => models().reduce((a, m) => a + m.size, 0);
  const missing = () => models().filter((m) => !m.installed);
  return (
    <div class="page">
      <PageHead
        title="Set up Sift"
        desc="Sift runs two models on your GPU through official llama.cpp builds: surya 2.5 reads tables, formulas and scans; Gemma 4 E4B describes images and checks the result. You choose where they go; nothing downloads before you press a button."
      />
      <Show when={ready()}>
        <Callout
          action={
            <button class="btn primary" onClick={() => setPage("convert")}>
              <Icon name="play" size={14} /> Start converting
            </button>
          }
        >
          Everything is in place. You can change any of it later on the Engine and Models pages.
        </Callout>
      </Show>
      <div class="box steps">
        <Step n={1} title="Where llama.cpp goes" done={steps()[0]} current={current() === 0}>
          <div class="box">
            <FolderSetting kind="engine" title="Engine folder" desc="One subfolder per installed build (about 0.6 GB each)." />
          </div>
        </Step>
        <Step n={2} title="Where the models go" done={steps()[1]} current={current() === 1}>
          <div class="box">
            <FolderSetting kind="models" title="Models folder" desc={`surya 2.5, its layout model and Gemma 4 E4B: about ${gb(total())}.`} />
          </div>
        </Step>
        <Step n={3} title="Install llama.cpp" done={steps()[2]} current={current() === 2}>
          <div class="row">
            <Show
              when={overview()?.engine_active}
              fallback={
                <button class="btn primary" disabled={engineBusy() || !overview()?.engine_dir} onClick={() => void installEngine()}>
                  <Icon name="download" size={14} /> Install the newest official build
                </button>
              }
            >
              <span class="badge ok">installed</span>
              <span class="mono small">{overview()?.engine_active}</span>
            </Show>
            <Show when={download()}>
              {(d) => (
                <span class="small mono muted">
                  {mb(d().done)} / {mb(d().total)}
                </span>
              )}
            </Show>
          </div>
          <Show when={download()}>{(d) => <Progress value={d().done} max={d().total} />}</Show>
          <Show when={engineLog().length && !overview()?.engine_active}>
            <pre class="console" style={{ "max-height": "120px", "min-height": "0" }}>
              {engineLog().join("\n")}
            </pre>
          </Show>
          <div class="box">
            <Setting compact title="Check for new builds when Sift starts" desc="Reads GitHub's release list; never downloads on its own.">
              <Toggle
                label="Check for new builds when Sift starts"
                checked={settings()?.engine.check_at_startup ?? false}
                onChange={(v) => saveSetting("engine", "check_at_startup", v)}
              />
            </Setting>
          </div>
        </Step>
        <Step n={4} title="Download the models" done={steps()[3]} current={current() === 3}>
          <div class="row">
            <Show when={missing().length} fallback={<span class="badge ok">all three ready</span>}>
              <button class="btn primary" disabled={!overview()?.models_dir || Object.keys(modelProgress()).length > 0} onClick={() => void downloadModel("all")}>
                <Icon name="download" size={14} /> Download {missing().length === models().length ? "all three" : `the ${missing().length} missing`} ({gb(missing().reduce((a, m) => a + m.size, 0))})
              </button>
            </Show>
          </div>
          <div class="stack">
            <For each={models()} keyed={(m) => m.id}>
              {(m) => {
                const p = () => modelProgress()[m().id];
                return (
                  <div class="stack" style={{ gap: "4px" }}>
                    <div class="row small">
                      <span class={["dot", { ok: m().installed, accent: !!p(), live: !!p() }]} />
                      <span>{m().name}</span>
                      <span class="spacer" />
                      <span class="muted mono">
                        {p() ? `${gb(p()!.done)} / ${gb(p()!.total)}` : m().installed ? "ready" : modelNote()[m().id]?.text ?? gb(m().size)}
                      </span>
                    </div>
                    <Show when={p()}>{(x) => <Progress value={x().done} max={x().total} />}</Show>
                  </div>
                );
              }}
            </For>
          </div>
        </Step>
      </div>
    </div>
  );
}
