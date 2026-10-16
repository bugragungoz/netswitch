import { fireEvent, render, screen, waitFor } from '@testing-library/preact';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { AppStat } from '../api/types';
import { setLanguage } from '../i18n';
import { confirmQueue } from '../state/dialogs';
import { InternetSwitch } from './InternetSwitch';

const setAppBlocked = vi.fn(async () => {});
vi.mock('../api', () => ({ api: { setAppBlocked: (...args: unknown[]) => setAppBlocked(...(args as [])) } }));
vi.mock('../state/monitor', () => ({ refresh: async () => {} }));

function app(over: Partial<AppStat> = {}): AppStat {
  return {
    key: 'c:\\apps\\demo.exe',
    name: 'Demo',
    path: 'C:\\Apps\\demo.exe',
    pids: [10],
    rate: { in: 0, out: 0 },
    total: { in: 0, out: 0 },
    history: [],
    connections: 0,
    listening: 0,
    hosts: 0,
    firstSeen: 0,
    lastActive: null,
    blocked: false,
    protected: false,
    ...over,
  };
}

beforeEach(() => {
  setLanguage('en');
  setAppBlocked.mockClear();
  confirmQueue.value = [];
});

describe('InternetSwitch', () => {
  it('asks before blocking and does nothing when the answer is no', async () => {
    render(<InternetSwitch app={app()} />);
    fireEvent.click(screen.getByRole('switch', { name: 'Internet access for Demo' }));
    await waitFor(() => expect(confirmQueue.value).toHaveLength(1));
    expect(confirmQueue.value[0]!.title).toBe('Block Demo?');
    expect(confirmQueue.value[0]!.body).toContain('C:\\Apps\\demo.exe');
    confirmQueue.value[0]!.resolve(false);
    await Promise.resolve();
    expect(setAppBlocked).not.toHaveBeenCalled();
  });

  it('blocks after a yes', async () => {
    render(<InternetSwitch app={app()} />);
    fireEvent.click(screen.getByRole('switch'));
    await waitFor(() => expect(confirmQueue.value).toHaveLength(1));
    confirmQueue.value[0]!.resolve(true);
    await waitFor(() => expect(setAppBlocked).toHaveBeenCalledWith('C:\\Apps\\demo.exe', 'Demo', true));
  });

  it('gives the internet back without asking', async () => {
    render(<InternetSwitch app={app({ blocked: true })} />);
    const sw = screen.getByRole('switch');
    expect(sw.getAttribute('aria-checked')).toBe('false');
    fireEvent.click(sw);
    await waitFor(() => expect(setAppBlocked).toHaveBeenCalledWith('C:\\Apps\\demo.exe', 'Demo', false));
    expect(confirmQueue.value).toHaveLength(0);
  });

  it('cannot switch off a part of Windows or a process without a program', () => {
    render(<InternetSwitch app={app({ name: 'Host', protected: true })} />);
    render(<InternetSwitch app={app({ name: 'System', path: null })} />);
    const [windows, system] = screen.getAllByRole('switch') as HTMLButtonElement[];
    expect(windows!.disabled).toBe(true);
    expect(windows!.textContent).toBe('Part of Windows');
    expect(system!.disabled).toBe(true);
    expect(system!.textContent).toBe('No program');
  });
});
