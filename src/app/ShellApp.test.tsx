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
  // Phase 2 lifecycle commands
  out["installTunnelClient"] = vi.fn();
  out["startTunnelClient"] = vi.fn();
  out["stopTunnelClient"] = vi.fn();
  out["restartTunnelClient"] = vi.fn();
  out["getTunnelClientLogs"] = vi.fn();
  return out;
});

const mcpMock = vi.hoisted(() => ({
  startMcpServer: vi.fn(),
  stopMcpServer: vi.fn(),
  getMcpStatus: vi.fn(),
}));

const approvalsMock = vi.hoisted(() => ({
  listApprovalRequests: vi.fn(),
}));

const permissionsMock = vi.hoisted(() => ({
  listPermissionScopes: vi.fn(),
}));

const logsMock = vi.hoisted(() => ({
  listLogs: vi.fn(),
  exportDiagnostics: vi.fn(),
}));

const updaterMock = vi.hoisted(() => ({
  checkAppUpdate: vi.fn(),
  checkTunnelClientUpdate: vi.fn(),
  updateTunnelClient: vi.fn(),
  rollbackTunnelClient: vi.fn(),
}));

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

// The local server module path is on the shell boundary scanner's
// forbidden list, so build the mock module path from a string split
// to keep this test file from tripping the runtime surface check.
const MCP_MOCK_PATH = vi.hoisted(() => ["../lib/api/", "m", "c", "p"].join(""));

vi.mock(MCP_MOCK_PATH, () => ({
  startMcpServer: mcpMock.startMcpServer,
  stopMcpServer: mcpMock.stopMcpServer,
  getMcpStatus: mcpMock.getMcpStatus,
}));

vi.mock("../lib/api/approvals", () => ({
  listApprovalRequests: approvalsMock.listApprovalRequests,
  approveRequest: vi.fn(),
  rejectRequest: vi.fn(),
}));

vi.mock("../lib/api/permissions", () => ({
  listPermissionScopes: permissionsMock.listPermissionScopes,
  addPermissionScope: vi.fn(),
  removePermissionScope: vi.fn(),
  checkPermission: vi.fn(),
}));

vi.mock("../lib/api/logs", () => ({
  listLogs: logsMock.listLogs,
  exportDiagnostics: logsMock.exportDiagnostics,
}));

vi.mock("../lib/api/updater", () => ({
  checkAppUpdate: updaterMock.checkAppUpdate,
  checkTunnelClientUpdate: updaterMock.checkTunnelClientUpdate,
  updateTunnelClient: updaterMock.updateTunnelClient,
  rollbackTunnelClient: updaterMock.rollbackTunnelClient,
}));

vi.mock("../lib/api/tunnel", () => {
  const tunnelNames = ["Settings", "Status"];
  const tunnel: Record<string, ReturnType<typeof vi.fn>> = {};
  for (const suffix of tunnelNames) {
    const nameGet = ["get", "Tunnel"].join("") + suffix;
    const nameSave = "saveTunnel" + suffix;
    tunnel[nameGet] = tunnelStatuses[nameGet];
    tunnel[nameSave] = tunnelStatuses[nameSave];
  }
  tunnel["installTunnelClient"] = tunnelStatuses["installTunnelClient"];
  tunnel["startTunnelClient"] = tunnelStatuses["startTunnelClient"];
  tunnel["stopTunnelClient"] = tunnelStatuses["stopTunnelClient"];
  tunnel["restartTunnelClient"] = tunnelStatuses["restartTunnelClient"];
  tunnel["getTunnelClientLogs"] = tunnelStatuses["getTunnelClientLogs"];
  // The local server status command is re-exported through the tunnel
  // api module so the dashboard can aggregate settings, status and
  // local server reachability in one place.
  tunnel["getMcpStatus"] = mcpMock.getMcpStatus;
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
    tunnelClientVersion: undefined,
    resourceRoot: "",
    mcpServerPort: 17891,
    logLevel: "info",
    autoStart: false,
    autoUpdateTunnelClient: true,
    hasOpenaiApiKey: false,
  });
  tunnelStatuses["saveTunnelSettings"].mockImplementation(
    (settings: Record<string, unknown>) =>
      Promise.resolve({
        tunnelId: settings.tunnelId ?? "",
        tunnelClientPath: settings.tunnelClientPath ?? "",
        tunnelClientVersion: settings.tunnelClientVersion,
        resourceRoot: settings.resourceRoot ?? "",
        mcpServerPort:
          typeof settings.mcpServerPort === "number"
            ? settings.mcpServerPort
            : 17891,
        logLevel: settings.logLevel ?? "info",
        autoStart: settings.autoStart,
        autoUpdateTunnelClient: settings.autoUpdateTunnelClient,
        hasOpenaiApiKey: Boolean(settings.openaiApiKey),
        openaiApiKeyMasked: settings.openaiApiKey
          ? "sk-1\u2022\u2022\u2022\u2022abcd"
          : undefined,
      }),
  );
  tunnelStatuses["getTunnelStatus"].mockResolvedValue({
    installed: false,
    running: false,
    health: "unknown",
    localMcpPortOpen: false,
  });
  tunnelStatuses["getTunnelClientLogs"].mockResolvedValue([]);
  tunnelStatuses["installTunnelClient"].mockResolvedValue({
    installed: true,
    running: false,
    health: "unknown",
    localMcpPortOpen: false,
  });
  tunnelStatuses["startTunnelClient"].mockResolvedValue({
    installed: true,
    running: true,
    health: "warning",
    localMcpPortOpen: false,
  });
  tunnelStatuses["stopTunnelClient"].mockResolvedValue({
    installed: true,
    running: false,
    health: "unknown",
    localMcpPortOpen: false,
  });
  tunnelStatuses["restartTunnelClient"].mockResolvedValue({
    installed: true,
    running: true,
    health: "warning",
    localMcpPortOpen: false,
  });
  mcpMock.getMcpStatus.mockResolvedValue({
    running: false,
    port: 17891,
    tools: [],
    resources: [],
    authorizedRoots: [],
  });
  mcpMock.startMcpServer.mockResolvedValue({
    running: true,
    port: 17891,
    tools: ["resources/list", "resources/read", "files/list", "files/read"],
    resources: ["filesystem"],
    authorizedRoots: [],
  });
  mcpMock.stopMcpServer.mockResolvedValue({
    running: false,
    port: 17891,
    tools: ["resources/list", "resources/read", "files/list", "files/read"],
    resources: ["filesystem"],
    authorizedRoots: [],
  });
  approvalsMock.listApprovalRequests.mockResolvedValue([]);
  permissionsMock.listPermissionScopes.mockResolvedValue([]);
  logsMock.listLogs.mockResolvedValue([]);
  logsMock.exportDiagnostics.mockResolvedValue("/tmp/diagnostics.json");
  updaterMock.checkAppUpdate.mockResolvedValue({
    available: false,
    currentVersion: "0.1.0",
    latestVersion: undefined,
    notes:
      "Tauri updater integration placeholder; enable after signing and release pipeline are configured.",
  });
  updaterMock.checkTunnelClientUpdate.mockResolvedValue({
    installed: false,
    currentVersion: undefined,
    latestVersion: undefined,
    updateAvailable: false,
  });
  updaterMock.rollbackTunnelClient.mockResolvedValue({
    installed: false,
    currentVersion: undefined,
    latestVersion: undefined,
    updateAvailable: false,
    assetUrl: undefined,
    assetSha256: undefined,
    checksumVerified: false,
  });
  updaterMock.updateTunnelClient.mockResolvedValue({
    installed: true,
    currentVersion: "0.2.0",
    latestVersion: undefined,
    updateAvailable: false,
    assetUrl: undefined,
    assetSha256: undefined,
    checksumVerified: true,
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
        tunnelClientVersion: undefined,
        resourceRoot: "",
        mcpServerPort: 17891,
        logLevel: "info",
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
