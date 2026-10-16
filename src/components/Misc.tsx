import type { ComponentChildren } from 'preact';
import { t } from '../i18n';
import { Icon } from './Icon';
import { IconButton } from './IconButton';

/** A label and one number, the form for a headline value. */
export function StatTile({ label, value, sub, icon }: { label: string; value: string; sub?: string; icon?: string }) {
  return (
    <div class="stat">
      <div class="stat-label">
        {icon ? <Icon name={icon} size={16} /> : null}
        <span>{label}</span>
      </div>
      <div class="stat-value">{value}</div>
      {sub ? <div class="stat-sub muted">{sub}</div> : null}
    </div>
  );
}

export function EmptyState({ icon, title, body, children }: { icon: string; title: string; body?: string; children?: ComponentChildren }) {
  return (
    <div class="empty-state">
      <div class="empty-icon">
        <Icon name={icon} size={24} />
      </div>
      <h2>{title}</h2>
      {body ? <p>{body}</p> : null}
      {children ? <div class="empty-actions">{children}</div> : null}
    </div>
  );
}

export function Note({ icon = 'info', tone, title, children }: { icon?: string; tone?: 'warn' | 'info'; title: string; children?: ComponentChildren }) {
  return (
    <div class={`bg-note ${tone ? `note-${tone}` : ''}`}>
      <Icon name={icon} size={18} class={tone ? `tone-${tone}` : 'muted'} />
      <p>
        <strong>{title}</strong>
        {children}
      </p>
    </div>
  );
}

/** The first letter of an app's name in a quiet circle; apps have no icons yet. */
export function AppAvatar({ name }: { name: string }) {
  const letter = (name.match(/[\p{L}\p{N}]/u)?.[0] ?? '?').toUpperCase();
  return (
    <span class="avatar" aria-hidden="true">
      {letter}
    </span>
  );
}

export function SearchBox({ value, onInput, placeholder }: { value: string; onInput: (v: string) => void; placeholder: string }) {
  return (
    <label class="search-box">
      <Icon name="search" size={16} />
      <input type="search" value={value} placeholder={placeholder} aria-label={placeholder} onInput={(e) => onInput((e.currentTarget as HTMLInputElement).value)} />
      {value ? <IconButton label={t('common.clearSearch')} icon="close" size="sm" onClick={() => onInput('')} /> : null}
    </label>
  );
}

export function PageHeader({ title, subtitle, children }: { title: string; subtitle?: string; children?: ComponentChildren }) {
  return (
    <header class="page-header">
      <div class="page-heading">
        <h1 class="view-title">{title}</h1>
        {subtitle ? <p class="muted page-subtitle">{subtitle}</p> : null}
      </div>
      {children ? <div class="page-actions">{children}</div> : null}
    </header>
  );
}
