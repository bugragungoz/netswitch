/**
 * A made-up computer for the plain browser: `npm run dev:web`, the UI tests and the screenshots.
 * Every app, path and address here is invented; the addresses come from the ranges reserved for
 * documentation (192.0.2.0/24, 198.51.100.0/24, 203.0.113.0/24) and a private home network.
 * The traffic follows simple patterns with seeded noise, so screenshots look the same each time.
 */
import type { Api } from './index';
import type { AppOnline, AppStat, BlockedApp, ConnectionStat, InOut, MonitorState, Protocol, Settings } from './types';

interface Host {
  ip: string;
  host: string | null;
  port: number;
  protocol: Protocol;
  share: number;
  local?: boolean;
  service?: string;
}

interface SimApp {
  key: string;
  name: string;
  path: string | null;
  pids: number[];
  listening: number;
  hosts: Host[];
  /** Bytes per second at second `t` of the simulation. */
  profile: (t: number) => InOut;
  blocked: boolean;
  protected: boolean;
  // Running state
  total: InOut;
  history: InOut[];
  firstSeen: number;
  lastActive: number | null;
  rate: InOut;
  flows: Map<string, { rate: InOut; total: InOut; lastActive: number | null }>;
}

const KB = 1000;
const MB = 1000 * 1000;

/** mulberry32: small seeded random numbers, so the demo is the same on every run. */
function seeded(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const rnd = seeded(20261016);
const jitter = (v: number, amount = 0.25) => Math.max(0, v * (1 + (rnd() - 0.5) * 2 * amount));
const io = (down: number, up: number): InOut => ({ in: Math.round(down), out: Math.round(up) });
/** The position of second `t` in a cycle of `n` seconds, also for the negative seconds before the page opened. */
const cycle = (t: number, n: number) => ((t % n) + n) % n;

function app(key: string, name: string, path: string | null, pids: number[], hosts: Host[], profile: (t: number) => InOut, extra: Partial<SimApp> = {}): SimApp {
  return {
    key, name, path, pids, hosts, profile, listening: 0, blocked: false, protected: false,
    total: io(0, 0), history: [], firstSeen: 0, lastActive: null, rate: io(0, 0), flows: new Map(), ...extra,
  };
}

function buildApps(): SimApp[] {
  return [
    app('c:\\program files\\google\\chrome\\application\\chrome.exe', 'Google Chrome', 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe', [6120, 6188, 7012, 7344],
      [
        { ip: '203.0.113.10', host: 'rr4---sn-demo.googlevideo.com', port: 443, protocol: 'tcp', share: 0.86 },
        { ip: '203.0.113.11', host: 'www.youtube.com', port: 443, protocol: 'tcp', share: 0.08 },
        { ip: '203.0.113.12', host: 'i.ytimg.com', port: 443, protocol: 'tcp', share: 0.05 },
        { ip: '203.0.113.13', host: 'fonts.gstatic.com', port: 443, protocol: 'udp', share: 0.01 },
      ],
      (t) => (cycle(t, 12) < 2 ? io(jitter(5.8 * MB, 0.1), jitter(70 * KB)) : io(jitter(1.1 * MB, 0.3), jitter(28 * KB))),
      { listening: 3 }),
    app('c:\\program files (x86)\\steam\\steam.exe', 'Steam', 'C:\\Program Files (x86)\\Steam\\steam.exe', [9020],
      [
        { ip: '198.51.100.21', host: 'cache1-fra1.steamcontent.com', port: 443, protocol: 'tcp', share: 0.52 },
        { ip: '198.51.100.22', host: 'cache2-fra1.steamcontent.com', port: 443, protocol: 'tcp', share: 0.47 },
        { ip: '198.51.100.25', host: 'api.steampowered.com', port: 443, protocol: 'tcp', share: 0.01 },
      ],
      (t) => (t > -170 ? io(jitter(7.4 * MB, 0.18), jitter(95 * KB)) : io(jitter(3 * KB), jitter(1 * KB))),
      { listening: 2 }),
    app('c:\\program files\\microsoft onedrive\\onedrive.exe', 'Microsoft OneDrive', 'C:\\Program Files\\Microsoft OneDrive\\OneDrive.exe', [4410],
      [
        { ip: '192.0.2.41', host: 'my.microsoftpersonalcontent.com', port: 443, protocol: 'tcp', share: 0.93 },
        { ip: '192.0.2.40', host: 'graph.microsoft.com', port: 443, protocol: 'tcp', share: 0.07 },
      ],
      (t) => (cycle(t, 40) < 26 ? io(jitter(14 * KB), jitter(420 * KB, 0.2)) : io(jitter(2 * KB), jitter(4 * KB)))),
    app('c:\\users\\demo\\appdata\\roaming\\spotify\\spotify.exe', 'Spotify', 'C:\\Users\\demo\\AppData\\Roaming\\Spotify\\Spotify.exe', [11032, 11240],
      [
        { ip: '198.51.100.60', host: 'audio-ak.spotifycdn.com', port: 443, protocol: 'tcp', share: 0.92 },
        { ip: '198.51.100.61', host: 'spclient.wg.spotify.com', port: 443, protocol: 'tcp', share: 0.08 },
      ],
      (t) => (cycle(t, 30) < 3 ? io(jitter(1.3 * MB, 0.1), jitter(9 * KB)) : io(jitter(1.5 * KB), jitter(0.8 * KB))),
      { listening: 1 }),
    app('c:\\users\\demo\\appdata\\local\\discord\\app-1.0.9200\\discord.exe', 'Discord', 'C:\\Users\\demo\\AppData\\Local\\Discord\\app-1.0.9200\\Discord.exe', [8800, 8812],
      [
        { ip: '203.0.113.80', host: 'frankfurt4410.discord.media', port: 50004, protocol: 'udp', share: 0.97 },
        { ip: '203.0.113.81', host: 'gateway.discord.gg', port: 443, protocol: 'tcp', share: 0.03 },
      ],
      () => io(jitter(10 * KB, 0.15), jitter(8.5 * KB, 0.15))),
    app('c:\\windows\\system32\\svchost.exe', 'Host Process for Windows Services', 'C:\\Windows\\System32\\svchost.exe', [1188, 2264, 3340],
      [
        { ip: '192.0.2.90', host: '7.tlu.dl.delivery.mp.microsoft.com', port: 80, protocol: 'tcp', share: 0.72 },
        { ip: '192.168.1.34', host: 'desktop-living.local', port: 7680, protocol: 'tcp', share: 0.25, local: true },
        { ip: '192.168.1.1', host: 'router.local', port: 53, protocol: 'udp', share: 0.03, local: true },
      ],
      (t) => (cycle(t, 50) < 18 ? io(jitter(540 * KB, 0.3), jitter(22 * KB)) : io(jitter(4 * KB), jitter(2 * KB))),
      { listening: 14, protected: true }),
    app('pid:4', 'System', null, [4],
      [{ ip: '192.168.1.20', host: 'nas.local', port: 445, protocol: 'tcp', share: 1, local: true }],
      (t) => (cycle(t, 20) < 6 ? io(jitter(130 * KB), jitter(12 * KB)) : io(0, 0)),
      { listening: 9 }),
    app('c:\\users\\demo\\appdata\\local\\programs\\microsoft vs code\\code.exe', 'Visual Studio Code', 'C:\\Users\\demo\\AppData\\Local\\Programs\\Microsoft VS Code\\Code.exe', [12880],
      [
        { ip: '198.51.100.100', host: 'update.code.visualstudio.com', port: 443, protocol: 'tcp', share: 0.6 },
        { ip: '198.51.100.101', host: 'marketplace.visualstudio.com', port: 443, protocol: 'tcp', share: 0.4 },
      ],
      (t) => (cycle(t, 90) < 4 ? io(jitter(60 * KB), jitter(6 * KB)) : io(0, 0))),
    app('c:\\program files (x86)\\microsoft\\edgewebview\\application\\msedgewebview2.exe', 'Microsoft Edge WebView2', 'C:\\Program Files (x86)\\Microsoft\\EdgeWebView\\Application\\msedgewebview2.exe', [5120],
      [{ ip: '192.0.2.120', host: 'login.live.com', port: 443, protocol: 'tcp', share: 1 }],
      () => io(jitter(2.5 * KB, 0.5), jitter(1.2 * KB, 0.5))),
    app('c:\\program files\\qbittorrent\\qbittorrent.exe', 'qBittorrent', 'C:\\Program Files\\qBittorrent\\qbittorrent.exe', [3900],
      [{ ip: '203.0.113.150', host: null, port: 51413, protocol: 'tcp', share: 1 }],
      () => io(jitter(900 * KB), jitter(600 * KB)),
      { listening: 2, blocked: true }),
    app('c:\\users\\demo\\appdata\\roaming\\zoom\\bin\\zoom.exe', 'Zoom Workplace', 'C:\\Users\\demo\\AppData\\Roaming\\Zoom\\bin\\Zoom.exe', [7700],
      [{ ip: '198.51.100.140', host: 'zoom.us', port: 443, protocol: 'tcp', share: 1 }],
      (t) => (cycle(t, 120) < 2 ? io(jitter(3 * KB), jitter(1 * KB)) : io(0, 0)),
      { listening: 1 }),
  ];
}

const apps = buildApps();
const blockedFromScript: BlockedApp = {
  name: 'Adobe Photoshop 2025',
  programs: [
    'C:\\Program Files\\Adobe\\Adobe Photoshop 2025\\Photoshop.exe',
    'C:\\Program Files\\Adobe\\Adobe Photoshop 2025\\node.exe',
    'C:\\Program Files\\Adobe\\Adobe Photoshop 2025\\Required\\UXP\\com.adobe.ccx.start\\CCXProcess.exe',
  ],
  rules: 6,
  complete: true,
};
let extraBlocked: BlockedApp[] = [blockedFromScript];
let systemHistory: InOut[] = [];
let systemTotal: InOut = io(0, 0);
let systemRate: InOut = io(0, 0);
let clock = 0; // seconds of simulation
let lastStepAt = 0;
let started = 0;

function stepOnce(now: number) {
  let sum = io(0, 0);
  for (const a of apps) {
    const r = a.blocked ? io(0, 0) : a.profile(clock);
    a.rate = r;
    a.total = io(a.total.in + r.in, a.total.out + r.out);
    a.history = [...a.history.slice(-59), r];
    if (r.in + r.out > 0) a.lastActive = now;
    for (const h of a.hosts) {
      const key = `${h.ip}|${h.port}|${h.protocol}`;
      const f = a.flows.get(key) ?? { rate: io(0, 0), total: io(0, 0), lastActive: null };
      f.rate = io(r.in * h.share, r.out * h.share);
      f.total = io(f.total.in + f.rate.in, f.total.out + f.rate.out);
      if (f.rate.in + f.rate.out > 0) f.lastActive = now;
      a.flows.set(key, f);
    }
    sum = io(sum.in + r.in, sum.out + r.out);
  }
  // The adapters also carry what no app owns (ARP, ICMP, other devices' broadcasts).
  systemRate = io(sum.in + jitter(3 * KB), sum.out + jitter(1.5 * KB));
  systemTotal = io(systemTotal.in + systemRate.in, systemTotal.out + systemRate.out);
  systemHistory = [...systemHistory.slice(-299), systemRate];
  clock++;
}

function ensureStarted() {
  if (started) return;
  started = Date.now();
  // Five minutes of history before the page opened, so the chart starts full.
  clock = -300;
  for (let i = 0; i < 300; i++) stepOnce(started - (300 - i) * 1000);
  for (const a of apps) a.firstSeen = started - 300_000 + Math.floor(rnd() * 60_000);
  lastStepAt = started;
}

function advance() {
  ensureStarted();
  const now = Date.now();
  const steps = Math.min(10, Math.floor((now - lastStepAt) / 1000));
  for (let i = 0; i < steps; i++) stepOnce(lastStepAt + (i + 1) * 1000);
  if (steps > 0) lastStepAt += steps * 1000;
}

function blockedList(): BlockedApp[] {
  const own = apps
    .filter((a) => a.blocked && a.path)
    .map((a) => ({ name: a.name, programs: [a.path!], rules: 2, complete: true }));
  return [...own, ...extraBlocked].sort((x, y) => x.name.localeCompare(y.name));
}

function state(): MonitorState {
  advance();
  const now = lastStepAt;
  const appStats: AppStat[] = apps.map((a) => ({
    key: a.key, name: a.name, path: a.path, pids: a.pids, rate: a.rate, total: a.total, history: a.history,
    connections: a.blocked ? 0 : a.hosts.filter((h) => a.flows.get(`${h.ip}|${h.port}|${h.protocol}`)?.lastActive).length,
    listening: a.listening, hosts: a.hosts.length, firstSeen: a.firstSeen, lastActive: a.lastActive, blocked: a.blocked, protected: a.protected,
  }));
  appStats.sort((x, y) => y.rate.in + y.rate.out - (x.rate.in + x.rate.out) || y.total.in + y.total.out - (x.total.in + x.total.out));
  const connections: ConnectionStat[] = [];
  for (const a of apps) {
    for (const h of a.hosts) {
      const f = a.flows.get(`${h.ip}|${h.port}|${h.protocol}`);
      if (!f || !f.lastActive) continue;
      const open = !a.blocked && (h.protocol === 'tcp' || now - f.lastActive < 30_000);
      connections.push({
        app: a.key, appName: a.name, protocol: h.protocol, localPort: 49152 + (h.ip.length * 97 + h.port) % 16000,
        remoteIp: h.ip, remotePort: h.port, host: h.host, service: h.port === 443 ? (h.protocol === 'udp' ? 'quic' : 'https') : h.port === 80 ? 'http' : h.port === 53 ? 'dns' : h.port === 445 ? 'smb' : h.port === 7680 ? 'delivery-optimization' : null,
        localNetwork: !!h.local, state: h.protocol === 'tcp' ? (open ? 'established' : 'timeWait') : null, open,
        rate: f.rate, total: f.total, lastActive: f.lastActive,
      });
    }
  }
  connections.sort((x, y) => Number(y.open) - Number(x.open) || y.rate.in + y.rate.out - (x.rate.in + x.rate.out) || y.total.in + y.total.out - (x.total.in + x.total.out));
  return {
    supported: true,
    admin: true,
    traceError: null,
    errors: [],
    snapshot: { at: now, perApp: true, rate: systemRate, total: systemTotal, history: systemHistory, apps: appStats, connections },
    blocked: blockedList(),
  };
}

const SETTINGS_KEY = 'netswitch-demo-settings';
const DEFAULT_SETTINGS: Settings = { theme: 'system', language: 'system', notifyNewApps: true, dailyBackup: true, rateUnit: 'bytes' };

function readSettings(): Settings {
  try {
    const raw = localStorage.getItem(SETTINGS_KEY);
    return raw ? { ...DEFAULT_SETTINGS, ...(JSON.parse(raw) as Partial<Settings>) } : DEFAULT_SETTINGS;
  } catch {
    return DEFAULT_SETTINGS;
  }
}

const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));

export const mockApi: Api = {
  simulated: true,
  async getState() {
    return state();
  },
  async getSettings() {
    return readSettings();
  },
  async setSettings(settings) {
    try {
      localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
    } catch {
      // Private windows may refuse storage; the setting then lasts until the page closes.
    }
    return settings;
  },
  async setAppBlocked(path, _name, blocked) {
    await wait(250);
    const a = apps.find((x) => x.path === path);
    if (!a) throw { code: 'not_found', detail: path };
    if (blocked && a.protected) throw { code: 'protected_program', detail: a.name };
    a.blocked = blocked;
  },
  async unblockApplication(name) {
    await wait(200);
    let removed = 0;
    for (const a of apps) {
      if (a.blocked && a.name === name) {
        a.blocked = false;
        removed += 2;
      }
    }
    const before = extraBlocked.length;
    extraBlocked = extraBlocked.filter((b) => {
      if (b.name !== name) return true;
      removed += b.rules;
      return false;
    });
    if (!removed && before === extraBlocked.length) throw { code: 'not_found', detail: name };
    return removed;
  },
  async unblockAll() {
    await wait(300);
    const removed = blockedList().reduce((n, b) => n + b.rules, 0);
    for (const a of apps) a.blocked = false;
    extraBlocked = [];
    return removed;
  },
  async backupFirewall() {
    await wait(400);
    const d = new Date();
    const pad = (n: number) => String(n).padStart(2, '0');
    return `C:\\Users\\demo\\Documents\\netSwitch\\Backups\\FirewallBackup-${d.getFullYear()}${pad(d.getMonth() + 1)}${pad(d.getDate())}-${pad(d.getHours())}${pad(d.getMinutes())}${pad(d.getSeconds())}.wfw`;
  },
  async revealProgram() {},
  async openFirewallConsole() {},
  async openUrl(url) {
    window.open(url, '_blank', 'noopener');
  },
  async onAppsOnline(callback) {
    // A new app goes online a little after the page opens, unless a test asked for quiet.
    if (new URLSearchParams(location.search).has('quiet')) return () => {};
    const timer = setTimeout(() => callback([{ key: 'c:\\program files\\demo\\updater.exe', name: 'Demo Updater' } satisfies AppOnline]), 20_000);
    return () => clearTimeout(timer);
  },
  async notify() {},
};
