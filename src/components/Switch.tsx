import type { ComponentChildren } from 'preact';

export interface SwitchProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: ComponentChildren;
  disabled?: boolean;
  hint?: string;
}

/** Bugra Switch (added in Bugra): role="switch", the track fills with accent when on. */
export function Switch({ checked, onChange, label, disabled, hint }: SwitchProps) {
  return (
    <div class="switch-row">
      <button type="button" role="switch" class="bg-switch" aria-checked={checked} disabled={disabled} onClick={() => onChange(!checked)}>
        <span class="bg-switch-track" aria-hidden="true">
          <span class="bg-switch-thumb" />
        </span>
        <span>{label}</span>
      </button>
      {hint ? <p class="bg-field-msg">{hint}</p> : null}
    </div>
  );
}
