//! Create bead modal - lazygit-style multi-field form
//!
//! Layout:
//! ╭─Title──────────────────────────────────────────────────────╮
//! │ Task title█                                                │
//! ╰────────────────────────────────────────────────────────────╯
//! ╭─Description────────────────Press <tab> to switch fields────╮
//! │ Description text...                                        │
//! │                                                            │
//! ╰────────────────────────────────────────────────────────────╯
//! ╭─Options────────────────────────────────────────────────────╮
//! │ Type: task ▾   Priority: P2 ▾   Labels: +add               │
//! ╰────────────────────────────Press <ctrl+s> to create────────╯

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use tui_textarea::TextArea;

use crate::data::BeadType;
use crate::ui::Theme;

/// Which field is focused in the create modal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CreateField {
    #[default]
    Title,
    Description,
    Type,
    Priority,
    Labels,
}

impl CreateField {
    fn next(self) -> Self {
        match self {
            Self::Title => Self::Description,
            Self::Description => Self::Type,
            Self::Type => Self::Priority,
            Self::Priority => Self::Labels,
            Self::Labels => Self::Title,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Title => Self::Labels,
            Self::Description => Self::Title,
            Self::Type => Self::Description,
            Self::Priority => Self::Type,
            Self::Labels => Self::Priority,
        }
    }
}

/// State for the create bead modal
#[derive(Debug, Clone)]
pub struct CreateModal {
    /// Currently focused field
    pub focus: CreateField,
    /// Title input
    pub title: TextArea<'static>,
    /// Description input
    pub description: TextArea<'static>,
    /// Selected bead type
    pub bead_type: BeadType,
    /// Selected priority (0-4)
    pub priority: u8,
    /// Labels (comma-separated in input, parsed to vec)
    pub labels: TextArea<'static>,
    /// Whether the modal is open
    pub open: bool,
}

impl Default for CreateModal {
    fn default() -> Self {
        Self {
            focus: CreateField::default(),
            title: TextArea::default(),
            description: TextArea::default(),
            bead_type: BeadType::default(),
            priority: 2,
            labels: TextArea::default(),
            open: false,
        }
    }
}

impl CreateModal {
    pub fn new() -> Self {
        Self {
            priority: 2, // Default to P2
            ..Default::default()
        }
    }

    /// Open the modal and reset state
    pub fn open(&mut self) {
        self.open = true;
        self.focus = CreateField::Title;
        self.title = TextArea::default();
        self.description = TextArea::default();
        self.labels = TextArea::default();
        self.bead_type = BeadType::Task;
        self.priority = 2;
    }

    /// Open the modal pre-filled with bead data for editing
    pub fn open_with_bead(&mut self, bead: &crate::data::Bead) {
        self.open = true;
        self.focus = CreateField::Title;

        // Pre-fill title
        self.title = TextArea::from(vec![bead.title.clone()]);

        // Pre-fill description
        if let Some(desc) = &bead.description {
            let lines: Vec<String> = desc.lines().map(|s| s.to_string()).collect();
            self.description = TextArea::from(lines);
        } else {
            self.description = TextArea::default();
        }

        // Pre-fill labels
        if !bead.labels.is_empty() {
            let labels_str = bead.labels.join(", ");
            self.labels = TextArea::from(vec![labels_str]);
        } else {
            self.labels = TextArea::default();
        }

        // Set type and priority
        self.bead_type = bead.bead_type;
        self.priority = bead.priority;
    }

    /// Close the modal
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Check if we can submit (title is required)
    pub fn can_submit(&self) -> bool {
        !self.title.lines().join("\n").trim().is_empty()
    }

    /// Get the title
    pub fn get_title(&self) -> String {
        self.title.lines().join("\n")
    }

    /// Get the description (None if empty)
    pub fn get_description(&self) -> Option<String> {
        let desc = self.description.lines().join("\n");
        let trimmed = desc.trim();
        if trimmed.is_empty() { None } else { Some(desc) }
    }

    /// Get labels as a vector
    pub fn get_labels(&self) -> Vec<String> {
        self.labels
            .lines()
            .join("\n")
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Handle pasted text for the currently focused field.
    pub fn handle_paste(&mut self, text: &str) {
        match self.focus {
            CreateField::Title => {
                let single_line = text
                    .lines()
                    .map(str::trim_end)
                    .collect::<Vec<_>>()
                    .join(" ");
                let _ = self.title.insert_str(single_line);
            }
            CreateField::Description => {
                let _ = self.description.insert_str(text);
            }
            CreateField::Labels => {
                let labels_line = text
                    .lines()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join(", ");
                let _ = self.labels.insert_str(labels_line);
            }
            CreateField::Type | CreateField::Priority => {}
        }
    }

    /// Handle a key event, returns true if modal should close and submit
    pub fn handle_key(&mut self, key: KeyEvent) -> ModalAction {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        match key.code {
            // Cancel
            KeyCode::Esc => {
                self.close();
                return ModalAction::Cancelled;
            }

            // Submit with Ctrl+S or Ctrl+Enter
            KeyCode::Char('s') if ctrl => {
                if self.can_submit() {
                    return ModalAction::Submit;
                }
                return ModalAction::None;
            }
            KeyCode::Enter if ctrl => {
                if self.can_submit() {
                    return ModalAction::Submit;
                }
                return ModalAction::None;
            }

            // Tab to switch fields
            KeyCode::Tab if shift => {
                // Some terminals send Tab with shift modifier
                self.focus = self.focus.prev();
                return ModalAction::None;
            }
            KeyCode::BackTab => {
                // Most terminals send BackTab for Shift+Tab
                self.focus = self.focus.prev();
                return ModalAction::None;
            }
            KeyCode::Tab => {
                self.focus = self.focus.next();
                return ModalAction::None;
            }

            // Field-specific handling
            _ => {
                self.handle_field_key(key);
            }
        }

        ModalAction::None
    }

    fn handle_field_key(&mut self, key: KeyEvent) {
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        match self.focus {
            CreateField::Title => {
                // Plain Enter in title moves to description, but Shift+Enter adds newline
                if key.code == KeyCode::Enter && !shift {
                    self.focus = CreateField::Description;
                } else {
                    self.title.input(key);
                }
            }
            CreateField::Description => {
                // Allow Enter for newlines in description
                self.description.input(key);
            }
            CreateField::Type => {
                // Cycle through types with left/right or j/k
                match key.code {
                    KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('k') | KeyCode::Up => {
                        self.bead_type = self.prev_type();
                    }
                    KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('j') | KeyCode::Down => {
                        self.bead_type = self.next_type();
                    }
                    _ => {}
                }
            }
            CreateField::Priority => {
                // Cycle through priorities with left/right or j/k
                match key.code {
                    KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('k') | KeyCode::Up => {
                        self.priority = self.priority.saturating_sub(1);
                    }
                    KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('j') | KeyCode::Down => {
                        self.priority = (self.priority + 1).min(4);
                    }
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        let p = c.to_digit(10).unwrap() as u8;
                        if p <= 4 {
                            self.priority = p;
                        }
                    }
                    _ => {}
                }
            }
            CreateField::Labels => {
                // Don't allow newlines in labels field
                if key.code != KeyCode::Enter {
                    self.labels.input(key);
                }
            }
        }
    }

    fn next_type(&self) -> BeadType {
        match self.bead_type {
            BeadType::Task => BeadType::Bug,
            BeadType::Bug => BeadType::Feature,
            BeadType::Feature => BeadType::Epic,
            BeadType::Epic => BeadType::Story,
            BeadType::Story => BeadType::Task,
        }
    }

    fn prev_type(&self) -> BeadType {
        match self.bead_type {
            BeadType::Task => BeadType::Story,
            BeadType::Bug => BeadType::Task,
            BeadType::Feature => BeadType::Bug,
            BeadType::Epic => BeadType::Feature,
            BeadType::Story => BeadType::Epic,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CreateField, CreateModal};

    #[test]
    fn paste_title_flattens_newlines() {
        let mut modal = CreateModal::new();
        modal.focus = CreateField::Title;

        modal.handle_paste("one\ntwo");

        assert_eq!(modal.get_title(), "one two");
    }

    #[test]
    fn paste_description_preserves_newlines() {
        let mut modal = CreateModal::new();
        modal.focus = CreateField::Description;

        modal.handle_paste("line one\nline two");

        assert_eq!(modal.description.lines(), &["line one", "line two"]);
    }

    #[test]
    fn paste_labels_turns_newlines_into_commas() {
        let mut modal = CreateModal::new();
        modal.focus = CreateField::Labels;

        modal.handle_paste("ui\nbug");

        assert_eq!(modal.get_labels(), vec!["ui", "bug"]);
    }
}

/// Action to take after handling a key
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalAction {
    None,
    Submit,
    Cancelled,
}

/// Render the create modal
pub fn render_create_modal(frame: &mut Frame, area: Rect, theme: &Theme, modal: &CreateModal) {
    // Calculate modal size - take up most of the screen
    let modal_width = (area.width - 4).min(80);
    let modal_height = (area.height - 4).min(20);
    let x = (area.width - modal_width) / 2;
    let y = (area.height - modal_height) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    // Clear the area
    frame.render_widget(Clear, modal_area);

    // Split into title (3 lines), description (flexible), options (3 lines)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(5),    // Description
            Constraint::Length(3), // Options
        ])
        .split(modal_area);

    // Render title field
    render_title_field(frame, chunks[0], theme, modal);

    // Render description field
    render_description_field(frame, chunks[1], theme, modal);

    // Render options field
    render_options_field(frame, chunks[2], theme, modal);
}

fn render_title_field(frame: &mut Frame, area: Rect, theme: &Theme, modal: &CreateModal) {
    let focused = modal.focus == CreateField::Title;
    let border_color = if focused {
        theme.focused_border
    } else {
        theme.border
    };

    let mut textarea = modal.title.clone();
    textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(border_color))
            .title(" Title ")
            .title_style(Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
    );
    textarea.set_style(Style::default().fg(theme.fg));
    textarea.set_cursor_line_style(Style::default()); // Disable underline
    if !focused {
        textarea.set_cursor_style(Style::default());
    }
    frame.render_widget(&textarea, area);
}

fn render_description_field(frame: &mut Frame, area: Rect, theme: &Theme, modal: &CreateModal) {
    let focused = modal.focus == CreateField::Description;
    let border_color = if focused {
        theme.focused_border
    } else {
        theme.border
    };

    // Title with hint
    let title = if focused {
        " Description ─── Press <tab> to switch fields "
    } else {
        " Description "
    };

    let mut textarea = modal.description.clone();
    textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(border_color))
            .title(title)
            .title_style(Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
    );
    textarea.set_style(Style::default().fg(theme.fg));
    textarea.set_cursor_line_style(Style::default()); // Disable underline
    if !focused {
        textarea.set_cursor_style(Style::default());
    }
    frame.render_widget(&textarea, area);
}

fn render_options_field(frame: &mut Frame, area: Rect, theme: &Theme, modal: &CreateModal) {
    // Check if any option field is focused
    let type_focused = modal.focus == CreateField::Type;
    let priority_focused = modal.focus == CreateField::Priority;
    let labels_focused = modal.focus == CreateField::Labels;
    let any_focused = type_focused || priority_focused || labels_focused;

    let border_color = if any_focused {
        theme.focused_border
    } else {
        theme.border
    };

    // Title with submit hint
    let title = " Options ─── Press <ctrl+s> to create ";

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(border_color))
        .title(title)
        .title_style(Style::default().fg(theme.fg).add_modifier(Modifier::BOLD));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Build options line
    let mut spans = Vec::new();

    // Type
    spans.push(Span::styled("Type: ", Style::default().fg(theme.muted)));
    let type_style = if type_focused {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD | Modifier::REVERSED)
    } else {
        Style::default().fg(theme.fg)
    };
    spans.push(Span::styled(format!(" {} ", modal.bead_type), type_style));

    spans.push(Span::raw("   "));

    // Priority
    spans.push(Span::styled("Priority: ", Style::default().fg(theme.muted)));
    let priority_style = if priority_focused {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD | Modifier::REVERSED)
    } else {
        Style::default().fg(theme.priority_color(modal.priority))
    };
    spans.push(Span::styled(
        format!(" P{} ", modal.priority),
        priority_style,
    ));

    spans.push(Span::raw("   "));

    // Labels
    spans.push(Span::styled("Labels: ", Style::default().fg(theme.muted)));
    let labels_text = modal.labels.lines().join("\n");
    let display_text = if labels_text.is_empty() {
        "(none)".to_string()
    } else {
        labels_text
    };

    let label_style = if labels_focused {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.fg)
    };
    spans.push(Span::styled(display_text, label_style));

    let line = Line::from(spans);
    let para = Paragraph::new(line);
    frame.render_widget(para, inner);
}
