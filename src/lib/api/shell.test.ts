import { beforeEach, describe, expect, it, vi } from "vitest";
import { shellApi } from "./shell";
import { invoke } from "@tauri-apps/api/core";
import { APP_BRAND } from "../brand/brand";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const invokeMock = vi.mocked(invoke);

describe("shellApi", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("calls get_app_info", async () => {
    invokeMock.mockResolvedValueOnce({
      name: APP_BRAND.displayName,
      version: "0.1.0",
    });

    await expect(shellApi.getAppInfo()).resolves.toEqual({
      name: APP_BRAND.displayName,
      version: "0.1.0",
    });

    expect(invokeMock).toHaveBeenCalledWith("get_app_info");
  });

  it("calls open_external with url", async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await shellApi.openExternal("https://github.com");

    expect(invokeMock).toHaveBeenCalledWith("open_external", {
      url: "https://github.com",
    });
  });

  it("calls get_settings", async () => {
    invokeMock.mockResolvedValueOnce({
      theme: "system",
      startMinimized: false,
    });

    await expect(shellApi.getSettings()).resolves.toEqual({
      theme: "system",
      startMinimized: false,
    });

    expect(invokeMock).toHaveBeenCalledWith("get_settings");
  });

  it("calls save_settings with settings payload", async () => {
    const settings = {
      theme: "dark" as const,
      startMinimized: true,
    };

    invokeMock.mockResolvedValueOnce(settings);

    await expect(shellApi.saveSettings(settings)).resolves.toEqual(settings);

    expect(invokeMock).toHaveBeenCalledWith("save_settings", {
      settings,
    });
  });

  it("calls update_tray_menu", async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await shellApi.updateTrayMenu();

    expect(invokeMock).toHaveBeenCalledWith("update_tray_menu");
  });
});
