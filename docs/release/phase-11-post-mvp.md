# Phase 11: Post-MVP Capability Extension

After MVP is stable, this phase widens the tunnel-mcp surface:

- **Workspace profiles** map a directory to a specific permission set so
  that different projects get independent scopes.
- **Extra MCP tools**: `files.search`, `files.patch`, readonly `git.status`
  / `git.diff`, and `package.json` inspection.
- **Command approval**: every shell command must be whitelisted and
  require approval; the policy cannot silently disable approval.
- **App integrations** open a path in VS Code, Finder/Explorer or
  xdg-open. All invocations are passive — they never write user files.
- **Policy presets** (Safe / Developer / Power User) live alongside the
  permission store; this phase only lands the data contracts and leaves
  the per-preset preset payloads to a later phase.
- **Agent wiring** with ForgeAgent is not done here. We expose the same
  Approval / Logs / Permissions models so a future integration only
  needs to wire IPC.

## Modules

| Path | Purpose |
|------|---------|
| `src-tauri/src/product/workspace/` | workspace profile store |
| `src-tauri/src/product/commands/approval.rs` | command policy + validation |
| `src-tauri/src/product/integrations/app_open.rs` | IDE / file-manager openers |
| `src-tauri/src/product/mcp/extra_tools.rs` | `files.search`, readonly git, `package.json` |
| `src/lib/workspace/types.ts` | frontend types |
| `src/lib/api/workspace.ts` | frontend invoke bridge |

## Non-goals

- No long-running `command.run` IPC yet — Phase 11 only locks the policy
  surface so a future phase can wire it without re-planning.
- No preset factory — Safe / Developer / Power User land in Phase 12.
- No ForgeAgent cross-process wiring.
