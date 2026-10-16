/**
 * Bytes and speeds for people. Units are decimal (1 MB = 1,000,000 bytes), as network speeds are
 * sold and measured; speeds can be shown in bytes (MB/s) or bits (Mbit/s).
 */
import { formatNumber, t } from '../i18n';

export type RateUnit = 'bytes' | 'bits';

const BYTE_UNITS = ['B', 'KB', 'MB', 'GB', 'TB'];
const BIT_UNITS = ['bit/s', 'kbit/s', 'Mbit/s', 'Gbit/s', 'Tbit/s'];

/** A value and its unit: one decimal below 10, none above, so columns stay calm. */
function scaled(value: number, units: string[]): string {
  let i = 0;
  let v = Math.max(0, value);
  while (v >= 1000 && i < units.length - 1) {
    v /= 1000;
    i++;
  }
  const digits = i === 0 || v >= 10 ? 0 : 1;
  return `${formatNumber(v, { maximumFractionDigits: digits, minimumFractionDigits: digits })} ${units[i]}`;
}

export function formatBytes(bytes: number): string {
  return scaled(bytes, BYTE_UNITS);
}

export function formatRate(bytesPerSecond: number, unit: RateUnit): string {
  return unit === 'bits' ? scaled(bytesPerSecond * 8, BIT_UNITS) : `${scaled(bytesPerSecond, BYTE_UNITS)}/s`;
}

/** "now", "12 s ago", "4 min ago", "2 h ago". */
export function formatAgo(at: number | null | undefined, now: number): string {
  if (!at) return t('time.never');
  const s = Math.max(0, Math.round((now - at) / 1000));
  if (s < 2) return t('time.now');
  if (s < 60) return t('time.secondsAgo', { count: s });
  if (s < 3600) return t('time.minutesAgo', { count: Math.floor(s / 60) });
  return t('time.hoursAgo', { count: Math.floor(s / 3600) });
}

/** An address with its port; IPv6 in brackets. */
export function formatEndpoint(ip: string, port: number): string {
  return ip.includes(':') ? `[${ip}]:${port}` : `${ip}:${port}`;
}

/** A tick value for a rate axis: a clean round step above the peak. */
export function niceCeiling(value: number): number {
  if (value <= 0) return 1000;
  const exp = Math.pow(10, Math.floor(Math.log10(value)));
  for (const step of [1, 2, 2.5, 5, 10]) {
    if (value <= step * exp) return step * exp;
  }
  return 10 * exp;
}
