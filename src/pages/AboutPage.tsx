import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Page } from "../components/layout/Page";
import { Section } from "../components/layout/Section";
import { Button } from "../components/ui/button";
import { shellApi } from "../lib/api/shell";
import { queryKeys } from "../lib/query/queryKeys";
import {
  checkAppUpdate,
  checkTunnelClientUpdate,
  rollbackTunnelClient,
  updateTunnelClient,
} from "../lib/api/updater";
import { APP_BRAND } from "../lib/brand/brand";
import { openExternal } from "../lib/platform/external";
import type { ShellPageProps } from "../app/routes";
import type {
  TunnelClientVersionStatus,
  UpdateCheckResult,
} from "../lib/updater/types";

const DEFAULT_TUNNEL_CLIENT_MANIFEST_URL =
  "https://github.com/baicie/tunnel-client/releases/latest/download/manifest.json";

export function AboutPage(_props: ShellPageProps) {
  const appInfoQuery = useQuery({
    queryKey: queryKeys.appInfo,
    queryFn: shellApi.getAppInfo,
  });

  const [appUpdate, setAppUpdate] = useState<UpdateCheckResult | null>(null);
  const [clientUpdate, setClientUpdate] =
    useState<TunnelClientVersionStatus | null>(null);
  const [updateError, setUpdateError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function runUpdate(action: () => Promise<unknown>) {
    setBusy(true);
    setUpdateError(null);
    try {
      await action();
    } catch (err) {
      setUpdateError(String(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <Page title="About" description={APP_BRAND.description}>
      <Section title="Application">
        {appInfoQuery.isLoading ? (
          <p className="text-sm text-muted-foreground">Loading...</p>
        ) : (
          <dl className="grid grid-cols-[120px_1fr] gap-2 text-sm">
            <dt className="text-muted-foreground">Name</dt>
            <dd>{appInfoQuery.data?.name ?? APP_BRAND.displayName}</dd>
            <dt className="text-muted-foreground">Version</dt>
            <dd>{appInfoQuery.data?.version ?? "0.0.0"}</dd>
            <dt className="text-muted-foreground">Identifier</dt>
            <dd>{appInfoQuery.data?.identifier ?? APP_BRAND.identifier}</dd>
          </dl>
        )}

        <div className="mt-5 flex flex-wrap gap-2">
          <Button
            variant="default"
            onClick={() => openExternal(APP_BRAND.repositoryUrl)}
          >
            Open Repository
          </Button>
          <Button
            variant="outline"
            disabled={busy}
            onClick={() =>
              runUpdate(async () => setAppUpdate(await checkAppUpdate()))
            }
          >
            Check App Update
          </Button>
        </div>

        {appUpdate ? (
          <pre className="mt-3 overflow-x-auto rounded bg-muted px-2 py-1 text-xs">
            {JSON.stringify(appUpdate, null, 2)}
          </pre>
        ) : null}
      </Section>

      <Section title="tunnel-client Update">
        <p className="text-sm text-muted-foreground">
          Tunnel-client ships its own manifest so the desktop shell can be
          upgraded independently of the embedded binary.
        </p>

        <div className="mt-3 flex flex-wrap gap-2">
          <Button
            variant="outline"
            disabled={busy}
            onClick={() =>
              runUpdate(async () =>
                setClientUpdate(
                  await checkTunnelClientUpdate(
                    DEFAULT_TUNNEL_CLIENT_MANIFEST_URL,
                  ),
                ),
              )
            }
          >
            Check tunnel-client Update
          </Button>

          <Button
            variant="default"
            disabled={busy || clientUpdate?.updateAvailable === false}
            onClick={() =>
              runUpdate(async () =>
                setClientUpdate(
                  await updateTunnelClient(DEFAULT_TUNNEL_CLIENT_MANIFEST_URL),
                ),
              )
            }
          >
            Update tunnel-client
          </Button>

          <Button
            variant="outline"
            disabled={busy}
            onClick={() =>
              runUpdate(async () =>
                setClientUpdate(await rollbackTunnelClient()),
              )
            }
          >
            Rollback tunnel-client
          </Button>
        </div>

        {clientUpdate ? (
          <pre className="mt-3 overflow-x-auto rounded bg-muted px-2 py-1 text-xs">
            {JSON.stringify(clientUpdate, null, 2)}
          </pre>
        ) : null}
      </Section>

      {updateError ? (
        <p role="alert" className="text-sm text-red-500">
          {updateError}
        </p>
      ) : null}
    </Page>
  );
}
