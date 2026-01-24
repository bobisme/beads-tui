//! Color themes for beads-tui
//!
//! Inspired by lazygit's neutral, professional aesthetic:
//! - Green border for focused pane
//! - Simple unicode glyphs (no emojis)
//! - Neutral color scheme

#![allow(dead_code)]

use ratatui::style::Color;

/// A color theme for the application
#[derive(Debug, Clone)]
pub struct Theme {
    /// Theme name
    pub name: &'static str,
    /// Background color
    pub bg: Color,
    /// Primary foreground color
    pub fg: Color,
    /// Muted/secondary text color
    pub muted: Color,
    /// Accent/highlight color
    pub accent: Color,
    /// Border color (unfocused panes)
    pub border: Color,
    /// Border color for focused pane (lazygit uses green)
    pub focused_border: Color,
    /// Selection/highlight background
    pub selection_bg: Color,
    /// Selection foreground
    pub selection_fg: Color,
    /// Status: open
    pub status_open: Color,
    /// Status: in progress
    pub status_in_progress: Color,
    /// Status: blocked
    pub status_blocked: Color,
    /// Status: closed
    pub status_closed: Color,
    /// Priority critical (P0)
    pub priority_critical: Color,
    /// Priority high (P1)
    pub priority_high: Color,
    /// Priority medium (P2)
    pub priority_medium: Color,
    /// Priority low (P3+)
    pub priority_low: Color,
}

/// Lazygit-inspired theme (default) - neutral with green focused borders
pub const LAZYGIT: Theme = Theme {
    name: "Lazygit",
    bg: Color::Reset, // Use terminal default
    fg: Color::White,
    muted: Color::Gray, // Lighter than DarkGray for visibility on selection
    accent: Color::Cyan,
    border: Color::DarkGray,      // Unfocused panes
    focused_border: Color::Green, // Focused pane (lazygit signature)
    selection_bg: Color::DarkGray,
    selection_fg: Color::Cyan,
    status_open: Color::White,
    status_in_progress: Color::Cyan,
    status_blocked: Color::Red,
    status_closed: Color::Green, // Green for completed!
    priority_critical: Color::Red,
    priority_high: Color::Yellow,
    priority_medium: Color::White,
    priority_low: Color::Gray, // Lighter for visibility on selection
};

/// Tokyo Night theme
pub const TOKYO_NIGHT: Theme = Theme {
    name: "Tokyo Night",
    bg: Color::Rgb(26, 27, 38),
    fg: Color::Rgb(169, 177, 214),
    muted: Color::Rgb(86, 95, 137),
    accent: Color::Rgb(122, 162, 247),
    border: Color::Rgb(59, 66, 97),
    focused_border: Color::Rgb(158, 206, 106), // Green
    selection_bg: Color::Rgb(41, 46, 66),
    selection_fg: Color::Rgb(192, 202, 245),
    status_open: Color::Rgb(169, 177, 214),
    status_in_progress: Color::Rgb(125, 207, 255),
    status_blocked: Color::Rgb(247, 118, 142),
    status_closed: Color::Rgb(158, 206, 106), // Green for completed
    priority_critical: Color::Rgb(247, 118, 142),
    priority_high: Color::Rgb(255, 158, 100),
    priority_medium: Color::Rgb(224, 175, 104),
    priority_low: Color::Rgb(158, 206, 106),
};

/// Dracula theme
pub const DRACULA: Theme = Theme {
    name: "Dracula",
    bg: Color::Rgb(40, 42, 54),
    fg: Color::Rgb(248, 248, 242),
    muted: Color::Rgb(98, 114, 164),
    accent: Color::Rgb(189, 147, 249),
    border: Color::Rgb(68, 71, 90),
    focused_border: Color::Rgb(80, 250, 123), // Green
    selection_bg: Color::Rgb(68, 71, 90),
    selection_fg: Color::Rgb(248, 248, 242),
    status_open: Color::Rgb(248, 248, 242),
    status_in_progress: Color::Rgb(139, 233, 253),
    status_blocked: Color::Rgb(255, 85, 85),
    status_closed: Color::Rgb(80, 250, 123), // Green for completed
    priority_critical: Color::Rgb(255, 85, 85),
    priority_high: Color::Rgb(255, 184, 108),
    priority_medium: Color::Rgb(241, 250, 140),
    priority_low: Color::Rgb(80, 250, 123),
};

/// Nord theme
pub const NORD: Theme = Theme {
    name: "Nord",
    bg: Color::Rgb(46, 52, 64),
    fg: Color::Rgb(216, 222, 233),
    muted: Color::Rgb(76, 86, 106),
    accent: Color::Rgb(136, 192, 208),
    border: Color::Rgb(59, 66, 82),
    focused_border: Color::Rgb(163, 190, 140), // Green
    selection_bg: Color::Rgb(67, 76, 94),
    selection_fg: Color::Rgb(236, 239, 244),
    status_open: Color::Rgb(216, 222, 233),
    status_in_progress: Color::Rgb(136, 192, 208),
    status_blocked: Color::Rgb(191, 97, 106),
    status_closed: Color::Rgb(163, 190, 140), // Green for completed
    priority_critical: Color::Rgb(191, 97, 106),
    priority_high: Color::Rgb(208, 135, 112),
    priority_medium: Color::Rgb(235, 203, 139),
    priority_low: Color::Rgb(163, 190, 140),
};

/// All available themes (Lazygit is default)
pub const THEMES: &[Theme] = &[LAZYGIT, TOKYO_NIGHT, DRACULA, NORD];

impl Theme {
    /// Get a color for a priority level
    pub fn priority_color(&self, priority: u8) -> Color {
        match priority {
            0 => self.priority_critical,
            1 => self.priority_high,
            2 => self.priority_medium,
            _ => self.priority_low,
        }
    }
}
