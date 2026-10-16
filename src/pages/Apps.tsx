import { useComputed, useSignal } from '@preact/signals';
import { api } from '../api';
import type { AppStat } from '../api/types';
import { collator, formatNumber, t } from '../i18n';
import { formatAgo, formatBytes, formatRate } from '../lib/format';
import { apps, connections, snapshot } from '../state/monitor';
import { rateUnit } from '../state/settings';
import { toast } from '../state/toasts';
import { appFilter, appSearch, selectedApp, showConnectionsOf } from '../state/ui';
import { Button } from '../components/Button';
import { Chip } from '../components/Chip';
import { Icon } from '../components/Icon';
import { IconButton } from '../components/IconButton';
import { AppAvatar, EmptyState, PageHeader, SearchBox } from '../components/Misc';
import { TrafficChart } from '../components/TrafficChart';
import { InternetSwitch } from './InternetSwitch';

type SortKey = 'name' | 'down' | 'up' | 'total' | 'connections';

const VALUE: Record<Exclude<SortKey, 'name'>, (a: AppStat) => number> = {
  down: (a) => a.rate.in,
  up: (a) => a.rate.out,
  total: (a) => a.total.in + a.total.out,
  connections: (a) => a.connections,
};

function SortHeader({ label, k, sort, align, extra }: { label: string; k: SortKey; sort: { value: { key: SortKey | null; desc: boolean } }; align?: 'end'; extra?: boolean }) {
  const active = sort.value.key === k;
  return (
    <div class={`th ${align === 'end' ? 'is-end' : ''} ${extra ? 'is-extra' : ''}`} role="columnheader" aria-sort={active ? (sort.value.desc ? 'descending' : 'ascending') : 'none'}>
      <button
        type="button"
        class="th-sort"
        onClick={() => (sort.value = { key: k, desc: active ? !sort.value.desc : k !== 'name' })}
      >
        <span class="truncate">{label}</span>
        <Icon name={active && !sort.value.desc ? 'arrowUp' : 'arrowDown'} size={13} class={`sort-icon ${active ? 'is-active' : ''}`} />
      </button>
    </div>
  );
}

function AppPanel({ app, onClose }: { app: AppStat; onClose: () => void }) {
  const unit = rateUnit.value;
  const now = snapshot.value?.at ?? Date.now();
  const hosts = connections.value.filter((c) => c.app === app.key).slice(0, 12);
  const copyPath = async () => {
    if (!app.path) return;
    try {
      await navigator.clipboard.writeText(app.path);
      toast(t('common.copied'), { tone: 'ok' });
    } catch {
      toast(app.path);
    }
  };
  return (
    <aside class="detail-panel" aria-label={app.name}>
      <header class="panel-head">
        <AppAvatar name={app.name} />
        <h2 class="panel-title">{app.name}</h2>
        <IconButton label={t('common.close')} icon="close" size="sm" onClick={onClose} />
      </header>
      <div class="panel-body">
        <div class="panel-switch">
          <InternetSwitch app={app} />
        </div>
        {app.protected ? <p class="muted panel-hint">{t('apps.protectedHint')}</p> : !app.path ? <p class="muted panel-hint">{t('apps.noPath')}</p> : null}

        <section class="panel-section">
          <h3 class="panel-label">{t('apps.lastMinute')}</h3>
          <TrafficChart history={app.history} span={60} unit={unit} label={t('apps.lastMinute')} height={90} compact />
          <dl class="facts">
            <div><dt>{t('common.download')}</dt><dd class="num">{formatRate(app.rate.in, unit)} <span class="muted">- {formatBytes(app.total.in)}</span></dd></div>
            <div><dt>{t('common.upload')}</dt><dd class="num">{formatRate(app.rate.out, unit)} <span class="muted">- {formatBytes(app.total.out)}</span></dd></div>
            <div><dt>{t('apps.firstSeen')}</dt><dd>{formatAgo(app.firstSeen, now)}</dd></div>
            <div><dt>{t('apps.lastActive')}</dt><dd>{formatAgo(app.lastActive, now)}</dd></div>
            <div><dt>{t('apps.processes')}</dt><dd class="mono">{app.pids.length ? app.pids.join(', ') : '-'}</dd></div>
            <div><dt>{t('apps.listening')}</dt><dd class="num">{formatNumber(app.listening)}</dd></div>
          </dl>
        </section>

        {app.path ? (
          <section class="panel-section">
            <h3 class="panel-label">{t('apps.program')}</h3>
            <p class="mono path">{app.path}</p>
            <div class="panel-actions">
              <Button size="sm" icon="folder" onClick={() => void api.revealProgram(app.path!)}>
                {t('apps.reveal')}
              </Button>
              <Button size="sm" icon="copy" onClick={() => void copyPath()}>
                {t('apps.copyPath')}
              </Button>
            </div>
          </section>
        ) : null}

        <section class="panel-section">
          <div class="panel-label-row">
            <h3 class="panel-label">{t('apps.talksTo')}</h3>
            {hosts.length ? (
              <Button size="sm" variant="quiet" onClick={() => showConnectionsOf(app.name)}>
                {t('apps.showConnections')}
              </Button>
            ) : null}
          </div>
          {hosts.length === 0 ? (
            <p class="muted">{t('apps.noHosts')}</p>
          ) : (
            <ul class="host-list">
              {hosts.map((c) => (
                <li key={`${c.protocol}|${c.remoteIp}|${c.remotePort}|${c.localPort}`}>
                  <span class="host-name mono truncate" title={c.host ? `${c.host} (${c.remoteIp})` : c.remoteIp}>
                    {c.host ?? c.remoteIp}
                  </span>
                  <span class="muted mono host-port">{c.service ?? c.remotePort}</span>
                  <span class="num host-rate">{formatRate(c.rate.in + c.rate.out, unit)}</span>
                </li>
              ))}
            </ul>
          )}
        </section>
      </div>
    </aside>
  );
}

export function Apps() {
  const sort = useSignal<{ key: SortKey | null; desc: boolean }>({ key: null, desc: true });
  const unit = rateUnit.value;

  const shown = useComputed(() => {
    const q = appSearch.value.trim().toLowerCase();
    let list = apps.value;
    if (appFilter.value === 'online') list = list.filter((a) => a.rate.in + a.rate.out > 0);
    if (appFilter.value === 'blocked') list = list.filter((a) => a.blocked);
    if (q) list = list.filter((a) => a.name.toLowerCase().includes(q) || (a.path ?? '').toLowerCase().includes(q));
    const { key, desc } = sort.value;
    if (key) {
      const dir = desc ? -1 : 1;
      list = [...list].sort((a, b) => (key === 'name' ? collator.value.compare(a.name, b.name) : VALUE[key](a) - VALUE[key](b)) * dir);
    }
    return list;
  });

  const selected = apps.value.find((a) => a.key === selectedApp.value) ?? null;
  const query = appSearch.value.trim();

  return (
    <div class="view">
      <div class="view-head">
        <PageHeader title={t('apps.title')} subtitle={t('apps.subtitle')} />
        <div class="toolbar">
          <SearchBox value={appSearch.value} onInput={(v) => (appSearch.value = v)} placeholder={t('apps.searchPlaceholder')} />
          <div class="chip-row" role="group" aria-label={t('apps.filterLabel')}>
            <Chip selected={appFilter.value === 'all'} onClick={() => (appFilter.value = 'all')}>{t('apps.filterAll')}</Chip>
            <Chip selected={appFilter.value === 'online'} onClick={() => (appFilter.value = 'online')}>{t('apps.filterOnline')}</Chip>
            <Chip selected={appFilter.value === 'blocked'} onClick={() => (appFilter.value = 'blocked')}>{t('apps.filterBlocked')}</Chip>
          </div>
          <span class="grow" />
          <span class="toolbar-count muted">{t('apps.count', { count: shown.value.length })}</span>
        </div>
      </div>

      <div class={`split ${selected ? 'has-panel' : ''}`}>
        {shown.value.length === 0 ? (
          <EmptyState
            icon="apps"
            title={query ? t('apps.emptySearch', { query }) : appFilter.value === 'blocked' ? t('apps.emptyBlocked') : appFilter.value === 'online' ? t('apps.emptyOnline') : t('apps.empty')}
          />
        ) : (
          <div class="table apps-table" role="table" aria-label={t('apps.title')}>
            <div class="thead" role="row">
              <SortHeader label={t('apps.colApp')} k="name" sort={sort} />
              <SortHeader label={t('apps.colDownload')} k="down" sort={sort} align="end" />
              <SortHeader label={t('apps.colUpload')} k="up" sort={sort} align="end" />
              <SortHeader label={t('apps.colTotal')} k="total" sort={sort} align="end" extra />
              <SortHeader label={t('apps.colConnections')} k="connections" sort={sort} align="end" extra />
              <div class="th" role="columnheader">{t('apps.colInternet')}</div>
            </div>
            {shown.value.map((a) => (
              <div
                key={a.key}
                role="row"
                class={`tr ${a.key === selectedApp.value ? 'is-open' : ''} ${a.blocked ? 'is-blocked' : ''}`}
                tabIndex={0}
                aria-selected={a.key === selectedApp.value}
                onClick={() => (selectedApp.value = a.key === selectedApp.value ? null : a.key)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    selectedApp.value = a.key === selectedApp.value ? null : a.key;
                  }
                }}
              >
                <div class="td td-app" role="cell">
                  <AppAvatar name={a.name} />
                  <span class="app-text">
                    <span class="app-name truncate">{a.name}</span>
                    <span class="app-path muted mono truncate" title={a.path ?? undefined}>{a.path ?? a.key}</span>
                  </span>
                </div>
                <div class="td is-end num" role="cell">{formatRate(a.rate.in, unit)}</div>
                <div class="td is-end num" role="cell">{formatRate(a.rate.out, unit)}</div>
                <div class="td is-end num is-extra" role="cell">{formatBytes(a.total.in + a.total.out)}</div>
                <div class="td is-end num is-extra" role="cell">{formatNumber(a.connections)}</div>
                <div class="td" role="cell">
                  <InternetSwitch app={a} />
                </div>
              </div>
            ))}
          </div>
        )}
        {selected ? <AppPanel app={selected} onClose={() => (selectedApp.value = null)} /> : null}
      </div>
    </div>
  );
}
