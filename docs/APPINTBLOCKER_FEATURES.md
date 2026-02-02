# AppIntBlockerGUI Feature Reference

Reference list of features from the WPF AppIntBlockerGUI project for future implementation in this Tauri app.

## Implemented in app-blocker-gui

- Block Application (folder, exe/dll, exclusions)
- Manage Rules (list, remove by app, remove all)
- Restore Point creation
- Windows Firewall (open wf.msc)
- Dark/Light theme
- Custom window frame
- UAC handling; single elevation when app run as admin

## To Implement (from AppIntBlockerGUI)

- **Settings**: App settings, theme persistence, options
- **Network Monitor**: Real-time per-process bandwidth, one-click block from monitor (requires Rust backend for network stats / NetStat or similar)
- **Restore Points**: List/restore snapshots (WPF has snapshot list; we only have create for now)
- **Sidebar navigation**: Optional sidebar like WPF (Block, Manage, Restore, Windows Firewall, Settings, Network Monitor)
- **System status**: Uptime, active rules count, admin mode indicator in UI

## Technical Notes

- Network Monitor in WPF uses .NET network APIs; Tauri equivalent would need Windows APIs via Rust (e.g. GetTcpTable/GetUdpTable or a crate).
- Restore point listing/restore: PowerShell or WMI from Rust.
