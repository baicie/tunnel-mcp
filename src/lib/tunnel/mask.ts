export function maskSecret(value?: string): string | undefined {
  if (!value) return undefined;
  if (value.length <= 8) return "••••";
  return `${value.slice(0, 4)}••••${value.slice(-4)}`;
}
