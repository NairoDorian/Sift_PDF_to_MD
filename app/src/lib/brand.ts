/**
 * Sift's mark: a strainer (rim, bowl and a diagonal net) over two lines of text. The PDF goes in at
 * the top, the text comes out underneath.
 *
 * One geometry, three consumers (ZER0's rule: the mark is drawn from one source, never redrawn by
 * hand): the window (`components/Brand.tsx`), the app icons (`scripts/gen-icons.ts` hands
 * `markSvg({ tile: true })` to `tauri icon`) and the README art (`docs/brand/`, same script).
 *
 * Drawn on a 32-unit grid. The net is two families of 45-degree lines, 5 units apart, crossing on
 * the centre line, clipped to the bowl; it reads as mesh from 32 px up and as a shaded bowl below.
 */

export const GRID = 32;

/** The rim: a straight top edge a little wider than the bowl. */
export const RIM = "M3.5 7.2H28.5";
/** The bowl: half an ellipse under the rim (bottom at y = 16.7). */
export const BOWL = "M5.5 7.2A10.5 9.5 0 0 0 26.5 7.2";
/** The bowl's area, the net's clip. */
export const BOWL_AREA = `${BOWL}Z`;

/** x - y = c and x + y = 32 - c: mirror images about x = 16, one crossing at (16, 12). */
const NET_C = [-11, -6, -1, 4, 9, 14, 19];
export const NET = NET_C.map((c) => `M${c + 4} 4L${c + 20} 20`)
  .concat(NET_C.map((c) => `M${28 - c} 4L${12 - c} 20`))
  .join("");

/** What comes through: two lines of text, the second longer, set left like a paragraph. */
export const TEXT_LINES = "M10 21.8H18.2M10 25.4H23.2";

export const STROKE = { frame: 2.6, net: 1, text: 2.4 } as const;

/** The app tile and the shipped default colours (`styles.css` dark theme). */
export const BRAND = {
  tile: "#111316",
  ink: "#f2ede4",
  accent: "#f5a524",
  /** the accent on a light ground */
  accentInk: "#b45f06",
} as const;

export type MarkOptions = {
  /** Draw the rounded app tile behind the mark (the app icon). */
  tile?: boolean;
  ink?: string;
  accent?: string;
  /** Output size in px (the viewBox stays the 32-unit grid). */
  size?: number;
  /** Unique id prefix for the clip path, when several marks share one document. */
  id?: string;
};

/** The mark as a standalone SVG document. */
export function markSvg(o: MarkOptions = {}): string {
  const ink = o.ink ?? BRAND.ink;
  const accent = o.accent ?? BRAND.accent;
  const size = o.size ?? 1024;
  const id = o.id ?? "sift-bowl";
  // On the tile the mark is scaled to 84 % and centred, the margin Windows' own icons keep.
  const body =
    `<defs><clipPath id="${id}"><path d="${BOWL_AREA}"/></clipPath></defs>` +
    `<g clip-path="url(#${id})"><path d="${NET}" fill="none" stroke="${ink}" stroke-width="${STROKE.net}" stroke-opacity="0.82"/></g>` +
    `<path d="${RIM}${BOWL}" fill="none" stroke="${ink}" stroke-width="${STROKE.frame}" stroke-linecap="round" stroke-linejoin="round"/>` +
    `<path d="${TEXT_LINES}" fill="none" stroke="${accent}" stroke-width="${STROKE.text}" stroke-linecap="round"/>`;
  const tile = o.tile ? `<rect width="32" height="32" rx="7" fill="${BRAND.tile}"/>` : "";
  const placed = o.tile ? `<g transform="translate(16 16) scale(0.84) translate(-16 -16.3)">${body}</g>` : body;
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 ${GRID} ${GRID}">${tile}${placed}</svg>`;
}
