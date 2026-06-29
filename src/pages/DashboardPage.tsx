import { useQuery } from "@tanstack/react-query";
import { Page } from "../components/layout/Page";
import { Section } from "../components/layout/Section";
import { shellApi } from "../lib/api/shell";
import { queryKeys } from "../lib/query/queryKeys";
import { APP_BRAND } from "../lib/brand/brand";
import type { ShellPageProps } from "../app/routes";

export function DashboardPage(_props: ShellPageProps) {
  const appInfoQuery = useQuery({
    queryKey: queryKeys.appInfo,
    queryFn: shellApi.getAppInfo,
  });

  return (
    <Page
      title="Dashboard"
      description="A clean desktop shell ready for product features."
    >
      <Section title="Status">
        <p className="text-sm text-muted-foreground">
          The shell runtime is ready. No legacy business module should be
          loaded.
        </p>
      </Section>

      <Section title="App Info">
        {appInfoQuery.isLoading ? (
          <p className="text-sm text-muted-foreground">Loading...</p>
        ) : (
          <dl className="grid grid-cols-[120px_1fr] gap-2 text-sm">
            <dt className="text-muted-foreground">Name</dt>
            <dd>{appInfoQuery.data?.name ?? APP_BRAND.displayName}</dd>
            <dt className="text-muted-foreground">Version</dt>
            <dd>{appInfoQuery.data?.version ?? "0.0.0"}</dd>
          </dl>
        )}
      </Section>
    </Page>
  );
}
