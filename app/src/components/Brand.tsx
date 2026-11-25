import type { JSX } from "@solidjs/web";
import { BOWL, BOWL_AREA, NET, RIM, STROKE, TEXT_LINES } from "../lib/brand";

let instances = 0;

/** The mark, drawn from lib/brand.ts (the same geometry as the app icon): the strainer in the
 * text colour, its two lines of text in the accent. */
export function BrandMark(props: { size?: number; title?: string }): JSX.Element {
  const clip = `sift-bowl-${++instances}`;
  return (
    <svg
      width={props.size ?? 28}
      height={props.size ?? 28}
      viewBox="0 0 32 32"
      role={props.title ? "img" : undefined}
      aria-label={props.title}
      aria-hidden={props.title ? undefined : "true"}
      style={{ flex: "none" }}
    >
      <defs>
        <clipPath id={clip}>
          <path d={BOWL_AREA} />
        </clipPath>
      </defs>
      <g clip-path={`url(#${clip})`}>
        <path d={NET} fill="none" stroke="currentColor" stroke-width={STROKE.net} stroke-opacity="0.82" />
      </g>
      <path d={RIM + BOWL} fill="none" stroke="currentColor" stroke-width={STROKE.frame} stroke-linecap="round" stroke-linejoin="round" />
      <path d={TEXT_LINES} fill="none" stroke="var(--accent)" stroke-width={STROKE.text} stroke-linecap="round" />
    </svg>
  );
}

export function Lockup(props: { size?: number }): JSX.Element {
  return (
    <span class="row" style={{ gap: "10px", "flex-wrap": "nowrap" }}>
      <BrandMark size={props.size ?? 28} />
      <span class="wordmark">Sift</span>
    </span>
  );
}
