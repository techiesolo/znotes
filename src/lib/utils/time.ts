const UNITS: [limit: number, divisor: number, label: string][] = [
  [60, 1, "s"],
  [3600, 60, "m"],
  [86400, 3600, "h"],
  [86400 * 30, 86400, "d"],
];

export function formatRelative(unixSeconds: number, nowSeconds = Date.now() / 1000): string {
  const diff = Math.max(0, Math.floor(nowSeconds - unixSeconds));
  if (diff < 5) return "just now";
  for (const [limit, divisor, label] of UNITS) {
    if (diff < limit) return `${Math.floor(diff / divisor)}${label} ago`;
  }
  return new Date(unixSeconds * 1000).toLocaleDateString();
}
