# Architecture

This document describes the architecture and boundaries of Desktop Shell Template.

## Overview

The template is split into:

```txt
React renderer
Tauri commands
Shell logic
Template configuration
Checks
```

High-level flow:

```txt
React page
  -> src/lib/api/shell.ts
    -> Tauri command
      -> src-tauri/src/commands/*
        -> src-tauri/src/shell/*
```

## Frontend structure

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
```

## Frontend boundaries

### `src/app`

Owns app composition:

```txt
ShellApp
routes
layout
```

It should not directly call Tauri invoke.

### `src/pages`

Owns pages.

Default pages:

```txt
Dashboard
Settings
About
```

Pages can use `shellApi`, React Query, and shell components.

### `src/components/ui`

Pure UI only.

Allowed:

```txt
Button
Card
Switch
small native controls when a dependency is not needed
```

Examples:

```txt
Use a native <select> for simple choices.
Use the existing Switch wrapper for boolean settings.
Add Dialog / Tooltip / Select only when the product really needs them,
and update dependency + boundary checks in the same change.
```

Not allowed:

```txt
ProviderCard
McpList
PromptEditor
UsageChart
SubscriptionPanel
```

### `src/lib/api`

Only API boundary.

Rule:

```txt
Tauri invoke must only appear in src/lib/api/shell.ts.
```

### `src/lib/brand`

Generated brand identity from `template.config.ts`.

Do not edit generated files manually.

### `src/lib/platform`

Platform wrappers:

```txt
window controls
external open
```

### `src/lib/settings`

Shell settings model and hooks.

## Rust structure

```txt
src-tauri/src/
  main.rs
  lib.rs
  error.rs

  commands/
    app.rs
    settings.rs
    shell.rs
    mod.rs

  shell/
    app_info.rs
    brand.rs
    external_url.rs
    logging.rs
    paths.rs
    runtime_boundary.rs
    settings_store.rs
    tray.rs
    updater.rs
    panic_hook.rs
    mod.rs
```

## Rust boundaries

### `lib.rs`

Only Tauri builder composition:

```txt
plugins
invoke_handler
setup
run
```

It should not contain product-specific logic.

### `commands/`

Tauri command adapters.

They should:

```txt
1. Validate command input if needed.
2. Call shell logic.
3. Convert errors to String.
```

They should not contain large business logic.

### `shell/`

Pure shell logic.

Examples:

```txt
app_info
brand
external_url
settings_store
tray
logging
updater
runtime_boundary
```

Most tests should target `shell/`.

## Template identity

Single source:

```txt
template.config.ts
```

Generated outputs:

```txt
src/lib/brand/templateConfig.ts
src/lib/brand/brand.ts
src/lib/settings/settings.ts
src-tauri/src/shell/brand.rs
package.json
src-tauri/Cargo.toml
src-tauri/tauri.conf.json
README.md
```

Run:

```bash
pnpm sync:template
```

Check:

```bash
pnpm check:template-config
```

## Command boundary

Allowed default commands:

```txt
get_app_info
open_external
get_settings
save_settings
update_tray_menu
```

When adding a command:

```txt
1. Add shell logic under src-tauri/src/shell/.
2. Add command adapter under src-tauri/src/commands/.
3. Register in lib.rs.
4. Add to runtime boundary.
5. Add shellApi method.
6. Add tests.
```

## Forbidden legacy concepts

The template should not include:

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

## Dependency boundary

The minimal shell should not include heavy product dependencies.

Frontend examples to avoid:

```txt
@codemirror/*
@dnd-kit/*
recharts
i18next
react-i18next
flexsearch
framer-motion
cmdk
```

Rust examples to avoid:

```txt
reqwest
axum
hyper
rusqlite
rquickjs
zip
brotli
zstd
```

Run:

```bash
pnpm check:template-deps
```

## Testing strategy

Frontend tests:

```txt
ShellApp route switching
shellApi command names
settings/theme behavior
brand config
template dependency checks
```

Rust tests:

```txt
brand boundary
runtime boundary
settings store
external URL validation
dependency boundary
template config sync
```

## Design rule

The shell template should be boring.

If a feature is product-specific, keep it out of the template by default.
