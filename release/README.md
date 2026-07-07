# Rutken Static Analyzer

Rutken is a Rust-based APK analyzer focused on fast container inspection, DEX decoding, and command-line workflows.

## Commands

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
