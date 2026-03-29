use gtk::{
    gdk, style_context_add_provider_for_display, CssProvider, STYLE_PROVIDER_PRIORITY_USER,
};

use crate::ui::theme;

pub(super) fn install_css(app_font_size: u32) {
    let provider = CssProvider::new();
    let ui_scale = ui_scale(app_font_size);
    let css = format!(
        "
        window.magma-window {{
            background: {window_bg};
            border: 1px solid {window_edge};
            border-radius: 12px;
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
            transition: background 140ms ease;
            -gtk-icon-size: 0px;
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
            background: #FF5F56;
        }}

        headerbar.magma-header windowcontrols button.minimize {{
            background: #FFBD2E;
        }}

        headerbar.magma-header windowcontrols button.maximize {{
            background: #27C93F;
        }}

        headerbar.magma-header windowcontrols button.close:hover {{
            background: #FF3B30;
        }}

        headerbar.magma-header windowcontrols button.minimize:hover {{
            background: #E5A323;
        }}

        headerbar.magma-header windowcontrols button.maximize:hover {{
            background: #1CAD30;
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
            background: #FF5F56;
        }}

        headerbar.magma-header button.titlebutton.minimize {{
            background: #FFBD2E;
        }}

        headerbar.magma-header button.titlebutton.maximize {{
            background: #27C93F;
        }}

        headerbar.magma-header button.titlebutton.close:hover {{
            background: #FF3B30;
        }}

        headerbar.magma-header button.titlebutton.minimize:hover {{
            background: #E5A323;
        }}

        headerbar.magma-header button.titlebutton.maximize:hover {{
            background: #1CAD30;
        }}

        button.magma-header-settings {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 50%;
            min-height: 24px;
            min-width: 24px;
            padding: 4px;
            box-shadow: none;
            opacity: 0.4;
            transition: opacity 140ms ease;
        }}

        button.magma-header-settings:hover {{
            opacity: 1.0;
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
            border: 1px solid {border};
            border-radius: 18px;
            padding: 10px;
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
            background: rgba(255, 255, 255, 0.04);
            min-width: 10px;
            margin: 0 4px;
            border-radius: 999px;
            transition: background 140ms ease;
        }}

        paned.magma-split-pane > separator:hover {{
            background: rgba(255, 77, 77, 0.18);
        }}

        box.magma-mux-root {{
            background: transparent;
        }}

        box.magma-mux-bar {{
            background: rgba(255, 255, 255, 0.02);
            border: 1px solid rgba(255, 255, 255, 0.05);
            border-radius: 999px;
            padding: 4px;
            margin: 0 0 4px 0;
        }}

        button.magma-mux-session,
        button.magma-mux-action {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 999px;
            min-height: 22px;
            min-width: 22px;
            padding: 0 10px;
            box-shadow: none;
            opacity: 0.6;
            transition: opacity 140ms ease, background 140ms ease, color 140ms ease;
        }}

        button.magma-mux-session:hover,
        button.magma-mux-action:hover {{
            opacity: 1.0;
            background: rgba(255, 255, 255, 0.04);
        }}

        button.magma-mux-session.active {{
            opacity: 1.0;
            background: rgba(255, 77, 77, 0.1);
            color: {accent};
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
            border: 1px solid rgba(255, 255, 255, 0.04);
            border-radius: 12px;
            background: rgba(255, 255, 255, 0.015);
            transition: border-color 180ms ease, background 180ms ease, opacity 180ms ease;
        }}

        .magma-handle:hover {{
            background: rgba(255, 255, 255, 0.025);
            border-color: rgba(255, 255, 255, 0.08);
        }}

        .magma-handle.collapsed {{
            background: rgba(255, 255, 255, 0.01);
        }}

        .magma-handle.collapsed:hover {{
            background: rgba(255, 77, 77, 0.05);
            border-color: rgba(255, 77, 77, 0.18);
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
            transition: opacity 180ms ease, background 180ms ease, color 180ms ease;
        }}

        button.magma-handle-segment:hover {{
            opacity: 0.86;
            background: rgba(255, 255, 255, 0.035);
        }}

        button.magma-handle-segment.active {{
            opacity: 1.0;
            background: rgba(255, 77, 77, 0.08);
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
            background: rgba(255, 77, 77, 0.035);
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
            border: 1px solid rgba(255, 255, 255, 0.04);
            border-radius: 12px;
            background: rgba(255, 255, 255, 0.015);
            transition: border-color 180ms ease, background 180ms ease, opacity 180ms ease;
        }}

        .magma-left-handle:hover {{
            background: rgba(255, 255, 255, 0.025);
            border-color: rgba(255, 255, 255, 0.08);
        }}

        .magma-left-handle.collapsed {{
            background: rgba(255, 255, 255, 0.01);
        }}

        .magma-left-handle.collapsed:hover {{
            background: rgba(255, 77, 77, 0.05);
            border-color: rgba(255, 77, 77, 0.18);
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
            background: rgba(255, 255, 255, 0.035);
        }}

        button.magma-left-handle-segment.active {{
            opacity: 1.0;
            background: rgba(255, 77, 77, 0.08);
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
            background: rgba(255, 77, 77, 0.035);
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
            transition: opacity 140ms ease, background 140ms ease;
        }}

        button.magma-folder-action:hover {{
            opacity: 1.0;
            background: rgba(255, 255, 255, 0.04);
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
            transition: background 120ms ease;
        }}

        button.magma-folder-item:hover {{
            background: rgba(255, 255, 255, 0.04);
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
        .ft-source   {{ color: #E8C87A; }}
        .ft-test     {{ color: #7FB685; }}
        .ft-build    {{ color: #B0896E; }}
        .ft-dep      {{ color: {text_dim}; }}
        .ft-doc      {{ color: #8AB4D6; }}
        .ft-asset    {{ color: #C79DD6; }}
        .ft-git      {{ color: #E87070; }}
        .ft-config   {{ color: #B0896E; }}
        .ft-secret   {{ color: #E87070; opacity: 0.85; }}
        .ft-lock     {{ color: {text_dim}; }}

        .ft-rust     {{ color: #E87050; }}
        .ft-js       {{ color: #E8C87A; }}
        .ft-ts       {{ color: #5B9BD5; }}
        .ft-html     {{ color: #E87050; }}
        .ft-css      {{ color: #8AB4D6; }}
        .ft-json     {{ color: #B0896E; }}
        .ft-python   {{ color: #7FB685; }}
        .ft-go       {{ color: #6CC7D6; }}
        .ft-c        {{ color: #8AB4D6; }}
        .ft-java     {{ color: #E87050; }}
        .ft-shell    {{ color: #7FB685; }}
        .ft-image    {{ color: #C79DD6; }}
        .ft-archive  {{ color: #B0896E; }}
        .ft-binary   {{ color: {text_dim}; }}

        /* ── Git status dots ── */

        .magma-git-dot {{
            font-size: 8px;
            margin: 0 2px 0 4px;
            opacity: 0.9;
        }}

        .git-modified  {{ color: #E8C87A; }}
        .git-staged    {{ color: #7FB685; }}
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
            background: transparent;
            border: 1px solid {border};
            border-radius: 999px;
            padding: 4px 16px;
            margin: 0 0 8px 0;
            transition: border-color 140ms ease;
        }}

        box.magma-input-pill:focus-within {{
            border-color: {accent};
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
            transition: opacity 140ms ease, background 140ms ease;
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
            background: rgba(0, 0, 0, 0.95);
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
            background: rgba(255, 255, 255, 0.018);
            border: 1px solid rgba(255, 255, 255, 0.05);
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
            background: rgba(0, 0, 0, 0.94);
            border: 1px solid {border};
            border-radius: 14px;
            padding: 12px;
            min-width: 360px;
        }}

        entry.magma-switcher-entry {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-bottom: 1px solid rgba(255, 255, 255, 0.08);
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
            background: rgba(255, 255, 255, 0.05);
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
            padding: 0 4px;
        }}

        .magma-tab-item {{
            background: transparent;
            border-radius: 0;
            padding: 6px 14px;
            margin: 0 2px;
            transition: background 140ms ease, opacity 140ms ease, border-color 140ms ease;
            border-bottom: 2px solid transparent;
            opacity: 0.4;
        }}

        .magma-tab-item:hover {{
            background: rgba(255, 255, 255, 0.03);
            opacity: 0.8;
        }}

        .magma-tab-item.active {{
            background: transparent;
            border-bottom-color: {accent};
            opacity: 1.0;
        }}

        button.magma-tab-close-button {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 50%;
            min-height: 20px;
            min-width: 20px;
            padding: 0;
            margin-left: 2px;
            opacity: 0;
            box-shadow: none;
            transition: opacity 140ms ease, background 140ms ease;
        }}

        .magma-tab-item:hover button.magma-tab-close-button,
        .magma-tab-item.active button.magma-tab-close-button {{
            opacity: 0.4;
        }}

        button.magma-tab-close-button:hover {{
            background: rgba(255, 255, 255, 0.1);
            opacity: 1.0;
        }}

        .magma-tab-item.dragging {{
            opacity: 0.4;
        }}

        .magma-tab-item.drop-target {{
            background: rgba(255, 255, 255, 0.05);
            border-bottom-color: {accent};
        }}

        button.magma-add-tab-button {{
            background: transparent;
            color: {text_primary};
            border: none;
            padding: 6px;
            min-height: 28px;
            min-width: 28px;
            opacity: 0.5;
            margin: 0 8px;
            border-radius: 50%;
            transition: opacity 140ms ease, background 140ms ease;
        }}

        button.magma-add-tab-button:hover {{
            opacity: 1.0;
            background: rgba(255, 255, 255, 0.05);
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
            border-bottom: 1px solid rgba(255, 77, 77, 0.35);
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
            color: #FFBD2E;
        }}

        box.magma-input-pill.search-active {{
            border-color: rgba(255, 189, 46, 0.3);
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
            color: #7FB685;
            opacity: 0.7;
        }}

        label.magma-notice-error {{
            color: {accent};
            opacity: 0.9;
        }}

        label.magma-status-ok {{
            color: #7FB685;
        }}

        label.magma-status-error {{
            color: {accent};
        }}

        label.magma-status-running {{
            color: #E5C07B;
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
            border-bottom: 1px solid rgba(255, 255, 255, 0.035);
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
            border: 1px solid rgba(255, 255, 255, 0.05);
            border-radius: 8px;
            min-height: 28px;
            min-width: 28px;
            padding: 4px;
            box-shadow: none;
            opacity: 0.5;
            transition: opacity 140ms ease, background 140ms ease, border-color 140ms ease;
        }}

        button.magma-view-header-action:hover {{
            background: rgba(255, 255, 255, 0.025);
            border-color: rgba(255, 255, 255, 0.09);
            opacity: 1.0;
        }}

        button.magma-view-action,
        button.magma-view-open {{
            background: rgba(255, 255, 255, 0.02);
            color: {text_primary};
            border: 1px solid rgba(255, 255, 255, 0.04);
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
            background: rgba(255, 77, 77, 0.05);
            border-color: rgba(255, 77, 77, 0.12);
        }}

        .magma-view-scope {{
            padding: 0 8px 4px 8px;
        }}

        .magma-view-scope-chip {{
            background: rgba(255, 255, 255, 0.03);
            color: rgba(255, 255, 255, 0.40);
            border: 1px solid rgba(255, 255, 255, 0.05);
            border-radius: 10px;
            padding: 1px 10px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            box-shadow: none;
            min-height: 0;
            transition: background 120ms ease, color 120ms ease, border-color 120ms ease;
        }}

        .magma-view-scope-chip:hover {{
            color: rgba(255, 255, 255, 0.65);
            border-color: rgba(255, 255, 255, 0.10);
            background: rgba(255, 255, 255, 0.05);
        }}

        .magma-view-scope-chip-active {{
            background: rgba(255, 77, 77, 0.08);
            color: {accent};
            border-color: rgba(255, 77, 77, 0.16);
        }}

        .magma-view-scope-chip-active:hover {{
            background: rgba(255, 77, 77, 0.12);
            color: {accent};
        }}

        .magma-view-file-scroller {{
            margin-bottom: 8px;
            background: rgba(255, 255, 255, 0.012);
            border: 1px solid rgba(255, 255, 255, 0.035);
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
            background: rgba(255, 77, 77, 0.03);
        }}

        row.magma-view-file-row:selected {{
            background: rgba(255, 77, 77, 0.06);
            border: 1px solid rgba(255, 77, 77, 0.12);
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
            background: rgba(4, 4, 5, 0.98);
            border: 1px solid rgba(255, 255, 255, 0.04);
            border-radius: 16px;
            padding: 0;
        }}

        .magma-view-preview-chrome {{
            padding: 14px 16px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.045);
            background: rgba(10, 10, 12, 0.98);
        }}

        .magma-view-preview-actions {{
            margin-left: 12px;
            background: rgba(255, 255, 255, 0.02);
            border: 1px solid rgba(255, 255, 255, 0.04);
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
            background: rgba(255, 255, 255, 0.03);
            border-color: rgba(255, 255, 255, 0.05);
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
            background: rgba(255, 255, 255, 0.035);
            border-color: rgba(255, 255, 255, 0.06);
            opacity: 0.92;
        }}

        button.magma-view-preview-button-primary:hover {{
            background: rgba(255, 255, 255, 0.05);
            border-color: rgba(255, 255, 255, 0.09);
        }}

        .magma-view-preview-stack {{
            background: transparent;
        }}

        .magma-view-preview-surface,
        .magma-view-code-scroller {{
            background: rgba(3, 3, 4, 1);
            border: none;
        }}

        textview.magma-view-code {{
            background: rgba(3, 3, 4, 1);
            color: {text_primary};
        }}

        .magma-view-empty-state {{
            padding: 42px 24px;
            background: rgba(3, 3, 4, 1);
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
            background: rgba(3, 3, 4, 1);
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
            background: rgba(255, 255, 255, 0.03);
        }}

        .magma-web-tab.active {{
            opacity: 1.0;
            background: rgba(255, 255, 255, 0.04);
            border-color: rgba(255, 255, 255, 0.06);
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
            background: rgba(255, 77, 77, 0.15);
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
            color: #27C93F;
            opacity: 0.7;
        }}

        .magma-web-ssl.insecure {{
            color: #FFBD2E;
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
            background: rgba(255, 255, 255, 0.02);
            border: 1px solid rgba(255, 255, 255, 0.06);
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
            background: rgba(255, 255, 255, 0.02);
            border: 1px solid rgba(255, 255, 255, 0.06);
            border-radius: 16px;
            padding: 6px;
        }}

        .magma-web-nav {{
            background: rgba(255, 255, 255, 0.02);
            border: 1px solid rgba(255, 255, 255, 0.04);
            border-radius: 12px;
            padding: 2px;
            margin-right: 6px;
        }}

        .magma-web-address-shell {{
            background: rgba(0, 0, 0, 0.22);
            border: 1px solid rgba(255, 255, 255, 0.04);
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
            background: rgba(255, 255, 255, 0.06);
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
            background: rgba(255, 255, 255, 0.01);
            border: 1px solid rgba(255, 255, 255, 0.05);
            border-radius: 16px;
            padding: 0;
        }}

        .magma-webview {{
            background: rgba(0, 0, 0, 0.38);
            border: none;
            border-radius: 16px;
            margin-top: 0;
        }}

        .magma-logr-header {{
            padding: 4px 0 12px 0;
            margin-bottom: 2px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.035);
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
            transition: opacity 140ms ease, background 140ms ease, border-color 140ms ease, transform 140ms ease;
        }}

        .magma-logr-tab:hover {{
            opacity: 0.88;
            background: rgba(255, 255, 255, 0.02);
            border-color: rgba(255, 255, 255, 0.05);
        }}

        .magma-logr-tab.active {{
            opacity: 1.0;
            background: rgba(255, 77, 77, 0.035);
            border-color: rgba(255, 77, 77, 0.12);
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
            background: rgba(255, 255, 255, 0.05);
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
            background: rgba(255, 255, 255, 0.018);
            border: 1px solid rgba(255, 255, 255, 0.04);
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
            border-color: rgba(255, 77, 77, 0.14);
            background: rgba(255, 77, 77, 0.045);
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
            background: rgba(255, 255, 255, 0.05);
        }}

        row.magma-logr-popover-row:focus {{
            background: rgba(255, 77, 77, 0.10);
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
            background: rgba(255, 255, 255, 0.02);
            color: {text_primary};
            border: 1px solid rgba(255, 255, 255, 0.04);
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
            background: rgba(255, 77, 77, 0.05);
            border-color: rgba(255, 77, 77, 0.12);
        }}

        .magma-logr-controls {{
            background: rgba(255, 255, 255, 0.012);
            border: 1px solid rgba(255, 255, 255, 0.035);
            border-radius: 999px;
            padding: 4px 6px;
            margin-bottom: 6px;
        }}

        .magma-logr-stream-shell {{
            background: rgba(255, 255, 255, 0.018);
            border-radius: 999px;
            padding: 2px 10px 2px 8px;
            margin: 0 6px;
        }}

        .magma-logr-stream-icon {{
            color: #27C93F;
            -gtk-icon-size: 10px;
            opacity: 0.72;
        }}

        .magma-logr-stream-label {{
            color: #B0E4B7;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.82;
        }}

        entry.magma-logr-filter {{
            background: rgba(255, 255, 255, 0.018);
            color: {text_primary};
            border: 1px solid rgba(255, 255, 255, 0.04);
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
            background: rgba(255, 77, 77, 0.035);
            border-color: rgba(255, 77, 77, 0.15);
            box-shadow: none;
            outline: none;
        }}

        .magma-logr-status {{
            color: #FFBD2E;
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
            border-bottom: 1px solid rgba(255, 255, 255, 0.015);
            border-radius: 8px;
            transition: background 160ms ease, padding 160ms ease, border-color 160ms ease;
        }}

        .magma-log-entry:hover {{
            background: rgba(255, 77, 77, 0.03);
            border-bottom-color: rgba(255, 77, 77, 0.08);
        }}

        .magma-log-entry.expanded {{
            background: rgba(255, 77, 77, 0.04);
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
            background: rgba(255, 255, 255, 0.015);
            border: 1px solid rgba(255, 255, 255, 0.04);
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
            background: rgba(255, 255, 255, 0.08);
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
            background: rgba(255, 77, 77, 0.15);
            color: #FF4D4D;
        }}

        /* Log Level Colors */
        .log-error .log-level-dot {{ color: #FF5F56; background: rgba(255, 95, 86, 0.12); }}
        .log-error .log-body {{ color: #FF8079; opacity: 0.9; }}

        .log-warn .log-level-dot {{ color: #FFBD2E; background: rgba(255, 189, 46, 0.12); }}
        .log-warn .log-body {{ color: #FFD272; opacity: 0.85; }}

        .log-info .log-level-dot {{ color: #27C93F; background: rgba(39, 201, 63, 0.12); }}
        .log-info .log-body {{ color: #B0E4B7; opacity: 0.8; }}

        .log-debug .log-level-dot {{ color: {text_primary}; opacity: 0.3; background: rgba(255, 255, 255, 0.04); }}
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
            background: rgba(0, 0, 0, 0.78);
            border: 1px solid rgba(255, 255, 255, 0.08);
            border-radius: 22px;
            box-shadow: 0 28px 72px rgba(0, 0, 0, 0.42);
        }}

        .magma-setup-topbar {{
            padding: 12px 16px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.06);
            background: rgba(255, 255, 255, 0.015);
        }}

        .magma-setup-dot {{
            font-size: {font_9};
            opacity: 0.9;
        }}

        .magma-setup-dot.red {{
            color: rgba(255, 95, 86, 0.9);
        }}

        .magma-setup-dot.amber {{
            color: rgba(255, 189, 46, 0.9);
        }}

        .magma-setup-dot.green {{
            color: rgba(39, 201, 63, 0.9);
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
            border-bottom: 1px solid rgba(255, 255, 255, 0.05);
        }}

        .magma-setup-step {{
            background: transparent;
            border: none;
            border-bottom: 2px solid transparent;
            border-radius: 0;
            padding: 0 0 10px 0;
        }}

        .magma-setup-step.active {{
            border-bottom-color: rgba(255, 77, 77, 0.72);
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
            background: rgba(255, 255, 255, 0.018);
            border: 1px solid rgba(255, 255, 255, 0.055);
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
            padding: 4px 24px 14px 0;
            margin-bottom: 4px;
        }}

        button.magma-settings-back {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 4px;
            min-height: 28px;
            min-width: 28px;
            padding: 4px;
            box-shadow: none;
            opacity: 0.4;
            transition: opacity 140ms ease;
        }}

        button.magma-settings-back:hover {{
            opacity: 1.0;
        }}

        .magma-settings-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_18};
            font-weight: 700;
            text-transform: lowercase;
            letter-spacing: 0.04em;
            opacity: 0.95;
            margin-top: 4px;
        }}

        .magma-settings-content {{
            padding: 4px 18px 24px 0;
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
            border-bottom: 1px solid rgba(255, 77, 77, 0.12);
            margin-bottom: 12px;
        }}

        .magma-settings-row {{
            background: transparent;
            border: none;
            border-bottom: 1px solid rgba(255, 255, 255, 0.025);
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
            background: rgba(255, 255, 255, 0.015);
            color: {text_primary};
            border: 1px solid rgba(255, 255, 255, 0.04);
            border-radius: 8px;
            padding: 6px 10px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            box-shadow: none;
            outline: none;
            transition: border-color 140ms ease, background 140ms ease;
        }}

        entry.magma-settings-entry:focus {{
            background: rgba(255, 255, 255, 0.025);
            border-color: {accent};
        }}

        spinbutton.magma-settings-spin {{
            background: rgba(255, 255, 255, 0.015);
            color: {text_primary};
            border: 1px solid rgba(255, 255, 255, 0.04);
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
            background: rgba(255, 255, 255, 0.015);
            border: 1px solid rgba(255, 255, 255, 0.04);
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

        switch.magma-settings-switch {{
            background: rgba(255, 255, 255, 0.03);
            border: 1px solid rgba(255, 255, 255, 0.04);
            border-radius: 12px;
            min-width: 42px;
            min-height: 22px;
            padding: 0;
            transition: background 180ms cubic-bezier(0.4, 0, 0.2, 1), border-color 180ms ease;
        }}

        switch.magma-settings-switch:checked {{
            background: rgba(255, 77, 77, 0.08);
            border-color: rgba(255, 77, 77, 0.18);
        }}

        switch.magma-settings-switch > image {{
            color: transparent;
        }}

        switch.magma-settings-switch slider {{
            background: rgba(255, 255, 255, 0.22);
            border: none;
            border-radius: 50%;
            min-width: 14px;
            min-height: 14px;
            margin: 4px;
            transition: background 180ms cubic-bezier(0.4, 0, 0.2, 1), transform 180ms cubic-bezier(0.4, 0, 0.2, 1);
        }}

        switch.magma-settings-switch:checked slider {{
            background: {accent};
            box-shadow: 0 0 8px rgba(255, 77, 77, 0.35);
        }}

        button.magma-settings-link {{
            background: transparent;
            color: {text_primary};
            border: 1px solid rgba(255, 255, 255, 0.04);
            border-radius: 10px;
            padding: 6px 14px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            opacity: 0.6;
            transition: background 140ms ease, border-color 140ms ease, opacity 140ms ease;
        }}

        button.magma-settings-link:hover {{
            background: rgba(255, 255, 255, 0.02);
            opacity: 1.0;
            border-color: rgba(255, 255, 255, 0.1);
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
            background: rgba(255, 77, 77, 0.10);
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
            color: rgba(255, 255, 255, 0.5);
        }}

        .magma-git-remote-bar {{
            padding: 4px 0;
            border-bottom: 1px solid {border};
            margin-bottom: 4px;
        }}

        .magma-git-remote-button {{
            background: transparent;
            color: rgba(255, 255, 255, 0.6);
            border: 1px solid rgba(255, 255, 255, 0.08);
            border-radius: 4px;
            padding: 2px 10px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            box-shadow: none;
            min-height: 0;
        }}

        .magma-git-remote-button:hover {{
            color: {text_primary};
            border-color: rgba(255, 255, 255, 0.15);
        }}

        .magma-git-icon-btn {{
            background: transparent;
            color: rgba(255, 255, 255, 0.5);
            border: none;
            border-radius: 4px;
            padding: 2px 6px;
            box-shadow: none;
            min-height: 0;
            min-width: 0;
        }}

        .magma-git-icon-btn:hover {{
            color: {text_primary};
            background: rgba(255, 255, 255, 0.06);
        }}

        .magma-git-nav {{
            padding: 4px 0;
            border-bottom: 1px solid {border};
            margin-bottom: 4px;
        }}

        .magma-git-nav-button {{
            background: transparent;
            color: rgba(255, 255, 255, 0.4);
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
            color: rgba(255, 255, 255, 0.7);
        }}

        .magma-git-nav-button.active {{
            color: {accent};
            background: rgba(255, 77, 77, 0.08);
        }}

        .magma-git-status {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: rgba(255, 255, 255, 0.35);
            padding: 4px 0;
            border-top: 1px solid {border};
            margin-top: 4px;
            transition: color 200ms ease;
        }}

        .magma-git-status-ok {{
            color: #388C50;
        }}

        .magma-git-status-err {{
            color: {accent};
        }}

        .magma-git-status-busy {{
            color: #C89A1E;
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
            background: rgba(255, 255, 255, 0.03);
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            border: 1px solid rgba(255, 255, 255, 0.06);
            border-radius: 4px;
        }}

        .magma-git-commit-actions {{
            padding: 4px 0 0 0;
        }}

        .magma-git-commit-button {{
            background: {accent};
            color: #0b0b0b;
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
            background: #ff6666;
        }}

        .magma-git-commit-button:disabled {{
            background: rgba(255, 77, 77, 0.3);
            color: rgba(11, 11, 11, 0.5);
        }}

        .magma-git-action-btn {{
            background: transparent;
            color: rgba(255, 255, 255, 0.5);
            border: 1px solid rgba(255, 255, 255, 0.06);
            border-radius: 4px;
            padding: 2px 8px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            box-shadow: none;
            min-height: 0;
        }}

        .magma-git-action-btn:hover {{
            color: {text_primary};
            border-color: rgba(255, 255, 255, 0.12);
        }}

        .magma-git-file-section {{
            margin-top: 2px;
        }}

        .magma-git-file-section-header {{
            padding: 4px 2px;
        }}

        .magma-git-section-arrow {{
            font-size: {font_9};
            color: rgba(255, 255, 255, 0.3);
            min-width: 12px;
        }}

        .magma-git-section-title {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 600;
            color: rgba(255, 255, 255, 0.5);
            text-transform: uppercase;
        }}

        .magma-git-section-count {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: rgba(255, 255, 255, 0.3);
        }}

        .magma-git-file-list {{
            background: transparent;
        }}

        .magma-git-file-row-container {{
            margin: 0;
        }}

        .magma-git-file-row {{
            padding: 3px 4px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.02);
        }}

        .magma-git-file-row:hover {{
            background: rgba(255, 255, 255, 0.03);
        }}

        .magma-git-file-status {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 700;
            min-width: 16px;
            padding: 0 2px;
        }}

        .magma-git-file-status.status-added {{
            color: #388C50;
        }}

        .magma-git-file-status.status-modified {{
            color: #C89A1E;
        }}

        .magma-git-file-status.status-deleted {{
            color: {accent};
        }}

        .magma-git-file-status.status-other {{
            color: rgba(255, 255, 255, 0.4);
        }}

        .magma-git-file-status.status-renamed {{
            color: #7AA2F7;
        }}

        .magma-git-file-status.status-conflict {{
            color: {accent};
            font-weight: bold;
        }}

        .magma-git-conflict-row {{
            background: rgba(255, 77, 77, 0.08);
        }}

        .magma-git-conflict-hint {{
            font-size: {font_9};
            color: rgba(255, 255, 255, 0.35);
            font-style: italic;
        }}

        .magma-git-file-name {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: {text_primary};
        }}

        .magma-git-file-action {{
            background: transparent;
            color: rgba(255, 255, 255, 0.4);
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
            background: rgba(255, 255, 255, 0.06);
        }}

        .magma-git-file-discard {{
            background: transparent;
            color: rgba(255, 77, 77, 0.5);
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
            background: rgba(255, 77, 77, 0.1);
        }}

        /* ─── Diff ─────────────────────────────────────────────── */

        .magma-git-diff-root {{
            padding: 4px 0 4px 16px;
        }}

        .magma-git-hunk-header {{
            padding: 2px 4px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.04);
        }}

        .magma-git-hunk-header-text {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: rgba(255, 255, 255, 0.35);
        }}

        .magma-git-diff-line {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            padding: 0 4px;
        }}

        .magma-git-line-added {{
            color: #388C50;
            background: rgba(56, 140, 80, 0.08);
        }}

        .magma-git-line-removed {{
            color: {accent};
            background: rgba(255, 77, 77, 0.08);
        }}

        .magma-git-line-context {{
            color: rgba(255, 255, 255, 0.35);
        }}

        .magma-git-line-header {{
            color: rgba(255, 255, 255, 0.25);
        }}

        .magma-git-diff-stat {{
            padding: 4px 0;
        }}

        .magma-git-diff-stat-line {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: rgba(255, 255, 255, 0.45);
        }}

        .magma-git-stage-hunk-btn {{
            background: transparent;
            color: rgba(255, 255, 255, 0.4);
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
            border-bottom: 1px solid rgba(255, 255, 255, 0.02);
        }}

        .magma-git-commit-row:hover {{
            background: rgba(255, 255, 255, 0.03);
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
            color: #C89A1E;
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
            color: rgba(255, 255, 255, 0.35);
        }}

        .magma-git-commit-date {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 8px;
            color: rgba(255, 255, 255, 0.25);
        }}

        .magma-git-ref-badge {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 8px;
            color: rgba(255, 255, 255, 0.6);
            background: rgba(255, 255, 255, 0.06);
            border-radius: 3px;
            padding: 0 4px;
        }}

        .magma-git-ref-badge.ref-head {{
            color: {accent};
            background: rgba(255, 77, 77, 0.1);
        }}

        .magma-git-ref-badge.ref-tag {{
            color: #C89A1E;
            background: rgba(200, 154, 30, 0.1);
        }}

        .magma-git-commit-detail {{
            padding: 4px 0 4px 16px;
            border-top: 1px solid rgba(255, 255, 255, 0.04);
        }}

        .magma-git-load-more {{
            background: transparent;
            color: rgba(255, 255, 255, 0.4);
            border: 1px solid rgba(255, 255, 255, 0.06);
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
            background: rgba(255, 255, 255, 0.03);
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            border: 1px solid rgba(255, 255, 255, 0.06);
            border-radius: 4px;
            padding: 3px 8px;
        }}

        .magma-git-branch-list {{
            background: transparent;
        }}

        .magma-git-branch-row {{
            padding: 4px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.02);
        }}

        .magma-git-branch-row:hover {{
            background: rgba(255, 255, 255, 0.03);
        }}

        .magma-git-branch-current {{
            background: rgba(255, 77, 77, 0.04);
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
            color: rgba(255, 255, 255, 0.3);
        }}

        .magma-git-branch-remote .magma-git-branch-name {{
            color: rgba(255, 255, 255, 0.45);
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
            background: rgba(255, 255, 255, 0.03);
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            border: 1px solid rgba(255, 255, 255, 0.06);
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
            border-bottom: 1px solid rgba(255, 255, 255, 0.02);
        }}

        .magma-git-stash-row:hover {{
            background: rgba(255, 255, 255, 0.03);
        }}

        .magma-git-stash-index {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: #C89A1E;
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
            background: rgba(255, 255, 255, 0.03);
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            border: 1px solid rgba(255, 255, 255, 0.06);
            border-radius: 4px;
            padding: 3px 8px;
        }}

        .magma-git-search-count {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: rgba(255, 255, 255, 0.35);
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
            color: rgba(255, 255, 255, 0.45);
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            background: rgba(255, 255, 255, 0.04);
            border-radius: 3px;
            padding: 0 4px;
        }}

        .magma-agent-badge.is-live {{
            color: {accent};
            background: rgba(255, 77, 77, 0.08);
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
            background: rgba(255, 255, 255, 0.02);
        }}

        .magma-agent-strip-status {{
            min-width: 72px;
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
        }}

        .magma-agent-strip-message {{
            color: rgba(255, 255, 255, 0.58);
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
        }}

        .magma-agent-strip-toggle,
        .magma-agent-strip-action {{
            background: rgba(255, 255, 255, 0.03);
            border: 1px solid {border};
            border-radius: 6px;
            color: rgba(255, 255, 255, 0.78);
            padding: 4px 9px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
        }}

        .magma-agent-strip-toggle:hover,
        .magma-agent-strip-action:hover {{
            background: rgba(255, 255, 255, 0.06);
        }}

        .magma-agent-prompt {{
            margin-top: 10px;
            padding-top: 8px;
            border-top: 1px solid {border};
        }}

        .magma-agent-prompt-input {{
            min-height: 34px;
            background: rgba(255, 255, 255, 0.03);
            border: 1px solid {border};
            border-radius: 6px;
            color: {text_primary};
            padding: 5px 9px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
        }}

        .magma-agent-prompt-send {{
            min-height: 34px;
            background: rgba(255, 255, 255, 0.03);
            border: 1px solid {border};
            border-radius: 6px;
            color: rgba(255, 255, 255, 0.78);
            padding: 5px 11px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
        }}

        .magma-agent-prompt-send:hover {{
            background: rgba(255, 255, 255, 0.06);
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
            color: rgba(255, 255, 255, 0.5);
            padding: 6px 4px 2px 4px;
            background: rgba(255, 255, 255, 0.02);
        }}

        .magma-git-patch-hunk-row {{
            padding: 3px 4px 3px 12px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.02);
        }}

        .magma-git-patch-hunk-row:hover {{
            background: rgba(255, 255, 255, 0.03);
        }}

        .magma-git-patch-hunk-row.selected {{
            background: rgba(255, 77, 77, 0.08);
        }}

        .magma-git-patch-hunk-status {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            min-width: 14px;
        }}

        .magma-git-patch-hunk-status.status-unreviewed {{
            color: rgba(255, 255, 255, 0.2);
        }}

        .magma-git-patch-hunk-status.status-reviewed {{
            color: #388C50;
        }}

        .magma-git-patch-hunk-status.status-risky {{
            color: {accent};
        }}

        .magma-git-patch-hunk-status.status-followup {{
            color: #C89A1E;
        }}

        .magma-git-patch-hunk-label {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: rgba(255, 255, 255, 0.5);
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
            color: rgba(255, 255, 255, 0.35);
            padding: 4px 4px 2px 4px;
            text-transform: uppercase;
            border-top: 1px solid {border};
        }}

        .magma-git-patch-annotation-frame {{
            border: 1px solid rgba(255, 255, 255, 0.06);
            border-radius: 4px;
            margin: 0 4px 4px 4px;
            min-height: 48px;
        }}

        .magma-git-patch-annotation {{
            background: rgba(255, 255, 255, 0.03);
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
            color: rgba(255, 255, 255, 0.4);
            border: 1px solid rgba(255, 255, 255, 0.06);
            border-radius: 4px;
            padding: 2px 8px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            box-shadow: none;
            min-height: 0;
        }}

        .magma-git-patch-mark-btn:hover {{
            color: {text_primary};
            border-color: rgba(255, 255, 255, 0.12);
        }}

        .magma-git-patch-mark-btn.mark-reviewed:hover {{
            color: #388C50;
            border-color: rgba(56, 140, 80, 0.3);
        }}

        .magma-git-patch-mark-btn.mark-risky:hover {{
            color: {accent};
            border-color: rgba(255, 77, 77, 0.3);
        }}

        .magma-git-patch-mark-btn.mark-followup:hover {{
            color: #C89A1E;
            border-color: rgba(200, 154, 30, 0.3);
        }}

        .magma-git-patch-actions {{
            padding: 4px;
            border-top: 1px solid {border};
        }}

        .magma-git-patch-draft-label {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: rgba(255, 255, 255, 0.35);
        }}

        /* ─── Shared ───────────────────────────────────────────── */

        .magma-git-empty {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            color: rgba(255, 255, 255, 0.25);
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
            color: rgba(255, 255, 255, 0.38);
        }}

        .magma-agent-log-scroll {{
            background: transparent;
            border-top: 1px solid rgba(255, 255, 255, 0.05);
            border-bottom: 1px solid rgba(255, 255, 255, 0.05);
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
            color: #CEB8EF;
        }}

        .magma-agent-role-system {{
            color: rgba(255, 255, 255, 0.3);
        }}

        .magma-agent-role-action {{
            color: #81C784;
        }}

        .magma-agent-log-text {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            line-height: 1.45;
        }}

        .magma-agent-text-user {{
            color: rgba(255, 255, 255, 0.86);
        }}

        .magma-agent-text-agent {{
            color: #CEB8EF;
        }}

        .magma-agent-text-system {{
            color: rgba(255, 255, 255, 0.35);
        }}

        .magma-agent-text-action {{
            color: #A5D6A7;
        }}

        .magma-agent-console {{
            margin: 10px;
            padding: 10px;
            border: 1px solid rgba(255, 255, 255, 0.06);
            border-radius: 8px;
            background: rgba(255, 255, 255, 0.02);
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
            color: rgba(255, 255, 255, 0.5);
            line-height: 1.45;
        }}

        .magma-agent-pending {{
            padding: 10px;
            margin: 10px 10px 0 10px;
            border: 1px solid rgba(255, 77, 77, 0.14);
            border-radius: 8px;
            background: rgba(255, 77, 77, 0.04);
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
            color: rgba(255, 255, 255, 0.65);
            margin-bottom: 6px;
        }}

        .magma-agent-pending-actions {{
            margin-top: 2px;
        }}

        .magma-agent-confirm {{
            background: rgba(255, 77, 77, 0.1);
            color: {accent};
            border: 1px solid rgba(255, 77, 77, 0.16);
            border-radius: 6px;
            padding: 4px 16px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 600;
            box-shadow: none;
            min-height: 0;
        }}

        .magma-agent-confirm:hover {{
            background: rgba(255, 77, 77, 0.16);
            color: #FF8D8D;
        }}

        .magma-agent-reject {{
            background: rgba(255, 255, 255, 0.03);
            color: rgba(255, 255, 255, 0.40);
            border: 1px solid rgba(255, 255, 255, 0.06);
            border-radius: 6px;
            padding: 4px 16px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            box-shadow: none;
            min-height: 0;
        }}

        .magma-agent-reject:hover {{
            background: rgba(255, 77, 77, 0.06);
            color: rgba(255, 130, 130, 0.70);
            border-color: rgba(255, 77, 77, 0.12);
        }}

        .magma-agent-prompt {{
            padding: 10px;
            border-top: 1px solid rgba(255, 255, 255, 0.04);
            background: rgba(0, 0, 0, 0.35);
        }}

        .magma-agent-prompt-glyph {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            color: {accent};
            padding: 0 2px 0 0;
        }}

        .magma-agent-prompt-input {{
            background: rgba(0, 0, 0, 0.72);
            color: {text_primary};
            border: 1px solid rgba(255, 255, 255, 0.07);
            border-radius: 6px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            padding: 6px 10px;
            box-shadow: none;
            min-height: 0;
        }}

        .magma-agent-prompt-input:focus {{
            border-color: rgba(255, 77, 77, 0.25);
            background: rgba(0, 0, 0, 0.88);
        }}

        .magma-agent-prompt-send {{
            background: rgba(255, 255, 255, 0.04);
            color: rgba(255, 255, 255, 0.72);
            border: 1px solid rgba(255, 255, 255, 0.08);
            border-radius: 6px;
            padding: 4px 14px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 600;
            box-shadow: none;
            min-height: 0;
        }}

        .magma-agent-prompt-send:hover {{
            background: rgba(255, 255, 255, 0.08);
            color: {text_primary};
        }}

        /* ─── Notes pane ─────────────────────────────────────── */

        .magma-notes-pane {{
            background: #000000;
            border-left: 1px solid #1A1A1A;
            min-width: 392px;
        }}

        .magma-notes-header {{
            padding: 16px 14px;
            border-bottom: 1px solid #1A1A1A;
            background: #000000;
        }}

        .magma-notes-title-stack {{
            padding: 0;
        }}

        .magma-notes-title {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            font-weight: 800;
            color: #F3F3EF;
            text-transform: uppercase;
            letter-spacing: 1.2px;
        }}

        .magma-notes-subtitle {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: #6B6B66;
            margin-top: 2px;
        }}

        .magma-notes-count {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: #BABAB3;
            background: #0A0A0A;
            border: 1px solid #1A1A1A;
            border-radius: 999px;
            padding: 4px 10px;
        }}

        .magma-notes-new {{
            background: #0A0A0A;
            color: #F3F3EF;
            border: 1px solid #1A1A1A;
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
            background: #151515;
            border-color: #252525;
            color: #FFFFFF;
        }}

        .magma-notes-board {{
            background: #0A0A0A;
            color: #F3F3EF;
            border: 1px solid #1A1A1A;
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
            background: #151515;
            border-color: #252525;
            color: #FFFFFF;
        }}

        .magma-notes-recovery {{
            background: #0A0A0A;
            border-bottom: 1px solid #1A1A1A;
            padding: 10px 14px;
        }}

        .magma-notes-recovery-label {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: #BABAB3;
        }}

        .magma-notes-recovery-continue,
        .magma-notes-recovery-discard {{
            background: #0A0A0A;
            color: #F3F3EF;
            border: 1px solid #1A1A1A;
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
            background: #151515;
            border-color: #252525;
            color: #FFFFFF;
        }}

        .magma-notes-search {{
            background: #0A0A0A;
            color: #F3F3EF;
            border: 1px solid #1A1A1A;
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
            border-color: #252525;
            background: #0A0A0A;
        }}

        .magma-notes-list-area,
        .magma-notes-editor-area {{
            background: #000000;
        }}

        .magma-notes-list-scroll,
        .magma-notes-editor-scroll,
        .magma-notes-media-scroll {{
            background: #000000;
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
            color: #6B6B66;
            padding: 20px 14px;
        }}

        .magma-notes-tile {{
            background: #0A0A0A;
            border: 1px solid #1A1A1A;
            border-radius: 10px;
            padding: 12px;
            transition: all 120ms ease;
        }}

        .magma-notes-tile:hover {{
            background: #0F0F0F;
            border-color: #252525;
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
            color: #F3F3EF;
            border: none;
            box-shadow: none;
            padding: 0;
            min-height: 20px;
            min-width: 20px;
            transition: all 120ms ease;
        }}

        .magma-notes-tile-menu:hover {{
            color: #FFFFFF;
        }}

        .magma-notes-menu-popover {{
            background: transparent;
        }}

        .magma-notes-menu-panel,
        .magma-notes-emoji-panel {{
            background: #0A0A0A;
            border: 1px solid #1A1A1A;
            border-radius: 8px;
            padding: 8px;
        }}

        .magma-notes-menu-item {{
            background: transparent;
            color: #F3F3EF;
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
            background: #151515;
            border-color: #252525;
            color: #FFFFFF;
        }}

        .magma-notes-menu-delete {{
            color: #FF4D4D;
        }}

        .magma-notes-menu-delete:hover {{
            color: #FF6B6B;
            background: #1A0A0A;
            border-color: #5A2A2A;
        }}

        .magma-notes-row-title {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            color: #F3F3EF;
        }}

        .magma-notes-row-preview {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: #BABAB3;
            margin-top: 4px;
            line-height: 1.4;
        }}

        .magma-notes-row-meta {{
            margin-top: 8px;
        }}

        .magma-notes-row-time {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: #6B6B66;
        }}

        .magma-notes-row-pin {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: #388C50;
            font-weight: 600;
            background: #0A0A0A;
            border: 1px solid #1A1A1A;
            border-radius: 999px;
            padding: 2px 8px;
        }}

        .magma-notes-row-media {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: #F3F3EF;
            background: #0A0A0A;
            border: 1px solid #1A1A1A;
            border-radius: 999px;
            padding: 2px 8px;
        }}

        .magma-notes-editor-area {{
            border-top: 1px solid #1A1A1A;
            padding: 14px 14px 16px 14px;
        }}

        .magma-notes-editor-title {{
            background: #000000;
            color: #F3F3EF;
            border: none;
            border-bottom: 1px solid #1A1A1A;
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
            background: #000000;
            border-bottom-color: #252525;
        }}

        .magma-notes-editor-body {{
            background: #000000;
            color: #F3F3EF;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            caret-color: #FFFFFF;
            line-height: 1.5;
        }}

        .magma-notes-media-strip {{
            background: transparent;
            padding: 10px 0 4px 0;
        }}

        .magma-notes-media-card {{
            background: #0A0A0A;
            border: 1px solid #1A1A1A;
            border-radius: 8px;
            padding: 6px;
            transition: all 120ms ease;
        }}

        .magma-notes-media-card:hover {{
            border-color: #252525;
        }}

        .magma-notes-media-picture {{
            background: #000000;
            border-radius: 6px;
        }}

        .magma-notes-media-remove {{
            background: #1A0A0A;
            color: #FF4D4D;
            border: 1px solid #5A2A2A;
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
            background: #2A1515;
            color: #FF6B6B;
            border-color: #7A3A3A;
        }}

        .magma-notes-editor-actions {{
            padding: 12px 14px 0 14px;
        }}

        .magma-notes-editor-meta {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            color: #6B6B66;
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
            background: rgba(255, 77, 77, 0.88);
            border: 1px solid #7A3A3A;
            border-radius: 999px;
            min-height: 3px;
            min-width: 16px;
        }}

        .magma-notes-tool-scroll scrollbar slider:hover {{
            background: rgba(255, 107, 107, 0.96);
            border-color: #A24A4A;
        }}

        .magma-notes-tool-strip,
        .magma-notes-alignment-row,
        .magma-notes-list-row {{
            background: transparent;
        }}

        .magma-notes-tool-pill {{
            background: #000000;
            border: 1px solid #5A2A2A;
            border-radius: 999px;
            padding: 4px 0 6px 0;
            transition: all 120ms ease;
        }}

        .magma-notes-tool-pill:hover {{
            background: #000000;
            border-color: #5A2A2A;
        }}

        .magma-notes-tool {{
            background: transparent;
            color: #F3F3EF;
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
            color: #F3F3EF;
        }}

        .magma-notes-tool-icon {{
            min-width: 22px;
            padding: 3px 5px;
        }}

        .magma-notes-emoji-item {{
            background: #0A0A0A;
            color: #F3F3EF;
            border: 1px solid #1A1A1A;
            border-radius: 6px;
            box-shadow: none;
            min-height: 0;
            min-width: 28px;
            padding: 4px 6px;
            transition: all 120ms ease;
        }}

        .magma-notes-emoji-item:hover {{
            background: #151515;
            border-color: #252525;
            color: #FFFFFF;
        }}
        ",
        window_bg = css_color(theme::BG_PRIMARY),
        window_edge = css_color(theme::WINDOW_EDGE),
        titlebar_bg = css_color(theme::BG_TITLEBAR),
        surface = css_color(theme::SURFACE_BASE),
        border = css_color(theme::BORDER_STRONG),
        text_primary = css_color(theme::TEXT_PRIMARY),
        text_secondary = css_color(theme::TEXT_SECONDARY),
        text_dim = css_color(theme::TEXT_DIM),
        accent = css_color(theme::ACCENT),
        folder_title_size = px(9.0, ui_scale),
        folder_name_size = px(11.0, ui_scale),
        font_9 = px(9.0, ui_scale),
        font_10 = px(10.0, ui_scale),
        font_11 = px(11.0, ui_scale),
        font_12 = px(12.0, ui_scale),
        font_13 = px(13.0, ui_scale),
        font_18 = px(18.0, ui_scale),
        font_22 = px(22.0, ui_scale),
    );
    provider.load_from_data(&css);

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
