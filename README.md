# AVTIME

**NTP-synchronized precision clock display for aviation timepiece synchronization**

AVTIME is a terminal-based application that provides millisecond-accurate time display synchronized with Network Time Protocol (NTP) servers. Designed for aviators who need to accurately synchronize non-smart timepieces (mechanical watches, analog cockpit clocks, etc.) with precise time sources.

## Why AVTIME?

In aviation, accurate time synchronization is essential for:
- Flight planning and fuel calculations
- Navigation and position reporting
- Coordination with Air Traffic Control (ATC)
- Recording flight times for logbooks
- ETAs and time-critical operations

While modern avionics have built-in time synchronization, many pilots still use traditional analog watches and cockpit timers that require manual setting. AVTIME provides a quick, convenient reference for setting these timepieces with millisecond accuracy.

## Features

- **Millisecond precision**: Displays time in `HH:MM:SS.mmm` format
- **NTP synchronization**: Syncs with authoritative time servers (Google Time, pool.ntp.org, etc.)
- **Real-time offset display**: Shows your system clock's offset from true time
- **Visual sync status**: Color-coded indicators show sync freshness
  - 🟢 Green: Fresh sync (0-5s)
  - 🟡 Yellow: Recent sync (6-15s)
  - 🟣 Magenta: Aging sync (16-30s)
  - 🔴 Red: Stale sync (30+s)
- **Offset accuracy indicators**:
  - 🟢 Green: 0-10ms (excellent)
  - 🟡 Yellow: 11-50ms (good)
  - 🟣 Magenta: 51-100ms (fair)
  - 🔴 Red: 100+ms (poor)
- **Smooth updates**: Display refreshes every 10ms for smooth millisecond counting
- **Clean TUI**: Built with ratatui for a polished terminal interface

## Installation

### Prerequisites

- Rust toolchain (1.70 or later)

### Build from source

```bash
git clone https://github.com/yourusername/avtime.git
cd avtime
cargo build --release
```

The compiled binary will be in `target/release/avtime`

### Install globally

```bash
cargo install --git https://github.com/brendanmckenzie/avtime
```

## Usage

### Basic usage

```bash
avtime time.google.com
```

### Recommended NTP servers

**Google Public NTP:**
```bash
avtime time.google.com
```

**Pool.NTP.org (global pool):**
```bash
avtime pool.ntp.org
```

**Cloudflare NTP:**
```bash
avtime time.cloudflare.com
```

**Regional pools:**
```bash
avtime au.pool.ntp.org  # Australia
avtime us.pool.ntp.org  # United States
avtime europe.pool.ntp.org  # Europe
```

### Controls

- Press `q` or `Q` to quit
- Press `Ctrl+C` to quit

## Interface

The AVTIME display shows:

```
┌─ AVTIME ─────────────────────────────────┐
│           2026-01-09                     │
│  ┌──────────────────────────────────┐    │
│  │      12:34:56.789                │    │
│  └──────────────────────────────────┘    │
│                                          │
│  ┌─ Server ─────────────────────────┐    │
│  │ NTP Server: time.google.com      │    │
│  └──────────────────────────────────┘    │
│  ┌─ Offset ─────────────────────────┐    │
│  │ Clock Offset: +2 ms              │    │
│  └──────────────────────────────────┘    │
│  ┌─ Sync Status ────────────────────┐    │
│  │ ● Last sync: 3s ago              │    │
│  │ ████████████░░░░░░░░░░░░░         │    │
│  └──────────────────────────────────┘    │
│                                          │
│    Press 'q' or Ctrl+C to quit           │
└──────────────────────────────────────────┘
```

## Aviation Time Synchronization Tips

1. **Pre-flight setup**: Sync your timepieces during preflight planning when you have internet access
2. **Dual reference**: Keep both UTC and local time readily available
3. **Regular checks**: Re-sync if you notice drift over multiple flights
4. **Backup method**: Note the offset value in case you need to manually adjust later
5. **Cockpit clocks**: Use the millisecond display to set analog clocks precisely on the second

## Technical Details

- Built with Rust for performance and reliability
- Uses the `rsntp` crate for NTP client functionality
- Terminal UI powered by `ratatui`
- Asynchronous updates with `tokio`
- Auto re-syncs every 10 seconds to maintain accuracy
- Display updates at 100Hz for smooth millisecond counting

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues.

## Acknowledgments

Built for pilots who appreciate the blend of modern precision and traditional timepieces.

---

**Fly safe, stay on time.** ✈️
