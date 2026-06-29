# Desktop Shell Template

A minimal Tauri + React desktop application shell template.

It gives you a clean desktop app foundation with:

- Tauri 2 backend
- React renderer
- Shell layout
- Dashboard / Settings / About pages
- Window controls
- Theme settings
- App identity sync from one template config
- Shell boundary checks
- Frontend and Rust unit tests

## What is this

Desktop Shell Template is a reusable starter for building small to medium desktop applications.

It focuses on the shell layer:

```txt
app identity
window shell
layout
routing
settings
Tauri command boundary
tray placeholder
updater placeholder
template checks
```

It does not include any product-specific business modules.

## Features

```txt
1. React + Tauri desktop shell
2. Dashboard / Settings / About default pages
3. Shell layout with sidebar and titlebar
4. Theme mode setting
5. Tauri command wrapper through shellApi
6. Rust command boundary
7. App identity generated from template.config.ts
8. Brand and template dependency checks
9. Frontend unit tests
10. Rust unit tests
```

## Good for

This template is suitable for:

```txt
1. Local-first desktop tools
2. Developer tools
3. Admin consoles
4. Small internal desktop apps
5. Tauri proof-of-concepts
6. Desktop shells that later embed product modules
```

## Not good for

This template intentionally does not include:

```txt
1. Account system
2. Cloud backend
3. Database layer
4. MCP / provider / proxy / prompt management
5. Code editor
6. Chart dashboard
7. Command palette
8. i18n
9. Auto-update release pipeline
10. Complex plugin system
```

Those can be added later as product-specific modules.

## Quick Start

Install dependencies:

```bash
pnpm install
```

Start the desktop app:

```bash
pnpm dev
```

Run frontend checks:

```bash
pnpm check:template
```

Run Rust tests:

```bash
pnpm test:tauri
```

Run all checks:

```bash
pnpm check:all
```

## Rename Your App

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

Check whether generated files are in sync:

```bash
pnpm check:template-config
```

The sync command updates:

```txt
package.json
README.md
src/lib/brand/templateConfig.ts
src/lib/brand/brand.ts
src/lib/settings/settings.ts
src-tauri/Cargo.toml
src-tauri/tauri.conf.json
src-tauri/src/shell/brand.rs
```

## Project Structure

```txt
src/
  app/
    ShellApp.tsx
    routes.tsx
    layout/

  pages/
    DashboardPage.tsx
    SettingsPage.tsx
    AboutPage.tsx

  components/
    ui/
    layout/
    titlebar/
    settings/

  lib/
    api/
    brand/
    query/
    platform/
    settings/

src-tauri/
  src/
    commands/
    shell/
    error.rs
    lib.rs
    main.rs

docs/
  template-guide.md
  architecture.md
  checks.md

scripts/
  sync-template-config.mjs
  check-brand.mjs
  check-shell-boundary.mjs
  check-template-deps.mjs
  check-docs.mjs
```

## Add a Page

1. Add a page component under `src/pages/`.
2. Register it in `src/app/routes.tsx`.

Example:

```tsx
// src/pages/LogsPage.tsx
import { Page } from "../components/layout/Page";
import type { ShellPageProps } from "../app/routes";

export function LogsPage(_props: ShellPageProps) {
  return (
    <Page title="Logs" description="View local application logs.">
      <div>Logs page</div>
    </Page>
  );
}
```

```tsx
// src/app/routes.tsx
import { LogsPage } from "../pages/LogsPage";

export const routes = [
  // existing routes
  {
    id: "logs",
    label: "Logs",
    title: "Logs",
    component: LogsPage,
  },
];
```

## Add a Setting

1. Extend `ShellSettings`.
2. Update the Rust `ShellSettings` model.
3. Update `SettingsPage`.
4. Add tests.

See:

```txt
docs/template-guide.md
```

## Add a Shell Command

Frontend invoke calls must only exist in:

```txt
src/lib/api/shell.ts
```

Rust commands must be registered in:

```txt
src-tauri/src/commands/
```

Shell logic should live in:

```txt
src-tauri/src/shell/
```

See:

```txt
docs/template-guide.md
docs/architecture.md
```

## Build

Build the desktop app:

```bash
pnpm build
```

Build only the renderer:

```bash
pnpm build:renderer
```

## Release

This template includes updater configuration placeholders, but does not implement a full release pipeline.

Before real release, configure:

```txt
1. bundle identifier
2. signing
3. updater endpoint
4. release artifacts
5. latest.json generation
```

## Template Boundary

This template should not contain product-specific legacy modules such as:

```txt
provider
proxy
mcp
prompt
skills
usage
subscription
workspace
codex
gemini
claude
openclaw
opencode
hermes
```

Run:

```bash
pnpm check:shell-boundary
```

## Checks

Run the full template check:

```bash
pnpm check:template
```

This runs:

```txt
pnpm check:template-config
pnpm check:brand
pnpm check:shell-boundary
pnpm check:frontend-legacy
pnpm check:template-deps
pnpm check:docs
pnpm typecheck
pnpm test:unit
pnpm build:renderer
cd src-tauri && cargo check
cd src-tauri && cargo test
```

Frontend-only checks:

```bash
pnpm check:template:frontend
```

Rust-only checks:

```bash
pnpm check:template:rust
```

List all template check steps:

```bash
node scripts/check-template.mjs --list
```

## Documentation

```txt
docs/template-guide.md  - how to use and extend the template
docs/architecture.md    - architecture and module boundaries
docs/checks.md          - check commands and CI rules
```

## License

MIT

<!-- TEMPLATE_IDENTITY_START -->

## Template Identity

| Field            | Value                                                                         |
| ---------------- | ----------------------------------------------------------------------------- |
| App Name         | Desktop Shell                                                                 |
| Package Name     | desktop-shell                                                                 |
| Product Name     | Desktop Shell                                                                 |
| Identifier       | com.example.desktop-shell                                                     |
| Repository       | https://github.com/example/desktop-shell                                      |
| Deep Link Scheme | desktop-shell                                                                 |
| Updater Endpoint | https://github.com/example/desktop-shell/releases/latest/download/latest.json |

To change the application identity, edit `template.config.ts` and run:

```bash
pnpm sync:template
```

<!-- TEMPLATE_IDENTITY_END -->
