# Rutken Static Analyzer

Rutken is a Rust-based APK analyzer focused on fast container inspection, DEX decoding, and command-line workflows.

## Commands

Usage: `rutken <APK> <COMMAND> [ARGS]`

| Command | Description |
| --- | --- |
| `info` | Summary: SHA256, size, DEX file count, class count, native libs, package/SDK info |
| `manifest` | Render the decoded `AndroidManifest.xml` |
| `strings [--grep <pattern>]` | List decoded DEX strings, optionally filtered by a substring |
| `classes` | List all classes across DEX files |
| `search <query>` | Case-insensitive substring search over strings, class names, method names, and disassembled instructions |
| `disasm <query>` | Disassemble methods for classes whose name contains `<query>` |
| `dump [--json] [--raw] [--include strings]` | Full/partial report of the analyzed APK |
| `stats` | Hidden dev command: classes/methods/instructions/fields/XREF/CFG counts plus analysis time and peak RAM |

```bash
rutken app.apk info
rutken app.apk manifest
rutken app.apk strings
rutken app.apk strings --grep api
rutken app.apk classes
rutken app.apk search MainActivity
rutken app.apk disasm MainActivity
rutken app.apk dump --json
rutken app.apk dump --json --include strings
rutken app.apk dump --json --raw
rutken app.apk stats
```

## Info Output

The `info` command summarizes:

- SHA256
- file size
- DEX file count
- total classes
- package name
- minimum SDK
- target SDK
- detected native architectures

## JSON Modes

- `--json` produces a compact report for normal workflows.
- `--json --include strings` adds the decoded string table.
- `--json --raw` emits the full internal DEX model.

## Build

```bash
cargo build
cargo build --release
```

The release binary is produced at `target/release/cli`.

## Tests

```bash
cargo test
```
