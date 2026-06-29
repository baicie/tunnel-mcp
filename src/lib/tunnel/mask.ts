export function maskSecret(value?: string): string | undefined {
  const normalized = value?.trim();
  if (!normalized) return undefined;

  const chars = Array.from(normalized);
  if (chars.length <= 8) return "\u2022\u2022\u2022\u2022";

  return `${chars.slice(0, 4).join("")}\u2022\u2022\u2022\u2022${chars.slice(-4).join("")}`;
}
