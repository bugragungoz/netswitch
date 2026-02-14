# netSwitch

<table border="0">
  <tr>
    <td width="150px">
      <img src="src-tauri/icons/logo-white.png" alt="netSwitch Logo" width="128" height="128">
    </td>
    <td>
      <h3>Windows Firewall Rule Manager</h3>
      <p>
        Technical tool for managing Windows Firewall rules effectively. Built with Tauri, React, and Rust.
      </p>
      <p>
        <em>Built with <strong>Opus 4.5</strong> & <strong>Gemini 3 Pro</strong></em>
      </p>
      <div>
        <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT"></a>
        <a href="https://tauri.app/"><img src="https://img.shields.io/badge/Tauri-2.0-blue.svg" alt="Tauri"></a>
        <a href="https://react.dev/"><img src="https://img.shields.io/badge/React-18-blue.svg" alt="React"></a>
      </div>
    </td>
  </tr>
</table>

---

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

- **Frontend**: React 18, TypeScript, [shadcn/ui](https://ui.shadcn.com/)
- **Backend**: Rust (Tauri 2)
- **System Integration**: WinAPI, PowerShell
- **AI Development**: Code developed with **Opus 4.5** & **Gemini 3 Pro**.

## License

MIT License. See [LICENSE](LICENSE) for details.
