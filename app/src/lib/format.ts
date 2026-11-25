/** Display helpers: sizes, durations, paths (Windows or forward slashes). */

export const gb = (b: number) => `${(b / 1e9).toFixed(b >= 1e10 ? 1 : 2)} GB`;
export const mb = (b: number) => `${Math.round(b / 1e6)} MB`;
export const size = (b: number) => (b >= 1e9 ? gb(b) : b >= 1e6 ? mb(b) : `${Math.max(1, Math.round(b / 1e3))} KB`);

export function clock(ms: number): string {
  const s = Math.max(0, Math.floor(ms / 1000));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const ss = String(s % 60).padStart(2, "0");
  return h ? `${h}:${String(m).padStart(2, "0")}:${ss}` : `${m}:${ss}`;
}

export const seconds = (s: number) => (s >= 90 ? clock(s * 1000) : `${s.toFixed(1)} s`);

export function fileName(p: string): string {
  const parts = p.split(/[\\/]/).filter(Boolean);
  return parts.at(-1) ?? p;
}

export function dirName(p: string): string {
  const i = Math.max(p.lastIndexOf("\\"), p.lastIndexOf("/"));
  return i > 0 ? p.slice(0, i) : "";
}

/** `dir\name`, without doubling the separator when `dir` already ends with one (`S:\`). */
export function joinPath(dir: string, name: string): string {
  return `${dir.replace(/[\\/]+$/, "")}\\${name}`;
}

export function stem(p: string): string {
  const n = fileName(p);
  const i = n.lastIndexOf(".");
  return i > 0 ? n.slice(0, i) : n;
}

export const plural = (n: number, one: string, many = `${one}s`) => `${n} ${n === 1 ? one : many}`;
