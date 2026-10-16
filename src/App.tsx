import { useEffect } from 'preact/hooks';
import { api } from './api';
import { formatNumber, t } from './i18n';
import { formatRate } from './lib/format';
import { blocked, monitor, onlineApps, openConnections, readError, snapshot, startPolling } from './state/monitor';
import { loadSettings, rateUnit, settings } from './state/settings';
import { toast } from './state/toasts';
import { page, type Page } from './state/ui';
import { ConfirmHost } from './components/ConfirmHost';
import { Icon } from './components/Icon';
import { Snackbars } from './components/Snackbars';
import { Apps } from './pages/Apps';
import { Blocked } from './pages/Blocked';
import { Connections } from './pages/Connections';
import { Overview } from './pages/Overview';
import { REPO_URL, Settings } from './pages/Settings';

const NAV: { page: Page; icon: string; count?: () => number }[] = [
  { page: 'overview', icon: 'activity' },
  { page: 'apps', icon: 'apps', count: () => onlineApps.value.length },
  { page: 'connections', icon: 'link', count: () => openConnections.value.length },
  { page: 'blocked', icon: 'shieldOff', count: () => blocked.value.length },
  { page: 'settings', icon: 'settings' },
];

/** The wordmark's mark: a switch, on, in the accent ink. */
function Mark() {
  return (
    <svg class="wordmark-mark" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" aria-hidden="true">
      <rect x="2.5" y="6.5" width="19" height="11" rx="5.5" />
      <circle cx="16" cy="12" r="3.2" fill="currentColor" stroke="none" />
    </svg>
  );
}

function StatusBar() {
  const state = monitor.value;
  const snap = snapshot.value;
  const unit = rateUnit.value;
  let measuring: { icon: string; text: string; title?: string; tone?: string };
  if (readError.value) measuring = { icon: 'alert', text: t('status.noConnection', { detail: readError.value }), tone: 'tone-warn' };
  else if (state && !state.supported) measuring = { icon: 'monitor', text: t('status.notWindows') };
  else if (snap && !snap.perApp) measuring = { icon: 'alert', text: t('status.totalsOnly'), title: t('status.totalsOnlyHint', { reason: state?.traceError ?? '' }), tone: 'tone-warn' };
  else measuring = { icon: 'checkCircle', text: t('status.perApp'), tone: 'tone-ok' };
  const problems = state?.errors ?? [];

  return (
    <footer class="statusbar">
      <span class={`status-item ${measuring.tone ?? ''}`} title={measuring.title}>
        <Icon name={measuring.icon} size={14} />
        {measuring.text}
      </span>
      {api.simulated ? (
        <>
          <span class="sep" />
          <span class="status-item" title={t('status.demoHint')}>
            <Icon name="info" size={14} />
            {t('status.demo')}
          </span>
        </>
      ) : null}
      {problems.length ? (
        <>
          <span class="sep" />
          <span class="status-item tone-warn" title={problems.join('\n')}>
            <Icon name="alert" size={14} />
            {t('status.problems', { count: problems.length })}
          </span>
        </>
      ) : null}
      <span class="statusbar-gap" />
      {snap ? (
        <span class="status-item num">
          <Icon name="arrowDown" size={14} />
          {formatRate(snap.rate.in, unit)}
          <Icon name="arrowUp" size={14} />
          {formatRate(snap.rate.out, unit)}
        </span>
      ) : null}
      <span class="sep" />
      <span class="status-item mono">{t('status.version', { version: __APP_VERSION__ })}</span>
      <button type="button" class="status-item" onClick={() => void api.openUrl(REPO_URL)} aria-label={t('status.github')} title={t('status.github')}>
        <Icon name="github" size={14} />
      </button>
    </footer>
  );
}

export function App() {
  useEffect(() => {
    void loadSettings();
    startPolling();
    let off: (() => void) | null = null;
    void api
      .onAppsOnline((list) => {
        for (const a of list) {
          toast(t('notify.title', { name: a.name }), { tone: 'info' });
          if (settings.value?.notifyNewApps) void api.notify(t('notify.title', { name: a.name }), t('notify.body'));
        }
      })
      .then((unsubscribe) => (off = unsubscribe));
    return () => off?.();
  }, []);

  const current = page.value;
  return (
    <div class="app">
      <a class="skip-link" href="#main">
        {t('nav.label')}
      </a>
      <nav class="sidebar" aria-label={t('nav.label')}>
        <div class="wordmark">
          <Mark />
          <span class="wordmark-text">netSwitch</span>
        </div>
        <div class="nav">
          {NAV.map((item) => {
            const count = item.count?.();
            return (
              <button
                key={item.page}
                type="button"
                class="nav-item"
                aria-current={current === item.page ? 'page' : undefined}
                onClick={() => (page.value = item.page)}
              >
                <Icon name={item.icon} size={19} />
                <span>{t(`nav.${item.page}`)}</span>
                {count ? <span class="nav-count">{formatNumber(count)}</span> : null}
              </button>
            );
          })}
        </div>
      </nav>
      <main id="main" class="main" tabIndex={-1}>
        {current === 'overview' ? <Overview /> : null}
        {current === 'apps' ? <Apps /> : null}
        {current === 'connections' ? <Connections /> : null}
        {current === 'blocked' ? <Blocked /> : null}
        {current === 'settings' ? <Settings /> : null}
      </main>
      <StatusBar />
      <Snackbars />
      <ConfirmHost />
    </div>
  );
}
