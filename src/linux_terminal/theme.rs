use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ThemeMode {
    #[default]
    Dark,
    Light,
    AtomMaterial,
    Nord,
    Dracula,
    SolarizedDark,
    Monokai,
    GruvboxDark,
}

impl ThemeMode {
    pub(crate) const OPTIONS: [&'static str; 8] = [
        "dark",
        "light",
        "atom material",
        "nord",
        "dracula",
        "solarized dark",
        "monokai",
        "gruvbox dark",
    ];

    pub(crate) fn selected_index(self) -> u32 {
        match self {
            Self::Dark => 0,
            Self::Light => 1,
            Self::AtomMaterial => 2,
            Self::Nord => 3,
            Self::Dracula => 4,
            Self::SolarizedDark => 5,
            Self::Monokai => 6,
            Self::GruvboxDark => 7,
        }
    }

    pub(crate) fn from_index(index: u32) -> Self {
        match index {
            1 => Self::Light,
            2 => Self::AtomMaterial,
            3 => Self::Nord,
            4 => Self::Dracula,
            5 => Self::SolarizedDark,
            6 => Self::Monokai,
            7 => Self::GruvboxDark,
            _ => Self::Dark,
        }
    }

    pub(crate) fn prefers_dark_gtk(self) -> bool {
        // Only Light uses GTK light preference; all others are dark-based.
        !matches!(self, Self::Light)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Palette {
    pub(crate) bg_primary: u32,
    pub(crate) bg_titlebar: u32,
    pub(crate) surface_base: u32,
    pub(crate) accent: u32,
    pub(crate) text_primary: u32,
    pub(crate) text_secondary: u32,
    pub(crate) text_dim: u32,
    pub(crate) window_edge: u32,
    pub(crate) border_strong: u32,
    pub(crate) sem_yellow: u32,
    pub(crate) sem_green: u32,
    pub(crate) sem_orange: u32,
    pub(crate) sem_blue: u32,
    pub(crate) sem_magenta: u32,
    pub(crate) sem_brown: u32,
    #[allow(dead_code)]
    pub(crate) sem_gray: u32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TerminalPalette {
    pub(crate) foreground: u32,
    pub(crate) background: u32,
    pub(crate) ansi: [u32; 8],
}

pub(crate) fn palette(mode: ThemeMode) -> Palette {
    match mode {
        ThemeMode::Dark => Palette {
            bg_primary: 0x00000000,
            bg_titlebar: 0x00000000,
            surface_base: 0x000B0B0B,
            accent: 0x00FF4D4D,
            text_primary: 0x00F3F3EF,
            text_secondary: 0x00BABAB3,
            text_dim: 0x006B6B66,
            window_edge: 0x00E8E8E8,
            border_strong: 0x00242424,
            sem_yellow: 0x00E8C87A,
            sem_green: 0x007FB685,
            sem_orange: 0x00E87050,
            sem_blue: 0x005B9BD5,
            sem_magenta: 0x00C79DD6,
            sem_brown: 0x00B0896E,
            sem_gray: 0x006B6B66,
        },
        ThemeMode::Light => Palette {
            bg_primary: 0x00F6F2EB,
            bg_titlebar: 0x00F6F2EB,
            surface_base: 0x00FFFDF9,
            accent: 0x00C54242,
            text_primary: 0x001F1D1A,
            text_secondary: 0x005F584F,
            text_dim: 0x00878074,
            window_edge: 0x00C8BFB3,
            border_strong: 0x00C9C0B4,
            sem_yellow: 0x00B57C12,
            sem_green: 0x004E7757,
            sem_orange: 0x00B35E3F,
            sem_blue: 0x003E74AF,
            sem_magenta: 0x007D648C,
            sem_brown: 0x007B5E49,
            sem_gray: 0x00878074,
        },
        ThemeMode::AtomMaterial => Palette {
            bg_primary: 0x00151515,
            bg_titlebar: 0x00151515,
            surface_base: 0x001E1E1E,
            accent: 0x0089DDFF,
            text_primary: 0x00EEFFFF,
            text_secondary: 0x00B0BEC5,
            text_dim: 0x00546E7A,
            window_edge: 0x00546E7A,
            border_strong: 0x002C2C2C,
            sem_yellow: 0x00FFCB6B,
            sem_green: 0x00C3E88D,
            sem_orange: 0x00FF5370,
            sem_blue: 0x0082AAFF,
            sem_magenta: 0x00C792EA,
            sem_brown: 0x00F78C6C,
            sem_gray: 0x00546E7A,
        },
        ThemeMode::Nord => Palette {
            bg_primary: 0x001E222A,
            bg_titlebar: 0x001E222A,
            surface_base: 0x002B303C,
            accent: 0x0088C0D0,
            text_primary: 0x00ECEFF4,
            text_secondary: 0x00D8DEE9,
            text_dim: 0x004C566A,
            window_edge: 0x004C566A,
            border_strong: 0x00333A4A,
            sem_yellow: 0x00EBCB8B,
            sem_green: 0x00A3BE8C,
            sem_orange: 0x00BF616A,
            sem_blue: 0x0081A1C1,
            sem_magenta: 0x00B48EAD,
            sem_brown: 0x00D08770,
            sem_gray: 0x004C566A,
        },
        ThemeMode::Dracula => Palette {
            bg_primary: 0x00160F25,
            bg_titlebar: 0x00160F25,
            surface_base: 0x001D1432,
            accent: 0x00BD93F9,
            text_primary: 0x00F8F8F2,
            text_secondary: 0x00BFBFB6,
            text_dim: 0x006272A4,
            window_edge: 0x006272A4,
            border_strong: 0x002C1E4E,
            sem_yellow: 0x00F1FA8C,
            sem_green: 0x0050FA7B,
            sem_orange: 0x00FF5555,
            sem_blue: 0x00BD93F9,
            sem_magenta: 0x00FF79C6,
            sem_brown: 0x00FFB86C,
            sem_gray: 0x006272A4,
        },
        ThemeMode::SolarizedDark => Palette {
            bg_primary: 0x00001A21,
            bg_titlebar: 0x00001A21,
            surface_base: 0x0004242C,
            accent: 0x00268BD2,
            text_primary: 0x00FDF6E3,
            text_secondary: 0x0093A1A1,
            text_dim: 0x00586E75,
            window_edge: 0x00586E75,
            border_strong: 0x0006323D,
            sem_yellow: 0x00B58900,
            sem_green: 0x00859900,
            sem_orange: 0x00DC322F,
            sem_blue: 0x00268BD2,
            sem_magenta: 0x00D33682,
            sem_brown: 0x00CB4B16,
            sem_gray: 0x00586E75,
        },
        ThemeMode::Monokai => Palette {
            bg_primary: 0x00191A16,
            bg_titlebar: 0x00191A16,
            surface_base: 0x001E1E19,
            accent: 0x00F92672,
            text_primary: 0x00F8F8F2,
            text_secondary: 0x00CFCFC2,
            text_dim: 0x0075715E,
            window_edge: 0x0075715E,
            border_strong: 0x00292821,
            sem_yellow: 0x00E6DB74,
            sem_green: 0x00A6E22E,
            sem_orange: 0x00F92672,
            sem_blue: 0x0066D9EF,
            sem_magenta: 0x00AE81FF,
            sem_brown: 0x00FD971F,
            sem_gray: 0x0075715E,
        },
        ThemeMode::GruvboxDark => Palette {
            bg_primary: 0x001A1A1A,
            bg_titlebar: 0x001A1A1A,
            surface_base: 0x00272523,
            accent: 0x00FE8019,
            text_primary: 0x00EBDBB2,
            text_secondary: 0x00BDAE93,
            text_dim: 0x00665C54,
            window_edge: 0x00665C54,
            border_strong: 0x00383330,
            sem_yellow: 0x00D79921,
            sem_green: 0x00B8BB26,
            sem_orange: 0x00CC241D,
            sem_blue: 0x00458588,
            sem_magenta: 0x00B16286,
            sem_brown: 0x00D65D0E,
            sem_gray: 0x00665C54,
        },
    }
}

pub(crate) fn terminal_palette(mode: ThemeMode) -> TerminalPalette {
    match mode {
        ThemeMode::Dark => TerminalPalette {
            foreground: 0x00F3F3EF,
            background: 0x00000000,
            ansi: [
                0x00000000, // black
                0x00FF4D4D, // red
                0x0047A95A, // green
                0x00F0C94A, // yellow
                0x00D0D0D0, // blue
                0x00B3A4BA, // magenta
                0x0099B8C2, // cyan
                0x00F3F3EF, // white
            ],
        },
        ThemeMode::Light => TerminalPalette {
            foreground: 0x001F1D1A,
            background: 0x00FFFDF9,
            ansi: [
                0x00C9C0B4, 0x00C54242, 0x003A7A4F, 0x00B57C12,
                0x004F77C5, 0x007D648C, 0x004E8193, 0x001F1D1A,
            ],
        },
        ThemeMode::AtomMaterial => TerminalPalette {
            foreground: 0x00EEFFFF,
            background: 0x00151515,
            ansi: [
                0x00546E7A, // black
                0x00FF5370, // red
                0x00C3E88D, // green
                0x00FFCB6B, // yellow
                0x0082AAFF, // blue
                0x00C792EA, // magenta
                0x0089DDFF, // cyan
                0x00EEFFFF, // white
            ],
        },
        ThemeMode::Nord => TerminalPalette {
            foreground: 0x00ECEFF4,
            background: 0x001E222A,
            ansi: [
                0x003B4252, // black
                0x00BF616A, // red
                0x00A3BE8C, // green
                0x00EBCB8B, // yellow
                0x0081A1C1, // blue
                0x00B48EAD, // magenta
                0x0088C0D0, // cyan
                0x00ECEFF4, // white
            ],
        },
        ThemeMode::Dracula => TerminalPalette {
            foreground: 0x00F8F8F2,
            background: 0x00160F25,
            ansi: [
                0x0044475A, // black
                0x00FF5555, // red
                0x0050FA7B, // green
                0x00F1FA8C, // yellow
                0x00BD93F9, // blue
                0x00FF79C6, // magenta
                0x008BE9FD, // cyan
                0x00F8F8F2, // white
            ],
        },
        ThemeMode::SolarizedDark => TerminalPalette {
            foreground: 0x00FDF6E3,
            background: 0x00001A21,
            ansi: [
                0x00073642, // black
                0x00DC322F, // red
                0x00859900, // green
                0x00B58900, // yellow
                0x00268BD2, // blue
                0x00D33682, // magenta
                0x002AA198, // cyan
                0x00EEE8D5, // white
            ],
        },
        ThemeMode::Monokai => TerminalPalette {
            foreground: 0x00F8F8F2,
            background: 0x00191A16,
            ansi: [
                0x00272822, // black
                0x00F92672, // red
                0x00A6E22E, // green
                0x00E6DB74, // yellow
                0x0066D9EF, // blue
                0x00AE81FF, // magenta
                0x00A1EFE4, // cyan
                0x00F8F8F2, // white
            ],
        },
        ThemeMode::GruvboxDark => TerminalPalette {
            foreground: 0x00EBDBB2,
            background: 0x001A1A1A,
            ansi: [
                0x00282828, // black
                0x00CC241D, // red
                0x0098971A, // green
                0x00D79921, // yellow
                0x00458588, // blue
                0x00B16286, // magenta
                0x00689D6A, // cyan
                0x00EBDBB2, // white
            ],
        },
    }
}

