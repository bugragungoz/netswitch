# netSwitch

**Windows Firewall Rule Manager**

Technical tool for managing Windows Firewall rules effectively. Built with Tauri, React, and Rust.

## Features

- **Application Blocking**: Granular control over application internet access (inbound/outbound).
- **Firewall Management**: Direct interface for managing Windows Firewall rules.
- **Network Monitoring**: Real-time process connection tracking (TCP/UDP) and interface statistics.
- **System Tools**: Quick access to essential Windows network utilities (`wf.msc`, `ncpa.cpl`, etc.).
- **Restore Points**: Integrated system restore point creation for safety.

## Requirements

- **OS**: Windows 10/11
- **Permissions**: Administrator privileges required for firewall operations.

## Installation

Download the latest release from [Releases](https://github.com/bugragungoz/netswitch/releases).
Portable (`.exe`) and Installer (`.msi`/`.exe`) versions available.

## Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Tech Stack

- **Frontend**: React 18, TypeScript, shadcn/ui
- **Backend**: Rust (Tauri 2)
- **System Integration**: WinAPI, PowerShell

## License

MIT License. See [LICENSE](LICENSE) for details.
