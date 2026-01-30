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
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::data::BeadType;
use crate::ui::input::TextInput;
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
#[derive(Debug, Clone, Default)]
pub struct CreateModal {
    /// Currently focused field
    pub focus: CreateField,
    /// Title input
    pub title: TextInput,
    /// Description input
    pub description: TextInput,
    /// Selected bead type
    pub bead_type: BeadType,
    /// Selected priority (0-4)
    pub priority: u8,
    /// Labels (comma-separated in input, parsed to vec)
    pub labels: TextInput,
    /// Whether the modal is open
    pub open: bool,
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
        self.title.clear();
        self.description.clear();
        self.labels.clear();
        self.bead_type = BeadType::Task;
        self.priority = 2;
    }

    /// Close the modal
    pub fn close(&mut self) {
        self.open = false;
    }

    /// Check if we can submit (title is required)
    pub fn can_submit(&self) -> bool {
        !self.title.text().trim().is_empty()
    }

    /// Get the title
    pub fn get_title(&self) -> &str {
        self.title.text()
    }

    /// Get the description (None if empty)
    pub fn get_description(&self) -> Option<&str> {
        let desc = self.description.text().trim();
        if desc.is_empty() {
            None
        } else {
            Some(self.description.text())
        }
    }

    /// Get labels as a vector
    pub fn get_labels(&self) -> Vec<String> {
        self.labels
            .text()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
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
        match self.focus {
            CreateField::Title => {
                // Enter in title moves to description
                if key.code == KeyCode::Enter {
                    self.focus = CreateField::Description;
                } else {
                    self.title.handle_key(key);
                }
            }
            CreateField::Description => {
                // Allow Enter for newlines in description
                if key.code == KeyCode::Enter {
                    // Insert newline
                    self.description
                        .handle_key(KeyEvent::new(KeyCode::Char('\n'), KeyModifiers::NONE));
                } else {
                    self.description.handle_key(key);
                }
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
                self.labels.handle_key(key);
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

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(border_color))
        .title(" Title ")
        .title_style(Style::default().fg(theme.fg).add_modifier(Modifier::BOLD));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Render input with cursor
    let text = modal.title.text();
    let cursor = modal.title.cursor();
    let line = if focused {
        render_input_with_cursor(text, cursor, theme)
    } else {
        Line::from(Span::styled(
            text.to_string(),
            Style::default().fg(theme.fg),
        ))
    };

    let para = Paragraph::new(line);
    frame.render_widget(para, inner);
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

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(border_color))
        .title(title)
        .title_style(Style::default().fg(theme.fg).add_modifier(Modifier::BOLD));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Render multi-line input
    let text = modal.description.text();
    let cursor = modal.description.cursor();

    if focused {
        // For multi-line, we need to handle cursor position across lines
        let (before, after) = text.split_at(cursor.min(text.len()));
        let mut content = before.to_string();
        content.push('\u{2588}'); // Block cursor
        // Skip first character of after (covered by cursor)
        content.push_str(&after.chars().skip(1).collect::<String>());

        let para = Paragraph::new(content)
            .style(Style::default().fg(theme.fg))
            .wrap(Wrap { trim: false });
        frame.render_widget(para, inner);
    } else {
        let para = Paragraph::new(text)
            .style(Style::default().fg(theme.fg))
            .wrap(Wrap { trim: false });
        frame.render_widget(para, inner);
    }
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
    if labels_focused {
        // Show input with cursor
        let text = modal.labels.text();
        let cursor = modal.labels.cursor();
        let (before, after) = text.split_at(cursor.min(text.len()));
        spans.push(Span::styled(
            before.to_string(),
            Style::default().fg(theme.fg),
        ));
        spans.push(Span::styled(
            "\u{2588}".to_string(),
            Style::default().fg(theme.accent),
        ));
        // Skip first character of after (covered by cursor)
        let rest = after.chars().skip(1).collect::<String>();
        if !rest.is_empty() {
            spans.push(Span::styled(
                rest,
                Style::default().fg(theme.fg),
            ));
        }
    } else {
        let labels_text = if modal.labels.is_empty() {
            "(none)".to_string()
        } else {
            modal.labels.text().to_string()
        };
        spans.push(Span::styled(labels_text, Style::default().fg(theme.fg)));
    }

    let line = Line::from(spans);
    let para = Paragraph::new(line);
    frame.render_widget(para, inner);
}

fn render_input_with_cursor<'a>(text: &str, cursor: usize, theme: &Theme) -> Line<'a> {
    let (before, after) = text.split_at(cursor.min(text.len()));
    let mut spans = vec![Span::styled(before.to_string(), Style::default().fg(theme.fg))];

    // Cursor replaces the character at cursor position
    spans.push(Span::styled("\u{2588}".to_string(), Style::default().fg(theme.accent)));

    // Skip first character of after (covered by cursor)
    if let Some(rest) = after.chars().skip(1).collect::<String>().into() {
        spans.push(Span::styled(rest, Style::default().fg(theme.fg)));
    }

    Line::from(spans)
}
