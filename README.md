# Tunnel MCP

A local-first MCP gateway desktop client built on Tauri 2 and React.

It packages a managed `tunnel-client` sidecar with an embedded local MCP server so a remote AI (for example, ChatGPT Web) can reach a narrow, user-approved slice of local resources over a secure tunnel — never the whole disk.

This repository is forked from
[`baicie/tauri-template`](https://github.com/baicie/tauri-template) and inherits
the shell template's Tauri/React foundation, identity sync, and template
checks. The Tunnel MCP product layer is added on top without turning the shell
itself into product business code.

## What is this

Tunnel MCP is the desktop client half of a Local MCP Gateway. The desktop side
owns the local trust boundary:

```txt
local permission scopes
tunnel-client lifecycle
local MCP server lifecycle
resource authorisation
write approval
audit log
updater
```

It does not contain a code editor, an account system, a cloud backend, or a
plugin marketplace. Remote AI tools reach local resources only through the
tunnel and only for resources the user has explicitly approved.

## Current phase

```txt
Phase 0 - template fork and product identity: complete.
Phase 1 - product domain model, local settings, status surface: current.
Phase 2 onwards - managed tunnel-client lifecycle and local MCP runtime.
```

Phase 1 introduces the product layer above the shell: typed tunnel settings,
public masked settings, initial tunnel status, initial local MCP status, and the
frontend settings/dashboard surface. It does not download `tunnel-client`, start
an MCP server, or execute local tools yet.

## Features

```txt
1. Tauri 2 desktop shell
2. React renderer
3. Shell layout with sidebar and titlebar
4. Dashboard / Settings / About default pages
5. Theme and shell settings
6. App identity generated from template.config.ts
7. Shell command boundary enforced through shellApi
8. Tunnel product API adapters under src/lib/api/tunnel.ts
9. Local JSON settings store for Phase 1 product configuration
10. Public masked OpenAI key status, never raw key returned to frontend
11. Initial tunnel-client and local MCP server status commands
12. Frontend and Rust unit tests
13. Shell/product boundary checks
```

## Good for

```txt
1. Local-first desktop tools
2. Developer tools
3. Personal MCP gateways
4. Small internal desktop apps
5. Tauri proof-of-concepts that later embed tunnel-mcp product modules
```

## Not in scope for the shell

This shell layer intentionally does not include:

```txt
1. tunnel-client download or process management (Phase 2)
2. local MCP server tools or transport (Phase 3)
3. resource scopes or permission model (Phase 5)
4. write approval flow (Phase 5)
5. audit log UI (Phase 6)
6. account system, cloud backend, database layer
7. complex plugin marketplace
8. arbitrary shell command execution
9. auto-update release pipeline
```

Product modules live in product-layer paths and are kept behind explicit API and
runtime boundary lists.

## Quick Start

Install dependencies:

```bash
pnpm install
```

Start the desktop app:

```bash
pnpm dev
```

Run app-level checks:

```bash
pnpm check:app
```

Run the full template/product boundary maintenance check:

```bash
pnpm check:template
```

Run all checks (template + app):

```bash
pnpm check:all
```

## Identity

Application identity is generated from one config file:

```txt
template.config.ts
```

It controls:

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

To change the app name, identifier, deep-link scheme, or updater endpoint,
edit `template.config.ts` and run:

```bash
pnpm sync:template
```

Check whether generated files are still in sync:

```bash
pnpm check:template-config
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
      shell.ts
      tunnel.ts
    brand/
    query/
    platform/
    settings/
    tunnel/
      types.ts
      mask.ts

src-tauri/
  src/
    commands/
      app.rs
      settings.rs
      shell.rs
      tunnel.rs
    product/
      settings.rs
      status.rs
    shell/
    error.rs
    lib.rs
    main.rs

docs/
  template-guide.md
  architecture.md
  checks.md
  agents/issue-tracker.md

scripts/
  sync-template-config.mjs
  check-brand.mjs
  check-shell-boundary.mjs
  check-template-deps.mjs
  check-docs.mjs
```

## Product layer

Phase 1 product code is intentionally narrow:

```txt
src/lib/api/tunnel.ts              - frontend Tauri invoke adapter
src/lib/tunnel/types.ts            - frontend product DTOs
src/lib/tunnel/mask.ts             - frontend-only display masking helper
src-tauri/src/commands/tunnel.rs   - Tauri command adapter
src-tauri/src/product/settings.rs  - JSON settings store and public settings
src-tauri/src/product/status.rs    - initial status models
```

Rules:

```txt
1. shell code stays product-neutral
2. product commands are listed in PRODUCT_COMMANDS
3. frontend invoke calls stay inside API adapter modules
4. raw OpenAI key is accepted only by save_tunnel_settings
5. get_tunnel_settings returns only masked/public key state
6. blank OpenAI Key input preserves the existing key
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

1. Extend `ShellSettings` or product settings depending on ownership.
2. Update the matching Rust model.
3. Update the relevant page.
4. Add tests.

See:

```txt
docs/template-guide.md
```

## Add a Shell Command

Frontend invoke calls must only exist in API adapter modules:

```txt
src/lib/api/shell.ts
src/lib/api/tunnel.ts
```

Rust commands must be registered in:

```txt
src-tauri/src/commands/
```

Shell-neutral logic should live in:

```txt
src-tauri/src/shell/
```

Product logic should live in:

```txt
src-tauri/src/product/
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

This project carries updater configuration placeholders, but no real release
pipeline yet. Before a real release, configure:

```txt
1. bundle identifier (set via template.config.ts)
2. signing
3. updater endpoint (set via template.config.ts)
4. release artifacts
5. latest.json generation
```

## Shell Boundary

The shell layer must not contain product-specific business modules such as:

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

Phase 1 product paths are allowed to use Tunnel MCP vocabulary, but only under
explicit product directories or API adapter modules. Product commands must be
registered in `PRODUCT_COMMANDS` and mounted through `lib.rs`.

Run:

```bash
pnpm check:shell-boundary
```

## Checks

```bash
pnpm check:app          # app-level
pnpm check:template     # template/product boundary maintenance
pnpm check:all          # both
```

List all template check steps:

```bash
node scripts/check-template.mjs --list
```

## Documentation

```txt
docs/template-guide.md         - how to use and extend the shell
docs/architecture.md           - architecture and module boundaries
docs/checks.md                 - check commands and CI rules
docs/agents/issue-tracker.md   - GitHub issue tracker conventions
```

## License

MIT

<!-- TEMPLATE_IDENTITY_START -->

## Template Identity

| Field            | Value                                                                     |
| ---------------- | ------------------------------------------------------------------------- |
| App Name         | Tunnel MCP                                                                |
| Package Name     | tunnel-mcp                                                                |
| Product Name     | Tunnel MCP                                                                |
| Identifier       | com.baicie.tunnel-mcp                                                     |
| Repository       | https://github.com/baicie/tunnel-mcp                                      |
| Deep Link Scheme | tunnel-mcp                                                                |
| Updater Endpoint | https://github.com/baicie/tunnel-mcp/releases/latest/download/latest.json |

To change the application identity, edit `template.config.ts` and run:

```bash
pnpm sync:template
```

<!-- TEMPLATE_IDENTITY_END -->
