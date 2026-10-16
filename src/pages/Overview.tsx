import { useComputed } from '@preact/signals';
import { formatNumber, t } from '../i18n';
import { formatBytes, formatRate } from '../lib/format';
import { monitor, onlineApps, openConnections, snapshot } from '../state/monitor';
import { rateUnit } from '../state/settings';
import { openApp, page, showConnectionsOf } from '../state/ui';
import { TrafficChart } from '../components/TrafficChart';
import { AppAvatar, Note, PageHeader, StatTile } from '../components/Misc';
import { Button } from '../components/Button';
import { Icon } from '../components/Icon';
import { InternetSwitch } from './InternetSwitch';

const TOP = 6;

/** A thin bar for a share of the busiest one; the number beside it carries the value. */
function ShareBar({ value, max }: { value: number; max: number }) {
  const pct = max > 0 ? Math.max(2, Math.round((value / max) * 100)) : 0;
  return (
    <span class="share" aria-hidden="true">
      <span class="share-fill" style={{ width: `${pct}%` }} />
    </span>
  );
}

export function Overview() {
  const snap = snapshot.value;
  const state = monitor.value;
  const unit = rateUnit.value;

  const busiestApps = useComputed(() => onlineApps.value.slice(0, TOP));
  const busiestHosts = useComputed(() => {
    const byHost = new Map<string, { name: string; host: string | null; ip: string; apps: Set<string>; rate: number; down: number; up: number }>();
    for (const c of openConnections.value) {
      if (c.rate.in + c.rate.out === 0) continue;
      const key = c.host ?? c.remoteIp;
      const entry = byHost.get(key) ?? { name: key, host: c.host, ip: c.remoteIp, apps: new Set<string>(), rate: 0, down: 0, up: 0 };
      entry.apps.add(c.appName);
      entry.down += c.rate.in;
      entry.up += c.rate.out;
      entry.rate += c.rate.in + c.rate.out;
      byHost.set(key, entry);
    }
    return [...byHost.values()].sort((a, b) => b.rate - a.rate).slice(0, TOP);
  });

  if (state && !state.supported) {
    return (
      <div class="view-body">
        <PageHeader title={t('overview.title')} />
        <Note icon="monitor" title={t('overview.notWindowsTitle')}>
          {t('overview.notWindowsBody')}
        </Note>
      </div>
    );
  }

  const appMax = busiestApps.value[0] ? busiestApps.value[0].rate.in + busiestApps.value[0].rate.out : 0;
  const hostMax = busiestHosts.value[0]?.rate ?? 0;

  return (
    <div class="view-body">
      <PageHeader title={t('overview.title')} subtitle={t('overview.subtitle')} />
      {snap && !snap.perApp ? (
        <Note icon="alert" tone="warn" title={t('overview.perAppTitle')}>
          {t('overview.perAppBody')}
        </Note>
      ) : null}

      <section class="stats" aria-label={t('overview.title')}>
        <StatTile icon="arrowDown" label={t('overview.downloadNow')} value={formatRate(snap?.rate.in ?? 0, unit)} sub={t('overview.totalSince', { amount: formatBytes(snap?.total.in ?? 0) })} />
        <StatTile icon="arrowUp" label={t('overview.uploadNow')} value={formatRate(snap?.rate.out ?? 0, unit)} sub={t('overview.totalSince', { amount: formatBytes(snap?.total.out ?? 0) })} />
        <StatTile icon="apps" label={t('overview.appsOnline')} value={formatNumber(onlineApps.value.length)} />
        <StatTile icon="link" label={t('overview.connections')} value={formatNumber(openConnections.value.length)} />
      </section>

      <section class="card">
        <h2 class="card-title">{t('overview.chartTitle')}</h2>
        <TrafficChart history={snap?.history ?? []} span={300} unit={unit} label={t('overview.chartLabel')} />
      </section>

      <div class="two-col">
        <section class="card">
          <div class="card-head">
            <h2 class="card-title">{t('overview.busiestApps')}</h2>
            <Button size="sm" variant="quiet" onClick={() => (page.value = 'apps')}>
              {t('overview.seeAll')}
            </Button>
          </div>
          {busiestApps.value.length === 0 ? (
            <p class="muted card-empty">{t('overview.quiet')}</p>
          ) : (
            <ul class="rank-list">
              {busiestApps.value.map((a) => (
                <li key={a.key} class="rank-row">
                  <button type="button" class="rank-main" onClick={() => openApp(a.key)}>
                    <AppAvatar name={a.name} />
                    <span class="rank-text">
                      <span class="rank-name">{a.name}</span>
                      <span class="rank-meta num">
                        <span class="sr-only">{t('common.download')} </span>
                        {formatRate(a.rate.in, unit)} <span class="muted">/</span> <span class="sr-only">{t('common.upload')} </span>
                        {formatRate(a.rate.out, unit)}
                      </span>
                      <ShareBar value={a.rate.in + a.rate.out} max={appMax} />
                    </span>
                  </button>
                  <InternetSwitch app={a} compact />
                </li>
              ))}
            </ul>
          )}
        </section>

        <section class="card">
          <div class="card-head">
            <h2 class="card-title">{t('overview.busiestHosts')}</h2>
            <Button size="sm" variant="quiet" onClick={() => (page.value = 'connections')}>
              {t('overview.seeAll')}
            </Button>
          </div>
          {busiestHosts.value.length === 0 ? (
            <p class="muted card-empty">{t('overview.quiet')}</p>
          ) : (
            <ul class="rank-list">
              {busiestHosts.value.map((h) => (
                <li key={h.name} class="rank-row">
                  <button type="button" class="rank-main" onClick={() => showConnectionsOf(h.host ?? h.ip)}>
                    <span class="avatar" aria-hidden="true">
                      <Icon name="globe" size={16} />
                    </span>
                    <span class="rank-text">
                      <span class="rank-name mono">{h.host ?? h.ip}</span>
                      <span class="rank-meta">
                        <span class="num">{formatRate(h.down, unit)}</span> <span class="muted">/</span> <span class="num">{formatRate(h.up, unit)}</span>
                        <span class="muted"> - {[...h.apps].join(', ')}</span>
                      </span>
                      <ShareBar value={h.rate} max={hostMax} />
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </section>
      </div>
    </div>
  );
}
