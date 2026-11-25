/**
 * Sift's pre-commit gate (ZER0's `scripts/pre-commit.ts`, cut to Sift's tools): the checks of
 * AGENTS.md's routine in one command, stopping at the first failure.
 *
 *   bun scripts/precommit.ts          format, lint (warnings fail), unit tests, the window's
 *                                     type check and production build, workflow files parse
 *   bun scripts/precommit.ts --full   the same, then the release build and the golden test
 *   (from app/: bun run precommit, bun run precommit:full)
 *
 * As a git hook (opt in, per clone): `git config core.hooksPath .githooks`.
 *
 * No new dependency: cargo, bun and bash (for the golden test) are already the routine's tools.
 * The workflow check exists because a plain YAML value holding `": "` once made GitHub drop the
 * whole release workflow (docs/AUDIT.md).
 */
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

const ROOT = path.resolve(import.meta.dir, "..");
const APP = path.join(ROOT, "app");
const FULL = process.argv.includes("--full");

type Step = { label: string; run: () => { ok: boolean; out: string } };

const sh = (cmd: string, argv: string[], cwd = ROOT) => () => {
  const r = spawnSync(cmd, argv, { cwd, encoding: "utf8", shell: process.platform === "win32" });
  return { ok: r.status === 0, out: `${r.stdout ?? ""}${r.stderr ?? ""}` };
};

function workflowsParse(): { ok: boolean; out: string } {
  const dir = path.join(ROOT, ".github", "workflows");
  if (!fs.existsSync(dir)) return { ok: true, out: "" };
  const bad: string[] = [];
  for (const f of fs.readdirSync(dir).filter((f) => /\.ya?ml$/.test(f))) {
    try {
      const y = Bun.YAML.parse(fs.readFileSync(path.join(dir, f), "utf8")) as { on?: unknown; jobs?: unknown } | null;
      if (!y || y.on === undefined || y.jobs === undefined) bad.push(`${f}: no \`on\` or no \`jobs\``);
    } catch (e) {
      bad.push(`${f}: ${String(e)}`);
    }
  }
  return { ok: bad.length === 0, out: bad.join("\n") };
}

const steps: Step[] = [
  { label: "cargo fmt --check", run: sh("cargo", ["fmt", "--check"]) },
  { label: "clippy (warnings fail)", run: sh("cargo", ["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"]) },
  { label: "unit tests", run: sh("cargo", ["test", "-p", "sift-core", "--lib"]) },
  { label: "window: type check", run: sh("bun", ["x", "tsc", "-b"], APP) },
  { label: "window: production build", run: sh("bun", ["x", "vite", "build"], APP) },
  { label: "workflows parse", run: workflowsParse },
];
if (FULL) {
  steps.push({ label: "release build", run: sh("cargo", ["build", "--release", "-p", "sift-cli"]) });
  steps.push({ label: "golden test (14/14)", run: sh("bash", ["bench/golden/run.sh"]) });
}

const t0 = Date.now();
for (const s of steps) {
  const t = Date.now();
  const r = s.run();
  console.log(`${r.ok ? "ok  " : "FAIL"} ${s.label} (${((Date.now() - t) / 1000).toFixed(1)} s)`);
  if (!r.ok) {
    console.log(r.out.split("\n").slice(-40).join("\n"));
    process.exit(1);
  }
}
console.log(`All checks passed in ${((Date.now() - t0) / 1000).toFixed(0)} s${FULL ? "" : " (run with --full for the release build and the golden test)"}.`);
