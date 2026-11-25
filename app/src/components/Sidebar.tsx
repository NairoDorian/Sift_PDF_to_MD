import type { JSX } from "@solidjs/web";
import { For, Show } from "solid-js";
import { collapsed, converting, page, ready, refresh, setPage, setupSteps, toggleSidebar, updateAvailable, type Page } from "../lib/store";
import { BrandMark, Lockup } from "./Brand";
import { Icon, type IconName } from "./Icon";

/* ZER0's sidebar: the brand on top, one row per page with an accent bar on the current one,
   collapsible to icons (remembered). Setup shows only until Sift is ready. */

const ITEMS: { id: Page; label: string; icon: IconName }[] = [
  { id: "setup", label: "Setup", icon: "setup" },
  { id: "convert", label: "Convert", icon: "sieve" },
  { id: "engine", label: "Engine", icon: "engine" },
  { id: "models", label: "Models", icon: "models" },
  { id: "settings", label: "Settings", icon: "settings" },
  { id: "logs", label: "Logs", icon: "logs" },
  { id: "about", label: "About", icon: "about" },
];

export function Sidebar(): JSX.Element {
  const visible = () => ITEMS.filter((i) => i.id !== "setup" || !ready() || page() === "setup");
  const go = (p: Page) => {
    setPage(p);
    void refresh();
  };
  return (
    <aside class={["sidebar", { collapsed: collapsed() }]}>
      <div class="brand">
        <Show when={!collapsed()} fallback={<BrandMark size={28} title="Sift" />}>
          <Lockup size={28} />
        </Show>
      </div>
      <nav aria-label="Pages">
        <For each={visible()}>
          {(item) => (
            <button class="nav-item" aria-current={page() === item.id ? "page" : undefined} title={item.label} onClick={() => go(item.id)}>
              <Icon name={item.icon} size={19} />
              <span class="label">{item.label}</span>
              <Show when={item.id === "setup" && !ready()}>
                <span class="count">{`${setupSteps().filter(Boolean).length}/4`}</span>
              </Show>
              <Show when={item.id === "convert" && converting()}>
                <span class="pip" title="converting" />
              </Show>
              <Show when={item.id === "engine" && updateAvailable()}>
                <span class="pip" title="a newer llama.cpp build is available" />
              </Show>
            </button>
          )}
        </For>
      </nav>
      <button class="collapse" title={collapsed() ? "Expand the menu" : "Collapse the menu"} onClick={toggleSidebar}>
        <Icon name={collapsed() ? "expand" : "collapse"} size={14} />
        <Show when={!collapsed()}>
          <span>Collapse</span>
        </Show>
      </button>
    </aside>
  );
}
