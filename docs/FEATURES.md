# Magma Feature List

This document tracks the current user-facing feature set in the desktop application.

## Core Workspace

- Custom desktop window chrome with a dark terminal-focused style
- First-run setup flow with checkpoint restore
- Autosaving settings
- Dedicated About page
- Shared in-app version source

## Terminal Workspace

- Multi-tab terminal workspace
- Tab rename
- Tab reorder
- Quick tab switcher
- Split pane support inside a tab
- Pane-local terminal multiplexer sessions
- Pane focus switching
- Session create / close / switch inside a pane
- Direct session jump with `Ctrl+Alt+1..9`
- Terminal/app font size separation
- Cursor style and cursor blink settings
- Scrollback settings
- Unicode / UTF-8 shell environment fallback
- Clipboard integration
- Direct copy on terminal text highlight
- Command history in the input row
- In-app command completion notices
- Desktop command notifications
- Terminal inspector
- Experimental sixel image rendering toggle
- Ligature shaping toggle

## Side Panes

- `logr` pane for structured JSON log viewing
- Embedded web pane
- Default browser selection for web search/home behavior
- Lazy web view creation with shared browser context
- Width-clamped web pane behavior
- Compact segmented side-pane handle

## Logr

- File open and live follow
- Search and level filtering
- Export
- Graceful malformed JSON rendering

## Restore and Runtime

- Workspace restore for tabs
- Restore split position
- Restore active split pane
- Runtime shell resolution:
  - configured shell
  - bundled shell
  - `$SHELL`
  - `/bin/bash`
  - `/bin/sh`

## Packaging

- Linux bundle packaging scripts
- AppImage build script
- Install dependency script
