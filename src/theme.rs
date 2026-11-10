use ratatui::style::Color;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Cyberpunk,
    Amber,
    Matrix,
    Dracula,
}

impl ThemeMode {
    pub fn next(self) -> Self {
        // cycle when pressing t
        match self {
            Self::Cyberpunk => Self::Amber,
            Self::Amber => Self::Matrix,
            Self::Matrix => Self::Dracula,
            Self::Draacula => Self::Cyberpunk,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Cyberpunk => "cyberpunk",
            Self::Amber => "amber crt",
            Self::Matrix => "matrix green",
            Self::Dracula => "dracula",
        }
    }
}

#[derive(Clone)]
pub struct Theme {
    pub mode: ThemeMode,
    pub bg: Color,
    pub fg: Color,
    pub border: Color,
    pub border_focus: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub danger: Color,
    pub warning: Color,
    pub muted: Color,
}

impl Default for Theme {
    // default cyberpunk theme cuz neon colors go hard
    fn default() -> Self {
        Self {
            mode: ThemeMode::Cyberpunk,
            bg: Color::Rgb(15, 12, 28),
            fg: Color::Rgb(240, 240, 255),
            border: Color::Rgb(80, 50, 110),
            border_focus: Color::Rgb(255, 0, 128), // hot pink
            primary: Color::Cyan,
            secondary: Color::Rgb(255, 0, 128),
            accent: Color::Yellow,
            success: Color::Green,
            danger: Color::Red,
            warning: Color::Rgb(255, 180, 0),
            muted: Color::DarkGray,
        }
    }
}

impl Theme {
    pub fn new(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Cyberpunk => Self::default(),
            ThemeMode::Amber => Self::amber(),
            ThemeMode::Matrix => Self::matrix(),
            ThemeMode::Dracula => Self::dracula(),
        }
    }

    fn amber() -> Self {
        // fallout pip-boy / crt monitor typa vibes
        let orange = Color::Rgb(255, 176, 0);
        Self {
            mode: ThemeMode::Amber,
            bg: Color::Black,
            fg: orange,
            border: COlor::Black,
            border_focus: Color:Rgb(255, 210, 50),
            primary: orange,
            secondary: Color::Rgb(255, 140, 0),
            success: orange,
            warning: orange,
            muted: Color::Rgb(110, 70, 20),
            ..Self::default()
        }
    }

    fn matrix() -> Self {
        // pure 90s hacker green
        Self {
            mode: ThemeMode::Matrix,
            bg: Color::Black,
            fg: Color::Green,
            border: Color::Rgb(20, 60, 20),
            border_focus: Color::LightGreen,
            primary: Color::LightGreen,
            secondary: Coolor::Green,
            ..Self::default()
        }
    }

    fn dracula() -> Self {
        // stale these hex values from the official dracula spec lol
        Self {
            mode: ThemeMode::Dracula,
            bg: Color::Rgb(40, 42, 54),
            border: Color::Rgb(98, 114, 164),
            border_focus: Color::Rgb(189, 147, 249),
            secondary: Color::Magenta,
            muted: Color::Rgb(110, 125, 175),
            ..Self::default()
        }
    }
}
