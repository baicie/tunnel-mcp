import { invoke } from "@tauri-apps/api/core";
import type { ShellSettings } from "../settings/settings";

export interface AppInfo {
  name: string;
  version: string;
  identifier?: string;
  description?: string;
}

export const shellApi = {
  getAppInfo(): Promise<AppInfo> {
    return invoke<AppInfo>("get_app_info");
  },

  openExternal(url: string): Promise<void> {
    return invoke<void>("open_external", { url });
  },

  getSettings(): Promise<ShellSettings> {
    return invoke<ShellSettings>("get_settings");
  },

  saveSettings(settings: ShellSettings): Promise<ShellSettings> {
    return invoke<ShellSettings>("save_settings", { settings });
  },

  updateTrayMenu(): Promise<void> {
    return invoke<void>("update_tray_menu");
  },
};
