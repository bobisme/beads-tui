//! Detail panel widget for showing bead information

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use crate::data::{Bead, BeadStatus};
use crate::ui::Theme;

/// A panel showing detailed information about a bead
pub struct DetailPanel<'a> {
    bead: Option<&'a Bead>,
    theme: &'a Theme,
    focused: bool,
}

impl<'a> DetailPanel<'a> {
    pub fn new(bead: Option<&'a Bead>, theme: &'a Theme) -> Self {
        Self {
            bead,
            theme,
            focused: false,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    fn status_style(&self, status: &BeadStatus) -> Style {
        let color = match status {
            BeadStatus::Open => self.theme.status_open,
            BeadStatus::InProgress => self.theme.status_in_progress,
            BeadStatus::Blocked => self.theme.status_blocked,
            BeadStatus::Closed => self.theme.status_closed,
        };
        Style::default().fg(color)
    }

    fn render_metadata(&self, bead: &Bead) -> Text<'static> {
        let mut lines = Vec::new();

        // Title
        lines.push(Line::from(vec![Span::styled(
            bead.title.clone(),
            Style::default()
                .fg(self.theme.fg)
                .add_modifier(Modifier::BOLD),
        )]));

        lines.push(Line::raw(""));

        // ID and Status
        lines.push(Line::from(vec![
            Span::styled("ID: ", Style::default().fg(self.theme.muted)),
            Span::styled(bead.id.clone(), Style::default().fg(self.theme.accent)),
            Span::raw("  "),
            Span::styled("Status: ", Style::default().fg(self.theme.muted)),
            Span::styled(
                format!("{} {}", bead.status.icon(), bead.status),
                self.status_style(&bead.status),
            ),
        ]));

        // Type and Priority
        lines.push(Line::from(vec![
            Span::styled("Type: ", Style::default().fg(self.theme.muted)),
            Span::styled(
                bead.bead_type.to_string(),
                Style::default().fg(self.theme.fg),
            ),
            Span::raw("  "),
            Span::styled("Priority: ", Style::default().fg(self.theme.muted)),
            Span::styled(
                bead.priority_label(),
                Style::default()
                    .fg(self.theme.priority_color(bead.priority))
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        // Labels
        if !bead.labels.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Labels: ", Style::default().fg(self.theme.muted)),
                Span::styled(
                    bead.labels.join(", "),
                    Style::default().fg(self.theme.accent),
                ),
            ]));
        }

        // Assignee
        if let Some(ref assignee) = bead.assignee {
            lines.push(Line::from(vec![
                Span::styled("Assignee: ", Style::default().fg(self.theme.muted)),
                Span::styled(assignee.clone(), Style::default().fg(self.theme.fg)),
            ]));
        }

        lines.push(Line::raw(""));

        // Description
        if let Some(ref desc) = bead.description {
            lines.push(Line::from(vec![Span::styled(
                "Description:",
                Style::default()
                    .fg(self.theme.fg)
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::raw(""));
            for line in desc.lines() {
                lines.push(Line::raw(line.to_string()));
            }
        }

        // Dependencies section
        if !bead.blocked_by.is_empty() {
            lines.push(Line::raw(""));
            lines.push(Line::from(vec![Span::styled(
                "Blocked by:",
                Style::default()
                    .fg(self.theme.status_blocked)
                    .add_modifier(Modifier::BOLD),
            )]));
            for id in &bead.blocked_by {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("\u{2514}\u{2500} {}", id),
                        Style::default().fg(self.theme.status_blocked),
                    ),
                ]));
            }
        }

        if !bead.blocks.is_empty() {
            lines.push(Line::raw(""));
            lines.push(Line::from(vec![Span::styled(
                "Blocks:",
                Style::default()
                    .fg(self.theme.accent)
                    .add_modifier(Modifier::BOLD),
            )]));
            for id in &bead.blocks {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("\u{2514}\u{2500} {}", id),
                        Style::default().fg(self.theme.accent),
                    ),
                ]));
            }
        }

        if !bead.parent_ids.is_empty() {
            lines.push(Line::raw(""));
            lines.push(Line::from(vec![Span::styled(
                "Part of:",
                Style::default()
                    .fg(self.theme.muted)
                    .add_modifier(Modifier::BOLD),
            )]));
            for id in &bead.parent_ids {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("\u{2514}\u{2500} {}", id),
                        Style::default().fg(self.theme.muted),
                    ),
                ]));
            }
        }

        // Timestamps
        lines.push(Line::raw(""));
        if let Some(created) = bead.created_at {
            lines.push(Line::from(vec![
                Span::styled("Created: ", Style::default().fg(self.theme.muted)),
                Span::styled(
                    created.format("%Y-%m-%d %H:%M").to_string(),
                    Style::default().fg(self.theme.fg),
                ),
            ]));
        }
        if let Some(updated) = bead.updated_at {
            lines.push(Line::from(vec![
                Span::styled("Updated: ", Style::default().fg(self.theme.muted)),
                Span::styled(
                    updated.format("%Y-%m-%d %H:%M").to_string(),
                    Style::default().fg(self.theme.fg),
                ),
            ]));
        }

        Text::from(lines)
    }
}

impl Widget for DetailPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.focused {
            Style::default().fg(self.theme.accent)
        } else {
            Style::default().fg(self.theme.border)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(" Detail ")
            .title_style(
                Style::default()
                    .fg(self.theme.fg)
                    .add_modifier(Modifier::BOLD),
            );

        let inner = block.inner(area);
        block.render(area, buf);

        if let Some(bead) = self.bead {
            let text = self.render_metadata(bead);
            let para = Paragraph::new(text).wrap(Wrap { trim: false });
            para.render(inner, buf);
        } else {
            let text = Text::from(vec![Line::from(vec![Span::styled(
                "No bead selected",
                Style::default().fg(self.theme.muted),
            )])]);
            let para = Paragraph::new(text);
            para.render(inner, buf);
        }
    }
}
