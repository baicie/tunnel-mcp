export function maskSecret(value?: string): string | undefined {
  const normalized = value?.trim();

  if (!normalized) {
    return undefined;
  }

  const chars = Array.from(normalized);

  if (chars.length <= 8) {
    return "••••";
  }

  return `${chars.slice(0, 4).join("")}••••${chars.slice(-4).join("")}`;
}
