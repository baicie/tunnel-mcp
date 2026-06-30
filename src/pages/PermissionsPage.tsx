import { FormEvent, useEffect, useState } from "react";
import { Button } from "../components/ui/button";
import { Page } from "../components/layout/Page";
import { Section } from "../components/layout/Section";
import {
  addPermissionScope,
  listPermissionScopes,
  removePermissionScope,
} from "../lib/api/permissions";
import type {
  PermissionAccess,
  PermissionKind,
  PermissionScope,
} from "../lib/permissions/types";

const ACCESS_OPTIONS: PermissionAccess[] = ["read", "write", "readwrite"];
const KIND_OPTIONS: PermissionKind[] = ["filesystem", "command", "app"];

export function PermissionsPage() {
  const [scopes, setScopes] = useState<PermissionScope[]>([]);
  const [kind, setKind] = useState<PermissionKind>("filesystem");
  const [pattern, setPattern] = useState("");
  const [access, setAccess] = useState<PermissionAccess>("read");
  const [requireApproval, setRequireApproval] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function refresh() {
    setScopes(await listPermissionScopes());
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (!pattern.trim()) {
      setError("pattern is required");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await addPermissionScope({ kind, pattern, access, requireApproval });
      setPattern("");
      await refresh();
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  async function remove(id: string) {
    setBusy(true);
    setError(null);
    try {
      setScopes(await removePermissionScope(id));
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    refresh().catch((err) => setError(String(err)));
  }, []);

  return (
    <Page
      title="Permissions"
      description="Local resource scopes for the embedded MCP server. Sensitive paths are always denied."
    >
      <Section title="Add Scope">
        <form onSubmit={submit} className="grid gap-3 text-sm">
          <label className="grid gap-1">
            <span>Kind</span>
            <select
              value={kind}
              onChange={(event) =>
                setKind(event.target.value as PermissionKind)
              }
              className="rounded border border-border bg-background px-2 py-1"
            >
              {KIND_OPTIONS.map((option) => (
                <option key={option} value={option}>
                  {option}
                </option>
              ))}
            </select>
          </label>
          <label className="grid gap-1">
            <span>Directory or glob</span>
            <input
              value={pattern}
              onChange={(event) => setPattern(event.target.value)}
              placeholder="~/Documents/project-a/**"
              className="rounded border border-border bg-background px-2 py-1"
            />
          </label>
          <label className="grid gap-1">
            <span>Access</span>
            <select
              value={access}
              onChange={(event) =>
                setAccess(event.target.value as PermissionAccess)
              }
              className="rounded border border-border bg-background px-2 py-1"
            >
              {ACCESS_OPTIONS.map((option) => (
                <option key={option} value={option}>
                  {option}
                </option>
              ))}
            </select>
          </label>
          <label className="flex items-center gap-2">
            <input
              type="checkbox"
              checked={requireApproval}
              onChange={(event) => setRequireApproval(event.target.checked)}
            />
            <span>Require approval</span>
          </label>
          <div>
            <Button type="submit" disabled={busy}>
              Add
            </Button>
          </div>
        </form>
        {error ? (
          <p role="alert" className="pt-2 text-sm text-red-500">
            {error}
          </p>
        ) : null}
      </Section>

      <Section title="Active Scopes">
        {scopes.length ? (
          <ul className="grid gap-2 text-sm">
            {scopes.map((scope) => (
              <li
                key={scope.id}
                className="flex items-center justify-between gap-2 rounded border border-border px-3 py-2"
              >
                <span className="flex flex-col gap-1">
                  <code>{scope.pattern}</code>
                  <span className="text-xs text-muted-foreground">
                    {scope.kind} · {scope.access} ·{" "}
                    {scope.requireApproval ? "approval" : "pre-approved"}
                  </span>
                </span>
                <Button
                  variant="outline"
                  size="sm"
                  disabled={busy}
                  onClick={() => remove(scope.id)}
                >
                  Remove
                </Button>
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-sm text-muted-foreground">
            No permission scopes yet. Add one above to expose directories to the
            MCP server.
          </p>
        )}
      </Section>

      <Section title="Sensitive Path Denylist">
        <p className="text-sm text-muted-foreground">
          These paths are always denied, even if a scope matches them:
          <code className="ml-1">~/.ssh</code>,
          <code className="ml-1">~/.gnupg</code>,
          <code className="ml-1">~/Library/Keychains</code>,
          <code className="ml-1">~/.aws</code>,
          <code className="ml-1">~/.kube</code>,
          <code className="ml-1">~/.docker</code>,
          <code className="ml-1">.env</code>,
          <code className="ml-1">id_rsa</code>,
          <code className="ml-1">id_ed25519</code>,
          <code className="ml-1">%APPDATA%\Microsoft\Credentials</code>.
        </p>
      </Section>
    </Page>
  );
}
