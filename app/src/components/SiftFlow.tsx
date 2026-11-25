import type { JSX } from "@solidjs/web";

/* The sieve between the PDF card and the Markdown card on the Convert page: the same strainer as
   the mark (lib/brand.ts), drawn large with a finer net. While a conversion runs (`.running` on
   the flow around it, styles.css) the page's fragments drop into the bowl, the bowl shakes and
   characters of Markdown fall out through the net toward the document below. Static geometry,
   CSS keyframes only: no timers, nothing to clean up, and prefers-reduced-motion stops it all. */

const CX = 100;
const RIM_Y = 46;
const RX = 59;
const RY = 50;
const BOWL = `M${CX - RX} ${RIM_Y}A${RX} ${RY} 0 0 0 ${CX + RX} ${RIM_Y}`;
const FRAME = `M${CX - RX - 13} ${RIM_Y}H${CX + RX + 13}${BOWL}`;
const bowlY = (x: number) => RIM_Y + RY * Math.sqrt(Math.max(0, 1 - ((x - CX) / RX) ** 2));

/** The net: x - y = c and x + y = d, 10 units apart, crossing at the bowl's centre (100, 70). */
const NET = Array.from({ length: 25 }, (_, i) => {
  const k = i - 12;
  const c = 30 + k * 10;
  const d = 170 + k * 10;
  return `M${c + 30} 30L${c + 110} 110M${d - 30} 30L${d - 110} 110`;
}).join("");

/** What the PDF is made of, drifting into the bowl (rect x, width, delay). */
const FEEDS = [
  [64, 16, 0],
  [86, 22, -0.55],
  [112, 12, -1.05],
  [128, 18, -0.3],
  [74, 10, -0.8],
  [100, 14, -1.25],
] as const;

/** Grains in the bowl, jiggling with it. */
const GRAINS = [
  [80, 75],
  [91, 82],
  [103, 85],
  [114, 80],
  [123, 73],
  [96, 74],
  [109, 72],
  [86, 68],
] as const;

/** Markdown characters through the net: six columns, three in flight per column. */
const CHARS = "#|-*a1e`>t[]_sxo#n|";
const PERIOD = 1.7;
const BOTTOM = 212; // below the band: they land in the Markdown card
const GLYPHS = [70, 82, 94, 106, 118, 130].flatMap((x, c) =>
  [0, 1, 2].map((k) => {
    const y = Math.round(bowlY(x)) + 4;
    return {
      x,
      y,
      ch: CHARS[(c * 3 + k * 5) % CHARS.length],
      fall: BOTTOM - y,
      /** where it sits when nothing moves (Motion: Off) */
      rest: Math.round(((BOTTOM - 26 - y) * (k + 0.5)) / 3),
      delay: (-((k * PERIOD) / 3 + c * 0.29)).toFixed(2),
    };
  }),
);

export function SiftFlow(): JSX.Element {
  return (
    <svg class="sieve-svg" viewBox="0 0 200 186" aria-hidden="true">
      <defs>
        <clipPath id="sift-flow-bowl">
          <path d={`${BOWL}Z`} />
        </clipPath>
      </defs>
      {FEEDS.map(([x, w, delay]) => (
        <rect class="feed" x={x} y={10} width={w} height={2.6} style={`animation-delay:${delay}s`} />
      ))}
      <g class="sieve">
        <path class="sieve-body" d={`${BOWL}Z`} />
        <g clip-path="url(#sift-flow-bowl)">
          <path class="sieve-net" d={NET} fill="none" stroke-width="1.15" />
          {GRAINS.map(([x, y], i) => (
            <circle class="grain" cx={x} cy={y} r={i % 3 === 0 ? 2.2 : 1.7} style={`animation-delay:${(-i * 0.11).toFixed(2)}s`} />
          ))}
        </g>
        <path class="sieve-frame" d={FRAME} fill="none" stroke-width="4" stroke-linecap="round" stroke-linejoin="round" />
      </g>
      {GLYPHS.map((g) => (
        <text class="glyph" x={g.x} y={g.y} text-anchor="middle" style={`--fall:${g.fall}px;--rest:${g.rest}px;animation-delay:${g.delay}s`}>
          {g.ch}
        </text>
      ))}
    </svg>
  );
}
