import { useState } from 'preact/hooks';
import { api } from '../api';
import type { BlockedApp } from '../api/types';
import { t } from '../i18n';
import { errorMessage } from '../lib/errors';
import { confirm } from '../state/dialogs';
import { blocked, refresh } from '../state/monitor';
import { toast } from '../state/toasts';
import { page } from '../state/ui';
import { Button } from '../components/Button';
import { EmptyState, PageHeader } from '../components/Misc';

function BlockedRow({ app, busy, onUnblock }: { app: BlockedApp; busy: boolean; onUnblock: () => void }) {
  return (
    <li class="blocked-row">
      <div class="blocked-main">
        <div class="blocked-head">
          <h2 class="blocked-name">{app.name}</h2>
          <span class="badge">{t('blocked.rules', { count: app.rules })}</span>
          {app.complete ? null : (
            <span class="badge badge-warn" title={t('blocked.incompleteHint')}>
              {t('blocked.incomplete')}
            </span>
          )}
        </div>
        <ul class="program-list">
          {app.programs.map((p) => (
            <li key={p} class="mono">{p}</li>
          ))}
        </ul>
      </div>
      <Button onClick={onUnblock} disabled={busy} aria-label={t('blocked.unblockApp', { name: app.name })}>
        {t('blocked.unblock')}
      </Button>
    </li>
  );
}

export function Blocked() {
  const [busy, setBusy] = useState(false);
  const list = blocked.value;
  const totalRules = list.reduce((n, a) => n + a.rules, 0);

  const run = async (work: () => Promise<string | null>) => {
    setBusy(true);
    try {
      const message = await work();
      if (message) toast(message, { tone: 'ok' });
      await refresh();
    } catch (e) {
      toast(errorMessage(e), { tone: 'warn' });
    } finally {
      setBusy(false);
    }
  };

  const unblock = async (app: BlockedApp) => {
    const ok = await confirm({
      title: t('blocked.unblockTitle', { name: app.name }),
      body: t('blocked.unblockBody', { name: app.name, count: app.rules }),
      confirmLabel: t('blocked.unblock'),
      cancelLabel: t('common.cancel'),
    });
    if (ok) await run(async () => t('blocked.unblocked', { count: await api.unblockApplication(app.name) }));
  };

  const unblockAll = async () => {
    const ok = await confirm({
      title: t('blocked.unblockAllTitle'),
      body: t('blocked.unblockAllBody', { count: totalRules }),
      confirmLabel: t('blocked.unblockAll'),
      cancelLabel: t('common.cancel'),
      danger: true,
    });
    if (ok) await run(async () => t('blocked.unblocked', { count: await api.unblockAll() }));
  };

  const backup = () => run(async () => t('blocked.backupDone', { path: await api.backupFirewall() }));

  return (
    <div class="view-body">
      <PageHeader title={t('blocked.title')} subtitle={t('blocked.subtitle')}>
        <Button icon="backup" onClick={() => void backup()} disabled={busy}>
          {t('blocked.backup')}
        </Button>
        <Button icon="external" onClick={() => void api.openFirewallConsole()}>
          {t('blocked.openFirewall')}
        </Button>
        {list.length ? (
          <Button variant="danger" onClick={() => void unblockAll()} disabled={busy}>
            {t('blocked.unblockAll')}
          </Button>
        ) : null}
      </PageHeader>

      {list.length === 0 ? (
        <EmptyState icon="shield" title={t('blocked.empty')} body={t('blocked.emptyHint')}>
          <Button onClick={() => (page.value = 'apps')}>{t('nav.apps')}</Button>
        </EmptyState>
      ) : (
        <ul class="blocked-list">
          {list.map((a) => (
            <BlockedRow key={a.name} app={a} busy={busy} onUnblock={() => void unblock(a)} />
          ))}
        </ul>
      )}
      <p class="muted small-print">{t('blocked.restoreHint')}</p>
    </div>
  );
}
