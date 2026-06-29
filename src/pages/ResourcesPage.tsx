import { Page } from "../components/layout/Page";
import { Section } from "../components/layout/Section";

export function ResourcesPage() {
  return (
    <Page
      title="Resources"
      description="Authorized local directories and repositories will be listed here."
    >
      <Section title="Authorized Resources">
        <p className="text-sm text-muted-foreground">
          No resources are authorized yet. Phase 1 keeps this as an explicit
          empty state; later phases must require user approval before exposing
          any local directory or repository.
        </p>
      </Section>
    </Page>
  );
}
