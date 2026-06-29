import { useId } from "react";
import { Label } from "../ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../ui/select";
import type { ThemeMode } from "../../lib/settings/settings";

interface ThemeSelectProps {
  value: ThemeMode;
  onChange: (theme: ThemeMode) => void;
}

const THEME_OPTIONS: Array<{ value: ThemeMode; label: string }> = [
  { value: "system", label: "System" },
  { value: "light", label: "Light" },
  { value: "dark", label: "Dark" },
];

/**
 * Theme picker built on top of the shadcn-style `Select` primitive so
 * the look matches the rest of the shell (focus ring, popover border,
 * `bg-popover` content, etc.). Uses `useId` to wire the visible label
 * to the trigger so `getByLabelText('Theme')` keeps working in tests.
 */
export function ThemeSelect(props: ThemeSelectProps) {
  const labelId = useId();

  return (
    <div className="flex flex-col gap-2 text-sm">
      <Label id={labelId} className="font-medium">
        Theme
      </Label>

      <Select
        value={props.value}
        onValueChange={(next) => props.onChange(next as ThemeMode)}
      >
        <SelectTrigger aria-labelledby={labelId} aria-label="Theme">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {THEME_OPTIONS.map((option) => (
            <SelectItem key={option.value} value={option.value}>
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
