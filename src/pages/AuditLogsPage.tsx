import { Page } from "../components/layout/Page";
import { Section } from "../components/layout/Section";

export function AuditLogsPage() {
  return (
    <Page
      title="Audit Logs"
      description="Tunnel, MCP, resource access and approval events will be audited here."
    >
      <Section title="Recent Events">
        <p className="text-sm text-muted-foreground">
          No audit events yet. Phase 1 only reserves the information
          architecture; real event recording belongs to the audit phase.
        </p>
      </Section>
    </Page>
  );
}
