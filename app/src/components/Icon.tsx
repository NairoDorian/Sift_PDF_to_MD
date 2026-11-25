import type { JSX } from "@solidjs/web";

/* Sift's icons: drawn for this app on a 24-unit grid, 1.7 stroke, round caps (the weight of
   ZER0's icons; no icon library in the dependency list). */
const PATHS = {
  sieve: "M3 7.5h18M4.6 7.5a7.4 7 0 0 0 14.8 0M8.5 7.5l3.5 6.5 3.5-6.5M7 18.5h6M7 21h9",
  setup: "M10 6h10M10 12h10M10 18h10M4 6l1.5 1.5L8 5M4 12l1.5 1.5L8 11M4 18l1.5 1.5L8 17",
  engine: "M7 7h10v10H7zM10.5 10.5h3v3h-3zM10 3v4M14 3v4M10 17v4M14 17v4M3 10h4M3 14h4M17 10h4M17 14h4",
  models: "M12 3l8 4.5v9L12 21l-8-4.5v-9zM12 12l8-4.5M12 12v9M12 12L4 7.5",
  settings: "M4 7h8M16 7h4M4 17h3M11 17h9M12 5h4v4h-4zM7 15h4v4H7z",
  logs: "M4 5h16v14H4zM7.5 9.5l3 2.5-3 2.5M12.5 15h4",
  about: "M12 3a9 9 0 1 0 0 18a9 9 0 1 0 0-18zM12 11v5.5M12 7.6v.2",
  pdf: "M6 3h8l4 4v14H6zM14 3v4h4M9 12h6M9 15h6M9 18h3",
  folder: "M3 6h7l2 2h9v11H3z",
  markdown: "M6 3h8l4 4v14H6zM14 3v4h4M10 11v7M13.5 11v7M8.5 13.2h7M8.5 15.8h7",
  x: "M6 6l12 12M18 6L6 18",
  check: "M5 12.5l4.5 4.5L19 7",
  play: "M7 5v14l12-7z",
  stop: "M6.5 6.5h11v11h-11z",
  refresh: "M20 12a8 8 0 1 1-2.4-5.7L20 8.5M20 3.5v5h-5",
  download: "M12 4v11M7 10l5 5 5-5M5 20h14",
  trash: "M4 7h16M10 11v6M14 11v6M6 7l1 13h10l1-13M9 7V4h6v3",
  verify: "M12 3l8 3v6c0 4.5-3.4 8-8 9-4.6-1-8-4.5-8-9V6zM8.5 12l2.5 2.5 4.5-5",
  external: "M14 4h6v6M20 4l-9 9M18 14v6H4V6h6",
  open: "M5 4h9l5 5v11H5zM14 4v5h5M9 14l3 3 3-3M12 10v7",
  collapse: "M11 17l-5-5 5-5M18 17l-5-5 5-5",
  expand: "M13 7l5 5-5 5M6 7l5 5-5 5",
  copy: "M8 8h12v12H8zM16 8V4H4v12h4",
  report: "M8 3h8v4H8zM6 5H4v16h16V5h-2M8 12h8M8 16h5",
  alert: "M12 3l10 18H2zM12 10v5M12 18v.2",
  clock: "M12 3a9 9 0 1 0 0 18a9 9 0 1 0 0-18zM12 7v5l3.5 2",
  plus: "M12 5v14M5 12h14",
  undo: "M9 14l-5-5 5-5M4 9h10a6 6 0 0 1 0 12h-3",
} as const;

export type IconName = keyof typeof PATHS;

export function Icon(props: { name: IconName; size?: number; class?: string; title?: string }): JSX.Element {
  return (
    <svg
      width={props.size ?? 16}
      height={props.size ?? 16}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.7"
      stroke-linecap="round"
      stroke-linejoin="round"
      class={props.class}
      role={props.title ? "img" : undefined}
      aria-label={props.title}
      aria-hidden={props.title ? undefined : "true"}
    >
      <path d={PATHS[props.name]} />
    </svg>
  );
}
