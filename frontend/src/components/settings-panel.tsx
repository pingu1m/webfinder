import { ArrowLeft } from "lucide-react";
import { useSettingsStore, type Settings } from "@/stores/settings-store";

interface SettingsPanelProps {
  onClose: () => void;
}

export function SettingsPanel({ onClose }: SettingsPanelProps) {
  const settings = useSettingsStore();

  return (
    <div className="flex-1 flex flex-col min-h-0">
      <div className="flex items-center gap-2 px-4 py-2.5 border-b bg-muted/30 shrink-0">
        <button
          onClick={onClose}
          className="p-1 rounded hover:bg-muted transition-colors"
          title="Back to editor"
        >
          <ArrowLeft className="h-4 w-4" />
        </button>
        <span className="text-sm font-semibold">Settings</span>
      </div>

      <div className="flex-1 overflow-auto p-6">
        <div className="max-w-lg space-y-6">
          <ToggleSetting
            label="Auto-save"
            description="Automatically save files after editing (debounced). When off, use Cmd+S / Ctrl+S to save."
            checked={settings.autoSave}
            onChange={(v) => settings.update("autoSave", v)}
          />

          <NumberSetting
            label="Font size"
            description="Editor font size in pixels"
            value={settings.fontSize}
            min={8}
            max={72}
            onChange={(v) => settings.update("fontSize", v)}
          />

          <NumberSetting
            label="Tab size"
            description="Number of spaces per tab"
            value={settings.tabSize}
            min={1}
            max={16}
            onChange={(v) => settings.update("tabSize", v)}
          />

          <SelectSetting
            label="Word wrap"
            description="How the editor wraps long lines"
            value={settings.wordWrap}
            options={[
              { value: "on", label: "On" },
              { value: "off", label: "Off" },
              { value: "wordWrapColumn", label: "At column" },
              { value: "bounded", label: "Bounded" },
            ]}
            onChange={(v) => settings.update("wordWrap", v)}
          />

          <SelectSetting
            label="Theme"
            description="Application color theme"
            value={settings.theme}
            options={[
              { value: "light", label: "Light" },
              { value: "vs-dark", label: "Dark" },
              { value: "hc-black", label: "High Contrast" },
            ]}
            onChange={(v) => settings.update("theme", v)}
          />
        </div>
      </div>
    </div>
  );
}

function ToggleSetting({
  label,
  description,
  checked,
  onChange,
}: {
  label: string;
  description: string;
  checked: boolean;
  onChange: (value: boolean) => void;
}) {
  return (
    <div className="flex items-start justify-between gap-4">
      <div>
        <div className="text-sm font-medium">{label}</div>
        <div className="text-xs text-muted-foreground mt-0.5">{description}</div>
      </div>
      <button
        role="switch"
        aria-checked={checked}
        onClick={() => onChange(!checked)}
        className={`relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors ${
          checked ? "bg-primary" : "bg-input"
        }`}
      >
        <span
          className={`pointer-events-none block h-4 w-4 rounded-full bg-background shadow-sm transition-transform ${
            checked ? "translate-x-4" : "translate-x-0"
          }`}
        />
      </button>
    </div>
  );
}

function NumberSetting({
  label,
  description,
  value,
  min,
  max,
  onChange,
}: {
  label: string;
  description: string;
  value: number;
  min: number;
  max: number;
  onChange: (value: number) => void;
}) {
  return (
    <div className="flex items-start justify-between gap-4">
      <div>
        <div className="text-sm font-medium">{label}</div>
        <div className="text-xs text-muted-foreground mt-0.5">{description}</div>
      </div>
      <input
        type="number"
        min={min}
        max={max}
        value={value}
        onChange={(e) => {
          const n = parseInt(e.target.value, 10);
          if (!isNaN(n) && n >= min && n <= max) onChange(n);
        }}
        className="w-20 rounded-md border border-input bg-transparent px-2 py-1 text-sm text-right focus:outline-none focus:ring-1 focus:ring-ring"
      />
    </div>
  );
}

function SelectSetting({
  label,
  description,
  value,
  options,
  onChange,
}: {
  label: string;
  description: string;
  value: string;
  options: { value: string; label: string }[];
  onChange: (value: string) => void;
}) {
  return (
    <div className="flex items-start justify-between gap-4">
      <div>
        <div className="text-sm font-medium">{label}</div>
        <div className="text-xs text-muted-foreground mt-0.5">{description}</div>
      </div>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="rounded-md border border-input bg-transparent px-2 py-1 text-sm focus:outline-none focus:ring-1 focus:ring-ring"
      >
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
    </div>
  );
}
