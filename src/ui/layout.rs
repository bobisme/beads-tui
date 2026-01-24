//! Main layout for beads-tui

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::data::Bead;
use crate::ui::detail::DetailPanel;
use crate::ui::list::{BeadList, BeadListState};
use crate::ui::Theme;

/// Which pane is currently focused
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    List,
    Detail,
}

/// Render the main application layout
pub fn render_layout(
    frame: &mut ratatui::Frame,
    beads: &[Bead],
    list_state: &mut BeadListState,
    theme: &Theme,
    focus: Focus,
    split_percent: u16,
    filter: Option<&str>,
    show_help: bool,
) {
    let area = frame.area();

    // Main vertical layout: header, content, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Status bar
            Constraint::Min(3),    // Main content
            Constraint::Length(1), // Footer
        ])
        .split(area);

    // Render status bar
    render_status_bar(frame, chunks[0], beads, theme, filter);

    // Render main content (two panes)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(split_percent),
            Constraint::Percentage(100 - split_percent),
        ])
        .split(chunks[1]);

    // Render bead list
    let list = BeadList::new(beads, theme)
        .focused(focus == Focus::List)
        .filter(filter);
    frame.render_stateful_widget(list, content_chunks[0], list_state);

    // Render detail panel
    let selected_bead = list_state.selected().and_then(|i| {
        // Filter beads same way as list does to get correct index
        beads
            .iter()
            .filter(|b| {
                filter
                    .map(|f| b.title.to_lowercase().contains(&f.to_lowercase()))
                    .unwrap_or(true)
            })
            .nth(i)
    });
    let detail = DetailPanel::new(selected_bead, theme).focused(focus == Focus::Detail);
    frame.render_widget(detail, content_chunks[1]);

    // Render footer
    render_footer(frame, chunks[2], theme, filter.is_some());

    // Render help overlay if needed
    if show_help {
        render_help_overlay(frame, area, theme);
    }
}

fn render_status_bar(
    frame: &mut ratatui::Frame,
    area: Rect,
    beads: &[Bead],
    theme: &Theme,
    filter: Option<&str>,
) {
    let total = beads.len();
    let open = beads
        .iter()
        .filter(|b| b.status == crate::data::BeadStatus::Open)
        .count();
    let in_progress = beads
        .iter()
        .filter(|b| b.status == crate::data::BeadStatus::InProgress)
        .count();
    let blocked = beads
        .iter()
        .filter(|b| b.status == crate::data::BeadStatus::Blocked)
        .count();
    let closed = beads
        .iter()
        .filter(|b| b.status == crate::data::BeadStatus::Closed)
        .count();

    let mut spans = vec![
        Span::styled(
            " bu ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {} beads ", total), Style::default().fg(theme.fg)),
        Span::styled("\u{2502} ", Style::default().fg(theme.border)),
        Span::styled(
            format!("\u{25cf}{} ", open),
            Style::default().fg(theme.status_open),
        ),
        Span::styled(
            format!("\u{25d0}{} ", in_progress),
            Style::default().fg(theme.status_in_progress),
        ),
        Span::styled(
            format!("\u{26d4}{} ", blocked),
            Style::default().fg(theme.status_blocked),
        ),
        Span::styled(
            format!("\u{2714}{}", closed),
            Style::default().fg(theme.status_closed),
        ),
    ];

    if let Some(f) = filter {
        spans.push(Span::styled(
            format!(" \u{2502} Filter: {}", f),
            Style::default().fg(theme.accent),
        ));
    }

    let status = Paragraph::new(Line::from(spans)).style(Style::default().bg(theme.selection_bg));
    frame.render_widget(status, area);
}

fn render_footer(frame: &mut ratatui::Frame, area: Rect, theme: &Theme, filtering: bool) {
    let keys = if filtering {
        vec![("Esc", "clear"), ("Enter", "confirm")]
    } else {
        vec![
            ("j/k", "nav"),
            ("Enter", "detail"),
            ("Tab", "focus"),
            ("n", "new"),
            ("s", "status"),
            ("/", "search"),
            ("?", "help"),
            ("q", "quit"),
        ]
    };

    let spans: Vec<Span> = keys
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(
                    format!(" {} ", key),
                    Style::default()
                        .fg(theme.bg)
                        .bg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("{} ", desc), Style::default().fg(theme.muted)),
            ]
        })
        .collect();

    let footer = Paragraph::new(Line::from(spans));
    frame.render_widget(footer, area);
}

fn render_help_overlay(frame: &mut ratatui::Frame, area: Rect, theme: &Theme) {
    // Center a help box
    let help_width = 50.min(area.width.saturating_sub(4));
    let help_height = 16.min(area.height.saturating_sub(4));
    let x = (area.width - help_width) / 2;
    let y = (area.height - help_height) / 2;
    let help_area = Rect::new(x, y, help_width, help_height);

    // Clear the area
    let clear = Block::default().style(Style::default().bg(theme.bg));
    frame.render_widget(clear, help_area);

    let help_text = vec![
        Line::from(vec![Span::styled(
            "Keyboard Shortcuts",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::raw(""),
        Line::from(vec![
            Span::styled(
                "j/k, \u{2191}/\u{2193}  ",
                Style::default().fg(theme.accent),
            ),
            Span::raw("Move up/down"),
        ]),
        Line::from(vec![
            Span::styled("g/G          ", Style::default().fg(theme.accent)),
            Span::raw("First/last item"),
        ]),
        Line::from(vec![
            Span::styled("Enter        ", Style::default().fg(theme.accent)),
            Span::raw("Toggle detail panel"),
        ]),
        Line::from(vec![
            Span::styled("Tab          ", Style::default().fg(theme.accent)),
            Span::raw("Switch focus"),
        ]),
        Line::from(vec![
            Span::styled("n            ", Style::default().fg(theme.accent)),
            Span::raw("New bead"),
        ]),
        Line::from(vec![
            Span::styled("s            ", Style::default().fg(theme.accent)),
            Span::raw("Change status"),
        ]),
        Line::from(vec![
            Span::styled("/            ", Style::default().fg(theme.accent)),
            Span::raw("Search/filter"),
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
            Span::styled("q, Ctrl-C    ", Style::default().fg(theme.accent)),
            Span::raw("Quit"),
        ]),
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
                .border_style(Style::default().fg(theme.accent))
                .title(" Help ")
                .title_style(Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
        )
        .style(Style::default().bg(theme.bg));

    frame.render_widget(help, help_area);
}
