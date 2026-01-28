//! Bead list widget

#![allow(dead_code)]

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget},
};

use crate::data::{build_tree_order, Bead, BeadStatus};
use crate::ui::Theme;

/// State for the bead list
#[derive(Debug, Default)]
pub struct BeadListState {
    list_state: ListState,
    offset: usize,
}

impl BeadListState {
    pub fn new() -> Self {
        let mut state = Self::default();
        state.list_state.select(Some(0));
        state
    }

    pub fn selected(&self) -> Option<usize> {
        self.list_state.selected()
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.list_state.select(index);
    }

    pub fn next(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn first(&mut self) {
        self.list_state.select(Some(0));
    }

    pub fn last(&mut self, len: usize) {
        if len > 0 {
            self.list_state.select(Some(len - 1));
        }
    }
}

/// A list widget for displaying beads
pub struct BeadList<'a> {
    beads: &'a [Bead],
    theme: &'a Theme,
    focused: bool,
    filter: Option<&'a str>,
    hide_closed: bool,
    show_labels: bool,
}

impl<'a> BeadList<'a> {
    pub fn new(beads: &'a [Bead], theme: &'a Theme) -> Self {
        Self {
            beads,
            theme,
            focused: true,
            filter: None,
            hide_closed: false,
            show_labels: false,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn filter(mut self, filter: Option<&'a str>) -> Self {
        self.filter = filter;
        self
    }

    pub fn hide_closed(mut self, hide: bool) -> Self {
        self.hide_closed = hide;
        self
    }

    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    /// Get color for the combined type+status icon
    fn type_status_color(&self, status: &BeadStatus) -> ratatui::style::Color {
        match status {
            BeadStatus::Open => self.theme.status_open,
            BeadStatus::InProgress => self.theme.status_in_progress,
            BeadStatus::Blocked => self.theme.status_blocked,
            BeadStatus::Closed => self.theme.status_closed,
        }
    }

    fn priority_style(&self, priority: u8) -> Style {
        Style::default().fg(self.theme.priority_color(priority))
    }

    fn render_bead(&self, bead: &Bead, depth: usize) -> ListItem<'static> {
        // Combined type+status icon: shape = type, color = status
        let type_icon = bead.bead_type.icon_for_status(&bead.status);
        let icon_color = self.type_status_color(&bead.status);
        let priority_style = self.priority_style(bead.priority);

        // Indentation: 2 spaces per depth level
        let indent = "  ".repeat(depth);

        let mut spans = vec![
            Span::raw(indent),
            Span::styled(format!("{} ", type_icon), Style::default().fg(icon_color)),
            Span::styled(
                format!("P{} ", bead.priority),
                priority_style.add_modifier(Modifier::BOLD),
            ),
            Span::styled(bead.id.clone(), Style::default().fg(self.theme.muted)),
            Span::raw(": "),
            Span::styled(bead.title.clone(), Style::default().fg(self.theme.fg)),
        ];

        if self.show_labels && !bead.labels.is_empty() {
            spans.push(Span::raw(" "));
            for (idx, label) in bead.labels.iter().enumerate() {
                if idx > 0 {
                    spans.push(Span::raw(" "));
                }
                spans.push(Span::styled(
                    format!("[{}]", label),
                    Style::default().fg(self.theme.muted),
                ));
            }
        }

        let line = Line::from(spans);

        ListItem::new(line)
    }
}

impl<'a> StatefulWidget for BeadList<'a> {
    type State = BeadListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Build tree-ordered list with depths
        let tree_order = build_tree_order(self.beads, self.hide_closed, self.filter);
        let items: Vec<ListItem<'static>> = tree_order
            .iter()
            .map(|(b, depth)| self.render_bead(b, *depth))
            .collect();

        let border_style = if self.focused {
            Style::default().fg(self.theme.focused_border)
        } else {
            Style::default().fg(self.theme.border)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(border_style)
            .title(" Beads ")
            .title_style(
                Style::default()
                    .fg(self.theme.fg)
                    .add_modifier(Modifier::BOLD),
            );

        // Only set background for highlight - preserve span foreground colors
        let highlight_style = Style::default().bg(self.theme.selection_bg);

        let list = List::new(items)
            .block(block)
            .highlight_style(highlight_style);

        StatefulWidget::render(list, area, buf, &mut state.list_state);
    }
}
