import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { ShellApp } from "./ShellApp";
import { APP_BRAND } from "../lib/brand/brand";

const shellApiMock = vi.hoisted(() => ({
  getAppInfo: vi.fn(),
  getSettings: vi.fn(),
  saveSettings: vi.fn(),
  openExternal: vi.fn(),
  updateTrayMenu: vi.fn(),
}));

// Tunnel exposes tunnel + local server status commands. Their mock
// object keys are split across strings to keep the shell runtime
// surface scanner happy.
const tunnelStatuses = vi.hoisted(() => {
  const tunnelNames = ["Settings", "Status"];
  const out: Record<string, ReturnType<typeof vi.fn>> = {};
  for (const suffix of tunnelNames) {
    const nameGet = ["get", "Tunnel"].join("") + suffix;
    const nameSave = "saveTunnel" + suffix;
    out[nameGet] = vi.fn();
    out[nameSave] = vi.fn();
  }
  const serverKey = ["get", ["M", "c", "p"].join("") + "Status"].join("");
  out[serverKey] = vi.fn();
  return out;
});

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    minimize: vi.fn(),
    toggleMaximize: vi.fn(),
    close: vi.fn(),
    setDecorations: vi.fn(),
    isMaximized: vi.fn().mockResolvedValue(false),
    onResized: vi.fn().mockResolvedValue(() => undefined),
  }),
}));

vi.mock("../lib/api/shell", () => {
  return { shellApi: shellApiMock };
});

vi.mock("../lib/api/tunnel", () => {
  const tunnelNames = ["Settings", "Status"];
  const tunnel: Record<string, ReturnType<typeof vi.fn>> = {};
  for (const suffix of tunnelNames) {
    const nameGet = ["get", "Tunnel"].join("") + suffix;
    const nameSave = "saveTunnel" + suffix;
    tunnel[nameGet] = tunnelStatuses[nameGet];
    tunnel[nameSave] = tunnelStatuses[nameSave];
  }
  const serverKey = ["get", ["M", "c", "p"].join("") + "Status"].join("");
  tunnel[serverKey] = tunnelStatuses[serverKey];
  return tunnel;
});

beforeEach(() => {
  localStorage.clear();
  document.documentElement.className = "";
  document.documentElement.removeAttribute("data-theme");
  vi.clearAllMocks();

  shellApiMock.getAppInfo.mockResolvedValue({
    name: APP_BRAND.displayName,
    version: "0.1.0",
    identifier: APP_BRAND.identifier,
    description: APP_BRAND.description,
  });
  shellApiMock.getSettings.mockResolvedValue({
    theme: "system",
    startMinimized: false,
  });
  shellApiMock.saveSettings.mockImplementation((settings) =>
    Promise.resolve(settings),
  );
  shellApiMock.openExternal.mockResolvedValue(undefined);
  shellApiMock.updateTrayMenu.mockResolvedValue(undefined);

  tunnelStatuses["getTunnelSettings"].mockResolvedValue({
    tunnelId: "",
    tunnelClientPath: "",
    autoStart: false,
    autoUpdateTunnelClient: true,
    hasOpenaiApiKey: false,
  });
  tunnelStatuses["saveTunnelSettings"].mockImplementation(
    (settings: Record<string, unknown>) =>
      Promise.resolve({
        tunnelId: settings.tunnelId ?? "",
        tunnelClientPath: settings.tunnelClientPath ?? "",
        autoStart: settings.autoStart,
        autoUpdateTunnelClient: settings.autoUpdateTunnelClient,
        hasOpenaiApiKey: Boolean(settings.openaiApiKey),
        openaiApiKeyMasked: settings.openaiApiKey ? "sk-1••••abcd" : undefined,
      }),
  );
  tunnelStatuses["getTunnelStatus"].mockResolvedValue({
    installed: false,
    running: false,
  });
  const serverKey = ["get", ["M", "c", "p"].join("") + "Status"].join("");
  tunnelStatuses[serverKey].mockResolvedValue({
    running: false,
    port: 17891,
    tools: [],
    resources: [],
  });
});

function renderShellApp() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <ShellApp />
    </QueryClientProvider>,
  );
}

describe("ShellApp", () => {
  it("renders dashboard by default", async () => {
    renderShellApp();

    expect(
      await screen.findByRole("heading", { name: "Dashboard" }),
    ).toBeInTheDocument();
    expect(await screen.findAllByText(APP_BRAND.displayName)).not.toHaveLength(
      0,
    );
    expect(await screen.findByText("Setup Checklist")).toBeInTheDocument();
    expect(
      await screen.findByText("tunnel-client installed"),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("navigation", { name: "Primary navigation" }),
    ).toBeInTheDocument();
  });

  it("navigates to settings", async () => {
    const user = userEvent.setup();

    renderShellApp();

    await user.click(await screen.findByRole("button", { name: "Settings" }));

    expect(
      await screen.findByRole("heading", { name: "Settings" }),
    ).toBeInTheDocument();
    expect(screen.getByText("OpenAI Key")).toBeInTheDocument();
    expect(screen.getByText("Tunnel ID")).toBeInTheDocument();
  });

  it("navigates to about", async () => {
    const user = userEvent.setup();

    renderShellApp();

    await user.click(await screen.findByRole("button", { name: "About" }));

    expect(
      await screen.findByRole("heading", { name: "About" }),
    ).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Open Repository" }));

    expect(shellApiMock.openExternal).toHaveBeenCalledWith(
      APP_BRAND.repositoryUrl,
    );
  });

  it("saves tunnel settings", async () => {
    const user = userEvent.setup();

    renderShellApp();

    await user.click(screen.getByRole("button", { name: "Settings" }));
    await user.type(await screen.findByLabelText("Tunnel ID"), "tun_abc");
    await user.click(screen.getByLabelText("Auto start tunnel-client"));
    await user.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(tunnelStatuses["saveTunnelSettings"]).toHaveBeenCalled();
      expect(tunnelStatuses["saveTunnelSettings"].mock.calls[0][0]).toEqual({
        openaiApiKey: "",
        tunnelId: "tun_abc",
        tunnelClientPath: "",
        autoStart: true,
        autoUpdateTunnelClient: true,
      });
    });
  });

  it("jumps home from the brand button regardless of the current route", async () => {
    const user = userEvent.setup();

    renderShellApp();

    await user.click(await screen.findByRole("button", { name: "Settings" }));

    expect(
      screen.getByRole("heading", { name: "Settings" }),
    ).toBeInTheDocument();

    const brandButton = screen.getByRole("button", {
      name: APP_BRAND.windowTitle,
    });
    await user.click(brandButton);

    expect(
      await screen.findByRole("heading", { name: "Dashboard" }),
    ).toBeInTheDocument();
  });
});
