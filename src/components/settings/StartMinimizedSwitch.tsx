import { Switch } from "../ui/switch";

interface StartMinimizedSwitchProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
}

export function StartMinimizedSwitch(props: StartMinimizedSwitchProps) {
  return (
    <label className="flex items-center gap-2 text-sm">
      <Switch
        checked={props.checked}
        onCheckedChange={props.onChange}
        aria-label="Start minimized"
      />
      <span>Start minimized</span>
    </label>
  );
}
