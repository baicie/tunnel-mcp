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
    expect(await screen.findByText("0.1.0")).toBeInTheDocument();
    expect(screen.getByText("Ready")).toBeInTheDocument();
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
    expect(screen.getByText("Start minimized")).toBeInTheDocument();
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

  it("saves settings", async () => {
    const user = userEvent.setup();

    renderShellApp();

    await user.click(screen.getByRole("button", { name: "Settings" }));
    // ThemeSelect renders a Radix Select combobox; switch through the
    // popover instead of `selectOptions`, which only works on native
    // <select> elements.
    await user.click(screen.getByLabelText("Theme"));
    await user.click(await screen.findByRole("option", { name: "Dark" }));
    await user.click(screen.getByRole("button", { name: "Save Settings" }));

    await waitFor(() => {
      expect(shellApiMock.saveSettings).toHaveBeenCalled();
      expect(shellApiMock.saveSettings.mock.calls[0][0]).toEqual({
        theme: "dark",
        startMinimized: false,
      });
    });

    await waitFor(() => {
      expect(localStorage.getItem(`${APP_BRAND.packageName}.theme`)).toBe(
        "dark",
      );
      expect(document.documentElement.dataset.theme).toBe("dark");
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
