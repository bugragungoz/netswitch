/** The monitor's state, read once a second while the window is open. */
import { computed, signal } from '@preact/signals';
import { api } from '../api';
import type { AppStat, MonitorState } from '../api/types';

export const monitor = signal<MonitorState | null>(null);
/** Set when reading the state failed; cleared by the next good read. */
export const readError = signal<string | null>(null);

export const snapshot = computed(() => monitor.value?.snapshot ?? null);
export const apps = computed<AppStat[]>(() => snapshot.value?.apps ?? []);
export const connections = computed(() => snapshot.value?.connections ?? []);
export const blocked = computed(() => monitor.value?.blocked ?? []);
/** Apps with bytes in the last second. */
export const onlineApps = computed(() => apps.value.filter((a) => a.rate.in + a.rate.out > 0));
export const openConnections = computed(() => connections.value.filter((c) => c.open));

let timer: ReturnType<typeof setTimeout> | null = null;

export async function refresh(): Promise<void> {
  try {
    monitor.value = await api.getState();
    readError.value = null;
  } catch (e) {
    readError.value = e instanceof Error ? e.message : String(e);
  }
}

/** Reads the state now and then once a second. */
export function startPolling(): void {
  if (timer) return;
  const tick = async () => {
    await refresh();
    timer = setTimeout(tick, 1000);
  };
  void tick();
}

export function stopPolling(): void {
  if (timer) clearTimeout(timer);
  timer = null;
}
