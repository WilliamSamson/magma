# Obsidian

Obsidian is a GTK/VTE desktop terminal workspace written in Rust. It combines a focused terminal shell, a JSON log inspection pane, an embedded web viewer, and a first-run setup flow inside one desktop application.

## Current Scope

The project currently ships with these primary experiences:

- Embedded terminal workspace with tabs, split panes, and side tools
- Log workspace for viewing, filtering, and exporting newline-delimited JSON logs
- Embedded web viewer for in-app browsing beside the terminal
- First-run setup and autosaving settings for terminal/app behavior

## Features

- Custom desktop chrome with a dark, terminal-native visual system
- First-run setup flow with checkpoint restore
- Autosaving settings with separate terminal and app font sizes
- Embedded terminal tabs with:
  - tab rename
  - tab reorder
  - quick switcher
  - split panes
  - pane-local terminal multiplexer sessions
  - pane focus switching
- Terminal enhancements:
  - clipboard integration
  - command history
  - in-app command completion notices
  - desktop command notifications
  - terminal inspector
  - experimental sixel image rendering toggle
  - ligature shaping toggle
- Multiplexer controls:
  - pane-local session bar
  - new/close/switch terminal sessions inside a pane
  - keyboard session cycling
  - direct session jump with `Ctrl+Alt+1..9`
- Terminal selection copies directly on highlight
- Side panes:
  - `logr` log viewer
  - embedded web pane with selectable default browser
- Better workspace restore for tabs, split position, and active pane
- Global in-app version source
- File-based log viewing with live follow support
- Piped stdin support for shell-driven workflows
- Startup filters via repeated `--filter` arguments
- In-app search, level filtering, export, and help
- Graceful rendering of malformed JSON lines as visible error rows

See [docs/FEATURES.md](docs/FEATURES.md) for a fuller feature inventory.
See [docs/TERMINAL_WORKSPACE_GUIDE.md](docs/TERMINAL_WORKSPACE_GUIDE.md) for the full desktop terminal usage guide.

## Build

Requirements:

- Rust toolchain
- GTK4 development libraries
- VTE4 development libraries
- WebKitGTK 6.0 development libraries for the embedded web pane

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
- `docs/FEATURES.md`: current product feature list

## Fixtures

- `sample-logs.jsonl`: clean baseline fixture
- `sample-malformed.jsonl`: malformed-line fixture for error rendering

## Demo

A starter VHS script is included in `demo.tape`.
