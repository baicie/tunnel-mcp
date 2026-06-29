import { useEffect, useState } from "react";
import { Page } from "../components/layout/Page";
import { Section } from "../components/layout/Section";
import { Button } from "../components/ui/button";
import { ThemeSelect } from "../components/settings/ThemeSelect";
import { StartMinimizedSwitch } from "../components/settings/StartMinimizedSwitch";
import type { ShellPageProps } from "../app/routes";
import type { ShellSettings } from "../lib/settings/settings";

export function SettingsPage(props: ShellPageProps) {
  const [draft, setDraft] = useState<ShellSettings>(
    props.shellSettings.settings,
  );

  useEffect(() => {
    setDraft(props.shellSettings.settings);
  }, [props.shellSettings.settings]);

  return (
    <Page title="Settings" description="Configure shell-level preferences.">
      <Section title="General">
        <div className="flex flex-col gap-5">
          <ThemeSelect
            value={draft.theme}
            onChange={(theme) => setDraft((current) => ({ ...current, theme }))}
          />

          <StartMinimizedSwitch
            checked={draft.startMinimized}
            onChange={(startMinimized) =>
              setDraft((current) => ({ ...current, startMinimized }))
            }
          />

          <div>
            <Button
              variant="default"
              disabled={props.shellSettings.isSaving}
              onClick={() => props.shellSettings.saveSettings(draft)}
            >
              {props.shellSettings.isSaving ? "Saving..." : "Save Settings"}
            </Button>
          </div>
        </div>
      </Section>
    </Page>
  );
}
