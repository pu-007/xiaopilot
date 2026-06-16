# xiaopilot

Rust rewrite of [cut_in_xiaoai](https://github.com/pu-007/cut_in_xiaoai). Remote-control a PC and wake a server via MQTT (Bafa Cloud).

## Architecture

| Binary | Target | Purpose |
|--------|--------|---------|
| `xiaopilot-wol` | Linux x86 (iStoreOS) | Wake-on-LAN: sends magic packet on MQTT trigger |
| `xiaopilot-win` | Windows x86_64 | Shutdown / reboot / sleep / display switch / keyboard shortcuts |

Both connect to [Bafa Cloud MQTT](https://bemfa.com) (`bemfa.com:9501`), subscribe to a topic, and execute actions on matching MQTT payloads.

## Prerequisites

- [Rust](https://rustup.rs) (edition 2021)
- Linux: none (WOL uses raw UDP socket)
- Windows: `displayswitch.exe` (built-in), `powershell` (built-in)

## Quick Start

### 1. Configuration

Copy the example files and edit them:

```bash
cp .env.example .env
cp wol.yml.example wol.yml    # for xiaopilot-wol
cp win.yml.example win.yml    # for xiaopilot-win
```

**`.env`** — Bafa Cloud credentials:

```env
BAFA_ID=your_bafa_client_id
BAFA_USER=userName
BAFA_PASS=passwd
```

`BAFA_USER` / `BAFA_PASS` are optional and default to Bafa's standard `userName` / `passwd`.

**`wol.yml`** — WOL configuration:

```yaml
topic: "PC002"               # MQTT topic to subscribe
trigger: "on"                # MQTT payload that triggers WOL
mac: "00:11:22:33:44:55"     # target MAC address
broadcast: "192.168.100.255" # subnet broadcast (default: 255.255.255.255)
```

**`win.yml`** — Windows command map:

```yaml
topic: "PC002"

commands:
  "off":
    action: shutdown

  "on#1":
    action: reboot

  "on#2":
    action: monitor
    mode: copy

  "on#3":
    action: monitor
    mode: switch
    default_width: 2560

  "on#4":
    action: monitor
    mode: extend

  "on#5":
    action: monitor
    mode: pc

  "on#6":
    action: monitor
    mode: tv

  "on#7":
    action: key
    keys: ["Space"]

  "on#8":
    action: key
    keys: ["Alt", "Shift", "Escape"]

  "on#9":
    action: key
    keys: ["Alt", "F"]

  "on#12":
    action: sleep
```

### Actions reference

| Action | Description |
|--------|-------------|
| `shutdown` | `shutdown -s -t 0` |
| `reboot` | `shutdown -r -t 0` |
| `sleep` | S3 sleep to RAM (`SetSuspendState` with `ForceCritical=0`) |
| `monitor` | Display mode switch via `displayswitch.exe` |
| `key` | Keyboard simulation via `enigo` |

#### Monitor modes

| Mode | displayswitch arg |
|------|-------------------|
| `pc` | 1 (PC screen only) |
| `copy` | 2 (duplicate) |
| `extend` | 3 (extend) |
| `tv` | 4 (second screen only) |
| `switch` | Toggle PC↔TV based on current resolution vs `default_width` |

#### Keyboard keys

Supports: `Alt`, `Ctrl`, `Shift`, `Meta`/`Win`, `Space`, `Escape`, `Enter`, `Tab`, `Backspace`, `Delete`, `Insert`, `Home`, `End`, `PageUp`, `PageDown`, arrow keys, `CapsLock`, `NumLock`, `PrintScreen`, `Pause`, `F1`–`F12`, single characters.

Hotkey pattern: the last key is clicked while all preceding keys are held as modifiers.

Examples:
- `["Space"]` — press Space
- `["Alt", "F4"]` — Alt+F4
- `["Ctrl", "Shift", "Escape"]` — Ctrl+Shift+Esc

### 2. Build

Use [just](https://github.com/casey/just) (installed automatically via `cargo install just` if missing):

```bash
just setup       # one-time: install rustup targets + cargo-zigbuild + zig
just             # cross-compile both → bin/xiaopilot-wol + bin/xiaopilot-win.exe
just build-win   # windows only
just build-linux # linux only
just clean       # remove build artifacts
```

Manual build without `just`:

```bash
# Linux WOL (cross-compile from any OS via zig)
cargo zigbuild --release --target x86_64-unknown-linux-musl -p xiaopilot-wol

# Windows (native)
cargo build --release -p xiaopilot-win
```

Prerequisites for cross-compilation:
- `zig` — download from [ziglang.org](https://ziglang.org/download/) or `scoop install zig` / `winget install zig.zig`
- `cargo-zigbuild` — `cargo install cargo-zigbuild`

### 3. Run

```bash
# Linux
./bin/xiaopilot-wol

# Windows
.\bin\xiaopilot-win.exe
```

Config files are read from the current working directory at startup. The program exits with an error if the required YAML file is missing.

## Project Structure

```
xiaopilot/
├── Cargo.toml              # workspace root
├── justfile                # build recipes
├── .env.example
├── wol.yml.example
├── win.yml.example
├── xiaopilot-wol/          # Linux WOL binary
│   ├── Cargo.toml
│   └── src/main.rs
└── xiaopilot-win/          # Windows binary
    ├── Cargo.toml
    └── src/main.rs
```

## License

MIT
