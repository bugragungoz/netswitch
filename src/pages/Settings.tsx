import type { ComponentChildren } from 'preact';
import { api } from '../api';
import { LANGUAGES, t } from '../i18n';
import { errorMessage } from '../lib/errors';
import { saveSettings, settings } from '../state/settings';
import { toast } from '../state/toasts';
import { Button } from '../components/Button';
import { PageHeader } from '../components/Misc';
import { Switch } from '../components/Switch';
import type { Settings as SettingsType } from '../api/types';

export const REPO_URL = 'https://github.com/bugragungoz/netswitch';

function Segmented<T extends string>({ label, value, options, onChange }: { label: string; value: T; options: { value: T; label: string }[]; onChange: (v: T) => void }) {
  return (
    <div class="setting-row">
      <span class="setting-label" id={`seg-${label}`}>{label}</span>
      <div class="tabs" role="radiogroup" aria-labelledby={`seg-${label}`}>
        {options.map((o) => (
          <button key={o.value} type="button" role="radio" class="tab" aria-selected={o.value === value} aria-checked={o.value === value} onClick={() => onChange(o.value)}>
            {o.label}
          </button>
        ))}
      </div>
    </div>
  );
}

function Group({ title, children }: { title: string; children: ComponentChildren }) {
  return (
    <section class="settings-group">
      <h2 class="settings-title">{title}</h2>
      <div class="settings-body">{children}</div>
    </section>
  );
}

export function Settings() {
  const s = settings.value;
  if (!s) return null;
  const save = (patch: Partial<SettingsType>) => saveSettings(patch).catch((e) => toast(errorMessage(e), { tone: 'warn' }));

  return (
    <div class="view-body">
      <PageHeader title={t('settings.title')} />
      <Group title={t('settings.appearance')}>
        <Segmented
          label={t('settings.theme')}
          value={s.theme}
          onChange={(theme) => void save({ theme })}
          options={[
            { value: 'system', label: t('settings.themeSystem') },
            { value: 'dark', label: t('settings.themeDark') },
            { value: 'light', label: t('settings.themeLight') },
          ]}
        />
        <Segmented
          label={t('settings.language')}
          value={s.language}
          onChange={(language) => void save({ language })}
          options={[{ value: 'system' as const, label: t('settings.languageSystem') }, ...LANGUAGES.map((l) => ({ value: l.code as SettingsType['language'], label: l.nativeName }))]}
        />
        <Segmented
          label={t('settings.units')}
          value={s.rateUnit}
          onChange={(rateUnit) => void save({ rateUnit })}
          options={[
            { value: 'bytes', label: t('settings.unitBytes') },
            { value: 'bits', label: t('settings.unitBits') },
          ]}
        />
      </Group>
      <Group title={t('settings.notifications')}>
        <Switch checked={s.notifyNewApps} onChange={(notifyNewApps) => void save({ notifyNewApps })} label={t('settings.notifyNewApps')} hint={t('settings.notifyNewAppsHint')} />
      </Group>
      <Group title={t('settings.safety')}>
        <Switch checked={s.dailyBackup} onChange={(dailyBackup) => void save({ dailyBackup })} label={t('settings.dailyBackup')} hint={t('settings.dailyBackupHint')} />
      </Group>
      <Group title={t('settings.about')}>
        <p class="muted about-text">{t('settings.aboutBody')}</p>
        <p class="about-meta mono">
          {t('settings.version', { version: __APP_VERSION__ })} - {t('settings.license')}
        </p>
        <div>
          <Button icon="github" onClick={() => void api.openUrl(REPO_URL)}>
            {t('settings.source')}
          </Button>
        </div>
      </Group>
    </div>
  );
}
