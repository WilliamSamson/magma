# Graph Report - .  (2026-05-27)

## Corpus Check
- 100 files · ~100,179 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1232 nodes · 2124 edges · 68 communities detected
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## God Nodes (most connected - your core abstractions)
1. `run_git()` - 35 edges
2. `TabView` - 24 edges
3. `LogsFeature` - 18 edges
4. `MuxPaneView` - 16 edges
5. `get_color()` - 16 edges
6. `LogEntry` - 15 edges
7. `refresh_view()` - 15 edges
8. `SidePanes` - 14 edges
9. `WorkspaceView` - 13 edges
10. `build_editor_toolbar()` - 13 edges

## Surprising Connections (you probably didn't know these)
- `workspace_file()` --calls--> `config_root()`  [EXTRACTED]
  src/linux_terminal/persist.rs → src/linux_terminal/web/persist.rs
- `build_notes_pane()` --calls--> `build_header()`  [EXTRACTED]
  src/linux_terminal/notes/mod.rs → src/linux_terminal/agent/mod.rs
- `build_header()` --calls--> `save_note()`  [EXTRACTED]
  src/linux_terminal/agent/mod.rs → src/linux_terminal/notes/mod.rs
- `build_header()` --calls--> `refresh_list()`  [EXTRACTED]
  src/linux_terminal/agent/mod.rs → src/linux_terminal/notes/mod.rs
- `build_header()` --calls--> `show_editor()`  [EXTRACTED]
  src/linux_terminal/agent/mod.rs → src/linux_terminal/notes/mod.rs

## Communities

### Community 0 - "Community 0"
Cohesion: 0.03
Nodes (134): append_input_row(), apply_body_text(), apply_search(), backlog_path(), bind_refresh(), bind_selection(), bind_setup_navigation(), build_agent_pane() (+126 more)

### Community 1 - "Community 1"
Cohesion: 0.06
Nodes (53): BlameLine, BranchInfo, collect_file_hunks(), CommitInfo, DiffHunk, DiffLine, DiffLineKind, FileChange (+45 more)

### Community 2 - "Community 2"
Cohesion: 0.06
Nodes (31): accent(), accent_muted(), bg_primary(), bg_secondary(), bg_sidebar(), bg_titlebar(), border(), border_strong() (+23 more)

### Community 3 - "Community 3"
Cohesion: 0.13
Nodes (34): bind_clear_export(), bind_file_picker(), bind_filter(), bind_keyboard(), bind_play_stop(), bind_refresh(), bind_scroll_tracking(), build_logr_pane() (+26 more)

### Community 4 - "Community 4"
Cohesion: 0.09
Nodes (14): AgentPaneHost, cache_root(), clear_browser_storage(), create_private_file(), data_root(), ensure_private_dir(), ensure_private_file(), GitPaneHost (+6 more)

### Community 5 - "Community 5"
Cohesion: 0.14
Nodes (29): bind_mark_button(), bind_patch_keys(), build_patch_view(), clear_container(), clear_container_box(), clear_detail(), clear_session_file(), data_root() (+21 more)

### Community 6 - "Community 6"
Cohesion: 0.08
Nodes (11): App, ShellContext, category_for_path(), FileCategory, FileKind, kind_for_path(), scan_directory(), viewer_file_for_path() (+3 more)

### Community 7 - "Community 7"
Cohesion: 0.12
Nodes (22): clear_snapshot(), config_root(), default_socket_path(), default_status_path(), deserialize_optional_pane(), generate_session_id(), load_snapshot(), load_workspace() (+14 more)

### Community 8 - "Community 8"
Cohesion: 0.13
Nodes (25): bundled_shell(), bundled_shell_candidates(), is_executable(), resolve_executable(), resolve_shell(), bash_args(), build_launch(), current_utf8_locale() (+17 more)

### Community 9 - "Community 9"
Cohesion: 0.12
Nodes (4): build_split_view(), display_title(), stored_base_title(), TabView

### Community 10 - "Community 10"
Cohesion: 0.11
Nodes (16): Bitmap, blend(), Canvas, Canvas<'a>, draw_bitmap(), brand_width(), button_layout(), ControlButton (+8 more)

### Community 11 - "Community 11"
Cohesion: 0.16
Nodes (11): append_session(), clear_children(), close_active_session(), current_session(), FocusBinding, MuxBarContext, MuxPaneView, MuxState (+3 more)

### Community 12 - "Community 12"
Cohesion: 0.13
Nodes (16): AgentRuntimeHandle, apply_action(), ExecutorConfig, log_action(), LogEntry, LogRole, now_ms(), process_command() (+8 more)

### Community 13 - "Community 13"
Cohesion: 0.11
Nodes (23): active_pane_context(), active_session(), ActivePaneContext, approx_tokens(), build_workspace_context(), capture_tmux_lines(), git_context(), GitContext (+15 more)

### Community 14 - "Community 14"
Cohesion: 0.13
Nodes (10): build_revealer(), build_side_panes(), handle_button(), PaneButtons, PaneRevealers, set_active_button(), SidePaneKind, SidePanes (+2 more)

### Community 15 - "Community 15"
Cohesion: 0.13
Nodes (8): contains_ignore_case(), extract_text(), is_reserved_key(), LogEntry, parse_level(), stringify_object(), stringify_value(), summarize_fields()

### Community 16 - "Community 16"
Cohesion: 0.21
Nodes (20): bind_close_button(), bind_context_menu(), bind_find_signals(), bind_new_window(), bind_tab_click(), bind_tab_signals(), build_content_manager(), close_tab() (+12 more)

### Community 17 - "Community 17"
Cohesion: 0.19
Nodes (19): build_file_row(), build_folder_pane(), build_folder_row(), build_header(), build_tree_level(), collect_git_status(), file_icon(), file_tooltip() (+11 more)

### Community 18 - "Community 18"
Cohesion: 0.22
Nodes (19): activate_tab(), attach_drag(), attach_rename(), clear_children(), clear_drop_indicators(), current_index(), find_target_index(), finish_tab_rename() (+11 more)

### Community 19 - "Community 19"
Cohesion: 0.19
Nodes (8): action_button(), actions_box(), append_tab(), create_new_tab(), current_index(), notebook(), tab_bar_row(), WorkspaceView

### Community 20 - "Community 20"
Cohesion: 0.21
Nodes (18): apply_highlighting(), apply_tag(), bind_editor_actions(), build_editor(), char_count(), current_code_file(), EditorWidgets, find_comment_start() (+10 more)

### Community 21 - "Community 21"
Cohesion: 0.21
Nodes (1): LogsFeature

### Community 22 - "Community 22"
Cohesion: 0.17
Nodes (15): checkpoint_path(), clear(), load(), save(), SetupCheckpoint, build_inspector_panel(), display_font(), display_pty() (+7 more)

### Community 23 - "Community 23"
Cohesion: 0.2
Nodes (9): bind_keyboard(), bind_remote_op(), build_git_pane(), format_ahead_behind(), GitPaneView, refresh(), set_no_repo(), SubView (+1 more)

### Community 24 - "Community 24"
Cohesion: 0.16
Nodes (11): AgentModel, ensure_secure_permissions(), load_api_key(), load_model(), ModelRequest, NoopModel, OpenRouterModel, parse_actions() (+3 more)

### Community 25 - "Community 25"
Cohesion: 0.22
Nodes (14): build_appearance_step(), build_runtime_step(), build_topbar(), build_workspace_step(), display_row(), dot(), dropdown_field(), entry_field() (+6 more)

### Community 26 - "Community 26"
Cohesion: 0.25
Nodes (5): clear_rows(), current_index(), empty_row(), QuickSwitcher, switcher_row()

### Community 27 - "Community 27"
Cohesion: 0.18
Nodes (10): browser_profile(), BrowserProfile, home_info(), home_uri(), is_known_home_uri(), looks_like_host(), normalize_browser_id(), resolve_destination() (+2 more)

### Community 28 - "Community 28"
Cohesion: 0.26
Nodes (13): build_about_section(), build_about_view(), build_browser_view(), build_empty_search_panel(), build_main_page(), build_section_content(), build_section_nav(), build_section_stack() (+5 more)

### Community 29 - "Community 29"
Cohesion: 0.25
Nodes (13): build_icon(), cursor_icon_for_resize(), first_printable_char(), handle_tick(), handle_window_event(), header_logo_bytes(), main(), persist_window_size() (+5 more)

### Community 30 - "Community 30"
Cohesion: 0.19
Nodes (2): FilterState, InputMode

### Community 31 - "Community 31"
Cohesion: 0.2
Nodes (4): copy_terminal_selection(), format_path_display(), SessionView, wire_terminal_clipboard()

### Community 32 - "Community 32"
Cohesion: 0.18
Nodes (5): CodeLanguage, extension(), language_for_extension(), language_for_path(), supports_code_preview()

### Community 33 - "Community 33"
Cohesion: 0.19
Nodes (5): LogLevel, install_css(), resolve_alpha(), try_resolve_alpha_inner(), ui_scale()

### Community 34 - "Community 34"
Cohesion: 0.22
Nodes (8): ProfileId, TerminalProfile, apply_terminal_settings(), apply_terminal_theme(), build_terminal(), color_rgba(), rgba(), terminal_font_description()

### Community 35 - "Community 35"
Cohesion: 0.24
Nodes (8): action_row(), dropdown_row(), info_row(), setting_row(), spin_row(), switch_row(), text_row(), value_label()

### Community 36 - "Community 36"
Cohesion: 0.28
Nodes (11): apply_search(), bind_settings_search(), build_search_result(), build_search_view(), clear_list(), collect_text(), contains_ignore_case(), perform_search() (+3 more)

### Community 37 - "Community 37"
Cohesion: 0.18
Nodes (3): AppHeader, apply_window_button_tooltips(), build_header()

### Community 38 - "Community 38"
Cohesion: 0.32
Nodes (11): bind_add_tab(), bind_address(), bind_find_bar(), bind_home(), bind_keyboard(), bind_navigation(), bind_reload_stop(), bind_zoom() (+3 more)

### Community 39 - "Community 39"
Cohesion: 0.24
Nodes (8): find_empty_label(), PreviewWidgets, select_file(), set_empty_preview(), show_docx_preview(), show_error_preview(), show_info_preview(), update_preview_header()

### Community 40 - "Community 40"
Cohesion: 0.21
Nodes (4): Dock, DockOption, draw_pill_background(), Rect

### Community 41 - "Community 41"
Cohesion: 0.35
Nodes (10): build_status_widgets(), compact_command(), parse_status_triplet(), read_status_event(), show_desktop_notice(), show_notice(), StatusEvent, StatusWidgets (+2 more)

### Community 42 - "Community 42"
Cohesion: 0.27
Nodes (9): ctrl_key_byte(), handle_clipboard_shortcuts(), handle_history_navigation(), handle_terminal_clipboard_shortcuts(), InputHistory, InputHistoryState, paste_clipboard_into_entry(), show_next_history() (+1 more)

### Community 43 - "Community 43"
Cohesion: 0.31
Nodes (1): LevelFilters

### Community 44 - "Community 44"
Cohesion: 0.33
Nodes (5): build_left_pane(), build_revealer(), handle_button(), LeftPane, wrap_pane()

### Community 45 - "Community 45"
Cohesion: 0.42
Nodes (8): candidate_log_paths(), debug(), error(), info(), init(), open_log_file(), warn(), write_entry()

### Community 46 - "Community 46"
Cohesion: 0.36
Nodes (8): build_conflict_row(), build_file_row(), build_file_section(), build_staging_view(), build_untracked_row(), clear_list(), refresh_staging(), StagingWidgets

### Community 47 - "Community 47"
Cohesion: 0.22
Nodes (4): AgentAction, HunkRef, LogFilter, PaneType

### Community 48 - "Community 48"
Cohesion: 0.39
Nodes (8): execute_side_effect(), logr_filter_request_path(), select_hunk_patch(), stage_hunk(), take_logr_filter_request(), UiEffect, write_annotation(), write_logr_filter_request()

### Community 49 - "Community 49"
Cohesion: 0.46
Nodes (7): build_prompt_box(), compact_path(), connect_directory_updates(), current_path_display(), current_username(), fallback_path(), path_from_uri()

### Community 50 - "Community 50"
Cohesion: 0.48
Nodes (6): build_html(), decode_xml_entities(), extract_document_xml(), paragraph_text(), parse_paragraphs(), render_docx_html()

### Community 51 - "Community 51"
Cohesion: 0.43
Nodes (5): BranchWidgets, build_branch_row(), build_remote_branch_row(), clear_list(), refresh_branches()

### Community 52 - "Community 52"
Cohesion: 0.43
Nodes (5): load_recent_summaries(), now_ms(), repo_memory_dir(), SessionMemory, store_summary()

### Community 53 - "Community 53"
Cohesion: 0.33
Nodes (3): FontAtlas, FontMetrics, GlyphBitmap

### Community 54 - "Community 54"
Cohesion: 0.6
Nodes (5): follow_file(), read_next_line(), send_entry(), send_follow_error(), spawn_file_follower()

### Community 55 - "Community 55"
Cohesion: 0.53
Nodes (5): FollowConfig, load_file_source(), load_source(), LoadedSource, read_entries()

### Community 56 - "Community 56"
Cohesion: 0.47
Nodes (5): invalid_filter(), parse_args(), parse_filters(), ParsedArgs, StartupFilter

### Community 57 - "Community 57"
Cohesion: 0.53
Nodes (3): LinkMatcher, path_to_uri(), wire_open_actions()

### Community 58 - "Community 58"
Cohesion: 0.6
Nodes (5): about_label(), build_about_page(), centered_copy(), linked_label(), section_header()

### Community 59 - "Community 59"
Cohesion: 0.53
Nodes (5): build_commit_row(), build_graph_view(), clear_list(), GraphState, refresh_graph()

### Community 60 - "Community 60"
Cohesion: 0.47
Nodes (4): build_stash_row(), clear_list(), refresh_stash(), StashWidgets

### Community 61 - "Community 61"
Cohesion: 0.4
Nodes (4): ObserverConfig, repeated_error(), spawn(), WorkspaceEvent

### Community 62 - "Community 62"
Cohesion: 0.8
Nodes (4): bind_cursor_style(), build_appearance_section(), build_terminal_section(), preview_settings()

### Community 63 - "Community 63"
Cohesion: 0.5
Nodes (3): SourceState, UiState, ViewState

### Community 64 - "Community 64"
Cohesion: 0.83
Nodes (3): bind_header_state(), build_settings_page(), sync_header_state()

### Community 65 - "Community 65"
Cohesion: 1.0
Nodes (0): 

### Community 66 - "Community 66"
Cohesion: 1.0
Nodes (0): 

### Community 67 - "Community 67"
Cohesion: 1.0
Nodes (0): 

## Knowledge Gaps
- **113 isolated node(s):** `StoredWindowState`, `WindowChrome`, `UiState`, `ViewState`, `SourceState` (+108 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 65`** (2 nodes): `file_ops.rs`, `remove_line_at()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 66`** (2 nodes): `export.rs`, `write_filtered()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 67`** (1 nodes): `meta.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What connects `StoredWindowState`, `WindowChrome`, `UiState` to the rest of the system?**
  _113 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.03 - nodes in this community are weakly interconnected._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.06 - nodes in this community are weakly interconnected._
- **Should `Community 2` be split into smaller, more focused modules?**
  _Cohesion score 0.06 - nodes in this community are weakly interconnected._
- **Should `Community 3` be split into smaller, more focused modules?**
  _Cohesion score 0.13 - nodes in this community are weakly interconnected._
- **Should `Community 4` be split into smaller, more focused modules?**
  _Cohesion score 0.09 - nodes in this community are weakly interconnected._
- **Should `Community 5` be split into smaller, more focused modules?**
  _Cohesion score 0.14 - nodes in this community are weakly interconnected._