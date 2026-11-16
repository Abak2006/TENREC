export function formatBytes(bytes: number | null | undefined, digits = 1): string {
  if (bytes == null || !Number.isFinite(bytes)) return '–';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let v = Math.abs(bytes);
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i++;
  }
  return `${v.toFixed(i === 0 ? 0 : digits)} ${units[i]}`;
}

export const formatRate = (bps: number | null | undefined) => (bps == null ? '–' : `${formatBytes(bps)}/s`);

export const formatPercent = (v: number | null | undefined, digits = 0) =>
  v == null || !Number.isFinite(v) ? '–' : `${v.toFixed(digits)}%`;

export function formatDuration(seconds: number): string {
  const d = Math.floor(seconds / 86400);
  const h = Math.floor((seconds % 86400) / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (d > 0) return `${d}d ${h}h`;
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

export function formatRelative(iso: string | null | undefined, now = Date.now()): string {
  if (!iso) return '–';
  const t = Date.parse(iso);
  if (Number.isNaN(t)) return '–';
  const s = Math.round((now - t) / 1000);
  if (s < 10) return 'just now';
  if (s < 60) return `${s}s ago`;
  if (s < 3600) return `${Math.floor(s / 60)}m ago`;
  if (s < 86400) return `${Math.floor(s / 3600)}h ago`;
  return new Date(t).toLocaleDateString();
}

export function formatTime(iso: string | null | undefined): string {
  if (!iso) return '–';
  const t = new Date(iso);
  return Number.isNaN(t.getTime()) ? '–' : t.toLocaleString();
}

export const endpoint = (address: string, port: number) => (address.includes(':') ? `[${address}]:${port}` : `${address}:${port}`);
