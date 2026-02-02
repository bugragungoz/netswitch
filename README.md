# NetSwitch

Windows firewall rule manager and network control application built with Tauri, React, and TypeScript.

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

## Tech Stack

- **Frontend**: React 18, TypeScript, Tailwind CSS, shadcn/ui, Recharts
- **Backend**: Rust, Tauri 2
- **Platform**: Windows 10/11

## Requirements

- Windows 10 or later
- Administrator privileges for firewall operations

## Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Build Output

Production builds are located in:
- `src-tauri/target/release/NetSwitch.exe` (standalone)
- `src-tauri/target/release/bundle/nsis/` (installer)

## License

MIT
