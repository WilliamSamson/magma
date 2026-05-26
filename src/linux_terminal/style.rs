use gtk::{
    gdk, style_context_add_provider_for_display, CssProvider, STYLE_PROVIDER_PRIORITY_USER,
};

use super::{settings::Settings, theme};

pub(super) fn install_css(settings: &Settings) {
    let provider = CssProvider::new();
    let ui_scale = ui_scale(settings.app_font_size);
    let palette = theme::palette(settings.theme_mode);
    let css = format!(
        "
        @keyframes magma-fade-in {{
            from {{ opacity: 0; }}
            to {{ opacity: 1; }}
        }}

        window.magma-window {{
            background: {window_bg};
            border: 1px solid {window_edge};
            border-radius: 12px;
            animation: magma-fade-in 400ms ease-out;
        }}

        /* ── Global scrollbar — thin, themed ── */
        scrollbar {{
            background: transparent;
            border: none;
        }}

        scrollbar slider {{
            background: alpha({text_primary}, 0.10);
            border-radius: 999px;
            min-width: 6px;
            min-height: 6px;
            border: none;
            transition: all 250ms cubic-bezier(0.16, 1, 0.3, 1);
        }}

        scrollbar slider:hover {{
            background: alpha({text_primary}, 0.20);
            transform: scale(1.2);
        }}

        scrollbar.horizontal slider {{
            min-height: 6px;
        }}

        scrollbar.vertical slider {{
            min-width: 6px;
        }}

        headerbar.magma-header {{
            background: {titlebar_bg};
            border-bottom: 1px solid {border};
            min-height: 40px;
            padding: 4px 12px;
        }}

        headerbar.magma-header box {{
            background: transparent;
        }}

        headerbar.magma-header windowcontrols button {{
            background: {surface};
            border-radius: 50%;
            box-shadow: none;
            border: none;
            color: transparent;
            min-height: 14px;
            min-width: 14px;
            padding: 0;
            margin: 0 4px;
            transition: all 250ms cubic-bezier(0.16, 1, 0.3, 1);
            -gtk-icon-size: 0px;
        }}

        headerbar.magma-header windowcontrols button:hover {{
            transform: scale(1.2);
        }}

        headerbar.magma-header windowcontrols button image {{
            opacity: 0;
            -gtk-icon-size: 8px;
            transition: opacity 120ms ease;
        }}

        headerbar.magma-header windowcontrols button:hover image {{
            opacity: 1;
            color: rgba(0, 0, 0, 0.6);
        }}

        headerbar.magma-header windowcontrols button.close {{
            background: {sem_orange};
        }}

        headerbar.magma-header windowcontrols button.minimize {{
            background: {sem_yellow};
        }}

        headerbar.magma-header windowcontrols button.maximize {{
            background: {sem_green};
        }}

        headerbar.magma-header windowcontrols button.close:hover {{
            background: {sem_orange};
        }}

        headerbar.magma-header windowcontrols button.minimize:hover {{
            background: {sem_yellow};
        }}

        headerbar.magma-header windowcontrols button.maximize:hover {{
            background: {sem_green};
        }}

        headerbar.magma-header button.titlebutton {{
            background: {surface};
            border-radius: 50%;
            box-shadow: none;
            border: none;
            color: transparent;
            min-height: 14px;
            min-width: 14px;
            padding: 0;
            margin: 0 4px;
            transition: background 140ms ease;
            -gtk-icon-size: 0px;
        }}

        headerbar.magma-header button.titlebutton image {{
            opacity: 0;
            -gtk-icon-size: 8px;
            transition: opacity 120ms ease;
        }}

        headerbar.magma-header button.titlebutton:hover image {{
            opacity: 1;
            color: rgba(0, 0, 0, 0.6);
        }}

        headerbar.magma-header button.titlebutton.close {{
            background: {sem_orange};
        }}

        headerbar.magma-header button.titlebutton.minimize {{
            background: {sem_yellow};
        }}

        headerbar.magma-header button.titlebutton.maximize {{
            background: {sem_green};
        }}

        headerbar.magma-header button.titlebutton.close:hover {{
            background: {sem_orange};
        }}

        headerbar.magma-header button.titlebutton.minimize:hover {{
            background: {sem_yellow};
        }}

        headerbar.magma-header button.titlebutton.maximize:hover {{
            background: {sem_green};
        }}

        button.magma-header-settings,
        menubutton.magma-header-settings {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 50%;
            min-height: 24px;
            min-width: 24px;
            padding: 4px;
            box-shadow: none;
            opacity: 0.4;
            transition: all 250ms cubic-bezier(0.16, 1, 0.3, 1);
        }}

        menubutton.magma-header-settings > button {{
            background: transparent;
            border: none;
            box-shadow: none;
        }}

        button.magma-header-settings:hover,
        menubutton.magma-header-settings:hover {{
            opacity: 1.0;
            transform: scale(1.1) rotate(15deg);
        }}

        button.magma-header-close {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 4px;
            min-height: 18px;
            min-width: 18px;
            padding: 2px;
            box-shadow: none;
            opacity: 0.42;
            transition: all 250ms cubic-bezier(0.16, 1, 0.3, 1);
            -gtk-icon-size: 12px;
        }}

        button.magma-header-close:hover {{
            opacity: 1.0;
            transform: scale(1.1);
        }}

        .magma-logo {{
            opacity: 0.92;
        }}

        .magma-shell {{
            background: {window_bg};
            border-bottom-left-radius: 12px;
            border-bottom-right-radius: 12px;
        }}

        .magma-title {{
            color: {text_primary};
            font-weight: 700;
            letter-spacing: 0.04em;
        }}

        terminal.magma-terminal {{
            background: transparent;
            color: {text_primary};
            border: 1px solid alpha({text_primary}, 0.06);
            border-radius: {terminal_radius};
            padding: {terminal_padding};
        }}

        box.magma-workspace-actions {{
            background: transparent;
            border: none;
            margin: 0;
            padding: 0;
        }}

        separator.magma-separator {{
            background: {border};
            min-height: 1px;
            margin: 0 0 12px 0;
        }}

        separator.magma-v-separator {{
            background: {border};
            min-width: 1px;
            margin: 0 4px;
        }}

        paned.magma-split-pane > separator {{
            background: alpha({text_primary}, 0.04);
            min-width: {split_width};
            margin: 0 {split_margin};
            border-radius: 999px;
            transition: all 250ms cubic-bezier(0.16, 1, 0.3, 1);
        }}

        paned.magma-split-pane > separator:hover {{
            background: alpha({accent}, 0.18);
            transform: scaleX(1.4);
        }}

        box.magma-mux-root {{
            background: transparent;
        }}

        box.magma-mux-bar {{
            background: alpha({text_primary}, 0.015);
            border: 1px solid alpha({text_primary}, 0.06);
            border-bottom: 1px solid alpha({text_primary}, 0.04);
            border-radius: {bar_radius} {bar_radius} 0 0;
            padding: {bar_padding} calc({bar_padding} * 2);
            margin: 0;
        }}

        box.magma-mux-root terminal.magma-terminal {{
            border-radius: 0 0 {terminal_radius} {terminal_radius};
            border: 1px solid alpha({text_primary}, 0.06);
            border-top: none;
            background: alpha({text_primary}, 0.003);
            padding: {terminal_padding};
        }}

        button.magma-mux-session,
        button.magma-mux-action {{
            background: transparent;
            color: alpha({text_primary}, 0.6);
            border: 1px solid transparent;
            border-radius: 6px;
            min-height: {mux_btn_size};
            min-width: {mux_btn_size};
            padding: 2px {mux_btn_padding};
            box-shadow: none;
            opacity: 0.8;
            transition: all 200ms cubic-bezier(0.16, 1, 0.3, 1);
            font-family: monospace;
            font-weight: 700;
        }}

        button.magma-mux-session:hover,
        button.magma-mux-action:hover {{
            opacity: 1.0;
            background: alpha({text_primary}, 0.04);
            color: {text_primary};
        }}

        button.magma-mux-session.active {{
            opacity: 1.0;
            background: {window_bg};
            color: {accent};
            border: 1px solid alpha({text_primary}, 0.06);
            box-shadow: 0 2px 6px alpha(#000000, 0.15);
        }}

        button.magma-mux-action.close-session:hover {{
            color: {sem_orange};
            background: alpha({sem_orange}, 0.1);
        }}

        .magma-right-pane {{
            background: transparent;
            padding: 14px 14px 12px 14px;
            border: 1px solid {border};
            border-radius: 14px;
            margin: 0;
        }}

        .magma-handle {{
            min-width: 30px;
            margin: 0 3px;
            padding: 3px;
            border: 1px solid alpha({text_primary}, 0.04);
            border-radius: 12px;
            background: alpha({text_primary}, 0.015);
            transition: all 250ms cubic-bezier(0.16, 1, 0.3, 1);
        }}

        .magma-handle:hover {{
            background: alpha({text_primary}, 0.025);
            border-color: alpha({text_primary}, 0.08);
            transform: scaleX(1.05);
        }}

        .magma-handle.collapsed {{
            background: alpha({text_primary}, 0.01);
        }}

        .magma-handle.collapsed:hover {{
            background: alpha({accent}, 0.05);
            border-color: alpha({accent}, 0.18);
        }}

        button.magma-handle-segment {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 9px;
            min-width: 24px;
            min-height: 32px;
            padding: 0;
            box-shadow: none;
            opacity: 0.34;
            transition: all 250ms cubic-bezier(0.16, 1, 0.3, 1);
        }}

        button.magma-handle-segment:hover {{
            opacity: 0.86;
            background: alpha({text_primary}, 0.035);
            transform: scale(1.05);
        }}

        button.magma-handle-segment.active {{
            opacity: 1.0;
            background: alpha({accent}, 0.08);
        }}

        .magma-handle-icon {{
            color: {text_primary};
            -gtk-icon-size: 14px;
            opacity: inherit;
            transition: opacity 180ms ease, color 180ms ease;
        }}

        .magma-handle.collapsed button.magma-handle-segment {{
            opacity: 0.28;
        }}

        .magma-handle.collapsed:hover button.magma-handle-segment {{
            opacity: 0.72;
            background: alpha({accent}, 0.035);
        }}

        .magma-handle.collapsed:hover .magma-handle-icon {{
            color: {accent};
        }}

        button.magma-handle-segment.active .magma-handle-icon {{
            color: {accent};
        }}

        /* ── Left handle (folder pane toggle) ── */

        .magma-left-handle {{
            min-width: 30px;
            margin: 0 3px;
            padding: 3px;
            border: 1px solid alpha({text_primary}, 0.04);
            border-radius: 12px;
            background: alpha({text_primary}, 0.015);
            transition: border-color 180ms ease, background 180ms ease, opacity 180ms ease;
        }}

        .magma-left-handle:hover {{
            background: alpha({text_primary}, 0.025);
            border-color: alpha({text_primary}, 0.08);
        }}

        .magma-left-handle.collapsed {{
            background: alpha({text_primary}, 0.01);
        }}

        .magma-left-handle.collapsed:hover {{
            background: alpha({accent}, 0.05);
            border-color: alpha({accent}, 0.18);
        }}

        button.magma-left-handle-segment {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 9px;
            min-width: 24px;
            min-height: 32px;
            padding: 0;
            box-shadow: none;
            opacity: 0.34;
            transition: opacity 180ms ease, background 180ms ease, color 180ms ease;
        }}

        button.magma-left-handle-segment:hover {{
            opacity: 0.86;
            background: alpha({text_primary}, 0.035);
        }}

        button.magma-left-handle-segment.active {{
            opacity: 1.0;
            background: alpha({accent}, 0.08);
        }}

        .magma-left-handle-icon {{
            color: {text_primary};
            -gtk-icon-size: 14px;
            opacity: inherit;
            transition: opacity 180ms ease, color 180ms ease;
        }}

        .magma-left-handle.collapsed button.magma-left-handle-segment {{
            opacity: 0.28;
        }}

        .magma-left-handle.collapsed:hover button.magma-left-handle-segment {{
            opacity: 0.72;
            background: alpha({accent}, 0.035);
        }}

        .magma-left-handle.collapsed:hover .magma-left-handle-icon {{
            color: {accent};
        }}

        button.magma-left-handle-segment.active .magma-left-handle-icon {{
            color: {accent};
        }}

        /* ── Left pane (folder tree) ── */

        .magma-left-pane {{
            background: transparent;
            padding: 8px 14px 8px 4px;
            border: none;
            border-radius: 0;
            margin: 0;
        }}

        .magma-folder-header {{
            padding: 4px 4px 8px 4px;
            border-bottom: 1px solid {border};
        }}

        .magma-folder-title {{
            color: {text_secondary};
            font-size: {folder_title_size};
            font-weight: 600;
            letter-spacing: 0.06em;
            text-transform: uppercase;
        }}

        button.magma-folder-action {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 6px;
            min-width: 22px;
            min-height: 22px;
            padding: 2px;
            box-shadow: none;
            opacity: 0.4;
            transition: all 200ms cubic-bezier(0.16, 1, 0.3, 1);
        }}

        button.magma-folder-action:hover {{
            opacity: 1.0;
            background: alpha({text_primary}, 0.04);
            transform: scale(1.1);
        }}

        .magma-folder-action-icon {{
            -gtk-icon-size: 12px;
        }}

        .magma-folder-tree {{
            padding: 4px 0;
        }}

        button.magma-folder-item {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 6px;
            min-height: 26px;
            padding: 2px 8px;
            box-shadow: none;
            transition: all 200ms cubic-bezier(0.16, 1, 0.3, 1);
        }}

        button.magma-folder-item:hover {{
            background: alpha({text_primary}, 0.04);
            transform: translateX(4px);
        }}

        button.magma-folder-dir {{
            font-weight: 500;
        }}

        button.magma-folder-file {{
            opacity: 0.78;
        }}

        .magma-folder-chevron {{
            -gtk-icon-size: 10px;
            opacity: 0.5;
        }}

        .magma-folder-icon {{
            -gtk-icon-size: 14px;
            opacity: 0.7;
        }}

        .magma-folder-name {{
            font-size: {folder_name_size};
        }}

        .magma-folder-empty {{
            color: {text_dim};
            font-size: {folder_name_size};
            padding: 16px 8px;
        }}

        /* ── File-type colors (muted palette) ── */

        .ft-default  {{ color: {text_secondary}; opacity: 0.78; }}
        .ft-folder   {{ color: {text_primary}; }}
        .ft-source   {{ color: {sem_yellow}; }}
        .ft-test     {{ color: {sem_green}; }}
        .ft-build    {{ color: {sem_brown}; }}
        .ft-dep      {{ color: {text_dim}; }}
        .ft-doc      {{ color: {sem_blue}; }}
        .ft-asset    {{ color: {sem_magenta}; }}
        .ft-git      {{ color: {sem_orange}; }}
        .ft-config   {{ color: {sem_brown}; }}
        .ft-secret   {{ color: {sem_orange}; opacity: 0.85; }}
        .ft-lock     {{ color: {text_dim}; }}

        .ft-rust     {{ color: {sem_orange}; }}
        .ft-js       {{ color: {sem_yellow}; }}
        .ft-ts       {{ color: {sem_blue}; }}
        .ft-html     {{ color: {sem_orange}; }}
        .ft-css      {{ color: {sem_blue}; }}
        .ft-json     {{ color: {sem_brown}; }}
        .ft-python   {{ color: {sem_green}; }}
        .ft-go       {{ color: {sem_blue}; }}
        .ft-c        {{ color: {sem_blue}; }}
        .ft-java     {{ color: {sem_orange}; }}
        .ft-shell    {{ color: {sem_green}; }}
        .ft-image    {{ color: {sem_magenta}; }}
        .ft-archive  {{ color: {sem_brown}; }}
        .ft-binary   {{ color: {text_dim}; }}

        /* ── Git status dots ── */

        .magma-git-dot {{
            font-size: 8px;
            margin: 0 2px 0 4px;
            opacity: 0.9;
        }}

        .git-modified  {{ color: {sem_yellow}; }}
        .git-staged    {{ color: {sem_green}; }}
        .git-untracked {{ color: {text_dim}; }}
        .git-conflict  {{ color: {accent}; }}

        /* ── Breadcrumb bar ── */

        .magma-breadcrumb {{
            padding: 2px 4px 6px 4px;
            min-height: 18px;
        }}

        .magma-breadcrumb-sep {{
            color: {text_dim};
            font-size: {folder_title_size};
            margin: 0 1px;
            opacity: 0.5;
        }}

        .magma-breadcrumb-segment {{
            color: {text_dim};
            font-size: {folder_title_size};
        }}

        .magma-breadcrumb-active {{
            color: {text_secondary};
            font-weight: 600;
        }}

        box.magma-input-pill {{
            background: alpha({text_primary}, 0.012);
            border: 1px solid alpha({text_primary}, 0.06);
            border-radius: 8px;
            padding: 4px 12px;
            margin: 0 0 8px 0;
            transition: all 200ms cubic-bezier(0.16, 1, 0.3, 1);
        }}

        box.magma-input-pill:focus-within {{
            border-color: {accent};
            background: alpha({accent}, 0.015);
            box-shadow: 0 0 0 2px alpha({accent}, 0.15);
        }}

        box.magma-input-pill.terminal-active {{
            border-color: {accent};
        }}

        button.magma-workspace-button,
        button.magma-tool-button {{
            background: transparent;
            color: {text_primary};
            border-radius: 50%;
            padding: 8px;
            min-height: 28px;
            min-width: 28px;
            box-shadow: none;
            border: none;
            opacity: 0.6;
            transition: all 250ms cubic-bezier(0.16, 1, 0.3, 1);
        }}

        button.magma-search-toggle {{
            background: transparent;
            color: {text_primary};
            border-radius: 50%;
            padding: 2px;
            min-height: 18px;
            min-width: 18px;
            box-shadow: none;
            border: none;
            opacity: 0.35;
            transition: opacity 140ms ease;
        }}

        menubutton.magma-tool-menu > button {{
            background: transparent;
            color: {text_primary};
            border-radius: 50%;
            padding: 8px;
            min-height: 28px;
            min-width: 28px;
            box-shadow: none;
            border: none;
            opacity: 0.6;
            transition: opacity 140ms ease, background 140ms ease;
        }}

        button.magma-search-toggle:hover {{
            opacity: 0.8;
        }}

        button.magma-workspace-button:hover,
        button.magma-tool-button:hover {{
            background: {surface};
            opacity: 1.0;
        }}

        menubutton.magma-tool-menu > button:hover,
        menubutton.magma-tool-menu:checked > button {{
            background: {surface};
            opacity: 1.0;
        }}

        popover.magma-inspector-popover {{
            background: {surface};
            border: 1px solid {border};
            border-radius: 12px;
        }}

        popover.magma-inspector-popover > contents {{
            padding: 0;
            background: transparent;
            border-radius: 12px;
        }}

        .magma-inspector-panel {{
            background: transparent;
            padding: 12px;
            min-width: 280px;
        }}

        .magma-inspector-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            opacity: 0.92;
            margin-bottom: 2px;
        }}

        .magma-inspector-row {{
            background: transparent;
            border: 1px solid {border};
            border-radius: 10px;
            padding: 10px;
        }}

        .magma-inspector-key {{
            color: {accent};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 700;
            opacity: 0.78;
        }}

        .magma-inspector-value {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.62;
            line-height: 1.45;
        }}

        .magma-tab-bar-container {{
            background: transparent;
            margin: 0 12px;
            padding: 4px 0;
            min-height: 40px;
        }}

        .magma-tab-bar-scroller {{
            background: transparent;
            margin-right: 8px;
            min-height: 40px;
        }}

        .magma-switcher-overlay {{
            background: transparent;
        }}

        .magma-switcher-panel {{
            background: alpha({window_bg}, 0.94);
            border: 1px solid {border};
            border-radius: 14px;
            padding: 12px;
            min-width: 360px;
        }}

        entry.magma-switcher-entry {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-bottom: 1px solid alpha({text_primary}, 0.08);
            border-radius: 0;
            padding: 4px 2px 10px 2px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            box-shadow: none;
            outline: none;
        }}

        entry.magma-switcher-entry:focus {{
            border-bottom-color: {accent};
            box-shadow: none;
            outline: none;
        }}

        .magma-switcher-list {{
            background: transparent;
        }}

        row.magma-switcher-row {{
            background: transparent;
            border-radius: 8px;
            margin: 1px 0;
            padding: 0;
            transition: background 100ms ease;
        }}

        row.magma-switcher-row:hover,
        row.magma-switcher-row:selected {{
            background: alpha({text_primary}, 0.05);
        }}

        .magma-switcher-index {{
            color: {accent};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            min-width: 20px;
            opacity: 0.8;
        }}

        .magma-switcher-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            font-weight: 700;
            opacity: 0.82;
        }}

        .magma-switcher-empty {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            opacity: 0.35;
            padding: 8px 2px;
        }}

        .magma-tabs-list {{
            background: transparent;
            padding: 2px 4px;
        }}

        .magma-tab-item {{
            background: transparent;
            border-radius: 6px;
            border: 1px solid transparent;
            padding: 6px 12px;
            margin: 0 3px;
            transition: all 200ms cubic-bezier(0.16, 1, 0.3, 1);
            opacity: 0.5;
        }}

        .magma-tab-item:hover {{
            background: alpha({text_primary}, 0.035);
            opacity: 0.85;
            transform: scale(1.01) translateY(-0.5px);
        }}

        .magma-tab-item.active {{
            background: alpha({accent}, 0.05);
            border: 1px solid alpha({accent}, 0.15);
            box-shadow: 0 2px 6px alpha(#000000, 0.12);
            opacity: 1.0;
        }}

        .magma-tab-item.active label.magma-tab-label {{
            color: {accent};
        }}

        button.magma-tab-close-button {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 4px;
            min-height: 18px;
            min-width: 18px;
            padding: 0;
            margin-left: 6px;
            opacity: 0;
            box-shadow: none;
            transition: all 180ms ease;
        }}

        .magma-tab-item:hover button.magma-tab-close-button,
        .magma-tab-item.active button.magma-tab-close-button {{
            opacity: 0.5;
        }}

        button.magma-tab-close-button:hover {{
            background: alpha({sem_orange}, 0.15);
            color: {sem_orange};
            opacity: 1.0;
        }}

        .magma-tab-item.dragging {{
            opacity: 0.3;
        }}

        .magma-tab-item.drop-target {{
            background: alpha({accent}, 0.08);
            border-color: alpha({accent}, 0.2);
        }}

        button.magma-add-tab-button {{
            background: transparent;
            color: {text_primary};
            border: none;
            padding: 6px;
            min-height: 26px;
            min-width: 26px;
            opacity: 0.5;
            margin: 0 4px;
            border-radius: 6px;
            transition: all 180ms ease;
        }}

        button.magma-add-tab-button:hover {{
            opacity: 1.0;
            background: alpha({text_primary}, 0.05);
            transform: scale(1.05);
        }}

        notebook.magma-notebook > stack {{
            background: transparent;
        }}

        label.magma-tab-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            font-weight: 700;
            text-transform: lowercase;
        }}

        entry.magma-tab-rename-entry {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-bottom: 1px solid alpha({accent}, 0.35);
            border-radius: 0;
            padding: 2px 0;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            font-weight: 700;
            box-shadow: none;
            outline: none;
        }}

        entry.magma-tab-rename-entry:focus {{
            border-bottom-color: {accent};
            box-shadow: none;
            outline: none;
        }}

        entry.magma-entry.search-active {{
            color: {sem_yellow};
        }}

        box.magma-input-pill.search-active {{
            border-color: alpha({sem_yellow}, 0.3);
        }}

        label.magma-user-label {{
            color: {accent};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            font-weight: 700;
        }}

        label.magma-path-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            font-weight: 700;
        }}

        label.magma-status-label {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            margin: 0 6px 0 0;
        }}

        label.magma-notice-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.58;
            margin: 0 8px 0 0;
        }}

        label.magma-notice-ok {{
            color: {sem_green};
            opacity: 0.7;
        }}

        label.magma-notice-error {{
            color: {accent};
            opacity: 0.9;
        }}

        label.magma-status-ok {{
            color: {sem_green};
        }}

        label.magma-status-error {{
            color: {accent};
        }}

        label.magma-status-running {{
            color: {sem_yellow};
        }}

        entry.magma-entry {{
            background: transparent;
            color: {text_primary};
            border: none;
            padding: 8px 0;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            box-shadow: none;
            outline: none;
        }}

        entry.magma-entry:focus {{
            box-shadow: none;
            outline: none;
        }}

        /* Logr Pane */
        .magma-logr-root {{
            padding: 6px 8px;
        }}

        .magma-view-root {{
            padding: 0 4px 0 4px;
        }}

        .magma-view-header {{
            padding: 4px 0 12px 0;
            margin-bottom: 2px;
            border-bottom: 1px solid alpha({text_primary}, 0.035);
        }}

        .magma-view-heading {{
            min-width: 0;
        }}

        .magma-view-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            font-weight: 700;
            text-transform: lowercase;
            letter-spacing: 0.03em;
            opacity: 0.96;
        }}

        .magma-view-count {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 500;
            opacity: 0.54;
            line-height: 1.5;
        }}

        button.magma-view-header-action {{
            background: transparent;
            color: {text_primary};
            border: 1px solid alpha({text_primary}, 0.05);
            border-radius: 8px;
            min-height: 28px;
            min-width: 28px;
            padding: 4px;
            box-shadow: none;
            opacity: 0.5;
            transition: opacity 140ms ease, background 140ms ease, border-color 140ms ease;
        }}

        button.magma-view-header-action:hover {{
            background: alpha({text_primary}, 0.025);
            border-color: alpha({text_primary}, 0.09);
            opacity: 1.0;
        }}

        button.magma-view-action,
        button.magma-view-open {{
            background: alpha({text_primary}, 0.02);
            color: {text_primary};
            border: 1px solid alpha({text_primary}, 0.04);
            border-radius: 999px;
            min-height: 24px;
            min-width: 24px;
            padding: 3px 10px;
            box-shadow: none;
            opacity: 0.62;
            transition: opacity 140ms ease, background 140ms ease, border-color 140ms ease;
        }}

        button.magma-view-action:hover,
        button.magma-view-open:hover {{
            opacity: 1.0;
            background: alpha({accent}, 0.05);
            border-color: alpha({accent}, 0.12);
        }}

        .magma-view-scope {{
            padding: 0 8px 4px 8px;
        }}

        .magma-view-scope-chip {{
            background: alpha({text_primary}, 0.03);
            color: alpha({text_primary}, 0.40);
            border: 1px solid alpha({text_primary}, 0.05);
            border-radius: 10px;
            padding: 1px 10px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            box-shadow: none;
            min-height: 0;
            transition: background 120ms ease, color 120ms ease, border-color 120ms ease;
        }}

        .magma-view-scope-chip:hover {{
            color: alpha({text_primary}, 0.65);
            border-color: alpha({text_primary}, 0.10);
            background: alpha({text_primary}, 0.05);
        }}

        .magma-view-scope-chip-active {{
            background: alpha({accent}, 0.08);
            color: {accent};
            border-color: alpha({accent}, 0.16);
        }}

        .magma-view-scope-chip-active:hover {{
            background: alpha({accent}, 0.12);
            color: {accent};
        }}

        .magma-view-file-scroller {{
            margin-bottom: 8px;
            background: alpha({text_primary}, 0.012);
            border: 1px solid alpha({text_primary}, 0.035);
            border-radius: 14px;
            padding: 4px;
        }}

        .magma-view-file-list {{
            background: transparent;
        }}

        row.magma-view-file-row {{
            background: transparent;
            border-radius: 10px;
            margin: 0 0 2px 0;
            padding: 0;
            transition: background 140ms ease, border-color 140ms ease;
        }}

        row.magma-view-file-row:hover {{
            background: alpha({accent}, 0.03);
        }}

        row.magma-view-file-row:selected {{
            background: alpha({accent}, 0.06);
            border: 1px solid alpha({accent}, 0.12);
        }}

        .magma-view-file-card {{
            padding: 8px 10px;
        }}

        .magma-view-file-icon {{
            color: {text_primary};
            opacity: 0.5;
            -gtk-icon-size: 16px;
        }}

        .magma-view-file-name {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            opacity: 0.9;
        }}

        .magma-view-file-meta {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.42;
            margin-top: -1px;
        }}

        .magma-view-preview {{
            background: alpha({surface}, 0.98);
            border: 1px solid alpha({text_primary}, 0.04);
            border-radius: 16px;
            padding: 0;
        }}

        .magma-view-preview-chrome {{
            padding: 14px 16px;
            border-bottom: 1px solid alpha({text_primary}, 0.045);
            background: alpha({surface}, 0.98);
        }}

        .magma-view-preview-actions {{
            margin-left: 12px;
            background: alpha({text_primary}, 0.02);
            border: 1px solid alpha({text_primary}, 0.04);
            border-radius: 9px;
            padding: 3px;
        }}

        .magma-view-preview-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            font-weight: 700;
            opacity: 0.9;
        }}

        .magma-view-preview-meta {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.44;
        }}

        button.magma-view-preview-button {{
            background: transparent;
            color: {text_primary};
            border: 1px solid transparent;
            border-radius: 7px;
            min-height: 26px;
            padding: 4px 9px;
            box-shadow: none;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 600;
            letter-spacing: 0.01em;
            opacity: 0.72;
            transition: background 140ms ease, border-color 140ms ease, opacity 140ms ease;
        }}

        button.magma-view-preview-button:hover {{
            background: alpha({text_primary}, 0.03);
            border-color: alpha({text_primary}, 0.05);
            opacity: 1.0;
        }}

        button.magma-view-preview-button:disabled {{
            opacity: 0.28;
            background: transparent;
            border-color: transparent;
        }}

        button.magma-view-preview-button-secondary {{
            background: transparent;
        }}

        button.magma-view-preview-button-primary {{
            background: alpha({text_primary}, 0.035);
            border-color: alpha({text_primary}, 0.06);
            opacity: 0.92;
        }}

        button.magma-view-preview-button-primary:hover {{
            background: alpha({text_primary}, 0.05);
            border-color: alpha({text_primary}, 0.09);
        }}

        .magma-view-preview-stack {{
            background: transparent;
        }}

        .magma-view-preview-surface,
        .magma-view-code-scroller {{
            background: {window_bg};
            border: none;
        }}

        textview.magma-view-code {{
            background: {window_bg};
            color: {text_primary};
        }}

        .magma-view-empty-state {{
            padding: 42px 24px;
            background: {window_bg};
        }}

        .magma-view-empty-icon {{
            color: {text_primary};
            opacity: 0.08;
            margin-bottom: 12px;
        }}

        .magma-view-empty-text {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            opacity: 0.35;
            line-height: 1.5;
        }}

        .magma-view-info {{
            background: {window_bg};
            padding: 24px 18px;
        }}

        .magma-view-info-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 600;
        }}

        .magma-view-info-body {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.68;
            line-height: 1.45;
            margin-top: 2px;
        }}

        .magma-web-tab-row {{
            padding: 4px 0 4px 0;
        }}

        .magma-web-tab-scroll {{
        }}

        .magma-web-tabs {{
            padding: 0 2px;
        }}

        .magma-web-tab {{
            background: transparent;
            border: 1px solid transparent;
            border-radius: 6px;
            padding: 2px 6px;
            min-height: 22px;
            opacity: 0.45;
            transition: opacity 140ms ease, background 140ms ease, border-color 140ms ease;
        }}

        .magma-web-tab:hover {{
            opacity: 0.75;
            background: alpha({text_primary}, 0.03);
        }}

        .magma-web-tab.active {{
            opacity: 1.0;
            background: alpha({text_primary}, 0.04);
            border-color: alpha({text_primary}, 0.06);
        }}

        .magma-web-tab-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
        }}

        button.magma-web-tab-close {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 50%;
            min-height: 14px;
            min-width: 14px;
            padding: 1px;
            opacity: 0;
            box-shadow: none;
            transition: opacity 140ms ease, background 140ms ease;
        }}

        .magma-web-tab:hover button.magma-web-tab-close {{
            opacity: 0.4;
        }}

        button.magma-web-tab-close:hover {{
            opacity: 1.0;
            background: alpha({accent}, 0.15);
        }}

        button.magma-web-tab-add {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 50%;
            min-height: 22px;
            min-width: 22px;
            padding: 2px;
            opacity: 0.35;
            box-shadow: none;
            transition: opacity 140ms ease;
        }}

        button.magma-web-tab-add:hover {{
            opacity: 0.9;
        }}

        .magma-web-controls {{
            margin-bottom: 4px;
        }}

        .magma-web-ssl {{
            min-width: 14px;
            min-height: 14px;
            margin: 0 2px 0 0;
            opacity: 0.5;
        }}

        .magma-web-ssl.secure {{
            color: {sem_green};
            opacity: 0.7;
        }}

        .magma-web-ssl.insecure {{
            color: {sem_yellow};
            opacity: 0.7;
        }}

        progressbar.magma-web-progress {{
            min-height: 2px;
            margin: 0 2px 4px 2px;
        }}

        progressbar.magma-web-progress trough {{
            min-height: 2px;
            background: transparent;
            border: none;
            border-radius: 1px;
        }}

        progressbar.magma-web-progress progress {{
            min-height: 2px;
            background: {accent};
            border: none;
            border-radius: 1px;
        }}

        .magma-web-find-bar {{
            background: alpha({text_primary}, 0.02);
            border: 1px solid alpha({text_primary}, 0.06);
            border-radius: 8px;
            padding: 4px 6px;
            margin: 0 2px 4px 2px;
        }}

        entry.magma-web-find-entry {{
            background: transparent;
            color: {text_primary};
            border: none;
            padding: 2px 4px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            box-shadow: none;
            outline: none;
            opacity: 0.7;
        }}

        entry.magma-web-find-entry:focus {{
            opacity: 1.0;
            box-shadow: none;
            outline: none;
        }}

        .magma-web-find-matches {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.4;
            margin: 0 4px;
        }}

        .magma-web-bar {{
            background: alpha({text_primary}, 0.02);
            border: 1px solid alpha({text_primary}, 0.06);
            border-radius: 16px;
            padding: 6px;
        }}

        .magma-web-nav {{
            background: alpha({text_primary}, 0.02);
            border: 1px solid alpha({text_primary}, 0.04);
            border-radius: 12px;
            padding: 2px;
            margin-right: 6px;
        }}

        .magma-web-address-shell {{
            background: alpha({text_primary}, 0.08);
            border: 1px solid alpha({text_primary}, 0.04);
            border-radius: 12px;
            padding: 0 4px 0 6px;
        }}

        button.magma-web-button {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 10px;
            min-height: 28px;
            min-width: 28px;
            padding: 4px;
            opacity: 0.54;
            box-shadow: none;
            transition: opacity 140ms ease, background 140ms ease;
        }}

        button.magma-web-button:hover {{
            opacity: 1.0;
            background: alpha({text_primary}, 0.06);
        }}

        button.magma-web-button:disabled {{
            opacity: 0.18;
        }}

        button.magma-web-text-button {{
            min-width: 42px;
            padding: 4px 8px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 700;
        }}

        entry.magma-web-entry {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 0;
            padding: 7px 4px 7px 0;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            box-shadow: none;
            outline: none;
            opacity: 0.78;
            transition: opacity 140ms ease;
        }}

        entry.magma-web-entry:focus {{
            opacity: 1.0;
            box-shadow: none;
            outline: none;
        }}

        .magma-web-status {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.4;
            padding: 0 2px 8px 2px;
        }}

        .magma-web-frame {{
            background: alpha({text_primary}, 0.01);
            border: 1px solid alpha({text_primary}, 0.05);
            border-radius: 16px;
            padding: 0;
        }}

        .magma-webview {{
            background: {window_bg};
            border: none;
            border-radius: 16px;
            margin-top: 0;
        }}

        .magma-logr-header {{
            padding: 4px 0 12px 0;
            margin-bottom: 2px;
            border-bottom: 1px solid alpha({text_primary}, 0.035);
        }}

        .magma-logr-heading {{
            min-width: 0;
        }}

        .magma-logr-tab-row {{
            padding: 0 0 8px 0;
        }}

        .magma-logr-tabs {{
            background: transparent;
        }}

        .magma-logr-tab {{
            background: transparent;
            border: 1px solid transparent;
            border-radius: 12px;
            padding: 5px 9px;
            opacity: 0.56;
            transition: all 250ms cubic-bezier(0.16, 1, 0.3, 1);
        }}

        .magma-logr-tab:hover {{
            opacity: 0.88;
            background: alpha({text_primary}, 0.02);
            border-color: alpha({text_primary}, 0.05);
            transform: scale(1.03) translateY(-0.5px);
        }}

        .magma-logr-tab.active {{
            opacity: 1.0;
            background: alpha({accent}, 0.035);
            border-color: alpha({accent}, 0.12);
        }}

        .magma-logr-tab-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 700;
            letter-spacing: 0.02em;
            opacity: 0.72;
        }}

        .magma-logr-tab.active .magma-logr-tab-label {{
            color: {accent};
            opacity: 0.92;
        }}

        button.magma-logr-tab-close,
        button.magma-logr-tab-add {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 10px;
            min-height: 20px;
            min-width: 20px;
            padding: 1px;
            box-shadow: none;
            opacity: 0.28;
            transition: opacity 140ms ease, background 140ms ease;
        }}

        button.magma-logr-tab-close:hover,
        button.magma-logr-tab-add:hover {{
            opacity: 1.0;
            background: alpha({text_primary}, 0.05);
        }}

        .magma-logr-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_13};
            font-weight: 700;
            opacity: 0.96;
            text-transform: lowercase;
            letter-spacing: 0.03em;
        }}

        .magma-logr-count {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            opacity: 0.62;
        }}

        .magma-logr-picker {{
            background: alpha({text_primary}, 0.018);
            border: 1px solid alpha({text_primary}, 0.04);
            border-radius: 999px;
            padding: 4px 6px;
            margin: 0 0 6px 0;
        }}

        .magma-logr-inline-icon {{
            color: {accent};
            -gtk-icon-size: 12px;
            margin: 0 2px 0 2px;
            opacity: 0.7;
        }}

        menubutton.magma-logr-select > button {{
            background: transparent;
            border: 1px solid transparent;
            border-radius: 999px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            color: {text_primary};
            min-height: 24px;
            padding: 3px 10px;
            box-shadow: none;
            opacity: 0.82;
            transition: opacity 140ms ease, border-color 140ms ease, background 140ms ease;
        }}

        menubutton.magma-logr-select > button:hover {{
            opacity: 1.0;
            border-color: alpha({accent}, 0.14);
            background: alpha({accent}, 0.045);
        }}

        popover.magma-logr-popover {{
            background: {window_bg};
            border: 1px solid {border};
            border-radius: 6px;
            padding: 4px 0;
        }}

        popover.magma-logr-popover > contents {{
            background: {window_bg};
            border-radius: 6px;
            padding: 0;
        }}

        .magma-logr-popover-list {{
            background: transparent;
        }}

        .magma-logr-popover-row {{
            background: transparent;
            padding: 0;
            border-radius: 4px;
            margin: 1px 4px;
            transition: background 100ms ease;
        }}

        .magma-logr-popover-row:hover {{
            background: alpha({text_primary}, 0.05);
        }}

        row.magma-logr-popover-row:focus {{
            background: alpha({accent}, 0.10);
        }}

        .magma-logr-popover-item {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            padding: 5px 8px;
            opacity: 0.7;
        }}

        .magma-logr-popover-row:hover .magma-logr-popover-item {{
            opacity: 1.0;
        }}

        button.magma-logr-icon-btn {{
            background: alpha({text_primary}, 0.02);
            color: {text_primary};
            border: 1px solid alpha({text_primary}, 0.04);
            border-radius: 999px;
            min-height: 24px;
            min-width: 24px;
            padding: 3px;
            opacity: 0.62;
            box-shadow: none;
            transition: opacity 140ms ease, background 140ms ease, border-color 140ms ease;
        }}

        button.magma-logr-icon-btn:hover {{
            opacity: 1.0;
            background: alpha({accent}, 0.05);
            border-color: alpha({accent}, 0.12);
        }}

        .magma-logr-controls {{
            background: alpha({text_primary}, 0.012);
            border: 1px solid alpha({text_primary}, 0.035);
            border-radius: 999px;
            padding: 4px 6px;
            margin-bottom: 6px;
        }}

        .magma-logr-stream-shell {{
            background: alpha({text_primary}, 0.018);
            border-radius: 999px;
            padding: 2px 10px 2px 8px;
            margin: 0 6px;
        }}

        .magma-logr-stream-icon {{
            color: {sem_green};
            -gtk-icon-size: 10px;
            opacity: 0.72;
        }}

        .magma-logr-stream-label {{
            color: {sem_green};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.82;
        }}

        entry.magma-logr-filter {{
            background: alpha({text_primary}, 0.018);
            color: {text_primary};
            border: 1px solid alpha({text_primary}, 0.04);
            border-radius: 999px;
            padding: 6px 10px;
            margin-bottom: 6px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            box-shadow: none;
            outline: none;
            opacity: 0.82;
            transition: opacity 140ms ease, border-color 140ms ease, background 140ms ease;
        }}

        entry.magma-logr-filter:focus {{
            opacity: 1.0;
            background: alpha({accent}, 0.035);
            border-color: alpha({accent}, 0.15);
            box-shadow: none;
            outline: none;
        }}

        .magma-logr-status {{
            color: {sem_yellow};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.68;
            padding: 6px 0 0 0;
        }}

        .magma-logr-empty {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            opacity: 0.3;
            padding: 12px 4px;
        }}

        .magma-log-list {{
            background: transparent;
        }}

        .magma-log-entry {{
            padding: 3px 8px;
            border-bottom: 1px solid alpha({text_primary}, 0.015);
            border-radius: 8px;
            transition: background 160ms ease, padding 160ms ease, border-color 160ms ease;
        }}

        .magma-log-entry:hover {{
            background: alpha({accent}, 0.03);
            border-bottom-color: alpha({accent}, 0.08);
        }}

        .magma-log-entry.expanded {{
            background: alpha({accent}, 0.04);
            border-bottom-color: transparent;
            padding-top: 6px;
            transition: background 240ms cubic-bezier(0.4, 0, 0.2, 1);
        }}

        .magma-log-line-number {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.15;
            min-width: 28px;
            margin-right: 2px;
        }}

        .magma-log-entry:hover .magma-log-line-number {{
            opacity: 0.35;
        }}

        .magma-log-details {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.6;
            line-height: 1.4;
            background: alpha({text_primary}, 0.015);
            border: 1px solid alpha({text_primary}, 0.04);
            padding: 6px 10px;
            border-radius: 10px;
        }}

        .log-level-dot {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 700;
            min-width: 16px;
            min-height: 16px;
            border-radius: 999px;
            padding: 1px 0 0 0;
        }}

        .log-body {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            color: {text_primary};
            opacity: 0.7;
        }}

        .magma-log-fields {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.35;
            margin-left: 4px;
        }}

        button.magma-log-copy-btn {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 4px;
            padding: 2px;
            min-height: 22px;
            min-width: 22px;
            opacity: 0;
            transition: opacity 240ms cubic-bezier(0.4, 0, 0.2, 1), background 140ms ease;
            box-shadow: none;
        }}

        .magma-log-entry:hover button.magma-log-copy-btn {{
            opacity: 0.45;
        }}

        button.magma-log-copy-btn:hover {{
            opacity: 1.0;
            background: alpha({text_primary}, 0.08);
        }}

        button.magma-log-delete-btn {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 4px;
            padding: 2px;
            min-height: 22px;
            min-width: 22px;
            opacity: 0;
            transition: opacity 240ms cubic-bezier(0.4, 0, 0.2, 1), background 140ms ease;
            box-shadow: none;
        }}

        .magma-log-entry:hover button.magma-log-delete-btn {{
            opacity: 0.35;
        }}

        button.magma-log-delete-btn:hover {{
            opacity: 1.0;
            background: alpha({accent}, 0.15);
            color: {sem_orange};
        }}

        /* Log Level Colors */
        .log-error .log-level-dot {{ color: {sem_orange}; background: alpha({sem_orange}, 0.12); }}
        .log-error .log-body {{ color: {sem_orange}; opacity: 0.9; }}

        .log-warn .log-level-dot {{ color: {sem_yellow}; background: alpha({sem_yellow}, 0.12); }}
        .log-warn .log-body {{ color: {sem_yellow}; opacity: 0.85; }}

        .log-info .log-level-dot {{ color: {sem_green}; background: alpha({sem_green}, 0.12); }}
        .log-info .log-body {{ color: {sem_green}; opacity: 0.8; }}

        .log-debug .log-level-dot {{ color: {text_primary}; opacity: 0.3; background: alpha({text_primary}, 0.04); }}
        .log-debug .log-body {{ opacity: 0.4; }}

        /* Settings Page */
        .magma-settings-root {{
            padding: 16px 0 16px 24px;
        }}

        .magma-setup-root {{
            padding: 28px;
        }}

        .magma-setup-shell {{
            min-width: 760px;
            background: alpha({window_bg}, 0.94);
            border: 1px solid alpha({text_primary}, 0.08);
            border-radius: 22px;
        }}

        .magma-setup-topbar {{
            padding: 12px 16px;
            border-bottom: 1px solid alpha({text_primary}, 0.06);
            background: alpha({text_primary}, 0.015);
        }}

        .magma-setup-dot {{
            font-size: {font_9};
            opacity: 0.9;
        }}

        .magma-setup-dot.red {{
            color: alpha({sem_orange}, 0.9);
        }}

        .magma-setup-dot.amber {{
            color: alpha({sem_yellow}, 0.9);
        }}

        .magma-setup-dot.green {{
            color: alpha({sem_green}, 0.9);
        }}

        .magma-setup-topbar-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 700;
            letter-spacing: 0.16em;
            opacity: 0.36;
        }}

        .magma-setup-body {{
            padding: 28px 30px 30px 30px;
        }}

        .magma-setup-eyebrow {{
            color: {accent};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 700;
            letter-spacing: 0.14em;
            opacity: 0.9;
        }}

        .magma-setup-hero {{
            margin-bottom: 18px;
        }}

        .magma-setup-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_13};
            font-weight: 700;
            opacity: 0.98;
            margin: 2px 0 4px 0;
        }}

        .magma-setup-copy {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            opacity: 0.58;
            line-height: 1.6;
        }}

        .magma-setup-progress {{
            margin-bottom: 18px;
            padding-bottom: 6px;
            border-bottom: 1px solid alpha({text_primary}, 0.05);
        }}

        .magma-setup-step {{
            background: transparent;
            border: none;
            border-bottom: 2px solid transparent;
            border-radius: 0;
            padding: 0 0 10px 0;
        }}

        .magma-setup-step.active {{
            border-bottom-color: alpha({accent}, 0.72);
        }}

        .magma-setup-step-index {{
            color: {accent};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 700;
            letter-spacing: 0.14em;
            opacity: 0.36;
        }}

        .magma-setup-step-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            opacity: 0.42;
        }}

        .magma-setup-step.active .magma-setup-step-label {{
            opacity: 0.94;
        }}

        .magma-setup-step.active .magma-setup-step-index {{
            opacity: 0.82;
        }}

        .magma-setup-pages {{
            min-height: 286px;
        }}

        .magma-setup-page {{
            background: transparent;
            border: none;
            padding: 2px 0 0 0;
        }}

        .magma-setup-page-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            letter-spacing: 0.12em;
            opacity: 0.92;
        }}

        .magma-setup-page-copy {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.42;
            line-height: 1.55;
            margin-bottom: 10px;
        }}

        .magma-setup-setting {{
            background: alpha({text_primary}, 0.018);
            border: 1px solid alpha({text_primary}, 0.055);
            border-radius: 14px;
            padding: 14px 16px;
        }}

        .magma-setup-setting-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            opacity: 0.86;
        }}

        .magma-setup-setting-copy {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.4;
            line-height: 1.5;
        }}

        .magma-setup-value {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.62;
            line-height: 1.6;
        }}

        .magma-setup-footer {{
            margin-top: 16px;
            padding-top: 6px;
        }}

        button.magma-setup-secondary {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 0;
            padding: 4px 0;
            min-height: 28px;
            box-shadow: none;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            opacity: 0.54;
            transition: opacity 140ms ease, color 140ms ease;
        }}

        button.magma-setup-secondary:hover {{
            opacity: 1.0;
        }}

        button.magma-setup-action {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 0;
            padding: 4px 0;
            margin-top: 0;
            min-height: 28px;
            box-shadow: none;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            opacity: 0.9;
            transition: opacity 140ms ease, color 140ms ease;
        }}

        button.magma-setup-action:hover {{
            opacity: 1.0;
        }}

        .magma-setup-nav-content {{
            background: transparent;
        }}

        .magma-setup-nav-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            opacity: inherit;
        }}

        .magma-setup-nav-icon {{
            color: {text_primary};
            opacity: inherit;
        }}

        .magma-settings-header {{
            padding: 2px 24px 12px 0;
            margin-bottom: 4px;
        }}

        button.magma-settings-back {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 4px;
            min-height: 20px;
            min-width: 20px;
            padding: 2px;
            box-shadow: none;
            opacity: 0.4;
            transition: opacity 140ms ease;
            -gtk-icon-size: 12px;
        }}

        button.magma-settings-back:hover {{
            opacity: 1.0;
        }}

        .magma-settings-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_12};
            font-weight: 700;
            text-transform: lowercase;
            letter-spacing: 0.04em;
            opacity: 0.95;
            margin-top: 0;
        }}

        .magma-settings-main {{
            padding: 2px 10px 12px 0;
        }}

        .magma-settings-nav {{
            background: transparent;
            border: none;
            border-radius: 0;
            padding: 2px 0 0 0;
        }}

        .magma-settings-nav-heading {{
            color: {accent};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            text-transform: uppercase;
            letter-spacing: 0.14em;
            margin-bottom: 2px;
            opacity: 0.82;
        }}

        entry.magma-settings-nav-search {{
            min-width: 148px;
            margin-bottom: 8px;
            padding: 8px 10px;
            border-radius: 30px;
        }}

        button.magma-settings-nav-button {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 0;
            padding: 6px 0;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            text-transform: lowercase;
            box-shadow: none;
            opacity: 0.46;
            transition: all 200ms cubic-bezier(0.16, 1, 0.3, 1);
        }}

        button.magma-settings-nav-button:hover {{
            background: transparent;
            border-color: transparent;
            color: alpha({text_primary}, 0.82);
            opacity: 0.78;
            transform: translateX(4px);
        }}

        button.magma-settings-nav-button.active {{
            color: {accent};
            opacity: 1.0;
            transform: translateX(6px);
        }}

        .magma-settings-detail {{
            background: transparent;
            border: none;
            border-radius: 0;
        }}

        .magma-settings-content {{
            padding: 6px 8px 24px 8px;
        }}

        .magma-settings-empty {{
            padding: 28px 8px;
        }}

        .magma-settings-section {{
            color: {accent};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            text-transform: uppercase;
            letter-spacing: 0.14em;
            padding: 32px 0 8px 0;
            opacity: 0.82;
            border-bottom: 1px solid alpha({accent}, 0.12);
            margin-bottom: 12px;
        }}

        .magma-settings-row {{
            background: transparent;
            border: none;
            border-bottom: 1px solid alpha({text_primary}, 0.025);
            padding: 12px 0;
            margin-bottom: 2px;
            transition: opacity 160ms ease;
        }}

        .magma-settings-row:hover {{
            background: transparent;
            opacity: 1.0;
        }}

        .magma-settings-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            font-weight: 700;
            opacity: 0.9;
        }}

        .magma-settings-copy {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            opacity: 0.42;
            line-height: 1.5;
            margin-top: 2px;
        }}

        .magma-settings-value {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            opacity: 0.6;
        }}

        .magma-settings-about-copy {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            line-height: 1.6;
            opacity: 0.54;
        }}

        entry.magma-settings-entry {{
            background: alpha({text_primary}, 0.015);
            color: {text_primary};
            border: 1px solid alpha({text_primary}, 0.04);
            border-radius: 8px;
            padding: 6px 10px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            box-shadow: none;
            outline: none;
            transition: border-color 140ms ease, background 140ms ease;
        }}

        entry.magma-settings-entry:focus {{
            background: alpha({text_primary}, 0.025);
            border-color: {accent};
        }}

        spinbutton.magma-settings-spin {{
            background: alpha({text_primary}, 0.015);
            color: {text_primary};
            border: 1px solid alpha({text_primary}, 0.04);
            border-radius: 8px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            min-width: 90px;
            box-shadow: none;
        }}

        spinbutton.magma-settings-spin > text {{
            background: transparent;
        }}

        spinbutton.magma-settings-spin > button {{
            background: transparent;
            color: {text_primary};
            border: none;
            opacity: 0.4;
            transition: opacity 140ms ease;
        }}

        spinbutton.magma-settings-spin > button:hover {{
            opacity: 1.0;
        }}

        .magma-settings-dropdown {{
            background: transparent;
            border: 1px solid {border};
            border-radius: 8px;
            min-width: 140px;
        }}

        .magma-settings-dropdown > button {{
            background: transparent;
            color: {text_primary};
            border: none;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            padding: 6px 10px;
            opacity: 0.8;
            transition: opacity 140ms ease;
        }}

        popover > contents {{
            background: {surface};
            border: 1px solid {border};
            border-radius: 12px;
            padding: 8px;
            box-shadow: none;
        }}

        popover listview > row,
        popover list > row {{
            border-radius: 6px;
            padding: 2px 4px;
            transition: background 140ms ease;
        }}

        popover listview > row:hover,
        popover listview > row:selected,
        popover list > row:hover,
        popover list > row:selected {{
            background: {window_bg};
        }}

        popover label {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            color: {text_primary};
        }}

        switch.magma-settings-switch {{
            background: alpha({text_primary}, 0.03);
            border: 1px solid alpha({text_primary}, 0.04);
            border-radius: 12px;
            min-width: 42px;
            min-height: 22px;
            padding: 0;
            transition: background 180ms cubic-bezier(0.4, 0, 0.2, 1), border-color 180ms ease;
        }}

        switch.magma-settings-switch:checked {{
            background: alpha({accent}, 0.08);
            border-color: alpha({accent}, 0.18);
        }}

        switch.magma-settings-switch > image {{
            color: transparent;
        }}

        switch.magma-settings-switch slider {{
            background: alpha({text_primary}, 0.22);
            border: none;
            border-radius: 50%;
            min-width: 14px;
            min-height: 14px;
            margin: 4px;
            transition: background 180ms cubic-bezier(0.4, 0, 0.2, 1), transform 180ms cubic-bezier(0.4, 0, 0.2, 1);
        }}

        switch.magma-settings-switch:checked slider {{
            background: {accent};
            box-shadow: none;
        }}

        button.magma-settings-link {{
            background: transparent;
            color: {text_primary};
            border: 1px solid alpha({text_primary}, 0.04);
            border-radius: 10px;
            padding: 6px 14px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            opacity: 0.6;
            transition: background 140ms ease, border-color 140ms ease, opacity 140ms ease;
        }}

        button.magma-settings-link:hover {{
            background: alpha({text_primary}, 0.02);
            opacity: 1.0;
            border-color: alpha({text_primary}, 0.1);
        }}

        .magma-settings-about-page {{
            padding: 24px;
        }}

        .magma-settings-about-panel {{
        }}

        .magma-settings-about-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_22};
            font-weight: 700;
            text-transform: lowercase;
            letter-spacing: 0.06em;
            opacity: 0.96;
        }}

        .magma-settings-about-name {{
            color: {accent};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_12};
            font-weight: 700;
            letter-spacing: 0.08em;
            opacity: 0.9;
            margin-top: 8px;
        }}

        .magma-settings-about-meta {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            opacity: 0.38;
            margin-top: 4px;
        }}

        .magma-about-section-header {{
            color: {accent};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            font-weight: 700;
            text-transform: uppercase;
            letter-spacing: 0.1em;
            opacity: 0.8;
        }}

        .magma-about-credits-box {{
            opacity: 0.7;
        }}

        .magma-about-category-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            opacity: 0.4;
            text-transform: lowercase;
        }}

        .magma-about-license-text {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.35;
            line-height: 1.6;
        }}

        button.magma-settings-save {{
            background: transparent;
            color: {accent};
            border: 1px solid {accent};
            border-radius: 4px;
            padding: 6px 24px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            font-weight: 700;
            box-shadow: none;
            transition: background 140ms ease;
        }}

        button.magma-settings-save:hover {{
            background: alpha({accent}, 0.10);
        }}

        /* ─── Git Pane ─────────────────────────────────────────── */

        .magma-git-root {{
            padding: 8px;
        }}

        .magma-git-header {{
            padding: 4px 0 8px 0;
        }}

        .magma-git-title {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_13};
            font-weight: 700;
            color: {text_primary};
        }}

        .magma-git-branch-label {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            color: {accent};
            padding: 0 8px;
        }}

        .magma-git-ahead-behind {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            color: alpha({text_primary}, 0.5);
        }}

        .magma-git-remote-bar {{
            padding: 4px 0;
            border-bottom: 1px solid {border};
            margin-bottom: 4px;
        }}

        .magma-git-remote-button {{
            background: transparent;
            color: alpha({text_primary}, 0.6);
            border: 1px solid alpha({text_primary}, 0.08);
            border-radius: 4px;
            padding: 2px 10px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            box-shadow: none;
            min-height: 0;
        }}

        .magma-git-remote-button:hover {{
            color: {text_primary};
            border-color: alpha({text_primary}, 0.15);
        }}

        .magma-git-icon-btn {{
            background: transparent;
            color: alpha({text_primary}, 0.5);
            border: none;
            border-radius: 4px;
            padding: 2px 6px;
            box-shadow: none;
            min-height: 0;
            min-width: 0;
        }}

        .magma-git-icon-btn:hover {{
            color: {text_primary};
            background: alpha({text_primary}, 0.06);
        }}

        .magma-git-nav {{
            padding: 4px 0;
            border-bottom: 1px solid {border};
            margin-bottom: 4px;
        }}

        .magma-git-nav-button {{
            background: transparent;
            color: alpha({text_primary}, 0.4);
            border: none;
            border-radius: 4px;
            padding: 3px 8px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 600;
            box-shadow: none;
            min-height: 0;
        }}

        .magma-git-nav-button:hover {{
            color: alpha({text_primary}, 0.7);
        }}

        .magma-git-nav-button.active {{
            color: {accent};
            background: alpha({accent}, 0.08);
        }}

        .magma-git-status {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: alpha({text_primary}, 0.35);
            padding: 4px 0;
            border-top: 1px solid {border};
            margin-top: 4px;
            transition: color 200ms ease;
        }}

        .magma-git-status-ok {{
            color: {sem_green};
        }}

        .magma-git-status-err {{
            color: {accent};
        }}

        .magma-git-status-busy {{
            color: {sem_yellow};
        }}

        /* ─── Staging ──────────────────────────────────────────── */

        .magma-git-staging-root {{
            padding: 0;
        }}

        .magma-git-commit-box {{
            padding: 0 0 8px 0;
            border-bottom: 1px solid {border};
            margin-bottom: 4px;
        }}

        .magma-git-commit-entry {{
            background: alpha({text_primary}, 0.03);
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            border: 1px solid alpha({text_primary}, 0.06);
            border-radius: 4px;
        }}

        .magma-git-commit-actions {{
            padding: 4px 0 0 0;
        }}

        .magma-git-commit-button {{
            background: {accent};
            color: {text_dim};
            border: none;
            border-radius: 4px;
            padding: 3px 14px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            box-shadow: none;
            min-height: 0;
        }}

        .magma-git-commit-button:hover {{
            background: {sem_orange};
        }}

        .magma-git-commit-button:disabled {{
            background: alpha({accent}, 0.3);
            color: alpha({window_bg}, 0.5);
        }}

        .magma-git-action-btn {{
            background: transparent;
            color: alpha({text_primary}, 0.5);
            border: 1px solid alpha({text_primary}, 0.06);
            border-radius: 4px;
            padding: 2px 8px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            box-shadow: none;
            min-height: 0;
        }}

        .magma-git-action-btn:hover {{
            color: {text_primary};
            border-color: alpha({text_primary}, 0.12);
        }}

        .magma-git-file-section {{
            margin-top: 2px;
        }}

        .magma-git-file-section-header {{
            padding: 4px 2px;
        }}

        .magma-git-section-arrow {{
            font-size: {font_9};
            color: alpha({text_primary}, 0.3);
            min-width: 12px;
        }}

        .magma-git-section-title {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 600;
            color: alpha({text_primary}, 0.5);
            text-transform: uppercase;
        }}

        .magma-git-section-count {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: alpha({text_primary}, 0.3);
        }}

        .magma-git-file-list {{
            background: transparent;
        }}

        .magma-git-file-row-container {{
            margin: 0;
        }}

        .magma-git-file-row {{
            padding: 3px 4px;
            border-bottom: 1px solid alpha({text_primary}, 0.02);
        }}

        .magma-git-file-row:hover {{
            background: alpha({text_primary}, 0.03);
        }}

        .magma-git-file-status {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 700;
            min-width: 16px;
            padding: 0 2px;
        }}

        .magma-git-file-status.status-added {{
            color: {sem_green};
        }}

        .magma-git-file-status.status-modified {{
            color: {sem_yellow};
        }}

        .magma-git-file-status.status-deleted {{
            color: {accent};
        }}

        .magma-git-file-status.status-other {{
            color: alpha({text_primary}, 0.4);
        }}

        .magma-git-file-status.status-renamed {{
            color: {sem_blue};
        }}

        .magma-git-file-status.status-conflict {{
            color: {accent};
            font-weight: bold;
        }}

        .magma-git-conflict-row {{
            background: alpha({accent}, 0.08);
        }}

        .magma-git-conflict-hint {{
            font-size: {font_9};
            color: alpha({text_primary}, 0.35);
            font-style: italic;
        }}

        .magma-git-file-name {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: {text_primary};
        }}

        .magma-git-file-action {{
            background: transparent;
            color: alpha({text_primary}, 0.4);
            border: none;
            border-radius: 3px;
            padding: 1px 6px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 8px;
            box-shadow: none;
            min-height: 0;
            min-width: 0;
        }}

        .magma-git-file-action:hover {{
            color: {text_primary};
            background: alpha({text_primary}, 0.06);
        }}

        .magma-git-file-discard {{
            background: transparent;
            color: alpha({accent}, 0.5);
            border: none;
            border-radius: 3px;
            padding: 1px 4px;
            box-shadow: none;
            min-height: 0;
            min-width: 0;
            -gtk-icon-size: 12px;
        }}

        .magma-git-file-discard:hover {{
            color: {accent};
            background: alpha({accent}, 0.1);
        }}

        /* ─── Diff ─────────────────────────────────────────────── */

        .magma-git-diff-root {{
            padding: 4px 0 4px 16px;
        }}

        .magma-git-hunk-header {{
            padding: 2px 4px;
            border-bottom: 1px solid alpha({text_primary}, 0.04);
        }}

        .magma-git-hunk-header-text {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: alpha({text_primary}, 0.35);
        }}

        .magma-git-diff-line {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            padding: 0 4px;
        }}

        .magma-git-line-added {{
            color: {sem_green};
            background: alpha({sem_green}, 0.08);
        }}

        .magma-git-line-removed {{
            color: {accent};
            background: alpha({accent}, 0.08);
        }}

        .magma-git-line-context {{
            color: alpha({text_primary}, 0.35);
        }}

        .magma-git-line-header {{
            color: alpha({text_primary}, 0.25);
        }}

        .magma-git-diff-stat {{
            padding: 4px 0;
        }}

        .magma-git-diff-stat-line {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: alpha({text_primary}, 0.45);
        }}

        .magma-git-stage-hunk-btn {{
            background: transparent;
            color: alpha({text_primary}, 0.4);
            border: none;
            padding: 0 4px;
            font-size: 8px;
            box-shadow: none;
            min-height: 0;
            min-width: 0;
        }}

        /* ─── Graph / Log ──────────────────────────────────────── */

        .magma-git-graph-root {{
            padding: 0;
        }}

        .magma-git-graph-list {{
            background: transparent;
        }}

        .magma-git-commit-container {{
            margin: 0;
        }}

        .magma-git-commit-row {{
            padding: 4px;
            border-bottom: 1px solid alpha({text_primary}, 0.02);
        }}

        .magma-git-commit-row:hover {{
            background: alpha({text_primary}, 0.03);
        }}

        .magma-git-graph-art {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: {accent};
            min-width: 20px;
        }}

        .magma-git-commit-hash {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: {sem_yellow};
            min-width: 50px;
        }}

        .magma-git-commit-msg {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: {text_primary};
        }}

        .magma-git-commit-author {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 8px;
            color: alpha({text_primary}, 0.35);
        }}

        .magma-git-commit-date {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 8px;
            color: alpha({text_primary}, 0.25);
        }}

        .magma-git-ref-badge {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 8px;
            color: alpha({text_primary}, 0.6);
            background: alpha({text_primary}, 0.06);
            border-radius: 3px;
            padding: 0 4px;
        }}

        .magma-git-ref-badge.ref-head {{
            color: {accent};
            background: alpha({accent}, 0.1);
        }}

        .magma-git-ref-badge.ref-tag {{
            color: {sem_yellow};
            background: alpha({sem_yellow}, 0.1);
        }}

        .magma-git-commit-detail {{
            padding: 4px 0 4px 16px;
            border-top: 1px solid alpha({text_primary}, 0.04);
        }}

        .magma-git-load-more {{
            background: transparent;
            color: alpha({text_primary}, 0.4);
            border: 1px solid alpha({text_primary}, 0.06);
            border-radius: 4px;
            padding: 4px;
            margin: 4px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            box-shadow: none;
        }}

        .magma-git-load-more:hover {{
            color: {text_primary};
        }}

        /* ─── Branches ─────────────────────────────────────────── */

        .magma-git-branch-root {{
            padding: 0;
        }}

        .magma-git-branch-create {{
            padding: 4px 0;
            border-bottom: 1px solid {border};
            margin-bottom: 4px;
        }}

        .magma-git-branch-entry {{
            background: alpha({text_primary}, 0.03);
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            border: 1px solid alpha({text_primary}, 0.06);
            border-radius: 4px;
            padding: 3px 8px;
        }}

        .magma-git-branch-list {{
            background: transparent;
        }}

        .magma-git-branch-row {{
            padding: 4px;
            border-bottom: 1px solid alpha({text_primary}, 0.02);
        }}

        .magma-git-branch-row:hover {{
            background: alpha({text_primary}, 0.03);
        }}

        .magma-git-branch-current {{
            background: alpha({accent}, 0.04);
        }}

        .magma-git-branch-indicator {{
            font-size: {font_9};
            color: {accent};
            min-width: 12px;
        }}

        .magma-git-branch-name {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            color: {text_primary};
        }}

        .magma-git-branch-commit {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 8px;
            color: alpha({text_primary}, 0.3);
        }}

        .magma-git-branch-remote .magma-git-branch-name {{
            color: alpha({text_primary}, 0.45);
        }}

        /* ─── Stash ────────────────────────────────────────────── */

        .magma-git-stash-root {{
            padding: 0;
        }}

        .magma-git-stash-push {{
            padding: 4px 0;
            border-bottom: 1px solid {border};
            margin-bottom: 4px;
        }}

        .magma-git-stash-entry {{
            background: alpha({text_primary}, 0.03);
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            border: 1px solid alpha({text_primary}, 0.06);
            border-radius: 4px;
            padding: 3px 8px;
        }}

        .magma-git-stash-list {{
            background: transparent;
        }}

        .magma-git-stash-row-container {{
            margin: 0;
        }}

        .magma-git-stash-row {{
            padding: 4px;
            border-bottom: 1px solid alpha({text_primary}, 0.02);
        }}

        .magma-git-stash-row:hover {{
            background: alpha({text_primary}, 0.03);
        }}

        .magma-git-stash-index {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: {sem_yellow};
            min-width: 24px;
        }}

        .magma-git-stash-msg {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: {text_primary};
        }}

        /* ─── Search ───────────────────────────────────────────── */

        .magma-git-search-root {{
            padding: 0;
        }}

        .magma-git-search-bar {{
            padding: 4px 0;
            border-bottom: 1px solid {border};
            margin-bottom: 4px;
        }}

        .magma-git-search-mode {{
            font-size: {font_9};
        }}

        .magma-git-search-entry {{
            background: alpha({text_primary}, 0.03);
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            border: 1px solid alpha({text_primary}, 0.06);
            border-radius: 4px;
            padding: 3px 8px;
        }}

        .magma-git-search-count {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: alpha({text_primary}, 0.35);
            padding: 2px 0;
        }}

        .magma-git-search-results {{
            background: transparent;
        }}

        /* ─── Agent Pane ───────────────────────────────────────── */

        .magma-agent-pane {{
            padding: 8px;
        }}

        .magma-agent-header {{
            padding: 4px 0 8px 0;
            border-bottom: 1px solid {border};
            margin-bottom: 4px;
        }}

        .magma-agent-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_13};
            font-weight: 700;
            text-transform: lowercase;
            margin-right: 8px;
        }}

        .magma-agent-badge {{
            color: alpha({text_primary}, 0.45);
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            background: alpha({text_primary}, 0.04);
            border-radius: 3px;
            padding: 0 4px;
        }}

        .magma-agent-badge.is-live {{
            color: {accent};
            background: alpha({accent}, 0.08);
        }}

        .magma-agent-canvas {{
            min-height: 272px;
            margin-top: 6px;
        }}

        .magma-agent-dots {{
            min-height: 28px;
            margin-top: 2px;
        }}

        .magma-agent-strip {{
            min-height: 38px;
            margin-top: 8px;
            padding: 6px 8px;
            border-top: 1px solid {border};
            background: alpha({text_primary}, 0.02);
        }}

        .magma-agent-strip-status {{
            min-width: 72px;
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
        }}

        .magma-agent-strip-message {{
            color: alpha({text_primary}, 0.58);
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
        }}

        .magma-agent-strip-toggle,
        .magma-agent-strip-action {{
            background: alpha({text_primary}, 0.03);
            border: 1px solid {border};
            border-radius: 6px;
            color: alpha({text_primary}, 0.78);
            padding: 4px 9px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
        }}

        .magma-agent-strip-toggle:hover,
        .magma-agent-strip-action:hover {{
            background: alpha({text_primary}, 0.06);
        }}

        .magma-agent-prompt {{
            margin-top: 10px;
            padding-top: 8px;
            border-top: 1px solid {border};
        }}

        .magma-agent-prompt-input {{
            min-height: 34px;
            background: alpha({text_primary}, 0.03);
            border: 1px solid {border};
            border-radius: 6px;
            color: {text_primary};
            padding: 5px 9px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
        }}

        .magma-agent-prompt-send {{
            min-height: 34px;
            background: alpha({text_primary}, 0.03);
            border: 1px solid {border};
            border-radius: 6px;
            color: alpha({text_primary}, 0.78);
            padding: 5px 11px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
        }}

        .magma-agent-prompt-send:hover {{
            background: alpha({text_primary}, 0.06);
        }}

        /* ─── Patch ───────────────────────────────────────────── */

        .magma-git-patch-root {{
            padding: 0;
        }}

        .magma-git-patch-paned {{
            min-height: 200px;
        }}

        .magma-git-patch-queue {{
            border-right: 1px solid {border};
        }}

        .magma-git-patch-list {{
            background: transparent;
        }}

        .magma-git-patch-file-header {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 700;
            color: alpha({text_primary}, 0.5);
            padding: 6px 4px 2px 4px;
            background: alpha({text_primary}, 0.02);
        }}

        .magma-git-patch-hunk-row {{
            padding: 3px 4px 3px 12px;
            border-bottom: 1px solid alpha({text_primary}, 0.02);
        }}

        .magma-git-patch-hunk-row:hover {{
            background: alpha({text_primary}, 0.03);
        }}

        .magma-git-patch-hunk-row.selected {{
            background: alpha({accent}, 0.08);
        }}

        .magma-git-patch-hunk-status {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            min-width: 14px;
        }}

        .magma-git-patch-hunk-status.status-unreviewed {{
            color: alpha({text_primary}, 0.2);
        }}

        .magma-git-patch-hunk-status.status-reviewed {{
            color: {sem_green};
        }}

        .magma-git-patch-hunk-status.status-risky {{
            color: {accent};
        }}

        .magma-git-patch-hunk-status.status-followup {{
            color: {sem_yellow};
        }}

        .magma-git-patch-hunk-label {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: alpha({text_primary}, 0.5);
        }}

        .magma-git-patch-detail-root {{
            padding: 0 0 0 4px;
        }}

        .magma-git-patch-detail {{
            padding: 4px;
        }}

        .magma-git-patch-detail-file {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            color: {text_primary};
            padding: 4px 0;
        }}

        .magma-git-patch-annotation-label {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 600;
            color: alpha({text_primary}, 0.35);
            padding: 4px 4px 2px 4px;
            text-transform: uppercase;
            border-top: 1px solid {border};
        }}

        .magma-git-patch-annotation-frame {{
            border: 1px solid alpha({text_primary}, 0.06);
            border-radius: 4px;
            margin: 0 4px 4px 4px;
            min-height: 48px;
        }}

        .magma-git-patch-annotation {{
            background: alpha({text_primary}, 0.03);
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
        }}

        .magma-git-patch-status-row {{
            padding: 4px;
            border-top: 1px solid {border};
        }}

        .magma-git-patch-mark-btn {{
            background: transparent;
            color: alpha({text_primary}, 0.4);
            border: 1px solid alpha({text_primary}, 0.06);
            border-radius: 4px;
            padding: 2px 8px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            box-shadow: none;
            min-height: 0;
        }}

        .magma-git-patch-mark-btn:hover {{
            color: {text_primary};
            border-color: alpha({text_primary}, 0.12);
        }}

        .magma-git-patch-mark-btn.mark-reviewed:hover {{
            color: {sem_green};
            border-color: alpha({sem_green}, 0.3);
        }}

        .magma-git-patch-mark-btn.mark-risky:hover {{
            color: {accent};
            border-color: alpha({accent}, 0.3);
        }}

        .magma-git-patch-mark-btn.mark-followup:hover {{
            color: {sem_yellow};
            border-color: alpha({sem_yellow}, 0.3);
        }}

        .magma-git-patch-actions {{
            padding: 4px;
            border-top: 1px solid {border};
        }}

        .magma-git-patch-draft-label {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: alpha({text_primary}, 0.35);
        }}

        /* ─── Shared ───────────────────────────────────────────── */

        .magma-git-empty {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            color: alpha({text_primary}, 0.25);
            padding: 16px 8px;
        }}

        .magma-git-error {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            color: {accent};
            padding: 8px;
        }}

        /* ─── Agent pane ──────────────────────────────────────── */

        .magma-agent-pane {{
            background: transparent;
            padding: 0;
        }}

        .magma-agent-header {{
            padding: 10px 10px 8px 10px;
            border-bottom: 1px solid {border};
        }}

        .magma-agent-title {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            color: {text_primary};
            text-transform: uppercase;
            letter-spacing: 1.2px;
        }}

        .magma-agent-status {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: alpha({text_primary}, 0.38);
        }}

        .magma-agent-log-scroll {{
            background: transparent;
            border-top: 1px solid alpha({text_primary}, 0.05);
            border-bottom: 1px solid alpha({text_primary}, 0.05);
        }}

        .magma-agent-log {{
            padding: 8px 10px 10px 10px;
        }}

        .magma-agent-log-row {{
            padding: 4px 0;
        }}

        .magma-agent-log-badge {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 600;
            padding: 0;
            min-width: 14px;
            border-radius: 0;
            background: transparent;
        }}

        .magma-agent-role-user {{
            color: {accent};
        }}

        .magma-agent-role-agent {{
            color: {sem_magenta};
        }}

        .magma-agent-role-system {{
            color: alpha({text_primary}, 0.3);
        }}

        .magma-agent-role-action {{
            color: {sem_green};
        }}

        .magma-agent-log-text {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            line-height: 1.45;
        }}

        .magma-agent-text-user {{
            color: alpha({text_primary}, 0.86);
        }}

        .magma-agent-text-agent {{
            color: {sem_magenta};
        }}

        .magma-agent-text-system {{
            color: alpha({text_primary}, 0.35);
        }}

        .magma-agent-text-action {{
            color: {sem_green};
        }}

        .magma-agent-console {{
            margin: 10px;
            padding: 10px;
            border: 1px solid alpha({text_primary}, 0.06);
            border-radius: 8px;
            background: alpha({text_primary}, 0.02);
        }}

        .magma-agent-console-title {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            color: {text_primary};
            text-transform: uppercase;
            letter-spacing: 1px;
        }}

        .magma-agent-console-body {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: alpha({text_primary}, 0.5);
            line-height: 1.45;
        }}

        .magma-agent-pending {{
            padding: 10px;
            margin: 10px 10px 0 10px;
            border: 1px solid alpha({accent}, 0.14);
            border-radius: 8px;
            background: alpha({accent}, 0.04);
        }}

        .magma-agent-pending-label {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 600;
            color: {accent};
            margin-bottom: 4px;
        }}

        .magma-agent-pending-text {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: alpha({text_primary}, 0.65);
            margin-bottom: 6px;
        }}

        .magma-agent-pending-actions {{
            margin-top: 2px;
        }}

        .magma-agent-confirm {{
            background: alpha({accent}, 0.1);
            color: {accent};
            border: 1px solid alpha({accent}, 0.16);
            border-radius: 6px;
            padding: 4px 16px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 600;
            box-shadow: none;
            min-height: 0;
        }}

        .magma-agent-confirm:hover {{
            background: alpha({accent}, 0.16);
            color: {sem_orange};
        }}

        .magma-agent-reject {{
            background: alpha({text_primary}, 0.03);
            color: alpha({text_primary}, 0.40);
            border: 1px solid alpha({text_primary}, 0.06);
            border-radius: 6px;
            padding: 4px 16px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            box-shadow: none;
            min-height: 0;
        }}

        .magma-agent-reject:hover {{
            background: alpha({accent}, 0.06);
            color: alpha({sem_orange}, 0.70);
            border-color: alpha({accent}, 0.12);
        }}

        .magma-agent-prompt {{
            padding: 10px;
            border-top: 1px solid alpha({text_primary}, 0.04);
            background: alpha({text_primary}, 0.04);
        }}

        .magma-agent-prompt-glyph {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            color: {accent};
            padding: 0 2px 0 0;
        }}

        .magma-agent-prompt-input {{
            background: {surface};
            color: {text_primary};
            border: 1px solid alpha({text_primary}, 0.07);
            border-radius: 6px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            padding: 6px 10px;
            box-shadow: none;
            min-height: 0;
        }}

        .magma-agent-prompt-input:focus {{
            border-color: alpha({accent}, 0.25);
            background: alpha({window_bg}, 0.88);
        }}

        .magma-agent-prompt-send {{
            background: alpha({text_primary}, 0.04);
            color: alpha({text_primary}, 0.72);
            border: 1px solid alpha({text_primary}, 0.08);
            border-radius: 6px;
            padding: 4px 14px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 600;
            box-shadow: none;
            min-height: 0;
        }}

        .magma-agent-prompt-send:hover {{
            background: alpha({text_primary}, 0.08);
            color: {text_primary};
        }}

        /* ─── Notes pane ─────────────────────────────────────── */

        .magma-notes-pane {{
            background: {window_bg};
            border-left: 1px solid {border};
            min-width: 392px;
        }}

        .magma-notes-header {{
            padding: 16px 14px;
            border-bottom: 1px solid {border};
            background: {window_bg};
        }}

        .magma-notes-title-stack {{
            padding: 0;
        }}

        .magma-notes-title {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            font-weight: 800;
            color: {text_primary};
            text-transform: uppercase;
            letter-spacing: 1.2px;
        }}

        .magma-notes-subtitle {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: {text_dim};
            margin-top: 2px;
        }}

        .magma-notes-count {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: {text_secondary};
            background: {surface};
            border: 1px solid {border};
            border-radius: 999px;
            padding: 4px 10px;
        }}

        .magma-notes-new {{
            background: {surface};
            color: {text_primary};
            border: 1px solid {border};
            border-radius: 6px;
            padding: 6px 12px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 600;
            box-shadow: none;
            min-height: 0;
            text-transform: uppercase;
            letter-spacing: 0.6px;
            transition: all 120ms ease;
        }}

        .magma-notes-new:hover {{
            background: alpha({text_primary}, 0.07);
            border-color: alpha({text_primary}, 0.15);
            color: {text_primary};
        }}

        .magma-notes-board {{
            background: {surface};
            color: {text_primary};
            border: 1px solid {border};
            border-radius: 6px;
            padding: 6px 12px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 600;
            box-shadow: none;
            min-height: 0;
            text-transform: uppercase;
            letter-spacing: 0.6px;
            transition: all 120ms ease;
        }}

        .magma-notes-board:hover {{
            background: alpha({text_primary}, 0.07);
            border-color: alpha({text_primary}, 0.15);
            color: {text_primary};
        }}

        .magma-notes-recovery {{
            background: {surface};
            border-bottom: 1px solid {border};
            padding: 10px 14px;
        }}

        .magma-notes-recovery-label {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: {text_secondary};
        }}

        .magma-notes-recovery-continue,
        .magma-notes-recovery-discard {{
            background: {surface};
            color: {text_primary};
            border: 1px solid {border};
            border-radius: 999px;
            padding: 4px 10px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            box-shadow: none;
            min-height: 0;
            text-transform: uppercase;
            transition: all 120ms ease;
        }}

        .magma-notes-recovery-continue:hover,
        .magma-notes-recovery-discard:hover {{
            background: alpha({text_primary}, 0.07);
            border-color: alpha({text_primary}, 0.15);
            color: {text_primary};
        }}

        .magma-notes-search {{
            background: {surface};
            color: {text_primary};
            border: 1px solid {border};
            border-radius: 8px;
            margin: 12px 14px 10px 14px;
            padding: 10px 12px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            box-shadow: none;
            min-height: 0;
            transition: all 120ms ease;
        }}

        .magma-notes-search:focus {{
            border-color: alpha({text_primary}, 0.15);
            background: {surface};
        }}

        .magma-notes-list-area,
        .magma-notes-editor-area {{
            background: {window_bg};
        }}

        .magma-notes-list-scroll,
        .magma-notes-editor-scroll,
        .magma-notes-media-scroll {{
            background: {window_bg};
            border: none;
        }}

        .magma-notes-tiles {{
            background: transparent;
            padding: 0 14px 16px 14px;
        }}

        .magma-notes-tile-row {{
            background: transparent;
        }}

        .magma-notes-list-empty {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: {text_dim};
            padding: 20px 14px;
        }}

        .magma-notes-tile {{
            background: {surface};
            border: 1px solid {border};
            border-radius: 10px;
            padding: 12px;
            transition: all 120ms ease;
        }}

        .magma-notes-tile:hover {{
            background: alpha({text_primary}, 0.04);
            border-color: alpha({text_primary}, 0.15);
        }}

        .magma-notes-tile-compact {{
            min-height: 160px;
        }}

        .magma-notes-tile-tall {{
            min-height: 200px;
        }}

        .magma-notes-tile-wide {{
            min-height: 180px;
        }}

        .magma-notes-tile-feature {{
            min-height: 220px;
        }}

        .magma-notes-tile-top {{
            background: transparent;
            padding: 0;
        }}

        .magma-notes-tile-menu {{
            background: transparent;
            color: {text_primary};
            border: none;
            box-shadow: none;
            padding: 0;
            min-height: 20px;
            min-width: 20px;
            transition: all 120ms ease;
        }}

        .magma-notes-tile-menu:hover {{
            color: {text_primary};
        }}

        .magma-notes-menu-popover {{
            background: transparent;
        }}

        .magma-notes-menu-panel,
        .magma-notes-emoji-panel {{
            background: {surface};
            border: 1px solid {border};
            border-radius: 8px;
            padding: 8px;
        }}

        .magma-notes-menu-item {{
            background: transparent;
            color: {text_primary};
            border: 1px solid transparent;
            border-radius: 6px;
            box-shadow: none;
            min-height: 0;
            padding: 6px 10px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            text-transform: uppercase;
            transition: all 120ms ease;
        }}

        .magma-notes-menu-item:hover {{
            background: alpha({text_primary}, 0.07);
            border-color: alpha({text_primary}, 0.15);
            color: {text_primary};
        }}

        .magma-notes-menu-delete {{
            color: {sem_orange};
        }}

        .magma-notes-menu-delete:hover {{
            color: {sem_orange};
            background: alpha({sem_orange}, 0.08);
            border-color: alpha({sem_orange}, 0.28);
        }}

        .magma-notes-row-title {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            color: {text_primary};
        }}

        .magma-notes-row-preview {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: {text_secondary};
            margin-top: 4px;
            line-height: 1.4;
        }}

        .magma-notes-row-meta {{
            margin-top: 8px;
        }}

        .magma-notes-row-time {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: {text_dim};
        }}

        .magma-notes-row-pin {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: {sem_green};
            font-weight: 600;
            background: {surface};
            border: 1px solid {border};
            border-radius: 999px;
            padding: 2px 8px;
        }}

        .magma-notes-row-media {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: {text_primary};
            background: {surface};
            border: 1px solid {border};
            border-radius: 999px;
            padding: 2px 8px;
        }}

        .magma-notes-editor-area {{
            border-top: 1px solid {border};
            padding: 14px 14px 16px 14px;
        }}

        .magma-notes-editor-title {{
            background: {window_bg};
            color: {text_primary};
            border: none;
            border-bottom: 1px solid {border};
            border-radius: 0;
            padding: 0 0 12px 0;
            margin-bottom: 12px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_12};
            font-weight: 800;
            box-shadow: none;
            min-height: 0;
            transition: all 120ms ease;
        }}

        .magma-notes-editor-title:focus {{
            background: {window_bg};
            border-bottom-color: alpha({text_primary}, 0.15);
        }}

        .magma-notes-editor-body {{
            background: {window_bg};
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            caret-color: {text_primary};
            line-height: 1.5;
        }}

        .magma-notes-media-strip {{
            background: transparent;
            padding: 10px 0 4px 0;
        }}

        .magma-notes-media-card {{
            background: {surface};
            border: 1px solid {border};
            border-radius: 8px;
            padding: 6px;
            transition: all 120ms ease;
        }}

        .magma-notes-media-card:hover {{
            border-color: alpha({text_primary}, 0.15);
        }}

        .magma-notes-media-picture {{
            background: {window_bg};
            border-radius: 6px;
        }}

        .magma-notes-media-remove {{
            background: alpha({sem_orange}, 0.08);
            color: {sem_orange};
            border: 1px solid alpha({sem_orange}, 0.28);
            border-radius: 999px;
            padding: 2px 8px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            box-shadow: none;
            min-height: 0;
            text-transform: uppercase;
            transition: all 120ms ease;
        }}

        .magma-notes-media-remove:hover {{
            background: alpha({sem_orange}, 0.12);
            color: {sem_orange};
            border-color: alpha({sem_orange}, 0.44);
        }}

        .magma-notes-editor-actions {{
            padding: 12px 14px 0 14px;
        }}

        .magma-notes-editor-meta {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: {text_dim};
        }}

        .magma-notes-tool-scroll {{
            background: transparent;
            border: none;
            margin: 0 10px;
            min-height: 28px;
        }}

        .magma-notes-tool-scroll scrollbar {{
            background: transparent;
            border: none;
            min-height: 10px;
        }}

        .magma-notes-tool-scroll scrollbar.horizontal {{
            margin: 4px 8px 0 8px;
        }}

        .magma-notes-tool-scroll scrollbar slider {{
            background: alpha({accent}, 0.88);
            border: 1px solid alpha({sem_orange}, 0.44);
            border-radius: 999px;
            min-height: 3px;
            min-width: 16px;
        }}

        .magma-notes-tool-scroll scrollbar slider:hover {{
            background: alpha({sem_orange}, 0.96);
            border-color: alpha({sem_orange}, 0.6);
        }}

        .magma-notes-tool-strip,
        .magma-notes-alignment-row,
        .magma-notes-list-row {{
            background: transparent;
        }}

        .magma-notes-tool-pill {{
            background: {window_bg};
            border: 1px solid alpha({sem_orange}, 0.28);
            border-radius: 999px;
            padding: 4px 0 6px 0;
            transition: all 120ms ease;
        }}

        .magma-notes-tool-pill:hover {{
            background: {window_bg};
            border-color: alpha({sem_orange}, 0.28);
        }}

        .magma-notes-tool {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 999px;
            padding: 3px 6px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            box-shadow: none;
            min-height: 0;
            min-width: 24px;
            text-transform: uppercase;
            transition: all 120ms ease;
        }}

        .magma-notes-tool:hover {{
            background: transparent;
            color: {text_primary};
        }}

        .magma-notes-tool-icon {{
            min-width: 22px;
            padding: 3px 5px;
        }}

        .magma-notes-emoji-item {{
            background: {surface};
            color: {text_primary};
            border: 1px solid {border};
            border-radius: 6px;
            box-shadow: none;
            min-height: 0;
            min-width: 28px;
            padding: 4px 6px;
            transition: all 120ms ease;
        }}

        .magma-notes-emoji-item:hover {{
            background: alpha({text_primary}, 0.07);
            border-color: alpha({text_primary}, 0.15);
            color: {text_primary};
        }}
        ",
        window_bg = css_color(palette.bg_primary),
        sem_yellow = css_color(palette.sem_yellow),
        sem_green = css_color(palette.sem_green),
        sem_orange = css_color(palette.sem_orange),
        sem_blue = css_color(palette.sem_blue),
        sem_magenta = css_color(palette.sem_magenta),
        sem_brown = css_color(palette.sem_brown),
        window_edge = css_color(palette.window_edge),
        titlebar_bg = css_color(palette.bg_titlebar),
        surface = css_color(palette.surface_base),
        border = css_color(palette.border_strong),
        text_primary = css_color(palette.text_primary),
        text_secondary = css_color(palette.text_secondary),
        text_dim = css_color(palette.text_dim),
        accent = css_color(palette.accent),
        folder_title_size = px(9.0, ui_scale),
        folder_name_size = px(11.0, ui_scale),
        font_9 = px(9.0, ui_scale),
        font_10 = px(10.0, ui_scale),
        font_11 = px(11.0, ui_scale),
        font_12 = px(12.0, ui_scale),
        font_13 = px(13.0, ui_scale),
        font_22 = px(22.0, ui_scale),
        terminal_radius = px(14.0, ui_scale),
        terminal_padding = px(8.0, ui_scale),
        bar_radius = px(12.0, ui_scale),
        bar_padding = px(4.0, ui_scale),
        mux_btn_size = px(22.0, ui_scale),
        mux_btn_padding = px(8.0, ui_scale),
        split_width = px(8.0, ui_scale),
        split_margin = px(3.0, ui_scale),
    );
    // Post-process alpha(#HEX, factor) -> rgba(R, G, B, factor) for GTK4 compat
    let css = resolve_alpha(&css);
    provider.load_from_data(&css);

    if let Some(gtk_settings) = gtk::Settings::default() {
        gtk_settings.set_gtk_application_prefer_dark_theme(settings.theme_mode.prefers_dark_gtk());
    }

    if let Some(display) = gdk::Display::default() {
        style_context_add_provider_for_display(
            &display,
            &provider,
            STYLE_PROVIDER_PRIORITY_USER,
        );
    }
}

fn css_color(color: u32) -> String {
    format!("#{:06X}", color & 0x00FF_FFFF)
}

fn ui_scale(app_font_size: u32) -> f32 {
    app_font_size as f32 / 11.0
}

fn px(base: f32, scale: f32) -> String {
    format!("{:.1}px", base * scale)
}

/// Convert every `alpha(#RRGGBB, factor)` token in the CSS string into
/// `rgba(R, G, B, factor)` so that GTK4's CSS parser can resolve them.
/// GTK4 does not reliably support `alpha()` with hex colors in all contexts
/// (e.g. `box-shadow`, `border-color`), so we pre-resolve them here.
fn resolve_alpha(css: &str) -> String {
    let mut result = String::with_capacity(css.len());
    let mut remaining = css;
    while let Some(pos) = remaining.find("alpha(") {
        result.push_str(&remaining[..pos]);
        let after_alpha = &remaining[pos + 6..]; // skip "alpha("
        if let Some(end) = after_alpha.find(')') {
            let inner = &after_alpha[..end];
            if let Some(resolved) = try_resolve_alpha_inner(inner) {
                result.push_str(&resolved);
                remaining = &after_alpha[end + 1..];
                continue;
            }
        }
        // Could not parse — emit verbatim and advance past "alpha("
        result.push_str("alpha(");
        remaining = after_alpha;
    }
    result.push_str(remaining);
    result
}

/// Parse the interior of `alpha(#RRGGBB, factor)` and return `rgba(R,G,B,factor)`.
fn try_resolve_alpha_inner(inner: &str) -> Option<String> {
    let mut parts = inner.splitn(2, ',');
    let color_str = parts.next()?.trim();
    let factor_str = parts.next()?.trim();
    if !color_str.starts_with('#') || color_str.len() < 7 {
        return None;
    }
    let hex = u32::from_str_radix(&color_str[1..7], 16).ok()?;
    let r = (hex >> 16) & 0xFF;
    let g = (hex >> 8) & 0xFF;
    let b = hex & 0xFF;
    Some(format!("rgba({r}, {g}, {b}, {factor_str})"))
}
