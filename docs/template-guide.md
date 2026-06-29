# Template Guide

This guide explains how to use Desktop Shell Template to create a new desktop app.

## 1. Create your app

Clone or fork this repository.

Install dependencies:

```bash
pnpm install
```

Run the app:

```bash
pnpm dev
```

## 2. Rename the app

Edit:

```txt
template.config.ts
```

Example:

```ts
export default {
  appName: "Acme Desk",
  packageName: "acme-desk",
  productName: "Acme Desk",
  identifier: "com.acme.desk",
  description: "Acme desktop application.",
  repositoryUrl: "https://github.com/acme/acme-desk",
  deepLinkScheme: "acme-desk",
  updaterEndpoint:
    "https://github.com/acme/acme-desk/releases/latest/download/latest.json",
};
```

Then run:

```bash
pnpm sync:template
```

Check sync status:

```bash
pnpm check:template-config
```

## 3. Add a page

Create a page:

```tsx
// src/pages/LogsPage.tsx
import { Page } from "../components/layout/Page";
import { Section } from "../components/layout/Section";
import type { ShellPageProps } from "../app/routes";

export function LogsPage(_props: ShellPageProps) {
  return (
    <Page title="Logs" description="View local application logs.">
      <Section title="Application Logs">
        <p>Logs will be displayed here.</p>
      </Section>
    </Page>
  );
}
```

Register it:

```tsx
// src/app/routes.tsx
import { LogsPage } from "../pages/LogsPage";

export type RouteId = "dashboard" | "settings" | "about" | "logs";

export const routes = [
  {
    id: "dashboard",
    label: "Dashboard",
    title: "Dashboard",
    component: DashboardPage,
  },
  {
    id: "settings",
    label: "Settings",
    title: "Settings",
    component: SettingsPage,
  },
  {
    id: "about",
    label: "About",
    title: "About",
    component: AboutPage,
  },
  {
    id: "logs",
    label: "Logs",
    title: "Logs",
    component: LogsPage,
  },
];
```

Add a test:

```tsx
// src/app/ShellApp.test.tsx
it("navigates to logs", async () => {
  const user = userEvent.setup();

  renderApp();

  await user.click(screen.getByRole("button", { name: "Logs" }));

  expect(
    await screen.findByRole("heading", { name: "Logs" }),
  ).toBeInTheDocument();
});
```

Run:

```bash
pnpm test:unit
```

## 4. Add a setting

Frontend model:

```ts
// src/lib/settings/settings.ts
export interface ShellSettings {
  theme: ThemeMode;
  startMinimized: boolean;
  enableLogs: boolean;
}

export const defaultShellSettings: ShellSettings = {
  theme: "system",
  startMinimized: false,
  enableLogs: false,
};
```

Rust model:

```rust
// src-tauri/src/shell/settings_store.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShellSettings {
    pub theme: ThemeMode,
    pub start_minimized: bool,
    pub enable_logs: bool,
}

impl Default for ShellSettings {
    fn default() -> Self {
        Self {
            theme: ThemeMode::System,
            start_minimized: false,
            enable_logs: false,
        }
    }
}
```

Settings page:

```tsx
// src/pages/SettingsPage.tsx
<label className="flex items-center gap-2 text-sm">
  <Switch
    checked={draft.enableLogs}
    onCheckedChange={(checked) =>
      setDraft((current) => ({
        ...current,
        enableLogs: checked,
      }))
    }
    aria-label="Enable application logs"
  />
  <span>Enable logs</span>
</label>
```

Run:

```bash
pnpm typecheck
pnpm test:unit
cd src-tauri && cargo test
```

## 5. Add a Tauri command

Rule:

```txt
Frontend invoke only lives in src/lib/api/shell.ts.
Rust command adapter lives in src-tauri/src/commands/.
Business-free shell logic lives in src-tauri/src/shell/.
```

Add shell logic:

```rust
// src-tauri/src/shell/system_info.rs
#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub os: String,
}

pub fn get_system_info() -> SystemInfo {
    SystemInfo {
        os: std::env::consts::OS.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_info_should_have_os() {
        assert!(!get_system_info().os.is_empty());
    }
}
```

Export it:

```rust
// src-tauri/src/shell/mod.rs
pub mod system_info;
```

Add command:

```rust
// src-tauri/src/commands/shell.rs
#[tauri::command]
pub async fn get_system_info() -> Result<crate::shell::system_info::SystemInfo, String> {
    Ok(crate::shell::system_info::get_system_info())
}
```

Register command:

```rust
// src-tauri/src/lib.rs
.invoke_handler(tauri::generate_handler![
    commands::app::get_app_info,
    commands::app::open_external,
    commands::settings::get_settings,
    commands::settings::save_settings,
    commands::shell::update_tray_menu,
    commands::shell::get_system_info,
])
```

Update runtime boundary:

```rust
// src-tauri/src/shell/runtime_boundary.rs
pub const SHELL_ALLOWED_COMMANDS: &[&str] = &[
    "get_app_info",
    "open_external",
    "get_settings",
    "save_settings",
    "update_tray_menu",
    "get_system_info",
];
```

Add frontend API:

```ts
// src/lib/api/shell.ts
export interface SystemInfo {
  os: string;
}

export const shellApi = {
  // existing methods
  getSystemInfo(): Promise<SystemInfo> {
    return invoke<SystemInfo>("get_system_info");
  },
};
```

Run:

```bash
pnpm typecheck
pnpm test:unit
cd src-tauri && cargo test
```

## 6. Add UI components

Generic UI goes under:

```txt
src/components/ui/
```

Layout components go under:

```txt
src/components/layout/
```

Shell-specific components go under:

```txt
src/components/titlebar/
src/components/settings/
```

Avoid product-specific names in `components/ui`.

Good:

```txt
Button
Card
Switch
native select/input when simple enough
```

Add heavier primitives only when needed:

```txt
Dialog
Tooltip
Radix Select
```

Avoid:

```txt
ProviderCard
McpList
PromptEditor
UsageChart
SubscriptionPanel
```

## 7. Add dependencies

Before adding a dependency, ask:

```txt
1. Is it required by the minimal shell?
2. Is it product-specific?
3. Can it be optional?
4. Does it violate check-template-deps?
```

Run:

```bash
pnpm check:template-deps
```

For Rust:

```bash
cd src-tauri
cargo check
cargo test
cargo tree
```

## 8. Build

Build full app:

```bash
pnpm build
```

Build renderer only:

```bash
pnpm build:renderer
```

## 9. Update configuration

The updater endpoint is generated from:

```txt
template.config.ts
```

The generated Tauri config contains:

```json
{
  "plugins": {
    "updater": {
      "endpoints": ["..."]
    }
  }
}
```

This template does not generate release artifacts or latest.json. You need to add release automation for production.

## 10. Before committing

Run:

```bash
pnpm check:all
```

If you only changed docs:

```bash
pnpm check:docs
```
