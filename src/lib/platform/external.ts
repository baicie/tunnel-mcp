import { shellApi } from "../api/shell";

export function openExternal(url: string): Promise<void> {
  return shellApi.openExternal(url);
}
