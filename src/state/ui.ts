/** Where the window is: the page, the app open in the detail panel, and the page filters. */
import { signal } from '@preact/signals';

export type Page = 'overview' | 'apps' | 'connections' | 'blocked' | 'settings';

export const page = signal<Page>('overview');
export const selectedApp = signal<string | null>(null);

export const appSearch = signal('');
export const appFilter = signal<'online' | 'all' | 'blocked'>('all');

export const connectionSearch = signal('');
export const showTcp = signal(true);
export const showUdp = signal(true);
export const hideLocal = signal(false);

/** Opens an app's details on the Apps page. */
export function openApp(key: string): void {
  selectedApp.value = key;
  page.value = 'apps';
}

/** Shows the connections of one app. */
export function showConnectionsOf(appName: string): void {
  connectionSearch.value = appName;
  page.value = 'connections';
}
