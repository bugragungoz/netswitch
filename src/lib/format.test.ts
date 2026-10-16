import { describe, expect, it } from 'vitest';
import { setLanguage } from '../i18n';
import { formatAgo, formatBytes, formatEndpoint, formatRate, niceCeiling } from './format';

describe('format', () => {
  it('scales bytes with one decimal below ten', () => {
    setLanguage('en');
    expect(formatBytes(0)).toBe('0 B');
    expect(formatBytes(999)).toBe('999 B');
    expect(formatBytes(1500)).toBe('1.5 KB');
    expect(formatBytes(12_345_678)).toBe('12 MB');
    expect(formatBytes(2_500_000_000)).toBe('2.5 GB');
  });

  it('shows speeds in bytes or bits', () => {
    setLanguage('en');
    expect(formatRate(2_400_000, 'bytes')).toBe('2.4 MB/s');
    expect(formatRate(2_400_000, 'bits')).toBe('19 Mbit/s');
    expect(formatRate(100, 'bits')).toBe('800 bit/s');
  });

  it('uses the language for numbers', () => {
    setLanguage('tr');
    expect(formatBytes(1500)).toBe('1,5 KB');
    setLanguage('en');
  });

  it('says how long ago', () => {
    setLanguage('en');
    const now = 1_000_000;
    expect(formatAgo(null, now)).toBe('never');
    expect(formatAgo(now - 500, now)).toBe('now');
    expect(formatAgo(now - 12_000, now)).toBe('12 s ago');
    expect(formatAgo(now - 240_000, now)).toBe('4 min ago');
    expect(formatAgo(now - 7_200_000, now)).toBe('2 h ago');
  });

  it('writes endpoints', () => {
    expect(formatEndpoint('1.2.3.4', 443)).toBe('1.2.3.4:443');
    expect(formatEndpoint('2001:db8::1', 443)).toBe('[2001:db8::1]:443');
  });

  it('rounds axis tops to clean steps', () => {
    expect(niceCeiling(0)).toBe(1000);
    expect(niceCeiling(1_700_000)).toBe(2_000_000);
    expect(niceCeiling(2_100_000)).toBe(2_500_000);
    expect(niceCeiling(6_000)).toBe(10_000);
  });
});
