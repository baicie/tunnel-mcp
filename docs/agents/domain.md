# Domain Docs

This is a single-context repo for a reusable Tauri desktop shell template.

## Before Exploring

Read these first when they are relevant:

- `README.md` for product scope and common workflows.
- `docs/template-guide.md` for template usage.
- `docs/architecture.md` for frontend and Rust boundaries.
- `docs/checks.md` for validation commands.
- `docs/adr/` if ADRs are added later.

If `CONTEXT.md`, `CONTEXT-MAP.md`, or `docs/adr/` do not exist, proceed
silently. They can be added later when domain language or architectural
decisions need more structure.

## Vocabulary

Use the template's existing terms:

- "desktop shell template" for the project.
- "renderer" for React/Vite frontend code.
- "Tauri commands" for command adapters in `src-tauri/src/commands/`.
- "shell logic" for reusable Rust logic in `src-tauri/src/shell/`.
- "template identity" for values generated from `template.config.ts`.

Avoid product-specific vocabulary unless the repo is being forked into a
product app.
