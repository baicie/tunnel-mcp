import { Page } from "../components/layout/Page";
import { Section } from "../components/layout/Section";

export function PermissionsPage() {
  return (
    <Page
      title="Permissions"
      description="Permission scopes and access rules will be managed here."
    >
      <Section title="Permission Scopes">
        <p className="text-sm text-muted-foreground">
          No permission scopes exist yet. Phase 1 only defines the empty state;
          real caller/resource/operation policies belong to the permissions
          phase.
        </p>
      </Section>
    </Page>
  );
}
