# Obsidian

Obsidian is a desktop terminal workspace written in Rust. It combines a custom black-window shell with a log inspection interface for structured JSON log streams.

## Current Scope

The project currently ships with two primary experiences:

- Embedded terminal mode when launched without an input source
- Log workspace mode for viewing, filtering, and exporting newline-delimited JSON logs

## Features

- Custom desktop window chrome with a focused black theme
- Embedded shell session with pill-based command input
- File-based log viewing with live follow support
- Piped stdin support for shell-driven workflows
- Startup filters via repeated `--filter` arguments
- In-app search, level filtering, export, and help
- Graceful rendering of malformed JSON lines as visible error rows

## Build

Requirements:

- Rust toolchain
- GTK4 development libraries
- VTE4 development libraries

Build the project:

```bash
cargo build
```

Build an optimized release binary:

```bash
cargo build --release
```

Install locally:

```bash
cargo install --path .
```

## Run

Launch the embedded terminal window:

```bash
cargo run --
```

Open the log workspace against a file:

```bash
cargo run -- sample-logs.jsonl
```

Pipe logs from another command:

```bash
kubectl logs mypod -f | obsidian
```

Start with filters already applied:

```bash
cargo run -- --filter level=error --filter query=request sample-logs.jsonl
```

Supported startup filter keys:

- `level=trace|debug|info|warn|error`
- `query=<text>`
- `search=<text>`

## Log Workspace Controls

- `Up/Down`, `j/k`, `PageUp/PageDown`, `Home/End`: navigate
- Mouse wheel: scroll
- `/`: enter search mode
- `Enter` or `Esc`: exit search mode
- `t`, `d`, `i`, `w`, `e`: toggle level filters
- `c`: clear query and level filters
- `x`: export the filtered view to `obsidian-export.jsonl`
- `?`: toggle help
- `q`: quit

## Repository Layout

- `src/app.rs`: ratatui application shell
- `src/features/logs/`: log ingestion, filters, and viewer logic
- `src/linux_terminal/`: GTK/VTE terminal window implementation
- `src/renderer/`: custom window renderer and pixel-based chrome
- `src/ui/`: shared layout and theme primitives

## Fixtures

- `sample-logs.jsonl`: clean baseline fixture
- `sample-malformed.jsonl`: malformed-line fixture for error rendering

## Demo

A starter VHS script is included in `demo.tape`.
