import type { JSX } from "@solidjs/web";
import { createSignal, For, Show } from "solid-js";
import { call } from "../lib/api";
import { openPath, overview, pickFolder, refresh } from "../lib/store";
import { Icon } from "./Icon";

/* The building blocks of every page, after ZER0's components/ui (SettingsGroup, SettingContainer,
   ToggleSwitch, Button): a titled box of rows, each row a title, a description and a control. */

export function PageHead(props: { title: string; desc?: string; children?: JSX.Element }): JSX.Element {
  return (
    <header class="page-head">
      <div>
        <h1>{props.title}</h1>
        <Show when={props.desc}>
          <p>{props.desc}</p>
        </Show>
      </div>
      <div class="row">{props.children}</div>
    </header>
  );
}

export function Group(props: { title: string; aside?: JSX.Element; children: JSX.Element }): JSX.Element {
  return (
    <section class="group">
      <div class="group-head">
        <h2>{props.title}</h2>
        <div class="aside">{props.aside}</div>
      </div>
      <div class="box">{props.children}</div>
    </section>
  );
}

export function Setting(props: {
  title: JSX.Element;
  desc?: JSX.Element;
  children?: JSX.Element;
  disabled?: boolean;
  compact?: boolean;
}): JSX.Element {
  return (
    <div class={["setting", { disabled: props.disabled, compact: props.compact }]}>
      <div class="what">
        <div class="title">{props.title}</div>
        <Show when={props.desc}>
          <div class="desc">{props.desc}</div>
        </Show>
      </div>
      <div class="control">{props.children}</div>
    </div>
  );
}

export function Toggle(props: { checked: boolean; onChange: (v: boolean) => void; label: string; disabled?: boolean }): JSX.Element {
  return (
    <label class="switch">
      <input
        type="checkbox"
        role="switch"
        aria-label={props.label}
        checked={props.checked}
        disabled={props.disabled}
        onChange={(e) => props.onChange(e.currentTarget.checked)}
      />
      <span class="track" />
    </label>
  );
}

export function Seg<T extends string>(props: {
  value: T;
  options: readonly { value: T; label: string; title?: string }[];
  onChange: (v: T) => void;
  label: string;
  disabled?: boolean;
}): JSX.Element {
  return (
    <div class="seg" role="group" aria-label={props.label}>
      <For each={props.options}>
        {(o) => (
          <button
            type="button"
            title={o.title}
            disabled={props.disabled}
            aria-pressed={props.value === o.value ? "true" : "false"}
            onClick={() => props.onChange(o.value)}
          >
            {o.label}
          </button>
        )}
      </For>
    </div>
  );
}

export function Progress(props: { value?: number | null; max?: number }): JSX.Element {
  const pct = () => (props.value == null ? null : Math.max(0, Math.min(100, (props.value / (props.max || 1)) * 100)));
  return (
    <div class={["progress", { indeterminate: pct() == null }]} role="progressbar" aria-valuemin={0} aria-valuemax={100} aria-valuenow={pct() ?? undefined}>
      <i style={{ width: `${pct() ?? 30}%` }} />
    </div>
  );
}

export function Callout(props: { kind?: "info" | "warn" | "error"; children: JSX.Element; action?: JSX.Element }): JSX.Element {
  return (
    <div class={["callout", props.kind === "warn" ? "warn" : props.kind === "error" ? "error" : undefined]}>
      <div class="what">{props.children}</div>
      {props.action}
    </div>
  );
}

/** A destructive button that asks once, inline (no dialog): the first click arms it for a few
 * seconds, the second one acts. */
export function ConfirmButton(props: { label: string; confirm: string; onConfirm: () => void; disabled?: boolean }): JSX.Element {
  const [armed, setArmed] = createSignal(false);
  let timer: ReturnType<typeof setTimeout> | undefined;
  const click = () => {
    if (!armed()) {
      setArmed(true);
      timer = setTimeout(() => setArmed(false), 4000);
      return;
    }
    clearTimeout(timer);
    setArmed(false);
    props.onConfirm();
  };
  return (
    <button class={["btn sm danger", { armed: armed() }]} disabled={props.disabled} onClick={click} onBlur={() => setArmed(false)}>
      <Icon name={armed() ? "alert" : "trash"} size={13} /> {armed() ? props.confirm : props.label}
    </button>
  );
}

/** A model server's state as a dot: ready, starting (pulsing) or stopped. */
export function ServerDot(props: { state?: string }): JSX.Element {
  return <span class={["dot", { ok: props.state === "ready", accent: props.state === "starting", live: props.state === "starting" }]} />;
}

/** The engine or models folder: where it is, Change (with "move the files there" when one was
 * already chosen) and Open. The one control for each folder, on its own page (Engine, Models)
 * and in the first-run steps. */
export function FolderSetting(props: { kind: "engine" | "models"; title: string; desc?: string }): JSX.Element {
  const current = () => (props.kind === "engine" ? overview()?.engine_dir : overview()?.models_dir) ?? "";
  const [pending, setPending] = createSignal<string | null>(null);
  const [message, setMessage] = createSignal("");
  const apply = async (path: string, moveFiles: boolean) => {
    setPending(null);
    try {
      const r = await call<{ moved: string[]; old: string }>("set_folder", { kind: props.kind, path, moveFiles });
      setMessage(r.moved.length ? `Moved ${r.moved.length} item(s) from ${r.old}; the old folder was kept.` : "");
    } catch (err) {
      setMessage(String(err));
    }
    void refresh();
  };
  const choose = async () => {
    const p = await pickFolder(props.title);
    if (!p) return;
    if (current()) setPending(p);
    else await apply(p, false);
  };
  return (
    <>
      <Setting
        title={props.title}
        desc={
          <>
            <Show when={props.desc}>
              <div>{props.desc}</div>
            </Show>
            <div class="path">{current() || "Not chosen yet"}</div>
          </>
        }
      >
        <button class={["btn sm", { primary: !current() }]} onClick={choose}>
          {current() ? "Change..." : "Choose..."}
        </button>
        <Show when={current()}>
          <button class="btn sm icon ghost" title="Open in Explorer" onClick={() => openPath(current())}>
            <Icon name="external" size={14} />
          </button>
        </Show>
      </Setting>
      <Show when={pending()}>
        {(p) => (
          <Setting title="Move what is in the current folder there?" desc={<span class="path">{p()}</span>}>
            <button class="btn sm primary" onClick={() => apply(p(), true)}>
              Move the files
            </button>
            <button class="btn sm" onClick={() => apply(p(), false)}>
              Use it as it is
            </button>
            <button class="btn sm ghost" onClick={() => setPending(null)}>
              Cancel
            </button>
          </Setting>
        )}
      </Show>
      <Show when={message()}>
        <div class="setting compact">
          <span class="small muted">{message()}</span>
        </div>
      </Show>
    </>
  );
}

/** A document sheet: the PDF going in (with its scan line while Sift reads it), the Markdown
 * coming out, or a folder. */
export function DocGlyph(props: { kind: "pdf" | "md" | "folder"; dim?: boolean }): JSX.Element {
  const tag = () => (props.kind === "md" ? "MD" : "PDF");
  const tagFill = () => (props.kind === "md" ? "var(--accent-fill)" : "#e5484d");
  return (
    <span class="doc-glyph" style={{ opacity: props.dim ? "0.55" : "1" }}>
      <Show
        when={props.kind !== "folder"}
        fallback={
          <svg width="46" height="54" viewBox="0 0 46 54" aria-hidden="true">
            <path d="M2 10h15l4 5h23v33H2z" fill="var(--sunken)" stroke="var(--hairline-strong)" stroke-width="1.2" />
            <path d="M2 18h42" stroke="var(--hairline-strong)" stroke-width="1.2" />
            <path d="M9 27h12M9 32h18M9 37h9" stroke="var(--muted)" stroke-width="2" opacity="0.5" />
            <rect x="26" y="32" width="18" height="11" fill="#e5484d" />
            <text x="35" y="40.3" text-anchor="middle" fill="#fff" style={{ font: "700 7px var(--font-ui)" }}>
              PDF
            </text>
          </svg>
        }
      >
        <svg width="42" height="54" viewBox="0 0 42 54" aria-hidden="true">
          <path d="M3 1.5h25l11.5 11.5v39.5H3z" fill="var(--sunken)" stroke="var(--hairline-strong)" stroke-width="1.2" />
          <path d="M28 1.5V13h11.5" fill="none" stroke="var(--hairline-strong)" stroke-width="1.2" />
          <Show
            when={props.kind === "md"}
            fallback={<path d="M9 20h17M9 25h23M9 30h20M9 35h13" stroke="var(--muted)" stroke-width="2" opacity="0.5" />}
          >
            <path d="M9 20h10" stroke="var(--accent)" stroke-width="2.6" />
            <path d="M9 26h23M9 31h20M9 36h14" stroke="var(--muted)" stroke-width="2" opacity="0.5" />
          </Show>
          <rect x="0" y="40" width="22" height="11" fill={tagFill()} />
          <text x="11" y="48.3" text-anchor="middle" fill={props.kind === "md" ? "var(--accent-fill-ink)" : "#fff"} style={{ font: "700 7px var(--font-ui)" }}>
            {tag()}
          </text>
        </svg>
      </Show>
      <span class="scan" />
    </span>
  );
}
