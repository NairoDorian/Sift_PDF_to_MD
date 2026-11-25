/**
 * Sift's icons and brand art, regenerated from the one geometry in app/src/lib/brand.ts (the
 * window draws the same paths), ZER0's way (its scripts/gen-icons.ts):
 *
 *   bun scripts/gen-icons.ts
 *
 * 1. app/src-tauri/icons/: the app tile as an SVG handed to `tauri icon`, the bundler's own tool,
 *    which writes every size and the ICO container itself. Sift is Windows-only for now, so the
 *    files nothing reads (mobile, macOS, Store logos) are removed again.
 * 2. docs/brand/: the mark and the README banner (dark and light), an SVG that plays the same
 *    PDF -> sieve -> Markdown motion as the Convert page (CSS only; it stays still for readers
 *    who ask for reduced motion).
 *
 * No image library: `tauri icon` rasterises, everything else is SVG text.
 */
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { BOWL, BOWL_AREA, BRAND, NET, RIM, STROKE, markSvg } from "../app/src/lib/brand";

const ROOT = join(import.meta.dir, "..");
const APP = join(ROOT, "app");
const ICONS = join(APP, "src-tauri", "icons");
const DOCS = join(ROOT, "docs", "brand");
/** What tauri.conf.json's bundle.icon lists, plus the 1024 master `tauri icon` names icon.png. */
const KEEP = new Set(["32x32.png", "64x64.png", "128x128.png", "128x128@2x.png", "icon.png", "icon.ico"]);

// ---- 1. app icons ------------------------------------------------------------------------------
const staging = mkdtempSync(join(tmpdir(), "sift-icon-"));
try {
  const src = join(staging, "sift.svg");
  writeFileSync(src, markSvg({ tile: true, size: 1024 }));
  const r = spawnSync("bun", ["x", "tauri", "icon", src, "-o", ICONS], { cwd: APP, stdio: "inherit", shell: process.platform === "win32" });
  if (r.status !== 0) {
    console.error("tauri icon failed");
    process.exit(r.status ?? 1);
  }
} finally {
  rmSync(staging, { recursive: true, force: true });
}
for (const f of readdirSync(ICONS)) if (!KEEP.has(f)) rmSync(join(ICONS, f), { recursive: true, force: true });
console.log(`app/src-tauri/icons: ${[...KEEP].join(", ")}`);

// ---- 2. brand art --------------------------------------------------------------------------------
type Theme = { bg: string; edge: string; ink: string; muted: string; accent: string; fill: string; fillInk: string; sunken: string; line: string };
const THEMES: Record<"dark" | "light", Theme> = {
  dark: { bg: "#0d0f12", edge: "#ece7de22", ink: "#ece7de", muted: "#8d9097", accent: BRAND.accent, fill: BRAND.accent, fillInk: "#1b1305", sunken: "#090b0d", line: "#ece7de30" },
  light: { bg: "#f4f2ee", edge: "#16171a1f", ink: "#16171a", muted: "#6b6e74", accent: BRAND.accentInk, fill: BRAND.accent, fillInk: "#1b1305", sunken: "#ebe8e2", line: "#16171a2a" },
};
const FONT = `'Segoe UI Variable Display','Segoe UI',-apple-system,BlinkMacSystemFont,'Helvetica Neue',Arial,sans-serif`;
const MONO = `'Cascadia Mono',Consolas,'SF Mono',Menlo,monospace`;
const r1 = (n: number) => Math.round(n * 10) / 10;

/** The banner: the lockup on the left, the flow on the right (PDF, sieve, falling Markdown, MD). */
function banner(t: Theme): string {
  const W = 1280;
  const H = 400;
  // the flow column
  const CX = 1040;
  const RIM_Y = 168;
  const RX = 66;
  const RY = 50;
  const bowl = `M${CX - RX} ${RIM_Y}A${RX} ${RY} 0 0 0 ${CX + RX} ${RIM_Y}`;
  const net: string[] = [];
  for (let k = -11; k <= 11; k++) {
    const c = CX - (RIM_Y + 24) + k * 11;
    const d = CX + (RIM_Y + 24) + k * 11;
    net.push(`M${c + 140} 140L${c + 236} 236M${d - 140} 140L${d - 236} 236`);
  }
  const bowlY = (x: number) => RIM_Y + RY * Math.sqrt(Math.max(0, 1 - ((x - CX) / RX) ** 2));
  const chars = "#|-*a1e`>t[]_sx#n|o";
  const glyphs: string[] = [];
  [-36, -22, -8, 8, 22, 36].forEach((dx, c) => {
    for (let k = 0; k < 3; k++) {
      const x = CX + dx;
      const y = r1(bowlY(x) + 6);
      const delay = (-(k * 0.6 + c * 0.27)).toFixed(2);
      glyphs.push(`<text class="g" x="${x}" y="${y}" style="--f:${r1(300 - y)}px;--s:${r1(((300 - y) * (k + 0.5)) / 3)}px;animation-delay:${delay}s">${chars[(c * 3 + k * 5) % chars.length]}</text>`);
    }
  });
  const feeds = [
    [1010, 14, 0],
    [1030, 20, -0.5],
    [1054, 12, -1],
    [1020, 10, -0.75],
    [1046, 16, -0.25],
  ]
    .map(([x, w, d]) => `<rect class="feed" x="${x}" y="114" width="${w}" height="3" style="animation-delay:${d}s"/>`)
    .join("");
  const mdLines = [44, 30, 38, 26, 34]
    .map((w, i) => `<rect class="ln" x="${1012}" y="${318 + i * 11}" width="${w}" height="${i === 0 ? 4 : 3}" fill="${i === 0 ? t.accent : t.muted}" style="animation-delay:${(i * 0.5).toFixed(1)}s"/>`)
    .join("");
  const markScale = 5.4;
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" viewBox="0 0 ${W} ${H}" role="img" aria-label="Sift: PDFs to Markdown you can trust">
<style>
.sv{transform-box:fill-box;transform-origin:50% 35%;animation:shake .42s ease-in-out infinite alternate}
@keyframes shake{from{transform:translateX(-2.4px) rotate(-1.1deg)}to{transform:translateX(2.4px) rotate(1.1deg)}}
.g{fill:${t.accent};font:600 15px ${MONO};text-anchor:middle;opacity:0;animation:fall 1.8s linear infinite}
@keyframes fall{0%{transform:translateY(0);opacity:0}12%{opacity:.95}80%{opacity:.85}100%{transform:translateY(var(--f));opacity:0}}
.feed{fill:${t.muted};opacity:0;animation:feed 1.5s ease-in infinite}
@keyframes feed{0%{transform:translateY(-6px);opacity:0}25%{opacity:.6}100%{transform:translateY(40px);opacity:0}}
.ln{transform-box:fill-box;transform-origin:left;animation:type 2.5s ease-out infinite}
@keyframes type{0%{transform:scaleX(0)}40%,100%{transform:scaleX(1)}}
@media (prefers-reduced-motion:reduce){.sv,.g,.feed,.ln{animation:none}.feed{opacity:0}.g{opacity:.8;transform:translateY(var(--s))}}
</style>
<rect x=".5" y=".5" width="${W - 1}" height="${H - 1}" fill="${t.bg}" stroke="${t.edge}"/>
<defs><clipPath id="m"><path d="${BOWL_AREA}"/></clipPath><clipPath id="b"><path d="${bowl}Z"/></clipPath></defs>
<g transform="translate(84 112) scale(${markScale})">
<g clip-path="url(#m)"><path d="${NET}" fill="none" stroke="${t.ink}" stroke-width="${STROKE.net}" stroke-opacity=".82"/></g>
<path d="${RIM}${BOWL}" fill="none" stroke="${t.ink}" stroke-width="${STROKE.frame}" stroke-linecap="round" stroke-linejoin="round"/>
</g>
<text x="292" y="214" fill="${t.ink}" style="font:600 118px ${FONT};letter-spacing:-2px">Sift</text>
<text x="296" y="264" fill="${t.ink}" style="font:500 29px ${FONT}">PDFs to Markdown you can trust</text>
<text x="297" y="304" fill="${t.muted}" style="font:400 19px ${FONT}">Every value checked against the PDF. Every image kept and described. All local.</text>
<g>
<path d="M1006 30h48l14 14v62h-62z" fill="${t.sunken}" stroke="${t.line}" stroke-width="1.5"/>
<path d="M1054 30v14h14" fill="none" stroke="${t.line}" stroke-width="1.5"/>
<path d="M1016 56h34M1016 66h42M1016 76h38M1016 86h26" stroke="${t.muted}" stroke-width="3" opacity=".55"/>
<rect x="1000" y="88" width="30" height="14" fill="#e5484d"/><text x="1015" y="99" fill="#fff" style="font:700 10px ${FONT};text-anchor:middle">PDF</text>
</g>
${feeds}
<g class="sv">
<path d="${bowl}Z" fill="${t.ink}" opacity=".05"/>
<g clip-path="url(#b)"><path d="${net.join("")}" fill="none" stroke="${t.ink}" stroke-width="1.3" opacity=".34"/></g>
<path d="M${CX - RX - 14} ${RIM_Y}H${CX + RX + 14}${bowl}" fill="none" stroke="${t.ink}" stroke-width="4.5" stroke-linecap="round" stroke-linejoin="round"/>
</g>
${glyphs.join("")}
<g>
<path d="M1000 300h56l14 14v74h-70z" fill="${t.sunken}" stroke="${t.line}" stroke-width="1.5"/>
<path d="M1056 300v14h14" fill="none" stroke="${t.line}" stroke-width="1.5"/>
${mdLines}
<rect x="994" y="370" width="28" height="14" fill="${t.fill}"/><text x="1008" y="381" fill="${t.fillInk}" style="font:700 10px ${FONT};text-anchor:middle">MD</text>
</g>
</svg>
`;
}

mkdirSync(DOCS, { recursive: true });
writeFileSync(join(DOCS, "sift-icon.svg"), markSvg({ tile: true, size: 256, id: "sift-icon" }));
writeFileSync(join(DOCS, "sift-mark-dark.svg"), markSvg({ ink: THEMES.dark.ink, size: 128, id: "sift-mark" }));
writeFileSync(join(DOCS, "sift-mark-light.svg"), markSvg({ ink: THEMES.light.ink, size: 128, id: "sift-mark" }));
writeFileSync(join(DOCS, "banner-dark.svg"), banner(THEMES.dark));
writeFileSync(join(DOCS, "banner-light.svg"), banner(THEMES.light));
console.log("docs/brand: sift-icon.svg, sift-mark-dark.svg, sift-mark-light.svg, banner-dark.svg, banner-light.svg");
