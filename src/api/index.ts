/**
 * The backend: the Rust commands inside the app, or a simulation with made-up data when the page
 * runs in a plain browser (`npm run dev:web`, the UI tests and the screenshots).
 */
import type { AppOnline, MonitorState, Settings } from './types';
import { call } from './invoke';
import { mockApi } from './mock';

export interface Api {
  readonly simulated: boolean;
  getState(): Promise<MonitorState>;
  getSettings(): Promise<Settings>;
  setSettings(settings: Settings): Promise<Settings>;
  setAppBlocked(path: string, name: string, blocked: boolean): Promise<void>;
  unblockApplication(name: string): Promise<number>;
  unblockAll(): Promise<number>;
  backupFirewall(): Promise<string>;
  revealProgram(path: string): Promise<void>;
  openFirewallConsole(): Promise<void>;
  openUrl(url: string): Promise<void>;
  /** Calls back with apps that reach the internet for the first time; returns an unsubscribe. */
  onAppsOnline(callback: (apps: AppOnline[]) => void): Promise<() => void>;
  notify(title: string, body: string): Promise<void>;
}

const tauriApi: Api = {
  simulated: false,
  getState: () => call('get_state'),
  getSettings: () => call('get_settings'),
  setSettings: (settings) => call('set_settings', { settings }),
  setAppBlocked: (path, name, blocked) => call('set_app_blocked', { path, name, blocked }),
  unblockApplication: (name) => call('unblock_application', { name }),
  unblockAll: () => call('unblock_all'),
  backupFirewall: () => call('backup_firewall'),
  revealProgram: (path) => call('reveal_program', { path }),
  openFirewallConsole: () => call('open_firewall_console'),
  async openUrl(url) {
    const { openUrl } = await import('@tauri-apps/plugin-opener');
    await openUrl(url);
  },
  async onAppsOnline(callback) {
    const { listen } = await import('@tauri-apps/api/event');
    return listen<AppOnline[]>('apps-online', (e) => callback(e.payload));
  },
  async notify(title, body) {
    const n = await import('@tauri-apps/plugin-notification');
    let granted = await n.isPermissionGranted();
    if (!granted) granted = (await n.requestPermission()) === 'granted';
    if (granted) n.sendNotification({ title, body });
  },
};

export const inTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
export const api: Api = inTauri ? tauriApi : mockApi;
