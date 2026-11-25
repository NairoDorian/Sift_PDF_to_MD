import { render } from "@solidjs/web";
import App from "./App";
import { start } from "./lib/store";
import "./styles.css";

start();
render(() => <App />, document.getElementById("root")!);
