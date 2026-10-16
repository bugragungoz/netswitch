/** The shapes the Rust side sends (crates/netswitch-core/src/model.rs and src-tauri). */

/** Bytes, or bytes per second: received (in, download) and sent (out, upload). */
export interface InOut {
  in: number;
  out: number;
}

export type Protocol = 'tcp' | 'udp';

export type TcpState =
  | 'closed' | 'listen' | 'synSent' | 'synReceived' | 'established' | 'finWait1' | 'finWait2'
  | 'closeWait' | 'closing' | 'lastAck' | 'timeWait' | 'deleteTcb';

export interface AppStat {
  key: string;
  name: string;
  path: string | null;
  pids: number[];
  rate: InOut;
  total: InOut;
  /** Bytes per second over the last minute, oldest first. */
  history: InOut[];
  connections: number;
  listening: number;
  hosts: number;
  firstSeen: number;
  lastActive: number | null;
  blocked: boolean;
  protected: boolean;
}

export interface ConnectionStat {
  app: string;
  appName: string;
  protocol: Protocol;
  localPort: number;
  remoteIp: string;
  remotePort: number;
  host: string | null;
  service: string | null;
  localNetwork: boolean;
  state: TcpState | null;
  open: boolean;
  rate: InOut;
  total: InOut;
  lastActive: number | null;
}

export interface Snapshot {
  at: number;
  perApp: boolean;
  rate: InOut;
  total: InOut;
  /** Bytes per second of all adapters over the last five minutes, oldest first. */
  history: InOut[];
  apps: AppStat[];
  connections: ConnectionStat[];
}

export interface BlockedApp {
  name: string;
  programs: string[];
  rules: number;
  complete: boolean;
}

export interface MonitorState {
  supported: boolean;
  admin: boolean;
  traceError: string | null;
  errors: string[];
  snapshot: Snapshot;
  blocked: BlockedApp[];
}

export interface Settings {
  theme: 'system' | 'dark' | 'light';
  language: 'system' | 'en' | 'tr';
  notifyNewApps: boolean;
  dailyBackup: boolean;
  rateUnit: 'bytes' | 'bits';
}

export interface AppOnline {
  key: string;
  name: string;
}
