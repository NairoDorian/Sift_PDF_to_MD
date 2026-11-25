import type { JSX } from "@solidjs/web";
import { Group, PageHead, Seg, Setting } from "../components/ui";
import { Icon } from "../components/Icon";
import { dirName } from "../lib/format";
import {
  motion,
  openPath,
  overview,
  saveSetting,
  setMotion,
  setPage,
  setTheme,
  settings,
  systemReducesMotion,
  theme,
  type Motion,
  type Theme,
} from "../lib/store";

/* What has no other home. Every setting lives in one place only: the conversion options on
   Convert, the engine folder, backend and start-up check on Engine, the models folder on Models.
   This page links to them instead of repeating them. */

export default function SettingsPage(): JSX.Element {
  const file = () => overview()?.settings_file ?? "";
  return (
    <div class="page">
      <PageHead title="Settings" desc="Saved in one settings file, shared with the command line (sift config show)." />

      <div class="two-col">
        <Group title="Appearance">
          <Setting title="Theme" desc="System follows Windows">
            <Seg<Theme>
              label="Theme"
              value={theme()}
              options={[
                { value: "system", label: "System" },
                { value: "dark", label: "Dark" },
                { value: "light", label: "Light" },
              ]}
              onChange={setTheme}
            />
          </Setting>
          <Setting
            title="Motion"
            desc={
              motion() === "system"
                ? systemReducesMotion()
                  ? "Windows' animation effects are off: the sieve moves gently"
                  : "The sieve shakes and pours while Sift works"
                : motion() === "on"
                  ? "Every animation plays"
                  : "Nothing moves"
            }
          >
            <Seg<Motion>
              label="Motion"
              value={motion()}
              options={[
                { value: "system", label: "System" },
                { value: "on", label: "On" },
                { value: "off", label: "Off" },
              ]}
              onChange={setMotion}
            />
          </Setting>
        </Group>

        <Group title="Graphics memory">
          <Setting
            title="Run the two models"
            desc={
              settings()?.scheduler.models === "together"
                ? "Both at once: faster when the GPU has about 6.3 GB free"
                : "surya first, then Gemma: less memory, faster on 8 GB"
            }
          >
            <Seg
              label="Run the two models"
              value={settings()?.scheduler.models === "together" ? "together" : "sequential"}
              options={[
                { value: "sequential", label: "One by one" },
                { value: "together", label: "Together" },
              ]}
              onChange={(v) => saveSetting("scheduler", "models", v)}
            />
          </Setting>
        </Group>
      </div>

      <Group title="Where things are">
        <Setting compact title="Engine folder" desc={<span class="path">{overview()?.engine_dir || "not chosen yet"}</span>}>
          <button class="btn sm ghost" onClick={() => setPage("engine")}>
            Engine page <Icon name="expand" size={12} />
          </button>
        </Setting>
        <Setting compact title="Models folder" desc={<span class="path">{overview()?.models_dir || "not chosen yet"}</span>}>
          <button class="btn sm ghost" onClick={() => setPage("models")}>
            Models page <Icon name="expand" size={12} />
          </button>
        </Setting>
        <Setting
          compact
          title={overview()?.portable ? "Settings file (portable mode)" : "Settings file"}
          desc={<span class="path">{file()}</span>}
        >
          <button class="btn sm" disabled={!file()} onClick={() => openPath(dirName(file()))}>
            <Icon name="folder" size={13} /> Open folder
          </button>
        </Setting>
      </Group>

      <p class="small muted">
        Every other key (surya's and Gemma's server slots, sampling, the LLM pass's parts) keeps its measured default and is described in docs/settings.md; change one with{" "}
        <code>sift config set &lt;section.key&gt; &lt;value&gt;</code>.
      </p>
    </div>
  );
}
