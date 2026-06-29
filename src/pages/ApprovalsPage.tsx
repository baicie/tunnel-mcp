import { Page } from "../components/layout/Page";
import { Section } from "../components/layout/Section";

export function ApprovalsPage() {
  return (
    <Page
      title="Approvals"
      description="Pending resource and write approval requests will appear here."
    >
      <Section title="Pending Approvals">
        <p className="text-sm text-muted-foreground">
          No pending approvals. Phase 1 does not allow real resource access or
          writes, so this page is an empty approval-center skeleton.
        </p>
      </Section>
    </Page>
  );
}
