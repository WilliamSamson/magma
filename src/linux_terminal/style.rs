use gtk::{
    gdk, style_context_add_provider_for_display, CssProvider, STYLE_PROVIDER_PRIORITY_USER,
};

use crate::ui::theme;

pub(super) fn install_css(app_font_size: u32) {
    let provider = CssProvider::new();
    let ui_scale = ui_scale(app_font_size);
    let css = format!(
        "
        window.obsidian-window {{
            background: {window_bg};
            border: 1px solid {window_edge};
            border-radius: 12px;
        }}

        headerbar.obsidian-header {{
            background: {titlebar_bg};
            border-bottom: 1px solid {border};
            min-height: 40px;
            padding: 4px 12px;
        }}

        headerbar.obsidian-header box {{
            background: transparent;
        }}

        headerbar.obsidian-header windowcontrols button {{
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

        headerbar.obsidian-header windowcontrols button image {{
            opacity: 0;
            -gtk-icon-size: 8px;
            transition: opacity 120ms ease;
        }}

        headerbar.obsidian-header windowcontrols button:hover image {{
            opacity: 1;
            color: rgba(0, 0, 0, 0.6);
        }}

        headerbar.obsidian-header windowcontrols button.close {{
            background: #FF5F56;
        }}

        headerbar.obsidian-header windowcontrols button.minimize {{
            background: #FFBD2E;
        }}

        headerbar.obsidian-header windowcontrols button.maximize {{
            background: #27C93F;
        }}

        headerbar.obsidian-header windowcontrols button.close:hover {{
            background: #FF3B30;
        }}

        headerbar.obsidian-header windowcontrols button.minimize:hover {{
            background: #E5A323;
        }}

        headerbar.obsidian-header windowcontrols button.maximize:hover {{
            background: #1CAD30;
        }}

        headerbar.obsidian-header button.titlebutton {{
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

        headerbar.obsidian-header button.titlebutton image {{
            opacity: 0;
            -gtk-icon-size: 8px;
            transition: opacity 120ms ease;
        }}

        headerbar.obsidian-header button.titlebutton:hover image {{
            opacity: 1;
            color: rgba(0, 0, 0, 0.6);
        }}

        headerbar.obsidian-header button.titlebutton.close {{
            background: #FF5F56;
        }}

        headerbar.obsidian-header button.titlebutton.minimize {{
            background: #FFBD2E;
        }}

        headerbar.obsidian-header button.titlebutton.maximize {{
            background: #27C93F;
        }}

        headerbar.obsidian-header button.titlebutton.close:hover {{
            background: #FF3B30;
        }}

        headerbar.obsidian-header button.titlebutton.minimize:hover {{
            background: #E5A323;
        }}

        headerbar.obsidian-header button.titlebutton.maximize:hover {{
            background: #1CAD30;
        }}

        button.obsidian-header-settings {{
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

        button.obsidian-header-settings:hover {{
            opacity: 1.0;
        }}

        .obsidian-logo {{
            opacity: 0.92;
        }}

        .obsidian-shell {{
            background: {window_bg};
            border-bottom-left-radius: 12px;
            border-bottom-right-radius: 12px;
        }}

        .obsidian-title {{
            color: {text_primary};
            font-weight: 700;
            letter-spacing: 0.04em;
        }}

        terminal.obsidian-terminal {{
            background: transparent;
            color: {text_primary};
            border: 1px solid {border};
            border-radius: 18px;
            padding: 10px;
        }}

        box.obsidian-workspace-actions {{
            background: transparent;
            border: none;
            margin: 0;
            padding: 0;
        }}

        separator.obsidian-separator {{
            background: {border};
            min-height: 1px;
            margin: 0 0 12px 0;
        }}

        separator.obsidian-v-separator {{
            background: {border};
            min-width: 1px;
            margin: 0 4px;
        }}

        paned.obsidian-split-pane > separator {{
            background: rgba(255, 255, 255, 0.04);
            min-width: 10px;
            margin: 0 4px;
            border-radius: 999px;
            transition: background 140ms ease;
        }}

        paned.obsidian-split-pane > separator:hover {{
            background: rgba(255, 77, 77, 0.18);
        }}

        box.obsidian-mux-root {{
            background: transparent;
        }}

        box.obsidian-mux-bar {{
            background: rgba(255, 255, 255, 0.02);
            border: 1px solid rgba(255, 255, 255, 0.05);
            border-radius: 999px;
            padding: 4px;
            margin: 0 0 4px 0;
        }}

        button.obsidian-mux-session,
        button.obsidian-mux-action {{
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

        button.obsidian-mux-session:hover,
        button.obsidian-mux-action:hover {{
            opacity: 1.0;
            background: rgba(255, 255, 255, 0.04);
        }}

        button.obsidian-mux-session.active {{
            opacity: 1.0;
            background: rgba(255, 77, 77, 0.1);
            color: {accent};
        }}

        .obsidian-right-pane {{
            background: transparent;
            padding: 14px 14px 12px 14px;
            border: 1px solid {border};
            border-radius: 14px;
            margin: 0;
        }}

        .obsidian-handle {{
            min-width: 30px;
            margin: 0 3px;
            padding: 3px;
            border: 1px solid rgba(255, 255, 255, 0.04);
            border-radius: 12px;
            background: rgba(255, 255, 255, 0.015);
            transition: border-color 180ms ease, background 180ms ease, opacity 180ms ease;
        }}

        .obsidian-handle:hover {{
            background: rgba(255, 255, 255, 0.025);
            border-color: rgba(255, 255, 255, 0.08);
        }}

        .obsidian-handle.collapsed {{
            background: rgba(255, 255, 255, 0.01);
        }}

        .obsidian-handle.collapsed:hover {{
            background: rgba(255, 77, 77, 0.05);
            border-color: rgba(255, 77, 77, 0.18);
        }}

        button.obsidian-handle-segment {{
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

        button.obsidian-handle-segment:hover {{
            opacity: 0.86;
            background: rgba(255, 255, 255, 0.035);
        }}

        button.obsidian-handle-segment.active {{
            opacity: 1.0;
            background: rgba(255, 77, 77, 0.08);
        }}

        .obsidian-handle-icon {{
            color: {text_primary};
            -gtk-icon-size: 14px;
            opacity: inherit;
            transition: opacity 180ms ease, color 180ms ease;
        }}

        .obsidian-handle.collapsed button.obsidian-handle-segment {{
            opacity: 0.28;
        }}

        .obsidian-handle.collapsed:hover button.obsidian-handle-segment {{
            opacity: 0.72;
            background: rgba(255, 77, 77, 0.035);
        }}

        .obsidian-handle.collapsed:hover .obsidian-handle-icon {{
            color: {accent};
        }}

        button.obsidian-handle-segment.active .obsidian-handle-icon {{
            color: {accent};
        }}

        box.obsidian-input-pill {{
            background: transparent;
            border: 1px solid {border};
            border-radius: 999px;
            padding: 4px 16px;
            margin: 0 0 8px 0;
            transition: border-color 140ms ease;
        }}

        box.obsidian-input-pill:focus-within {{
            border-color: {accent};
        }}

        box.obsidian-input-pill.terminal-active {{
            border-color: {accent};
        }}

        button.obsidian-workspace-button,
        button.obsidian-tool-button {{
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

        button.obsidian-search-toggle {{
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

        menubutton.obsidian-tool-menu > button {{
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

        button.obsidian-search-toggle:hover {{
            opacity: 0.8;
        }}

        button.obsidian-workspace-button:hover,
        button.obsidian-tool-button:hover {{
            background: {surface};
            opacity: 1.0;
        }}

        menubutton.obsidian-tool-menu > button:hover,
        menubutton.obsidian-tool-menu:checked > button {{
            background: {surface};
            opacity: 1.0;
        }}

        popover.obsidian-inspector-popover {{
            background: rgba(0, 0, 0, 0.95);
            border: 1px solid {border};
            border-radius: 12px;
        }}

        popover.obsidian-inspector-popover > contents {{
            padding: 0;
            background: transparent;
            border-radius: 12px;
        }}

        .obsidian-inspector-panel {{
            background: transparent;
            padding: 12px;
            min-width: 280px;
        }}

        .obsidian-inspector-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            opacity: 0.92;
            margin-bottom: 2px;
        }}

        .obsidian-inspector-row {{
            background: rgba(255, 255, 255, 0.018);
            border: 1px solid rgba(255, 255, 255, 0.05);
            border-radius: 10px;
            padding: 10px;
        }}

        .obsidian-inspector-key {{
            color: {accent};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 700;
            opacity: 0.78;
        }}

        .obsidian-inspector-value {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.62;
            line-height: 1.45;
        }}

        .obsidian-tab-bar-container {{
            background: transparent;
            margin: 0 12px;
            padding: 4px 0;
            min-height: 40px;
        }}

        .obsidian-tab-bar-scroller {{
            background: transparent;
            margin-right: 8px;
            min-height: 40px;
        }}

        .obsidian-switcher-overlay {{
            background: transparent;
        }}

        .obsidian-switcher-panel {{
            background: rgba(0, 0, 0, 0.94);
            border: 1px solid {border};
            border-radius: 14px;
            padding: 12px;
            min-width: 360px;
        }}

        entry.obsidian-switcher-entry {{
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

        entry.obsidian-switcher-entry:focus {{
            border-bottom-color: {accent};
            box-shadow: none;
            outline: none;
        }}

        .obsidian-switcher-list {{
            background: transparent;
        }}

        row.obsidian-switcher-row {{
            background: transparent;
            border-radius: 8px;
            margin: 1px 0;
            padding: 0;
            transition: background 100ms ease;
        }}

        row.obsidian-switcher-row:hover,
        row.obsidian-switcher-row:selected {{
            background: rgba(255, 255, 255, 0.05);
        }}

        .obsidian-switcher-index {{
            color: {accent};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            min-width: 20px;
            opacity: 0.8;
        }}

        .obsidian-switcher-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            font-weight: 700;
            opacity: 0.82;
        }}

        .obsidian-switcher-empty {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            opacity: 0.35;
            padding: 8px 2px;
        }}

        .obsidian-tabs-list {{
            background: transparent;
            padding: 0 4px;
        }}

        .obsidian-tab-item {{
            background: transparent;
            border-radius: 0;
            padding: 6px 14px;
            margin: 0 2px;
            transition: background 140ms ease, opacity 140ms ease, border-color 140ms ease;
            border-bottom: 2px solid transparent;
            opacity: 0.4;
        }}

        .obsidian-tab-item:hover {{
            background: rgba(255, 255, 255, 0.03);
            opacity: 0.8;
        }}

        .obsidian-tab-item.active {{
            background: transparent;
            border-bottom-color: {accent};
            opacity: 1.0;
        }}

        button.obsidian-tab-close-button {{
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

        .obsidian-tab-item:hover button.obsidian-tab-close-button,
        .obsidian-tab-item.active button.obsidian-tab-close-button {{
            opacity: 0.4;
        }}

        button.obsidian-tab-close-button:hover {{
            background: rgba(255, 255, 255, 0.1);
            opacity: 1.0;
        }}

        .obsidian-tab-item.dragging {{
            opacity: 0.4;
        }}

        .obsidian-tab-item.drop-target {{
            background: rgba(255, 255, 255, 0.05);
            border-bottom-color: {accent};
        }}

        button.obsidian-add-tab-button {{
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

        button.obsidian-add-tab-button:hover {{
            opacity: 1.0;
            background: rgba(255, 255, 255, 0.05);
        }}

        notebook.obsidian-notebook > stack {{
            background: transparent;
        }}

        label.obsidian-tab-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            font-weight: 700;
            text-transform: lowercase;
        }}

        entry.obsidian-tab-rename-entry {{
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

        entry.obsidian-tab-rename-entry:focus {{
            border-bottom-color: {accent};
            box-shadow: none;
            outline: none;
        }}

        entry.obsidian-entry.search-active {{
            color: #FFBD2E;
        }}

        box.obsidian-input-pill.search-active {{
            border-color: rgba(255, 189, 46, 0.3);
        }}

        label.obsidian-user-label {{
            color: {accent};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            font-weight: 700;
        }}

        label.obsidian-path-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            font-weight: 700;
        }}

        label.obsidian-status-label {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            margin: 0 6px 0 0;
        }}

        label.obsidian-notice-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.58;
            margin: 0 8px 0 0;
        }}

        label.obsidian-notice-ok {{
            color: #7FB685;
            opacity: 0.7;
        }}

        label.obsidian-notice-error {{
            color: {accent};
            opacity: 0.9;
        }}

        label.obsidian-status-ok {{
            color: #7FB685;
        }}

        label.obsidian-status-error {{
            color: {accent};
        }}

        label.obsidian-status-running {{
            color: #E5C07B;
        }}

        entry.obsidian-entry {{
            background: transparent;
            color: {text_primary};
            border: none;
            padding: 8px 0;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            box-shadow: none;
            outline: none;
        }}

        entry.obsidian-entry:focus {{
            box-shadow: none;
            outline: none;
        }}

        /* Logr Pane */
        .obsidian-logr-root {{
            padding: 6px 8px;
        }}

        .obsidian-view-root {{
            padding: 0 4px 0 4px;
        }}

        .obsidian-view-header {{
            padding: 4px 0 12px 0;
            margin-bottom: 2px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.035);
        }}

        .obsidian-view-heading {{
            min-width: 0;
        }}

        .obsidian-view-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            font-weight: 700;
            text-transform: lowercase;
            letter-spacing: 0.03em;
            opacity: 0.96;
        }}

        .obsidian-view-count {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 500;
            opacity: 0.54;
            line-height: 1.5;
        }}

        button.obsidian-view-header-action {{
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

        button.obsidian-view-header-action:hover {{
            background: rgba(255, 255, 255, 0.025);
            border-color: rgba(255, 255, 255, 0.09);
            opacity: 1.0;
        }}

        button.obsidian-view-action,
        button.obsidian-view-open {{
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

        button.obsidian-view-action:hover,
        button.obsidian-view-open:hover {{
            opacity: 1.0;
            background: rgba(255, 77, 77, 0.05);
            border-color: rgba(255, 77, 77, 0.12);
        }}

        .obsidian-view-file-scroller {{
            margin-bottom: 8px;
            background: rgba(255, 255, 255, 0.012);
            border: 1px solid rgba(255, 255, 255, 0.035);
            border-radius: 14px;
            padding: 4px;
        }}

        .obsidian-view-file-list {{
            background: transparent;
        }}

        row.obsidian-view-file-row {{
            background: transparent;
            border-radius: 10px;
            margin: 0 0 2px 0;
            padding: 0;
            transition: background 140ms ease, border-color 140ms ease;
        }}

        row.obsidian-view-file-row:hover {{
            background: rgba(255, 77, 77, 0.03);
        }}

        row.obsidian-view-file-row:selected {{
            background: rgba(255, 77, 77, 0.06);
            border: 1px solid rgba(255, 77, 77, 0.12);
        }}

        .obsidian-view-file-card {{
            padding: 8px 10px;
        }}

        .obsidian-view-file-icon {{
            color: {text_primary};
            opacity: 0.5;
            -gtk-icon-size: 16px;
        }}

        .obsidian-view-file-name {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            opacity: 0.9;
        }}

        .obsidian-view-file-meta {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.42;
            margin-top: -1px;
        }}

        .obsidian-view-preview {{
            background: rgba(4, 4, 5, 0.98);
            border: 1px solid rgba(255, 255, 255, 0.04);
            border-radius: 16px;
            padding: 0;
            overflow: hidden;
        }}

        .obsidian-view-preview-chrome {{
            padding: 14px 16px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.045);
            background: rgba(10, 10, 12, 0.98);
        }}

        .obsidian-view-preview-actions {{
            margin-left: 12px;
            background: rgba(255, 255, 255, 0.02);
            border: 1px solid rgba(255, 255, 255, 0.04);
            border-radius: 9px;
            padding: 3px;
        }}

        .obsidian-view-preview-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            font-weight: 700;
            opacity: 0.9;
        }}

        .obsidian-view-preview-meta {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.44;
        }}

        button.obsidian-view-preview-button {{
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

        button.obsidian-view-preview-button:hover {{
            background: rgba(255, 255, 255, 0.03);
            border-color: rgba(255, 255, 255, 0.05);
            opacity: 1.0;
        }}

        button.obsidian-view-preview-button:disabled {{
            opacity: 0.28;
            background: transparent;
            border-color: transparent;
        }}

        button.obsidian-view-preview-button-secondary {{
            background: transparent;
        }}

        button.obsidian-view-preview-button-primary {{
            background: rgba(255, 255, 255, 0.035);
            border-color: rgba(255, 255, 255, 0.06);
            opacity: 0.92;
        }}

        button.obsidian-view-preview-button-primary:hover {{
            background: rgba(255, 255, 255, 0.05);
            border-color: rgba(255, 255, 255, 0.09);
        }}

        .obsidian-view-preview-stack {{
            background: transparent;
        }}

        .obsidian-view-preview-surface,
        .obsidian-view-code-scroller {{
            background: rgba(3, 3, 4, 1);
            border: none;
        }}

        textview.obsidian-view-code {{
            background: rgba(3, 3, 4, 1);
            color: {text_primary};
        }}

        .obsidian-view-empty-state {{
            padding: 42px 24px;
            background: rgba(3, 3, 4, 1);
        }}

        .obsidian-view-empty-icon {{
            color: {text_primary};
            opacity: 0.08;
            margin-bottom: 12px;
        }}

        .obsidian-view-empty-text {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            opacity: 0.35;
            line-height: 1.5;
        }}

        .obsidian-view-info {{
            background: rgba(3, 3, 4, 1);
            padding: 24px 18px;
        }}

        .obsidian-view-info-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 600;
        }}

        .obsidian-view-info-body {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.68;
            line-height: 1.45;
            margin-top: 2px;
        }}

        .obsidian-web-tab-row {{
            padding: 4px 0 4px 0;
        }}

        .obsidian-web-tab-scroll {{
            min-content-height: 28px;
        }}

        .obsidian-web-tabs {{
            padding: 0 2px;
        }}

        .obsidian-web-tab {{
            background: transparent;
            border: 1px solid transparent;
            border-radius: 6px;
            padding: 2px 6px;
            min-height: 22px;
            opacity: 0.45;
            transition: opacity 140ms ease, background 140ms ease, border-color 140ms ease;
        }}

        .obsidian-web-tab:hover {{
            opacity: 0.75;
            background: rgba(255, 255, 255, 0.03);
        }}

        .obsidian-web-tab.active {{
            opacity: 1.0;
            background: rgba(255, 255, 255, 0.04);
            border-color: rgba(255, 255, 255, 0.06);
        }}

        .obsidian-web-tab-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
        }}

        button.obsidian-web-tab-close {{
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

        .obsidian-web-tab:hover button.obsidian-web-tab-close {{
            opacity: 0.4;
        }}

        button.obsidian-web-tab-close:hover {{
            opacity: 1.0 !important;
            background: rgba(255, 77, 77, 0.15);
        }}

        button.obsidian-web-tab-add {{
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

        button.obsidian-web-tab-add:hover {{
            opacity: 0.9;
        }}

        .obsidian-web-controls {{
            margin-bottom: 4px;
        }}

        .obsidian-web-ssl {{
            min-width: 14px;
            min-height: 14px;
            margin: 0 2px 0 0;
            opacity: 0.5;
        }}

        .obsidian-web-ssl.secure {{
            color: #27C93F;
            opacity: 0.7;
        }}

        .obsidian-web-ssl.insecure {{
            color: #FFBD2E;
            opacity: 0.7;
        }}

        progressbar.obsidian-web-progress {{
            min-height: 2px;
            margin: 0 2px 4px 2px;
        }}

        progressbar.obsidian-web-progress trough {{
            min-height: 2px;
            background: transparent;
            border: none;
            border-radius: 1px;
        }}

        progressbar.obsidian-web-progress progress {{
            min-height: 2px;
            background: {accent};
            border: none;
            border-radius: 1px;
        }}

        .obsidian-web-find-bar {{
            background: rgba(255, 255, 255, 0.02);
            border: 1px solid rgba(255, 255, 255, 0.06);
            border-radius: 8px;
            padding: 4px 6px;
            margin: 0 2px 4px 2px;
        }}

        entry.obsidian-web-find-entry {{
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

        entry.obsidian-web-find-entry:focus {{
            opacity: 1.0;
            box-shadow: none;
            outline: none;
        }}

        .obsidian-web-find-matches {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.4;
            margin: 0 4px;
        }}

        .obsidian-web-bar {{
            background: rgba(255, 255, 255, 0.02);
            border: 1px solid rgba(255, 255, 255, 0.06);
            border-radius: 16px;
            padding: 6px;
        }}

        .obsidian-web-nav {{
            background: rgba(255, 255, 255, 0.02);
            border: 1px solid rgba(255, 255, 255, 0.04);
            border-radius: 12px;
            padding: 2px;
            margin-right: 6px;
        }}

        .obsidian-web-address-shell {{
            background: rgba(0, 0, 0, 0.22);
            border: 1px solid rgba(255, 255, 255, 0.04);
            border-radius: 12px;
            padding: 0 4px 0 6px;
        }}

        button.obsidian-web-button {{
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

        button.obsidian-web-button:hover {{
            opacity: 1.0;
            background: rgba(255, 255, 255, 0.06);
        }}

        button.obsidian-web-button:disabled {{
            opacity: 0.18;
        }}

        button.obsidian-web-text-button {{
            min-width: 42px;
            padding: 4px 8px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 700;
        }}

        entry.obsidian-web-entry {{
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

        entry.obsidian-web-entry:focus {{
            opacity: 1.0;
            box-shadow: none;
            outline: none;
        }}

        .obsidian-web-status {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.4;
            padding: 0 2px 8px 2px;
        }}

        .obsidian-web-frame {{
            background: rgba(255, 255, 255, 0.01);
            border: 1px solid rgba(255, 255, 255, 0.05);
            border-radius: 16px;
            padding: 0;
        }}

        .obsidian-webview {{
            background: rgba(0, 0, 0, 0.38);
            border: none;
            border-radius: 16px;
            margin-top: 0;
        }}

        .obsidian-logr-header {{
            padding: 4px 0 12px 0;
            margin-bottom: 2px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.035);
        }}

        .obsidian-logr-heading {{
            min-width: 0;
        }}

        .obsidian-logr-tab-row {{
            padding: 0 0 8px 0;
        }}

        .obsidian-logr-tabs {{
            background: transparent;
        }}

        .obsidian-logr-tab {{
            background: transparent;
            border: 1px solid transparent;
            border-radius: 12px;
            padding: 5px 9px;
            opacity: 0.56;
            transition: opacity 140ms ease, background 140ms ease, border-color 140ms ease, transform 140ms ease;
        }}

        .obsidian-logr-tab:hover {{
            opacity: 0.88;
            background: rgba(255, 255, 255, 0.02);
            border-color: rgba(255, 255, 255, 0.05);
        }}

        .obsidian-logr-tab.active {{
            opacity: 1.0;
            background: rgba(255, 77, 77, 0.035);
            border-color: rgba(255, 77, 77, 0.12);
        }}

        .obsidian-logr-tab-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 700;
            letter-spacing: 0.02em;
            opacity: 0.72;
        }}

        .obsidian-logr-tab.active .obsidian-logr-tab-label {{
            color: {accent};
            opacity: 0.92;
        }}

        button.obsidian-logr-tab-close,
        button.obsidian-logr-tab-add {{
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

        button.obsidian-logr-tab-close:hover,
        button.obsidian-logr-tab-add:hover {{
            opacity: 1.0;
            background: rgba(255, 255, 255, 0.05);
        }}

        .obsidian-logr-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_13};
            font-weight: 700;
            opacity: 0.96;
            text-transform: lowercase;
            letter-spacing: 0.03em;
        }}

        .obsidian-logr-count {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            opacity: 0.62;
        }}

        .obsidian-logr-picker {{
            background: rgba(255, 255, 255, 0.018);
            border: 1px solid rgba(255, 255, 255, 0.04);
            border-radius: 999px;
            padding: 4px 6px;
            margin: 0 0 6px 0;
        }}

        .obsidian-logr-inline-icon {{
            color: {accent};
            -gtk-icon-size: 12px;
            margin: 0 2px 0 2px;
            opacity: 0.7;
        }}

        menubutton.obsidian-logr-select > button {{
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

        menubutton.obsidian-logr-select > button:hover {{
            opacity: 1.0;
            border-color: rgba(255, 77, 77, 0.14);
            background: rgba(255, 77, 77, 0.045);
        }}

        popover.obsidian-logr-popover {{
            background: {window_bg};
            border: 1px solid {border};
            border-radius: 6px;
            padding: 4px 0;
        }}

        popover.obsidian-logr-popover > contents {{
            background: {window_bg};
            border-radius: 6px;
            padding: 0;
        }}

        .obsidian-logr-popover-list {{
            background: transparent;
        }}

        .obsidian-logr-popover-row {{
            background: transparent;
            padding: 0;
            border-radius: 4px;
            margin: 1px 4px;
            transition: background 100ms ease;
        }}

        .obsidian-logr-popover-row:hover {{
            background: rgba(255, 255, 255, 0.05);
        }}

        row.obsidian-logr-popover-row:focus {{
            background: rgba(255, 77, 77, 0.10);
        }}

        .obsidian-logr-popover-item {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            padding: 5px 8px;
            opacity: 0.7;
        }}

        .obsidian-logr-popover-row:hover .obsidian-logr-popover-item {{
            opacity: 1.0;
        }}

        button.obsidian-logr-icon-btn {{
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

        button.obsidian-logr-icon-btn:hover {{
            opacity: 1.0;
            background: rgba(255, 77, 77, 0.05);
            border-color: rgba(255, 77, 77, 0.12);
        }}

        .obsidian-logr-controls {{
            background: rgba(255, 255, 255, 0.012);
            border: 1px solid rgba(255, 255, 255, 0.035);
            border-radius: 999px;
            padding: 4px 6px;
            margin-bottom: 6px;
        }}

        .obsidian-logr-stream-shell {{
            background: rgba(255, 255, 255, 0.018);
            border-radius: 999px;
            padding: 2px 10px 2px 8px;
            margin: 0 6px;
        }}

        .obsidian-logr-stream-icon {{
            color: #27C93F;
            -gtk-icon-size: 10px;
            opacity: 0.72;
        }}

        .obsidian-logr-stream-label {{
            color: #B0E4B7;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.82;
        }}

        entry.obsidian-logr-filter {{
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

        entry.obsidian-logr-filter:focus {{
            opacity: 1.0;
            background: rgba(255, 77, 77, 0.035);
            border-color: rgba(255, 77, 77, 0.15);
            box-shadow: none;
            outline: none;
        }}

        .obsidian-logr-status {{
            color: #FFBD2E;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.68;
            padding: 6px 0 0 0;
        }}

        .obsidian-logr-empty {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            opacity: 0.3;
            padding: 12px 4px;
        }}

        .obsidian-log-list {{
            background: transparent;
        }}

        .obsidian-log-entry {{
            padding: 3px 8px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.015);
            border-radius: 8px;
            transition: background 160ms ease, padding 160ms ease, border-color 160ms ease;
        }}

        .obsidian-log-entry:hover {{
            background: rgba(255, 77, 77, 0.03);
            border-bottom-color: rgba(255, 77, 77, 0.08);
        }}

        .obsidian-log-entry.expanded {{
            background: rgba(255, 77, 77, 0.04);
            border-bottom-color: transparent;
            padding-top: 6px;
            transition: background 240ms cubic-bezier(0.4, 0, 0.2, 1);
        }}

        .obsidian-log-line-number {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.15;
            min-width: 28px;
            margin-right: 2px;
            text-align: right;
        }}

        .obsidian-log-entry:hover .obsidian-log-line-number {{
            opacity: 0.35;
        }}

        .obsidian-log-details {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.6;
            line-height: 1.4;
            background: rgba(255, 255, 255, 0.015);
            border: 1px solid rgba(255, 255, 255, 0.04);
            padding: 6px 10px;
            border-radius: 10px;
            /* Container containment */
            overflow-wrap: break-word;
            word-wrap: break-word;
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

        .obsidian-log-fields {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.35;
            margin-left: 4px;
        }}

        button.obsidian-log-copy-btn {{
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

        .obsidian-log-entry:hover button.obsidian-log-copy-btn {{
            opacity: 0.45;
        }}

        button.obsidian-log-copy-btn:hover {{
            opacity: 1.0 !important;
            background: rgba(255, 255, 255, 0.08);
        }}

        button.obsidian-log-delete-btn {{
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

        .obsidian-log-entry:hover button.obsidian-log-delete-btn {{
            opacity: 0.35;
        }}

        button.obsidian-log-delete-btn:hover {{
            opacity: 1.0 !important;
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
        .obsidian-settings-root {{
            padding: 16px 0 16px 24px;
        }}

        .obsidian-setup-root {{
            padding: 28px;
        }}

        .obsidian-setup-shell {{
            min-width: 760px;
            max-width: 760px;
            background: rgba(0, 0, 0, 0.78);
            border: 1px solid rgba(255, 255, 255, 0.08);
            border-radius: 22px;
            box-shadow: 0 28px 72px rgba(0, 0, 0, 0.42);
        }}

        .obsidian-setup-topbar {{
            padding: 12px 16px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.06);
            background: rgba(255, 255, 255, 0.015);
        }}

        .obsidian-setup-dot {{
            font-size: {font_9};
            opacity: 0.9;
        }}

        .obsidian-setup-dot.red {{
            color: rgba(255, 95, 86, 0.9);
        }}

        .obsidian-setup-dot.amber {{
            color: rgba(255, 189, 46, 0.9);
        }}

        .obsidian-setup-dot.green {{
            color: rgba(39, 201, 63, 0.9);
        }}

        .obsidian-setup-topbar-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 700;
            letter-spacing: 0.16em;
            opacity: 0.36;
        }}

        .obsidian-setup-body {{
            padding: 28px 30px 30px 30px;
        }}

        .obsidian-setup-eyebrow {{
            color: {accent};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 700;
            letter-spacing: 0.14em;
            opacity: 0.9;
        }}

        .obsidian-setup-hero {{
            margin-bottom: 18px;
        }}

        .obsidian-setup-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_13};
            font-weight: 700;
            opacity: 0.98;
            margin: 2px 0 4px 0;
        }}

        .obsidian-setup-copy {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            opacity: 0.58;
            line-height: 1.6;
        }}

        .obsidian-setup-progress {{
            margin-bottom: 18px;
            padding-bottom: 6px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.05);
        }}

        .obsidian-setup-step {{
            background: transparent;
            border: none;
            border-bottom: 2px solid transparent;
            border-radius: 0;
            padding: 0 0 10px 0;
        }}

        .obsidian-setup-step.active {{
            border-bottom-color: rgba(255, 77, 77, 0.72);
        }}

        .obsidian-setup-step-index {{
            color: {accent};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            font-weight: 700;
            letter-spacing: 0.14em;
            opacity: 0.36;
        }}

        .obsidian-setup-step-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            opacity: 0.42;
        }}

        .obsidian-setup-step.active .obsidian-setup-step-label {{
            opacity: 0.94;
        }}

        .obsidian-setup-step.active .obsidian-setup-step-index {{
            opacity: 0.82;
        }}

        .obsidian-setup-pages {{
            min-height: 286px;
        }}

        .obsidian-setup-page {{
            background: transparent;
            border: none;
            padding: 2px 0 0 0;
        }}

        .obsidian-setup-page-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            letter-spacing: 0.12em;
            opacity: 0.92;
        }}

        .obsidian-setup-page-copy {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.42;
            line-height: 1.55;
            margin-bottom: 10px;
        }}

        .obsidian-setup-setting {{
            background: rgba(255, 255, 255, 0.018);
            border: 1px solid rgba(255, 255, 255, 0.055);
            border-radius: 14px;
            padding: 14px 16px;
        }}

        .obsidian-setup-setting-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            opacity: 0.86;
        }}

        .obsidian-setup-setting-copy {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.4;
            line-height: 1.5;
        }}

        .obsidian-setup-value {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.62;
            line-height: 1.6;
        }}

        .obsidian-setup-footer {{
            margin-top: 16px;
            padding-top: 6px;
        }}

        button.obsidian-setup-secondary {{
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

        button.obsidian-setup-secondary:hover {{
            opacity: 1.0;
        }}

        button.obsidian-setup-action {{
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

        button.obsidian-setup-action:hover {{
            opacity: 1.0;
        }}

        .obsidian-setup-nav-content {{
            background: transparent;
        }}

        .obsidian-setup-nav-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            opacity: inherit;
        }}

        .obsidian-setup-nav-icon {{
            color: {text_primary};
            opacity: inherit;
        }}

        .obsidian-settings-header {{
            padding: 4px 24px 14px 0;
            margin-bottom: 4px;
        }}

        button.obsidian-settings-back {{
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

        button.obsidian-settings-back:hover {{
            opacity: 1.0;
        }}

        .obsidian-settings-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_18};
            font-weight: 700;
            text-transform: lowercase;
            letter-spacing: 0.04em;
            opacity: 0.95;
            margin-top: 4px;
        }}

        .obsidian-settings-content {{
            padding: 4px 18px 24px 0;
        }}

        .obsidian-settings-section {{
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

        .obsidian-settings-row {{
            background: transparent;
            border: none;
            border-bottom: 1px solid rgba(255, 255, 255, 0.025);
            padding: 12px 0;
            margin-bottom: 2px;
            transition: opacity 160ms ease;
        }}

        .obsidian-settings-row:hover {{
            background: transparent;
            opacity: 1.0;
        }}

        .obsidian-settings-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            font-weight: 700;
            opacity: 0.9;
        }}

        .obsidian-settings-copy {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            opacity: 0.42;
            line-height: 1.5;
            margin-top: 2px;
        }}

        .obsidian-settings-value {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            opacity: 0.6;
        }}

        .obsidian-settings-about-copy {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            line-height: 1.6;
            opacity: 0.54;
        }}

        entry.obsidian-settings-entry {{
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

        entry.obsidian-settings-entry:focus {{
            background: rgba(255, 255, 255, 0.025);
            border-color: {accent};
        }}

        spinbutton.obsidian-settings-spin {{
            background: rgba(255, 255, 255, 0.015);
            color: {text_primary};
            border: 1px solid rgba(255, 255, 255, 0.04);
            border-radius: 8px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            min-width: 90px;
            box-shadow: none;
        }}

        spinbutton.obsidian-settings-spin > text {{
            background: transparent;
        }}

        spinbutton.obsidian-settings-spin > button {{
            background: transparent;
            color: {text_primary};
            border: none;
            opacity: 0.4;
            transition: opacity 140ms ease;
        }}

        spinbutton.obsidian-settings-spin > button:hover {{
            opacity: 1.0;
        }}

        .obsidian-settings-dropdown {{
            background: rgba(255, 255, 255, 0.015);
            border: 1px solid rgba(255, 255, 255, 0.04);
            border-radius: 8px;
            min-width: 140px;
        }}

        .obsidian-settings-dropdown > button {{
            background: transparent;
            color: {text_primary};
            border: none;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            padding: 6px 10px;
            opacity: 0.8;
            transition: opacity 140ms ease;
        }}

        switch.obsidian-settings-switch {{
            background: rgba(255, 255, 255, 0.03);
            border: 1px solid rgba(255, 255, 255, 0.04);
            border-radius: 12px;
            min-width: 42px;
            min-height: 22px;
            padding: 0;
            transition: background 180ms cubic-bezier(0.4, 0, 0.2, 1), border-color 180ms ease;
        }}

        switch.obsidian-settings-switch:checked {{
            background: rgba(255, 77, 77, 0.08);
            border-color: rgba(255, 77, 77, 0.18);
        }}

        switch.obsidian-settings-switch > image {{
            color: transparent;
        }}

        switch.obsidian-settings-switch slider {{
            background: rgba(255, 255, 255, 0.22);
            border: none;
            border-radius: 50%;
            min-width: 14px;
            min-height: 14px;
            margin: 4px;
            transition: background 180ms cubic-bezier(0.4, 0, 0.2, 1), transform 180ms cubic-bezier(0.4, 0, 0.2, 1);
        }}

        switch.obsidian-settings-switch:checked slider {{
            background: {accent};
            box-shadow: 0 0 8px rgba(255, 77, 77, 0.35);
        }}

        button.obsidian-settings-link {{
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

        button.obsidian-settings-link:hover {{
            background: rgba(255, 255, 255, 0.02);
            opacity: 1.0;
            border-color: rgba(255, 255, 255, 0.1);
        }}

        .obsidian-settings-about-page {{
            padding: 24px;
        }}

        .obsidian-settings-about-panel {{
        }}

        .obsidian-settings-about-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_22};
            font-weight: 700;
            text-transform: lowercase;
            letter-spacing: 0.06em;
            opacity: 0.96;
        }}

        .obsidian-settings-about-name {{
            color: {accent};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_12};
            font-weight: 700;
            letter-spacing: 0.08em;
            opacity: 0.9;
            margin-top: 8px;
        }}

        .obsidian-settings-about-meta {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            opacity: 0.38;
            margin-top: 4px;
        }}

        .obsidian-about-section-header {{
            color: {accent};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_11};
            font-weight: 700;
            text-transform: uppercase;
            letter-spacing: 0.1em;
            opacity: 0.8;
        }}

        .obsidian-about-credits-box {{
            opacity: 0.7;
        }}

        .obsidian-about-category-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_10};
            font-weight: 700;
            opacity: 0.4;
            text-transform: lowercase;
        }}

        .obsidian-about-license-text {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: {font_9};
            opacity: 0.35;
            line-height: 1.6;
        }}

        button.obsidian-settings-save {{
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

        button.obsidian-settings-save:hover {{
            background: rgba(255, 77, 77, 0.10);
        }}
        ",
        window_bg = css_color(theme::BG_PRIMARY),
        window_edge = css_color(theme::WINDOW_EDGE),
        titlebar_bg = css_color(theme::BG_TITLEBAR),
        surface = css_color(theme::SURFACE_BASE),
        border = css_color(theme::BORDER_STRONG),
        text_primary = css_color(theme::TEXT_PRIMARY),
        accent = css_color(theme::ACCENT),
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
