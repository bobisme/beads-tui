//! Bead list widget

#![allow(dead_code)]

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget},
};

use crate::data::{Bead, BeadStatus};
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
}

impl<'a> BeadList<'a> {
    pub fn new(beads: &'a [Bead], theme: &'a Theme) -> Self {
        Self {
            beads,
            theme,
            focused: true,
            filter: None,
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

    fn status_style(&self, status: &BeadStatus) -> Style {
        let color = match status {
            BeadStatus::Open => self.theme.status_open,
            BeadStatus::InProgress => self.theme.status_in_progress,
            BeadStatus::Blocked => self.theme.status_blocked,
            BeadStatus::Closed => self.theme.status_closed,
        };
        Style::default().fg(color)
    }

    fn priority_style(&self, priority: u8) -> Style {
        Style::default().fg(self.theme.priority_color(priority))
    }

    fn render_bead(&self, bead: &Bead) -> ListItem<'static> {
        let status_icon = bead.status.icon();
        let status_style = self.status_style(&bead.status);
        let priority_style = self.priority_style(bead.priority);

        let line = Line::from(vec![
            Span::styled(format!("{} ", status_icon), status_style),
            Span::styled(
                format!("P{} ", bead.priority),
                priority_style.add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("[{}] ", bead.bead_type),
                Style::default().fg(self.theme.muted),
            ),
            Span::styled(bead.id.clone(), Style::default().fg(self.theme.muted)),
            Span::raw(": "),
            Span::styled(bead.title.clone(), Style::default().fg(self.theme.fg)),
        ]);

        ListItem::new(line)
    }
}

impl<'a> StatefulWidget for BeadList<'a> {
    type State = BeadListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let items: Vec<ListItem<'static>> = self
            .beads
            .iter()
            .filter(|b| {
                self.filter
                    .map(|f| b.title.to_lowercase().contains(&f.to_lowercase()))
                    .unwrap_or(true)
            })
            .map(|b| self.render_bead(b))
            .collect();

        let border_style = if self.focused {
            Style::default().fg(self.theme.accent)
        } else {
            Style::default().fg(self.theme.border)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(" Beads ")
            .title_style(
                Style::default()
                    .fg(self.theme.fg)
                    .add_modifier(Modifier::BOLD),
            );

        let highlight_style = Style::default()
            .bg(self.theme.selection_bg)
            .fg(self.theme.selection_fg)
            .add_modifier(Modifier::BOLD);

        let list = List::new(items)
            .block(block)
            .highlight_style(highlight_style)
            .highlight_symbol("> ");

        StatefulWidget::render(list, area, buf, &mut state.list_state);
    }
}
