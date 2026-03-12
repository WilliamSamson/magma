use gtk::{
    gdk, style_context_add_provider_for_display, CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION,
};

use crate::ui::theme;

pub(super) fn install_css() {
    let provider = CssProvider::new();
    let css = format!(
        "
        window.obsidian-window {{
            background: {window_bg};
            border: 1px solid {window_edge};
            border-radius: 12px;
            overflow: hidden;
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

        headerbar.obsidian-header button.titlebutton {{
            background: {surface};
            border-radius: 50%;
            box-shadow: none;
            color: transparent;
            min-height: 14px;
            min-width: 14px;
            padding: 0;
            margin: 0 4px;
            transition: background 140ms ease;
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

        .obsidian-logo {{
            opacity: 0.92;
        }}

        .obsidian-shell {{
            background: {window_bg};
            border-bottom-left-radius: 12px;
            border-bottom-right-radius: 12px;
            overflow: hidden;
        }}

        .obsidian-title {{
            color: {text_primary};
            font-weight: 700;
            letter-spacing: 0.04em;
        }}

        terminal.obsidian-terminal {{
            background: {terminal_bg};
            color: {text_primary};
            border: 1px solid {border};
            border-bottom: none;
            border-top-left-radius: 18px;
            border-top-right-radius: 18px;
            border-bottom-left-radius: 0px;
            border-bottom-right-radius: 0px;
            padding: 10px;
        }}

        separator.obsidian-separator {{
            background: {border};
            min-height: 1px;
            margin: 0 0 12px 0;
        }}

        box.obsidian-input-pill {{
            background: {terminal_bg};
            border: 1px solid {accent};
            border-radius: 999px;
            padding: 4px 16px;
            margin: 0 4px;
        }}

        label.obsidian-prompt-label {{
            color: {accent};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 11px;
            font-weight: 700;
        }}

        entry.obsidian-entry {{
            background: transparent;
            color: {text_primary};
            border: none;
            padding: 8px 0;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 11px;
            box-shadow: none;
            outline: none;
        }}

        entry.obsidian-entry:focus {{
            box-shadow: none;
            outline: none;
        }}
        ",
        window_bg = css_color(theme::BG_PRIMARY),
        window_edge = css_color(theme::WINDOW_EDGE),
        titlebar_bg = css_color(theme::BG_TITLEBAR),
        surface = css_color(theme::SURFACE_BASE),
        terminal_bg = css_color(theme::BG_SECONDARY),
        border = css_color(theme::BORDER_STRONG),
        text_primary = css_color(theme::TEXT_PRIMARY),
        accent = css_color(theme::ACCENT),
    );
    provider.load_from_data(&css);

    if let Some(display) = gdk::Display::default() {
        style_context_add_provider_for_display(
            &display,
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn css_color(color: u32) -> String {
    format!("#{:06X}", color & 0x00FF_FFFF)
}
