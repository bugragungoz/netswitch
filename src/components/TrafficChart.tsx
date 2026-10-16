/**
 * Download and upload speed over time: two series on one axis (both bytes per second).
 * Download is the one people watch, so it wears the accent; upload is a quiet neutral line. A
 * legend names both, a crosshair snaps to the nearest second and lists both values, and the
 * arrow keys move it when the chart has focus.
 */
import { useEffect, useRef, useState } from 'preact/hooks';
import type { InOut } from '../api/types';
import { t } from '../i18n';
import { formatRate, niceCeiling, type RateUnit } from '../lib/format';

interface Props {
  history: InOut[];
  /** Seconds the full width stands for; shorter histories sit at the right edge. */
  span: number;
  unit: RateUnit;
  label: string;
  height?: number;
  /** A small chart: no axes, no legend, no crosshair. */
  compact?: boolean;
}

const PAD = { top: 10, right: 12, bottom: 24, left: 64 };
const PAD_COMPACT = { top: 4, right: 2, bottom: 2, left: 2 };

function agoLabel(seconds: number): string {
  if (seconds <= 0) return t('overview.now');
  if (seconds < 60) return t('overview.secondsAgo', { count: seconds });
  return t('overview.minutesAgo', { count: Math.round(seconds / 60) });
}

export function TrafficChart({ history, span, unit, label, height = 220, compact = false }: Props) {
  const box = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(600);
  const [hover, setHover] = useState<number | null>(null);

  useEffect(() => {
    const el = box.current;
    if (!el || typeof ResizeObserver === 'undefined') return;
    const ro = new ResizeObserver(([entry]) => setWidth(Math.max(120, Math.floor(entry!.contentRect.width))));
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const pad = compact ? PAD_COMPACT : PAD;
  const w = Math.max(10, width - pad.left - pad.right);
  const h = Math.max(10, height - pad.top - pad.bottom);
  const peak = history.reduce((m, p) => Math.max(m, p.in, p.out), 0);
  const top = niceCeiling(peak);
  const n = history.length;
  // Index i (0 oldest) of n points sits `n - 1 - i` seconds before now.
  const x = (i: number) => pad.left + w - ((n - 1 - i) / Math.max(1, span - 1)) * w;
  const y = (v: number) => pad.top + h - (v / top) * h;

  const line = (key: 'in' | 'out') => history.map((p, i) => `${i ? 'L' : 'M'}${x(i).toFixed(1)} ${y(p[key]).toFixed(1)}`).join('');
  const area = (key: 'in' | 'out') => (n ? `${line(key)}L${x(n - 1).toFixed(1)} ${pad.top + h}L${x(0).toFixed(1)} ${pad.top + h}Z` : '');

  const pick = (clientX: number) => {
    const rect = box.current?.getBoundingClientRect();
    if (!rect || !n) return;
    const px = clientX - rect.left;
    const secondsAgo = Math.round(((pad.left + w - px) / w) * (span - 1));
    setHover(Math.min(n - 1, Math.max(0, n - 1 - secondsAgo)));
  };

  const onKey = (e: KeyboardEvent) => {
    if (!n) return;
    const current = hover ?? n - 1;
    const step = e.shiftKey ? 10 : 1;
    if (e.key === 'ArrowLeft') setHover(Math.max(0, current - step));
    else if (e.key === 'ArrowRight') setHover(Math.min(n - 1, current + step));
    else if (e.key === 'Home') setHover(0);
    else if (e.key === 'End') setHover(n - 1);
    else if (e.key === 'Escape') setHover(null);
    else return;
    e.preventDefault();
  };

  const ticks = [0, top / 2, top];
  const xTicks = compact ? [] : [span - 1, Math.round((span - 1) * 0.6), Math.round((span - 1) * 0.2), 0].filter((s, i, a) => a.indexOf(s) === i);
  const hoverPoint = hover !== null ? history[hover] : null;
  const latest = history[n - 1];

  return (
    <div class={`chart ${compact ? 'chart-compact' : ''}`}>
      {compact ? null : (
        <div class="chart-legend" aria-hidden="true">
          <span class="legend-item">
            <span class="legend-key legend-down" />
            {t('common.download')}
            <strong class="num">{formatRate(latest?.in ?? 0, unit)}</strong>
          </span>
          <span class="legend-item">
            <span class="legend-key legend-up" />
            {t('common.upload')}
            <strong class="num">{formatRate(latest?.out ?? 0, unit)}</strong>
          </span>
          <span class="legend-peak muted">{t('overview.peak', { value: formatRate(peak, unit) })}</span>
        </div>
      )}
      <div
        ref={box}
        class="chart-plot"
        style={{ height: `${height}px` }}
        tabIndex={compact ? -1 : 0}
        role="img"
        aria-label={`${label}. ${t('common.download')} ${formatRate(latest?.in ?? 0, unit)}, ${t('common.upload')} ${formatRate(latest?.out ?? 0, unit)}. ${t('overview.peak', { value: formatRate(peak, unit) })}.`}
        onPointerMove={compact ? undefined : (e) => pick(e.clientX)}
        onPointerLeave={compact ? undefined : () => setHover(null)}
        onKeyDown={compact ? undefined : onKey}
        onBlur={() => setHover(null)}
      >
        <svg width={width} height={height} aria-hidden="true">
          {compact
            ? null
            : ticks.map((v) => (
                <g key={v}>
                  <line class="chart-grid" x1={pad.left} x2={pad.left + w} y1={y(v)} y2={y(v)} />
                  <text class="chart-tick" x={pad.left - 8} y={y(v)} text-anchor="end" dominant-baseline="middle">
                    {formatRate(v, unit)}
                  </text>
                </g>
              ))}
          {xTicks.map((s) => (
            <text key={s} class="chart-tick" x={pad.left + w - (s / Math.max(1, span - 1)) * w} y={height - 6} text-anchor={s === 0 ? 'end' : s === span - 1 ? 'start' : 'middle'}>
              {agoLabel(s)}
            </text>
          ))}
          <path class="chart-area-up" d={area('out')} />
          <path class="chart-area-down" d={area('in')} />
          <path class="chart-line-up" d={line('out')} />
          <path class="chart-line-down" d={line('in')} />
          {hoverPoint && hover !== null ? (
            <g>
              <line class="chart-crosshair" x1={x(hover)} x2={x(hover)} y1={pad.top} y2={pad.top + h} />
              <circle class="chart-dot-up" cx={x(hover)} cy={y(hoverPoint.out)} r="4" />
              <circle class="chart-dot-down" cx={x(hover)} cy={y(hoverPoint.in)} r="4" />
            </g>
          ) : null}
        </svg>
        {hoverPoint && hover !== null ? (
          <div class="chart-tooltip" style={{ left: `${Math.min(width - 190, Math.max(0, x(hover) + 12))}px` }} role="status">
            <div class="muted">{agoLabel(n - 1 - hover)}</div>
            <div class="tooltip-row">
              <span class="legend-key legend-down" />
              <strong class="num">{formatRate(hoverPoint.in, unit)}</strong>
              <span class="muted">{t('common.download')}</span>
            </div>
            <div class="tooltip-row">
              <span class="legend-key legend-up" />
              <strong class="num">{formatRate(hoverPoint.out, unit)}</strong>
              <span class="muted">{t('common.upload')}</span>
            </div>
          </div>
        ) : null}
      </div>
    </div>
  );
}
