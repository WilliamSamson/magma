# Magma Terminal Workspace Guide
This guide covers the GTK/VTE desktop terminal workspace: setup, tabs, panes, side tools, settings, restore behavior, and the main shortcuts.
## Launch
Run the desktop workspace:
```bash
cargo run --
```
Build a release binary:
```bash
cargo build --release
```
Startup behavior:
- first launch opens setup
- later launches open the workspace directly
- window size, tabs, split state, pane focus, and session snapshots are restored when available
## First-Run Setup
Setup checkpoints are saved as you move through the flow and restored if setup is interrupted.
Steps:
1. `runtime`: shell command, resolved shell, config path
2. `workspace`: default search engine, side-pane open-on-start
3. `appearance`: terminal font size, app font size, cursor style, cursor blink
Finishing setup saves settings and enters the workspace.
## Workspace Layout
The window has four main areas:
1. window header: settings button
2. terminal workspace: tabs, terminal panes, split panes, pane-local session bars
3. side-pane handle: buttons for `logr`, `web`, and `view`
4. active side pane: only one side pane is visible at a time
## Terminal Workspace
Each tab starts with one terminal pane. A tab can be split into left and right panes, and each pane can manage its own internal session stack.
### Tabs
Actions:
- create, close, rename, and reorder tabs
- jump directly to tabs by number
- open the quick switcher
- cycle the display profile on the current tab

Shortcuts:
- `Ctrl+T`: new tab
- `Ctrl+W`: close tab
- `Ctrl+Shift+R`: rename tab
- `Ctrl+Shift+Left` / `Ctrl+Shift+Right`: move tab
- `Ctrl+Tab` / `Ctrl+Shift+Tab`: next or previous tab
- `Ctrl+1..9`: jump to tab
- `Ctrl+K`: toggle quick switcher
### Split Panes
Actions:
- add or remove the right split pane
- move focus between left and right panes
- restore focus to the last active pane on reopen

Shortcuts:
- `Ctrl+Alt+Left`: focus left pane
- `Ctrl+Alt+Right`: focus right pane
### Pane-Local Sessions
Each pane has its own session bar above the terminal.
Actions:
- create a new session in the active pane
- switch to the next or previous session
- close the active session
- jump directly to a numbered session

Shortcuts:
- `Ctrl+Alt+N`: new session
- `Ctrl+Alt+W`: close active session
- `Ctrl+Alt+PageDown`: next session
- `Ctrl+Alt+PageUp`: previous session
- `Ctrl+Alt+1..9`: jump to session
## Terminal Input Row
Each session includes an input row below the terminal.
Features:
- shows the current working directory
- sends typed commands with `Enter`
- stores in-app command history
- provides terminal output search
- shows status and command notices
- exposes the terminal inspector

Controls:
- `Up` / `Down`: command history
- `Ctrl+F`: search terminal output
- `Escape`: leave search mode or move focus out of the terminal
- `Ctrl+Shift+C`: copy selected terminal text
- `Ctrl+Shift+V`: paste into terminal
- `Ctrl+Shift+A`: select all terminal text
- `Shift+Insert`: paste into terminal

Forwarded control keys from the entry:
- `Ctrl+C`: interrupt
- `Ctrl+Z`: suspend
- `Ctrl+D`: EOF
- `Ctrl+L`: clear terminal
- `Ctrl+\`: quit signal
Selection behavior:
- selecting terminal text copies it to the primary selection and clipboard automatically
## Side Panes
The handle opens one pane at a time:
- `logr`: structured log viewer
- `web`: embedded browser
- `view`: file viewer for the active terminal directory
If `panel open on start` is enabled, Magma starts with `logr` open.
### Logr
Features:
- scans nearby `log`, `jsonl`, `json`, and `txt` files
- opens files in tabs
- supports filtering, live follow, clear, export, and jump-to-bottom
- renders malformed JSON lines as visible error rows

Shortcuts:
- `Ctrl+L`: focus filter
- `/`: focus filter
- `Ctrl+K`: clear current logr tab
- `Ctrl+T`: new logr tab
- `Ctrl+W`: close logr tab
- `Escape`: leave filter
### Web
Features:
- opens full URLs directly
- treats plain text as a search query using the selected default engine
- supports home, back, forward, reload, stop, tabs, zoom reset, zoom out, and in-page find

Shortcuts:
- `Ctrl+T`: new web tab
- `Ctrl+W`: close web tab
- `Ctrl+F`: toggle find bar
- `Escape`: close find bar
Search engine choices:
- `google`
- `duckduckgo`
- `bing`
- `brave`
### View
The viewer scans the active terminal directory and lists supported files sorted by last modification time.
Preview groups:
- images: `png`, `jpg`, `jpeg`, `gif`, `webp`, `bmp`, `svg`
- documents: `pdf`, `docx`, `doc`, `ppt`, `pptx`
- code and text: common source/config files such as `rs`, `rt`, `dart`, `js`, `ts`, `tsx`, `jsx`, `py`, `sh`, `json`, `toml`, `yaml`, `html`, `css`, `sql`, `xml`, `go`, `java`, `kt`, `swift`, `c/cpp`, `php`, `lua`, `zig`, `ps1`, `bat`, `cmd`, plus files like `Dockerfile`, `Makefile`, `.gitignore`, and `.env.production`

Viewer actions:
- refresh the directory scan
- preview images, PDFs, and DOCX
- show info fallbacks for office files without embedded preview
- preview and edit code/text files
- reload and save code/text files
- open the selected file externally
Notes:
- code preview wraps for narrower panes
- save and reload only appear for code/text previews
- unsupported files are hidden from the list
## Settings
Open settings from the window header. Changes autosave immediately.
Sections:
- `terminal`: scrollback lines, image rendering, ligatures
- `appearance`: font family, terminal font size, app font size, cursor style, cursor blink
- `browser`: default search engine for the web pane
- `shell`: shell command used for new sessions
- `logr`: whether the side pane opens on startup
- `about`: version, config path, credits, engine details
## Persistence
Magma saves:
- window size
- app settings
- workspace tabs
- active tab
- split-pane state and split position
- active pane focus
- pane-local session snapshots
- setup checkpoints while setup is incomplete
## Current Limits
- only one side pane is visible at a time
- the viewer only shows supported file types
- office files other than `docx` currently use info fallback instead of full embedded rendering
- the web pane has zoom out and reset, but no dedicated zoom-in control
## Related Docs
- [README.md](../README.md): build, run, and non-terminal log viewer entry points
- [FEATURES.md](./FEATURES.md): compact feature inventory
- [WEBVIEW_INSTALL_PACK.md](./WEBVIEW_INSTALL_PACK.md): WebKitGTK dependency notes
- [LINUX_BUNDLE_PACK.md](./LINUX_BUNDLE_PACK.md): Linux packaging notes
