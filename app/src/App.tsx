import { Dynamic, type JSX } from "@solidjs/web";
import { Footer } from "./components/Footer";
import { Sidebar } from "./components/Sidebar";
import { page, type Page } from "./lib/store";
import About from "./pages/About";
import Convert from "./pages/Convert";
import Engine from "./pages/Engine";
import Logs from "./pages/Logs";
import Models from "./pages/Models";
import SettingsPage from "./pages/Settings";
import Setup from "./pages/Setup";

/* Sift's window (MASTER_PLAN section 9) on ZER0's layout: the sidebar, the page, the status
   footer. Every option is a flag of `sift convert` or a key of the settings file; the work runs in
   the Rust core and the window shows its events (docs/events.md). */

const PAGES: Record<Page, () => JSX.Element> = {
  setup: Setup,
  convert: Convert,
  engine: Engine,
  models: Models,
  settings: SettingsPage,
  logs: Logs,
  about: About,
};

export default function App(): JSX.Element {
  return (
    <div class="shell">
      <Sidebar />
      <main>
        <Dynamic component={PAGES[page()]} />
      </main>
      <Footer />
    </div>
  );
}
