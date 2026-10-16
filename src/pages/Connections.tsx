import { useComputed } from '@preact/signals';
import type { ConnectionStat } from '../api/types';
import { t } from '../i18n';
import { formatAgo, formatBytes, formatEndpoint, formatRate } from '../lib/format';
import { connections, snapshot } from '../state/monitor';
import { rateUnit } from '../state/settings';
import { connectionSearch, hideLocal, openApp, showTcp, showUdp } from '../state/ui';
import { Chip } from '../components/Chip';
import { EmptyState, PageHeader, SearchBox } from '../components/Misc';
import { Switch } from '../components/Switch';

/** A word for the connection's state; TCP's eleven states fold into what people need to know. */
function stateLabel(c: ConnectionStat): string {
  if (c.protocol === 'udp') return t(c.open ? 'connections.state.active' : 'connections.state.idle');
  switch (c.state) {
    case 'established':
      return t('connections.state.open');
    case 'synSent':
    case 'synReceived':
      return t('connections.state.connecting');
    case 'finWait1':
    case 'finWait2':
    case 'closeWait':
    case 'closing':
    case 'lastAck':
    case 'timeWait':
      return t('connections.state.closing');
    default:
      return t('connections.state.closed');
  }
}

function matches(c: ConnectionStat, q: string): boolean {
  return (
    c.appName.toLowerCase().includes(q) ||
    (c.host ?? '').toLowerCase().includes(q) ||
    c.remoteIp.includes(q) ||
    String(c.remotePort) === q ||
    (c.service ?? '').includes(q)
  );
}

export function Connections() {
  const unit = rateUnit.value;
  const now = snapshot.value?.at ?? Date.now();
  const shown = useComputed(() => {
    const q = connectionSearch.value.trim().toLowerCase();
    return connections.value.filter(
      (c) => (c.protocol === 'tcp' ? showTcp.value : showUdp.value) && !(hideLocal.value && c.localNetwork) && (!q || matches(c, q)),
    );
  });
  const query = connectionSearch.value.trim();

  return (
    <div class="view">
      <div class="view-head">
        <PageHeader title={t('connections.title')} subtitle={t('connections.subtitle')} />
        <div class="toolbar">
          <SearchBox value={connectionSearch.value} onInput={(v) => (connectionSearch.value = v)} placeholder={t('connections.searchPlaceholder')} />
          <div class="chip-row" role="group" aria-label={t('connections.colProtocol')}>
            <Chip selected={showTcp.value} onClick={() => (showTcp.value = !showTcp.value)}>{t('connections.tcp')}</Chip>
            <Chip selected={showUdp.value} onClick={() => (showUdp.value = !showUdp.value)}>{t('connections.udp')}</Chip>
          </div>
          <Switch checked={hideLocal.value} onChange={(v) => (hideLocal.value = v)} label={t('connections.hideLocal')} />
          <span class="grow" />
          <span class="toolbar-count muted">{t('connections.count', { count: shown.value.length })}</span>
        </div>
      </div>

      {shown.value.length === 0 ? (
        <EmptyState icon="link" title={query ? t('connections.emptySearch', { query }) : t('connections.empty')} />
      ) : (
        <div class="table conn-table" role="table" aria-label={t('connections.title')}>
          <div class="thead" role="row">
            <div class="th" role="columnheader">{t('connections.colApp')}</div>
            <div class="th" role="columnheader">{t('connections.colRemote')}</div>
            <div class="th" role="columnheader">{t('connections.colPort')}</div>
            <div class="th" role="columnheader">{t('connections.colState')}</div>
            <div class="th is-end" role="columnheader">{t('connections.colDownload')}</div>
            <div class="th is-end" role="columnheader">{t('connections.colUpload')}</div>
            <div class="th is-end" role="columnheader">{t('connections.colTotal')}</div>
            <div class="th is-end" role="columnheader">{t('connections.colLastActive')}</div>
          </div>
          {shown.value.map((c) => (
            <div key={`${c.app}|${c.protocol}|${c.remoteIp}|${c.remotePort}|${c.localPort}`} role="row" class={`tr ${c.open ? '' : 'is-closed'}`}>
              <div class="td" role="cell">
                <button type="button" class="link-cell truncate" onClick={() => openApp(c.app)} title={c.appName}>
                  {c.appName}
                </button>
              </div>
              <div class="td td-remote" role="cell">
                <span class="remote-text">
                  <span class="mono truncate" title={c.host ?? undefined}>{c.host ?? formatEndpoint(c.remoteIp, c.remotePort)}</span>
                  {c.host ? <span class="muted mono truncate remote-ip">{c.remoteIp}</span> : null}
                </span>
                {c.localNetwork ? <span class="badge">{t('connections.localNetwork')}</span> : null}
              </div>
              <div class="td" role="cell">
                <span class="port-text mono">
                  <span>
                    {c.protocol.toUpperCase()} {c.remotePort}
                  </span>
                  {c.service ? (
                    <span class="muted truncate port-service" title={c.service}>
                      {c.service}
                    </span>
                  ) : null}
                </span>
              </div>
              <div class="td" role="cell">
                <span class={`state ${c.open ? 'is-open' : ''}`}>{stateLabel(c)}</span>
              </div>
              <div class="td is-end num" role="cell">{formatRate(c.rate.in, unit)}</div>
              <div class="td is-end num" role="cell">{formatRate(c.rate.out, unit)}</div>
              <div class="td is-end num" role="cell">{formatBytes(c.total.in + c.total.out)}</div>
              <div class="td is-end" role="cell">{formatAgo(c.lastActive, now)}</div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
