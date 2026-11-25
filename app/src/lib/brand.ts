/**
 * Sift's mark: a strainer, its rim, its bowl and a diagonal net. Nothing else: the strainer alone
 * says what Sift does (the two lines of text that came out under it were dropped, 2026-11-25).
 *
 * One geometry, three consumers (ZER0's rule: the mark is drawn from one source, never redrawn by
 * hand): the window (`components/Brand.tsx`), the app icons (`scripts/gen-icons.ts` hands
 * `markSvg({ tile: true })` to `tauri icon`) and the README art (`docs/brand/`, same script).
 *
 * Drawn on a 32-unit grid, centred on it (the drawn strainer spans y 9.9 to 22.0). The net is two
 * families of 45-degree lines, 5 units apart, crossing on the centre line, clipped to the bowl; it
 * reads as mesh from 32 px up and as a shaded bowl below.
 */

export const GRID = 32;

/** The rim: a straight top edge a little wider than the bowl. */
export const RIM = "M3.5 11.2H28.5";
/** The bowl: half an ellipse under the rim (bottom at y = 20.7). */
export const BOWL = "M5.5 11.2A10.5 9.5 0 0 0 26.5 11.2";
/** The bowl's area, the net's clip. */
export const BOWL_AREA = `${BOWL}Z`;

/** x - y = c - 4 and x + y = 36 - c: mirror images about x = 16, one crossing at (16, 16). */
const NET_C = [-11, -6, -1, 4, 9, 14, 19];
export const NET = NET_C.map((c) => `M${c + 4} 8L${c + 20} 24`)
  .concat(NET_C.map((c) => `M${28 - c} 8L${12 - c} 24`))
  .join("");

export const STROKE = { frame: 2.6, net: 1 } as const;

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
  /** Output size in px (the viewBox stays the 32-unit grid). */
  size?: number;
  /** Unique id prefix for the clip path, when several marks share one document. */
  id?: string;
};

/** The mark as a standalone SVG document. */
export function markSvg(o: MarkOptions = {}): string {
  const ink = o.ink ?? BRAND.ink;
  const size = o.size ?? 1024;
  const id = o.id ?? "sift-bowl";
  // On the tile the mark is scaled to 84 % around the centre, the margin Windows' own icons keep.
  const body =
    `<defs><clipPath id="${id}"><path d="${BOWL_AREA}"/></clipPath></defs>` +
    `<g clip-path="url(#${id})"><path d="${NET}" fill="none" stroke="${ink}" stroke-width="${STROKE.net}" stroke-opacity="0.82"/></g>` +
    `<path d="${RIM}${BOWL}" fill="none" stroke="${ink}" stroke-width="${STROKE.frame}" stroke-linecap="round" stroke-linejoin="round"/>`;
  const tile = o.tile ? `<rect width="32" height="32" rx="7" fill="${BRAND.tile}"/>` : "";
  const placed = o.tile ? `<g transform="translate(16 16) scale(0.84) translate(-16 -16)">${body}</g>` : body;
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 ${GRID} ${GRID}">${tile}${placed}</svg>`;
}
