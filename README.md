# NetSwitch

Windows firewall rule manager built with Tauri and PowerShell. Uses dynamic firewall scripts to block application internet access with granular control over directories, file types, and exclusions. Modern UI powered by shadcn/ui components.

> Built with Claude Opus 4.5

## Features

- **Block Applications**: Create firewall rules to block internet access for applications
  - Scan directories for executables (*.exe, *.dll)
  - Include/exclude subdirectories
  - Keywords and file exclusions
  - Inbound and outbound rule creation

- **Firewall Management**: View and manage Windows Firewall rules
  - Statistics overview (total, inbound, outbound, enabled, disabled)
  - NetSwitch-created rules management
  - Direct access to Windows Firewall console (wf.msc)

- **Network Monitor**: Real-time network activity monitoring
  - Active connections by process (TCP/UDP)
  - Network interface statistics
  - Live bandwidth graph
  - Interface filtering (hide virtual adapters)

- **System Tools**: Quick access to Windows network utilities
  - Windows Firewall (wf.msc)
  - Network Connections (ncpa.cpl)
  - Internet Properties (inetcpl.cpl)
  - Device Manager (devmgmt.msc)
  - Network Shell (netsh)
  - Resource Monitor (resmon)

- **System Restore**: Create system restore points before making changes

## Requirements

- Windows 10 or later
- Administrator privileges for firewall operations

## Installation

Download the latest installer from [Releases](https://github.com/bugragungoz/netswitch/releases).

## Development

```bash
npm install
npm run tauri dev
```

## License

MIT License - see [LICENSE](LICENSE)
