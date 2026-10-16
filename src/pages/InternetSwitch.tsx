/**
 * The switch that gives an app internet access or takes it away. Blocking asks first; giving it
 * back does not. Programs inside the Windows folder, and processes Windows names no program for,
 * cannot be switched off.
 */
import { useState } from 'preact/hooks';
import { api } from '../api';
import type { AppStat } from '../api/types';
import { t } from '../i18n';
import { errorMessage } from '../lib/errors';
import { confirm } from '../state/dialogs';
import { refresh } from '../state/monitor';
import { toast } from '../state/toasts';

export function InternetSwitch({ app, compact = false }: { app: AppStat; compact?: boolean }) {
  const [busy, setBusy] = useState(false);
  const on = !app.blocked;
  const disabled = busy || !app.path || (app.protected && on);
  const reason = !app.path ? t('apps.noPath') : app.protected && on ? t('apps.protectedHint') : undefined;
  const stateText = !app.path ? t('apps.noProgram') : app.protected && on ? t('apps.protected') : on ? t('apps.allowed') : t('apps.blockedState');

  const toggle = async () => {
    if (!app.path) return;
    if (on) {
      const ok = await confirm({
        title: t('apps.blockTitle', { name: app.name }),
        body: t('apps.blockBody', { path: app.path }),
        confirmLabel: t('apps.blockConfirm'),
        cancelLabel: t('common.cancel'),
        danger: true,
      });
      if (!ok) return;
    }
    setBusy(true);
    try {
      await api.setAppBlocked(app.path, app.name, on);
      toast(on ? t('apps.blocked', { name: app.name }) : t('apps.unblocked', { name: app.name }), { tone: 'ok' });
      await refresh();
    } catch (e) {
      toast(errorMessage(e), { tone: 'warn' });
    } finally {
      setBusy(false);
    }
  };

  return (
    <span class={`net-switch ${compact ? 'is-compact' : ''}`} title={reason}>
      <button
        type="button"
        role="switch"
        class="bg-switch"
        aria-checked={on}
        aria-label={t('apps.switchLabel', { name: app.name })}
        disabled={disabled}
        onClick={(e) => {
          e.stopPropagation();
          void toggle();
        }}
      >
        <span class="bg-switch-track" aria-hidden="true">
          <span class="bg-switch-thumb" />
        </span>
        {compact ? null : <span class="switch-state">{stateText}</span>}
      </button>
    </span>
  );
}
