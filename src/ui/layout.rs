//! Main layout for beads-tui

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::InputMode;
use crate::data::{build_tree_order, Bead};
use crate::ui::create_modal::{render_create_modal, CreateModal};
use crate::ui::detail::{DetailPanel, DetailState};
use crate::ui::list::{BeadList, BeadListState};
use crate::ui::Theme;

/// Which pane is currently focused
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    List,
    Detail,
}

/// Minimum width to show both panes
const MIN_DUAL_PANE_WIDTH: u16 = 60;

/// Render the main application layout
/// Returns (list_area, detail_area) for mouse handling
#[allow(clippy::too_many_arguments)]
pub fn render_layout(
    frame: &mut ratatui::Frame,
    beads: &[Bead],
    list_state: &mut BeadListState,
    detail_state: &mut DetailState,
    theme: &Theme,
    focus: Focus,
    split_percent: u16,
    filter: Option<&str>,
    show_help: bool,
    hide_closed: bool,
    show_detail: bool,
    input_mode: InputMode,
    search_text: &str,
    search_cursor: usize,
    create_modal: &CreateModal,
    reason_text: &str,
    reason_cursor: usize,
) -> (Rect, Rect) {
    let area = frame.area();
    let is_narrow = area.width < MIN_DUAL_PANE_WIDTH;

    // Main vertical layout: content + footer (no header)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // Main content
            Constraint::Length(1), // Footer
        ])
        .split(area);

    // Determine layout based on show_detail and terminal width
    let (list_area, detail_area) = if !show_detail {
        // Only show list (full width)
        (chunks[0], Rect::default())
    } else if is_narrow {
        // Narrow terminal: only show detail when it's open
        (Rect::default(), chunks[0])
    } else {
        // Normal: show both panes
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(split_percent),
                Constraint::Percentage(100 - split_percent),
            ])
            .split(chunks[0]);
        (content_chunks[0], content_chunks[1])
    };

    // Render bead list (if visible)
    if list_area.width > 0 {
        let list = BeadList::new(beads, theme)
            .focused(focus == Focus::List)
            .filter(filter)
            .hide_closed(hide_closed);
        frame.render_stateful_widget(list, list_area, list_state);
    }

    // Render detail panel (if visible)
    if detail_area.width > 0 {
        let tree_order = build_tree_order(beads, hide_closed, filter);
        let selected_bead = list_state
            .selected()
            .and_then(|i| tree_order.get(i).map(|(b, _)| *b));
        let detail = DetailPanel::new(selected_bead, theme).focused(focus == Focus::Detail);
        frame.render_stateful_widget(detail, detail_area, detail_state);
    }

    // Render footer
    render_footer(
        frame,
        chunks[1],
        theme,
        input_mode,
        search_text,
        search_cursor,
        hide_closed,
        show_detail,
        focus,
    );

    // Render help overlay if needed
    if show_help {
        render_help_overlay(frame, area, theme);
    }

    // Render create modal if in creating mode
    if input_mode == InputMode::Creating {
        render_create_modal(frame, area, theme, create_modal);
    }

    // Render reason input modal if closing or reopening
    if input_mode == InputMode::ClosingBead {
        render_reason_modal(frame, area, theme, "Close Bead", reason_text, reason_cursor);
    } else if input_mode == InputMode::ReopeningBead {
        render_reason_modal(
            frame,
            area,
            theme,
            "Reopen Bead",
            reason_text,
            reason_cursor,
        );
    }

    (list_area, detail_area)
}

fn render_footer(
    frame: &mut ratatui::Frame,
    area: Rect,
    theme: &Theme,
    input_mode: InputMode,
    input_text: &str,
    input_cursor: usize,
    hide_closed: bool,
    show_detail: bool,
    focus: Focus,
) {
    // Lazygit-style footer: "Key: desc | Key: desc | ..."
    let closed_label = if hide_closed {
        "show closed"
    } else {
        "hide closed"
    };
    let keys: Vec<(&str, &str)> = match input_mode {
        InputMode::Search => vec![("Esc", "cancel"), ("Enter", "confirm")],
        InputMode::Creating => vec![("Esc", "cancel"), ("Tab", "next field"), ("C-s", "create")],
        InputMode::ClosingBead | InputMode::ReopeningBead => {
            vec![("Esc", "cancel"), ("Enter", "confirm")]
        }
        InputMode::Normal if show_detail && focus == Focus::Detail => vec![
            ("j/k", "scroll"),
            ("Esc/h", "close"),
            ("x", "close/reopen"),
            ("?", "help"),
            ("q", "quit"),
        ],
        InputMode::Normal => vec![
            ("j/k", "nav"),
            ("Enter/l", "open"),
            ("a", "add"),
            ("c", closed_label),
            ("/", "filter"),
            ("?", "help"),
            ("q", "quit"),
        ],
    };

    let mut spans: Vec<Span> = Vec::new();

    for (i, (key, desc)) in keys.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" | ", Style::default().fg(theme.border)));
        }
        spans.push(Span::styled(
            key.to_string(),
            Style::default().fg(theme.accent),
        ));
        spans.push(Span::styled(
            format!(": {}", desc),
            Style::default().fg(theme.muted),
        ));
    }

    // Show input text if in search mode
    if input_mode == InputMode::Search {
        spans.push(Span::styled("  |  ", Style::default().fg(theme.border)));
        spans.push(Span::styled("/", Style::default().fg(theme.accent)));

        // Show text with cursor
        let (before, after) = input_text.split_at(input_cursor.min(input_text.len()));
        spans.push(Span::styled(
            before.to_string(),
            Style::default().fg(theme.fg),
        ));
        spans.push(Span::styled(
            "\u{2588}".to_string(), // Block cursor
            Style::default().fg(theme.accent),
        ));
        spans.push(Span::styled(
            after.to_string(),
            Style::default().fg(theme.fg),
        ));
    } else if input_mode == InputMode::Normal && !input_text.is_empty() {
        // Show active filter
        spans.push(Span::styled("  |  ", Style::default().fg(theme.border)));
        spans.push(Span::styled(
            format!("filter: {}", input_text),
            Style::default().fg(theme.fg),
        ));
    }

    // Calculate left side text width to see if we have room for version
    let left_text = Line::from(spans.clone());
    let left_width = left_text.width() as u16;

    // Version info (right-aligned if there's room)
    let version = env!("CARGO_PKG_VERSION");
    let version_text = format!("beads-tui {}", version);
    let version_width = version_text.len() as u16;

    // Only show version if there's at least 5 chars of padding between left and right
    if left_width + version_width + 5 <= area.width {
        let padding_width = area.width.saturating_sub(left_width + version_width);
        spans.push(Span::raw(" ".repeat(padding_width as usize)));
        spans.push(Span::styled(version_text, Style::default().fg(theme.muted)));
    }

    let footer = Paragraph::new(Line::from(spans));
    frame.render_widget(footer, area);
}

fn render_help_overlay(frame: &mut ratatui::Frame, area: Rect, theme: &Theme) {
    // Center a help box
    let help_width = 50.min(area.width.saturating_sub(4));
    let help_height = 18.min(area.height.saturating_sub(4));
    let x = (area.width - help_width) / 2;
    let y = (area.height - help_height) / 2;
    let help_area = Rect::new(x, y, help_width, help_height);

    // Clear the area
    frame.render_widget(Clear, help_area);

    let help_text = vec![
        Line::from(vec![Span::styled(
            "Keyboard Shortcuts",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("j/k          ", Style::default().fg(theme.accent)),
            Span::raw("Move up/down"),
        ]),
        Line::from(vec![
            Span::styled("u/d, b/f     ", Style::default().fg(theme.accent)),
            Span::raw("Page up/down (10 lines)"),
        ]),
        Line::from(vec![
            Span::styled("g/G          ", Style::default().fg(theme.accent)),
            Span::raw("First/last item"),
        ]),
        Line::from(vec![
            Span::styled("Tab          ", Style::default().fg(theme.accent)),
            Span::raw("Switch focus"),
        ]),
        Line::from(vec![
            Span::styled("a            ", Style::default().fg(theme.accent)),
            Span::raw("Add new task"),
        ]),
        Line::from(vec![
            Span::styled("x            ", Style::default().fg(theme.accent)),
            Span::raw("Close/reopen (detail pane)"),
        ]),
        Line::from(vec![
            Span::styled("c            ", Style::default().fg(theme.accent)),
            Span::raw("Toggle closed"),
        ]),
        Line::from(vec![
            Span::styled("/            ", Style::default().fg(theme.accent)),
            Span::raw("Filter"),
        ]),
        Line::from(vec![
            Span::styled("r            ", Style::default().fg(theme.accent)),
            Span::raw("Refresh"),
        ]),
        Line::from(vec![
            Span::styled("t            ", Style::default().fg(theme.accent)),
            Span::raw("Cycle theme"),
        ]),
        Line::from(vec![
            Span::styled("q            ", Style::default().fg(theme.accent)),
            Span::raw("Quit"),
        ]),
        Line::raw(""),
        Line::from(vec![Span::styled(
            "Mouse: click to select, wheel to scroll",
            Style::default().fg(theme.muted),
        )]),
        Line::raw(""),
        Line::from(vec![Span::styled(
            "Press any key to close",
            Style::default().fg(theme.muted),
        )]),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .border_style(Style::default().fg(theme.accent))
                .title(" Help ")
                .title_style(Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
        )
        .style(Style::default().bg(theme.bg));

    frame.render_widget(help, help_area);
}

fn render_reason_modal(
    frame: &mut ratatui::Frame,
    area: Rect,
    theme: &Theme,
    title: &str,
    text: &str,
    cursor: usize,
) {
    // Center a modal
    let width = 60.min(area.width.saturating_sub(4));
    let height = 7;
    let x = (area.width - width) / 2;
    let y = (area.height - height) / 2;
    let modal_area = Rect::new(x, y, width, height);

    // Clear the area
    frame.render_widget(Clear, modal_area);

    // Split into input area and hint area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input field
            Constraint::Length(1), // Padding
            Constraint::Length(1), // Hint
        ])
        .split(modal_area);

    // Input text with cursor
    let (before, after) = text.split_at(cursor.min(text.len()));
    let input_spans = vec![
        Span::raw(before),
        Span::styled("\u{2588}", Style::default().fg(theme.accent)), // Block cursor
        Span::raw(after),
    ];

    let input = Paragraph::new(Line::from(input_spans))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .border_style(Style::default().fg(theme.accent))
                .title(format!(" {} ", title))
                .title_style(Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
        )
        .style(Style::default().bg(theme.bg).fg(theme.fg));

    frame.render_widget(input, chunks[0]);

    // Hint text
    let hint = Paragraph::new(Line::from(vec![
        Span::styled("Enter", Style::default().fg(theme.accent)),
        Span::raw(" to confirm | "),
        Span::styled("Esc", Style::default().fg(theme.accent)),
        Span::raw(" to cancel"),
    ]))
    .style(Style::default().fg(theme.muted));

    frame.render_widget(hint, chunks[2]);
}
