import { useQuery } from "@tanstack/react-query";
import { Page } from "../components/layout/Page";
import { Section } from "../components/layout/Section";
import { Button } from "../components/ui/button";
import { shellApi } from "../lib/api/shell";
import { queryKeys } from "../lib/query/queryKeys";
import { APP_BRAND } from "../lib/brand/brand";
import { openExternal } from "../lib/platform/external";
import type { ShellPageProps } from "../app/routes";

export function AboutPage(_props: ShellPageProps) {
  const appInfoQuery = useQuery({
    queryKey: queryKeys.appInfo,
    queryFn: shellApi.getAppInfo,
  });

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

        <div className="mt-5">
          <Button
            variant="default"
            onClick={() => openExternal(APP_BRAND.repositoryUrl)}
          >
            Open Repository
          </Button>
        </div>
      </Section>
    </Page>
  );
}
