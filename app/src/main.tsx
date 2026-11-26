import { render } from "@solidjs/web";
import App from "./App";
import { start } from "./lib/store";
import "./styles.css";

// start() first: in the browser preview it installs the mock the first page's calls go to
void start().then(() => render(() => <App />, document.getElementById("root")!));
