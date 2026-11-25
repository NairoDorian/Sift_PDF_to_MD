/**
 * Sift's dependency updater: every dependency to its NEWEST published version, prereleases
 * included (alpha, beta, rc, next, canary, dev builds), Tauri 3 and Solid 2 included, then the
 * validation that proves the new resolution builds and still converts byte for byte. Adapted from
 * ZER0's scripts/update-deps.ts (same rules, cut to Sift's manifests: the Cargo workspace, its
 * crates, the app's Rust crate and the app's package.json).
 *
 *   bun scripts/update-deps.ts            newest incl. prereleases (the default), full validation
 *   bun scripts/update-deps.ts --quick    skip the release build + golden test
 *   bun scripts/update-deps.ts --stable   only stable versions
 *   bun scripts/update-deps.ts --dry-run  report what would change, write nothing
 *   (from app/: bun run update)
 *
 * Rules (each printed in the report when it applies):
 * - crates.io: the HIGHEST non-yanked version (prereleases included unless --stable).
 * - npm: the highest version among every dist-tag (`latest`, `next`, `beta`, `rc`, `alpha`, `dev`...).
 * - NPM_LINE_PINNED: the Solid packages stay on their major (Solid 2, its Vite plugin 3) and take
 *   the newer of their `next` and `latest` tags, never a downgrade. Both tags are read because each
 *   one has been the stale one: `next` carried Solid 2 while `latest` was 1.x, and since Solid 2.0
 *   went stable `next` sits at 2.0.0-rc.23 while `latest` moves (2.0.4, 2026-11-25). Following
 *   `next` alone held Sift on the release candidate (ZER0 reads both for the same reason).
 * - MAJOR_LOCKED: the Tauri family stays on major 3 (its newest alpha/beta/rc); a JS API must never
 *   talk to a different Tauri over IPC. Git dependencies (Tauri's plugins-workspace `v3` branch)
 *   follow their branch tip through `cargo update`.
 * - A crate whose new version does not resolve is put back to its previous spec, reported as held
 *   with the conflict, and `cargo update` runs again (ZER0's conflict handling).
 * - Transitive dependencies move too: `cargo update` rewrites the whole Cargo.lock, `bun update`
 *   the whole bun.lock (within package.json's exact pins, so no pin is rewritten).
 * - Report only: the toolchain (rustc and Bun against their newest releases) and the GitHub
 *   Actions in `.github/workflows` with a newer major (ZER0's audit).
 * Validation is not optional (a lockfile that resolves is not a lockfile that builds):
 *   cargo update (the whole lockfile) -> cargo check --workspace --all-targets -> unit tests
 *   bun update -> tsc -b -> vite build
 *   cargo build --release -p sift-cli -> golden test (bench/golden/run.sh: 7 PDFs byte-identical)
 */
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

const ROOT = path.resolve(import.meta.dir, "..");
const args = new Set(process.argv.slice(2));
const DRY = args.has("--dry-run");
const PRE = !args.has("--stable");
const QUICK = args.has("--quick");
const UA = { "User-Agent": "sift-update-deps" };

const NPM_LINE_PINNED: Record<string, { tags: string[]; major: number; why: string }> = {
  "solid-js": { tags: ["next", "latest"], major: 2, why: "stays on Solid 2: `next` and `latest` have each lagged the other" },
  "@solidjs/web": { tags: ["next", "latest"], major: 2, why: "the Solid 2 renderer, same line as solid-js" },
  "@solidjs/vite-plugin": { tags: ["next", "latest"], major: 3, why: "the Solid 2 compiler plugin (3.x); its `next` tag has lagged `latest` since next.35" },
};
const NPM_MAJOR_LOCKED_PREFIXES = ["@tauri-apps/"];
const CARGO_MAJOR_LOCKED = new Set<string>(["tauri", "tauri-build", "tauri-runtime-wry", "tauri-runtime", "tauri-utils", "tauri-macros", "tauri-codegen", "tauri-plugin"]);

type Change = { manifest: string; name: string; from: string; to: string; note?: string };
type Edit = { file: string; line: number; before: string; name: string };
const changes: Change[] = [];
const held: Change[] = [];
const edits: Edit[] = [];

function run(cmd: string, argv: string[], cwd = ROOT): { ok: boolean; out: string } {
  const r = spawnSync(cmd, argv, { cwd, encoding: "utf8", shell: process.platform === "win32" });
  return { ok: r.status === 0, out: `${r.stdout ?? ""}${r.stderr ?? ""}` };
}

// ---- semver -------------------------------------------------------------------------
type Ver = { major: number; minor: number; patch: number; pre: (string | number)[]; raw: string };
function parse(v: string): Ver | null {
  const m = /^v?(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?(?:\+.*)?$/.exec(v.trim());
  if (!m) return null;
  const pre = m[4] ? m[4].split(".").map((p) => (/^\d+$/.test(p) ? Number(p) : p)) : [];
  return { major: +m[1], minor: +m[2], patch: +m[3], pre, raw: v.trim() };
}
function cmp(a: Ver, b: Ver): number {
  for (const k of ["major", "minor", "patch"] as const) if (a[k] !== b[k]) return a[k] - b[k];
  if (!a.pre.length || !b.pre.length) return (a.pre.length ? -1 : 0) - (b.pre.length ? -1 : 0);
  for (let i = 0; i < Math.max(a.pre.length, b.pre.length); i++) {
    const x = a.pre[i], y = b.pre[i];
    if (x === undefined) return -1;
    if (y === undefined) return 1;
    if (typeof x === "number" && typeof y === "number") { if (x !== y) return x - y; continue; }
    if (typeof x === "number") return -1;
    if (typeof y === "number") return 1;
    if (x !== y) return x < y ? -1 : 1;
  }
  return 0;
}
const bare = (spec: string) => spec.replace(/^[=^~<>]+/, "").trim();
/** Cargo spec ("1", "1.0", "0.11.1", "9.0.0-pre3", "^1.2") -> full semver for comparison. */
function normSpec(spec: string): string {
  const s = bare(spec).split(",")[0].trim();
  const [core, ...pre] = s.split("-");
  const parts = core.split(".");
  while (parts.length < 3) parts.push("0");
  return parts.slice(0, 3).join(".") + (pre.length ? `-${pre.join("-")}` : "");
}

// ---- crates.io ----------------------------------------------------------------------
/** The HIGHEST non-yanked version (crates.io's `newest_version` is the most recently PUBLISHED
 * one: tauri-build reported 2.7.4 while 3.0.0-alpha.4 existed, 2026-11-07). */
async function crateNewest(name: string): Promise<string | null> {
  const r = await fetch(`https://crates.io/api/v1/crates/${name}`, { headers: UA });
  if (!r.ok) return null;
  const j: any = await r.json();
  const vs = ((j.versions ?? []) as any[])
    .filter((v) => !v.yanked)
    .map((v) => parse(String(v.num)))
    .filter((v): v is Ver => !!v && (PRE || !v.pre.length));
  return vs.sort(cmp).at(-1)?.raw ?? null;
}

/** `name = "x"` or `name = { version = "x", ... }` lines in any [*dependencies] table. */
async function updateCargoManifest(file: string): Promise<void> {
  const lines = fs.readFileSync(file, "utf8").split("\n");
  let inDeps = false;
  for (let i = 0; i < lines.length; i++) {
    const t = lines[i].trim();
    if (t.startsWith("[")) { inDeps = /dependencies\]$/.test(t); continue; }
    if (!inDeps || t.startsWith("#")) continue;
    const m = /^([A-Za-z0-9_-]+)\s*=\s*(?:"([^"]+)"|\{(.*)\})/.exec(t);
    if (!m) continue;
    const name = m[1];
    const inlineVer = m[3] ? /version\s*=\s*"([^"]+)"/.exec(m[3])?.[1] : undefined;
    if (m[3] && /\bgit\s*=/.test(m[3])) {
      changes.push({ manifest: path.relative(ROOT, file), name, from: "git", to: "branch tip", note: "follows its git branch through cargo update" });
      continue;
    }
    const spec = m[2] ?? inlineVer;
    if (!spec || (m[3] && /\bpath\s*=/.test(m[3]) && !inlineVer)) continue;
    const newest = await crateNewest(name);
    const cur = parse(normSpec(spec));
    const nv = newest ? parse(newest) : null;
    if (!nv || !cur) continue;
    if (CARGO_MAJOR_LOCKED.has(name) && nv.major !== cur.major) {
      held.push({ manifest: path.relative(ROOT, file), name, from: spec, to: newest!, note: "major-locked (the Tauri family stays on major 3)" });
      continue;
    }
    if (cmp(nv, cur) <= 0) continue;
    const prefix = spec.startsWith("=") ? "=" : "";
    const next = `${prefix}${newest!.split("+")[0]}`; // build metadata is ignored by cargo
    edits.push({ file, line: i, before: lines[i], name });
    lines[i] = lines[i].replace(`"${spec}"`, `"${next}"`);
    changes.push({ manifest: path.relative(ROOT, file), name, from: spec, to: next });
  }
  if (!DRY) fs.writeFileSync(file, lines.join("\n"));
}

/** Put a crate back to its previous spec (its new version does not resolve). */
function holdCrate(name: string, why: string): boolean {
  const mine = edits.filter((e) => e.name === name);
  if (!mine.length) return false;
  for (const e of mine) {
    const lines = fs.readFileSync(e.file, "utf8").split("\n");
    lines[e.line] = e.before;
    fs.writeFileSync(e.file, lines.join("\n"));
  }
  for (let i = changes.length - 1; i >= 0; i--) {
    if (changes[i].name === name) {
      held.push({ ...changes[i], note: `held: ${why}` });
      changes.splice(i, 1);
    }
  }
  for (let i = edits.length - 1; i >= 0; i--) if (edits[i].name === name) edits.splice(i, 1);
  return true;
}

// ---- npm ----------------------------------------------------------------------------
async function npmBest(name: string, current: string): Promise<{ v: string; note?: string } | null> {
  const r = await fetch(`https://registry.npmjs.org/${name.replace("/", "%2F")}`, { headers: UA });
  if (!r.ok) return null;
  const j: any = await r.json();
  const tags: Record<string, string> = j["dist-tags"] ?? {};
  const cur = parse(bare(current));
  const pinned = NPM_LINE_PINNED[name];
  if (pinned) {
    const onLine = pinned.tags
      .map((t) => (tags[t] ? parse(tags[t]) : null))
      .filter((v): v is Ver => !!v && v.major === pinned.major && (PRE || !v.pre.length))
      .sort(cmp);
    const pv = onLine.at(-1);
    const seen = pinned.tags.map((t) => `${t} ${tags[t] ?? "-"}`).join(", ");
    if (pv && (!cur || cmp(pv, cur) > 0)) return { v: pv.raw, note: `line-pinned to ${pinned.major}.x (${seen}): ${pinned.why}` };
    return null;
  }
  const cands = Object.entries(tags).filter(([t]) => PRE || t === "latest").map(([, v]) => parse(v)).filter(Boolean) as Ver[];
  let best = cands.sort(cmp).at(-1);
  if (!best) return null;
  if (cur && NPM_MAJOR_LOCKED_PREFIXES.some((p) => name.startsWith(p)) && best.major !== cur.major) {
    const same = cands.filter((c) => c.major === cur.major).sort(cmp).at(-1);
    if (!same) return null;
    best = same;
  }
  if (cur && cmp(best, cur) <= 0) return null;
  return { v: best.raw };
}

async function updatePackageJson(file: string): Promise<boolean> {
  if (!fs.existsSync(file)) return false;
  const pkg = JSON.parse(fs.readFileSync(file, "utf8"));
  for (const section of ["dependencies", "devDependencies"]) {
    for (const [name, spec] of Object.entries<string>(pkg[section] ?? {})) {
      if (/^(git|file|link|workspace):|\//.test(spec) && !spec.startsWith("@")) continue;
      const best = await npmBest(name, spec);
      if (!best) continue;
      const keepCaret = spec.startsWith("^") && !parse(best.v)?.pre.length;
      pkg[section][name] = `${keepCaret ? "^" : ""}${best.v}`;
      changes.push({ manifest: path.relative(ROOT, file), name, from: spec, to: pkg[section][name], note: best.note });
    }
  }
  if (!DRY) fs.writeFileSync(file, JSON.stringify(pkg, null, 2) + "\n");
  return true;
}

// ---- main ---------------------------------------------------------------------------
const manifests = [path.join(ROOT, "Cargo.toml"), ...fs.readdirSync(path.join(ROOT, "crates")).map((c) => path.join(ROOT, "crates", c, "Cargo.toml"))];
if (fs.existsSync(path.join(ROOT, "app", "src-tauri", "Cargo.toml"))) manifests.push(path.join(ROOT, "app", "src-tauri", "Cargo.toml"));
console.log(`Sift update-deps (${PRE ? "newest incl. prereleases" : "stable only"}${DRY ? ", dry run" : ""}${QUICK ? ", quick" : ""})`);
for (const m of manifests.filter((m) => fs.existsSync(m))) await updateCargoManifest(m);
const hasApp = await updatePackageJson(path.join(ROOT, "app", "package.json"));

const report = () => {
  for (const c of changes) console.log(`  ${c.manifest}: ${c.name} ${c.from} -> ${c.to}${c.note ? `  (${c.note})` : ""}`);
  for (const h of held) console.log(`  held ${h.name} ${h.from} (newest ${h.to}): ${h.note}`);
  if (!changes.some((c) => c.from !== "git")) console.log("  everything is already at its newest version");
};

/** Report only: the toolchain against its newest releases. Updating it is the maintainer's call
 * (`rustup update`, `bun upgrade`): both are machine-wide, shared with every other project. */
async function toolchainReport(): Promise<void> {
  const rustc = run("rustc", ["--version"]).out.trim();
  // the active toolchain (stable or a nightly lane), as `rustup check` reports it
  const active = run("rustup", ["show", "active-toolchain"]).out.trim().split(/\s+/)[0] ?? "";
  const lines = run("rustup", ["check"]).out.split("\n").map((l) => l.trim());
  const mine = lines.find((l) => active && l.startsWith(active));
  console.log(`  toolchain: ${rustc || "rustc not found"}${mine ? `  [rustup: ${mine}]` : ""}`);
  const bunNow = run("bun", ["--version"]).out.trim();
  try {
    const r = await fetch("https://registry.npmjs.org/bun", { headers: { ...UA, Accept: "application/vnd.npm.install-v1+json" } });
    const newest: string | undefined = r.ok ? ((await r.json()) as any)["dist-tags"]?.latest : undefined;
    const a = parse(bunNow), b = newest ? parse(newest) : null;
    console.log(`  toolchain: bun ${bunNow}${a && b && cmp(b, a) > 0 ? `  [newer: ${newest}, run \`bun upgrade\`]` : "  [newest]"}`);
  } catch {
    console.log(`  toolchain: bun ${bunNow}`);
  }
}

/** Report only (ZER0's audit): GitHub Actions in .github/workflows that have a newer major. */
async function actionsReport(): Promise<void> {
  const dir = path.join(ROOT, ".github", "workflows");
  if (!fs.existsSync(dir)) return;
  const refs = new Map<string, number>();
  for (const f of fs.readdirSync(dir).filter((f) => /\.ya?ml$/.test(f))) {
    for (const m of fs.readFileSync(path.join(dir, f), "utf8").matchAll(/uses:\s*([\w.-]+\/[\w.-]+)@v(\d+)/g)) refs.set(m[1], Number(m[2]));
  }
  const headers: Record<string, string> = { ...UA, Accept: "application/vnd.github+json" };
  if (process.env.GITHUB_TOKEN) headers.Authorization = `Bearer ${process.env.GITHUB_TOKEN}`;
  for (const [repo, major] of refs) {
    try {
      const r = await fetch(`https://api.github.com/repos/${repo}/releases/latest`, { headers });
      const tag: string | undefined = r.ok ? ((await r.json()) as any).tag_name : undefined;
      const newest = tag ? Number(/^v?(\d+)/.exec(tag)?.[1]) : NaN;
      if (newest > major) console.log(`  action ${repo}@v${major}: v${newest} exists (${tag}); move it by hand after reading its notes`);
    } catch {
      /* report only: an unreachable API is not a failure */
    }
  }
}

if (DRY) {
  report();
  await toolchainReport();
  await actionsReport();
  process.exit(0);
}

// cargo update over the whole lockfile; a crate that does not resolve is held and we retry
for (let attempt = 0; ; attempt++) {
  const r = run("cargo", ["update"]);
  if (r.ok) {
    console.log("ok   cargo update");
    break;
  }
  const conflict = /failed to select a version for (?:the requirement )?`([A-Za-z0-9_-]+)/.exec(r.out);
  if (attempt < 8 && conflict && holdCrate(conflict[1], "its newest version does not resolve with the rest (cargo update)")) {
    console.log(`     cargo update: ${conflict[1]} does not resolve; held at its previous spec, retrying`);
    continue;
  }
  console.log("FAIL cargo update");
  console.log(r.out.split("\n").slice(-25).join("\n"));
  report();
  process.exit(1);
}
report();
await toolchainReport();
await actionsReport();

const steps: [string, string, string[], string][] = [
  ["cargo check", "cargo", ["check", "--workspace", "--all-targets"], ROOT],
  ["unit tests", "cargo", ["test", "-p", "sift-core", "--lib"], ROOT],
];
if (hasApp) {
  // `bun update`, not `bun install`: install keeps every locked sub-dependency where it was
  steps.push(["bun update", "bun", ["update"], path.join(ROOT, "app")]);
  steps.push(["typecheck", "bun", ["x", "tsc", "-b"], path.join(ROOT, "app")]);
  steps.push(["vite build", "bun", ["x", "vite", "build"], path.join(ROOT, "app")]);
}
if (!QUICK) {
  steps.push(["release build", "cargo", ["build", "--release", "-p", "sift-cli"], ROOT]);
  steps.push(["golden test", "bash", ["bench/golden/run.sh"], ROOT]);
}
for (const [label, cmd, argv, cwd] of steps) {
  const r = run(cmd, argv, cwd);
  console.log(`${r.ok ? "ok  " : "FAIL"} ${label}`);
  if (!r.ok) {
    console.log(r.out.split("\n").slice(-30).join("\n"));
    process.exit(1);
  }
}
console.log("All dependencies updated and validated.");
