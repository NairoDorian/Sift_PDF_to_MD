import type { JSX } from "@solidjs/web";
import { For, Show } from "solid-js";
import { Icon } from "../components/Icon";
import { ConfirmButton, FolderSetting, Group, PageHead, Progress } from "../components/ui";
import { gb } from "../lib/format";
import {
  checkRevisions,
  checkingRevisions,
  deleteModel,
  downloadModel,
  modelNote,
  modelProgress,
  models,
  overview,
  revisions,
  verifyModel,
} from "../lib/store";

/* The three pinned models (catalog/*.toml): each from its own Hugging Face repository at a tested
   revision, every file checked against its sha256. */

export default function Models(): JSX.Element {
  const total = () => models().reduce((a, m) => a + m.size, 0);
  const missing = () => models().filter((m) => !m.installed);
  const downloading = () => Object.keys(modelProgress()).length > 0;
  return (
    <div class="page">
      <PageHead title="Models" desc="Each model comes from its own Hugging Face repository at a pinned, tested revision, and every file is checked against its sha256. Using a model means accepting its license." />

      <Group title="Storage" aside={<span class="small muted mono">{gb(total())} for the three</span>}>
        <FolderSetting kind="models" title="Models folder" />
      </Group>

      <section class="group">
        <div class="group-head">
          <h2>The models</h2>
          <div class="aside">
            <button class="btn sm" disabled={checkingRevisions()} onClick={() => void checkRevisions()}>
              <Show when={checkingRevisions()} fallback={<Icon name="refresh" size={13} />}>
                <span class="spinner" />
              </Show>
              Check for new revisions
            </button>
            <Show when={missing().length}>
              <button class="btn sm primary" disabled={!overview()?.models_dir || downloading()} onClick={() => void downloadModel("all")}>
                <Icon name="download" size={13} /> Download {missing().length === models().length ? "all" : "the missing"} ({gb(missing().reduce((a, m) => a + m.size, 0))})
              </button>
            </Show>
          </div>
        </div>
        <div class="cards">
          <For each={models()} keyed={(m) => m.id}>
            {(m) => {
              const p = () => modelProgress()[m().id];
              const n = () => modelNote()[m().id];
              return (
                <div class="box model-card">
                  <div class="top">
                    <div class="what">
                      <div class="name">{m().name}</div>
                      <div class="small muted">{m().role}</div>
                    </div>
                    <Show when={p()} fallback={<span class={["badge", m().installed ? "ok" : "warn"]}>{m().installed ? "installed" : "missing"}</span>}>
                      <span class="badge accent">downloading</span>
                    </Show>
                  </div>
                  <div class="facts">
                    <span>
                      size <b>{gb(m().size)}</b>
                    </span>
                    <span>
                      revision <b>{m().revision.slice(0, 8)}</b>
                    </span>
                    <span>{m().license}</span>
                  </div>
                  <Show when={p()}>
                    {(x) => (
                      <div class="stack" style={{ gap: "4px" }}>
                        <Progress value={x().done} max={x().total} />
                        <span class="small muted mono">
                          {x().phase} {x().file.split("/").pop()} · {gb(x().done)} / {gb(x().total)}
                        </span>
                      </div>
                    )}
                  </Show>
                  <div class="row">
                    <Show
                      when={m().installed}
                      fallback={
                        <button class="btn sm primary" disabled={!overview()?.models_dir || !!p()} onClick={() => void downloadModel(m().id)}>
                          <Icon name="download" size={13} /> Download
                        </button>
                      }
                    >
                      <button class="btn sm" onClick={() => void verifyModel(m().id)}>
                        <Icon name="verify" size={13} /> Verify
                      </button>
                      <ConfirmButton label="Delete" confirm={`Delete ${gb(m().size)}?`} onConfirm={() => void deleteModel(m().id)} />
                    </Show>
                  </div>
                  <Show when={n() || revisions()[m().id] || (!m().installed && m().missing.length)}>
                    <div class="small">
                      <Show when={n()}>{(x) => <div class={x().level === "error" ? "error-text" : x().level === "ok" ? "ok-text" : "muted"}>{x().text}</div>}</Show>
                      <Show when={revisions()[m().id]}>
                        <div class="muted">{revisions()[m().id]}</div>
                      </Show>
                      <Show when={!m().installed && !n() && m().missing.length}>
                        <div class="muted">missing: {m().missing.join(", ")}</div>
                      </Show>
                    </div>
                  </Show>
                </div>
              );
            }}
          </For>
        </div>
      </section>
    </div>
  );
}
