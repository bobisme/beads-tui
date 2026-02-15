//! Application state and main loop

use std::io::{self, IsTerminal, Stdout};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossterm::{
    event::{
        DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use ratatui::{Terminal, backend::CrosstermBackend, layout::Rect};

use crate::data::{Bead, BeadStatus, BeadStore, BrCli, build_tree_order};
use crate::event;
use crate::ui::layout::Focus;
use crate::ui::{
    BeadListState, CreateModal, DetailState, ModalAction, THEMES, Theme, render_layout,
};
use tui_textarea::TextArea;

/// Input mode for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Search,
    Creating,
    Editing,
    ClosingBead,
    ReopeningBead,
    AddingComment,
}

const MIN_SPLIT_PERCENT: u16 = 20;
const MAX_SPLIT_PERCENT: u16 = 80;

/// Application state
pub struct App {
    /// Path to the beads database
    db_path: PathBuf,
    /// All loaded beads
    beads: Vec<Bead>,
    /// List widget state
    list_state: BeadListState,
    /// Detail panel state (scroll position)
    detail_state: DetailState,
    /// Current theme index
    theme_idx: usize,
    /// Current focus
    focus: Focus,
    /// Split percentage (left pane width)
    split_percent: u16,
    /// Current input mode
    input_mode: InputMode,
    /// Text input for search
    search_input: TextArea<'static>,
    /// Create modal state
    create_modal: CreateModal,
    /// ID of bead being edited (if in Editing mode)
    editing_bead_id: Option<String>,
    /// Reason input for closing/reopening beads
    reason_input: TextArea<'static>,
    /// Comment input for adding comments
    comment_input: TextArea<'static>,
    /// Show labels in list view
    show_labels: bool,
    /// Show help overlay
    show_help: bool,
    /// Hide closed beads
    hide_closed: bool,
    /// Show detail pane
    show_detail: bool,
    /// Should the app quit
    should_quit: bool,
    /// Refresh interval
    refresh_interval: Duration,
    /// Last refresh time
    last_refresh: Instant,
    /// Layout areas for mouse handling
    list_area: Rect,
    detail_area: Rect,
    /// Whether the pane split is currently being dragged with the mouse
    split_resize_active: bool,
}

impl App {
    /// Create a new app instance
    pub fn new(db_path: PathBuf, refresh_secs: u64) -> Result<Self> {
        let store = BeadStore::open(&db_path)?;
        let beads = store.load_all()?;

        Ok(Self {
            db_path,
            beads,
            list_state: BeadListState::new(),
            detail_state: DetailState::new(),
            theme_idx: 0,
            focus: Focus::List,
            split_percent: 40,
            input_mode: InputMode::Normal,
            search_input: TextArea::default(),
            create_modal: CreateModal::new(),
            editing_bead_id: None,
            reason_input: TextArea::default(),
            comment_input: TextArea::default(),
            show_labels: true,
            show_help: false,
            hide_closed: true,  // Start with closed beads hidden
            show_detail: false, // Start with only list visible
            should_quit: false,
            refresh_interval: Duration::from_secs(refresh_secs),
            last_refresh: Instant::now(),
            list_area: Rect::default(),
            detail_area: Rect::default(),
            split_resize_active: false,
        })
    }

    /// Get the current theme
    fn theme(&self) -> &Theme {
        &THEMES[self.theme_idx]
    }

    /// Reload beads from database
    fn refresh(&mut self) -> Result<()> {
        let store = BeadStore::open(&self.db_path)?;
        self.beads = store.load_all()?;
        self.last_refresh = Instant::now();
        Ok(())
    }

    /// Get the current filter text (if searching or has active filter)
    fn filter(&self) -> Option<String> {
        let text = self.search_input.lines().join("\n");
        if text.is_empty() { None } else { Some(text) }
    }

    /// Get filtered beads count (uses tree order for consistency)
    fn filtered_len(&self) -> usize {
        build_tree_order(&self.beads, self.hide_closed, self.filter().as_deref()).len()
    }

    /// Handle a key event
    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        // Help overlay takes precedence
        if self.show_help {
            self.show_help = false;
            return Ok(());
        }

        // Input mode handling (search, create, close/reopen)
        match self.input_mode {
            InputMode::Search => {
                match key.code {
                    KeyCode::Esc => {
                        self.input_mode = InputMode::Normal;
                        self.search_input = TextArea::default();
                    }
                    KeyCode::Enter => {
                        self.input_mode = InputMode::Normal;
                        // Keep the filter text in search_input
                    }
                    _ => {
                        let old_len = self.search_input.lines().join("\n").len();
                        self.search_input.input(key);
                        // Reset selection when filter changes
                        if self.search_input.lines().join("\n").len() != old_len {
                            self.list_state.first();
                        }
                    }
                }
                return Ok(());
            }
            InputMode::Creating => {
                match self.create_modal.handle_key(key) {
                    ModalAction::Submit => {
                        self.create_bead()?;
                        self.input_mode = InputMode::Normal;
                        self.create_modal.close();
                        self.editing_bead_id = None;
                    }
                    ModalAction::Cancelled => {
                        self.input_mode = InputMode::Normal;
                        self.editing_bead_id = None;
                    }
                    ModalAction::None => {}
                }
                return Ok(());
            }
            InputMode::Editing => {
                match self.create_modal.handle_key(key) {
                    ModalAction::Submit => {
                        self.update_bead()?;
                        self.input_mode = InputMode::Normal;
                        self.create_modal.close();
                        self.editing_bead_id = None;
                    }
                    ModalAction::Cancelled => {
                        self.input_mode = InputMode::Normal;
                        self.editing_bead_id = None;
                    }
                    ModalAction::None => {}
                }
                return Ok(());
            }
            InputMode::ClosingBead => {
                match key.code {
                    KeyCode::Esc => {
                        self.input_mode = InputMode::Normal;
                        self.reason_input = TextArea::default();
                    }
                    KeyCode::Enter if !shift => {
                        self.close_bead()?;
                        self.input_mode = InputMode::Normal;
                        self.reason_input = TextArea::default();
                    }
                    _ => {
                        self.reason_input.input(key);
                    }
                }
                return Ok(());
            }
            InputMode::ReopeningBead => {
                match key.code {
                    KeyCode::Esc => {
                        self.input_mode = InputMode::Normal;
                        self.reason_input = TextArea::default();
                    }
                    KeyCode::Enter if !shift => {
                        self.reopen_bead()?;
                        self.input_mode = InputMode::Normal;
                        self.reason_input = TextArea::default();
                    }
                    _ => {
                        self.reason_input.input(key);
                    }
                }
                return Ok(());
            }
            InputMode::AddingComment => {
                match key.code {
                    KeyCode::Esc => {
                        self.input_mode = InputMode::Normal;
                        self.comment_input = TextArea::default();
                    }
                    KeyCode::Enter if !shift => {
                        self.add_comment()?;
                        self.input_mode = InputMode::Normal;
                        self.comment_input = TextArea::default();
                    }
                    _ => {
                        self.comment_input.input(key);
                    }
                }
                return Ok(());
            }
            InputMode::Normal => {}
        }

        // Normal mode
        match key.code {
            // Quit
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if ctrl => {
                self.should_quit = true;
            }

            // Suspend (Ctrl+Z)
            KeyCode::Char('z') if ctrl => {
                return Err(anyhow::anyhow!("__SUSPEND__"));
            }

            // Navigation - single line (focus-aware)
            KeyCode::Up | KeyCode::Char('k') if !ctrl => match self.focus {
                Focus::List => self.list_state.previous(self.filtered_len()),
                Focus::Detail => self.detail_state.scroll_up(1),
            },
            KeyCode::Down | KeyCode::Char('j') if !ctrl => match self.focus {
                Focus::List => self.list_state.next(self.filtered_len()),
                Focus::Detail => self.detail_state.scroll_down(1),
            },

            // Navigation - page (10 lines, focus-aware)
            KeyCode::Char('u') | KeyCode::Char('b') => match self.focus {
                Focus::List => self.scroll_up(10),
                Focus::Detail => self.detail_state.scroll_up(10),
            },
            KeyCode::Char('d') | KeyCode::Char('f') => match self.focus {
                Focus::List => self.scroll_down(10),
                Focus::Detail => self.detail_state.scroll_down(10),
            },
            KeyCode::Char('k') if ctrl => match self.focus {
                Focus::List => self.scroll_up(10),
                Focus::Detail => self.detail_state.scroll_up(10),
            },
            KeyCode::Char('j') if ctrl => match self.focus {
                Focus::List => self.scroll_down(10),
                Focus::Detail => self.detail_state.scroll_down(10),
            },
            KeyCode::PageUp => match self.focus {
                Focus::List => self.scroll_up(10),
                Focus::Detail => self.detail_state.scroll_up(10),
            },
            KeyCode::PageDown => match self.focus {
                Focus::List => self.scroll_down(10),
                Focus::Detail => self.detail_state.scroll_down(10),
            },

            // Navigation - first/last
            KeyCode::Home | KeyCode::Char('g') => match self.focus {
                Focus::List => self.list_state.first(),
                Focus::Detail => self.detail_state.reset(),
            },
            KeyCode::End | KeyCode::Char('G') => match self.focus {
                Focus::List => self.list_state.last(self.filtered_len()),
                Focus::Detail => {
                    // Scroll to a very large number - ratatui will clamp it
                    self.detail_state.scroll_down(10000);
                }
            },

            // Open detail pane
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right if self.focus == Focus::List => {
                self.show_detail = true;
                self.focus = Focus::Detail;
                self.detail_state.reset();
            }

            // Close detail pane
            KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left if self.focus == Focus::Detail => {
                self.show_detail = false;
                self.focus = Focus::List;
            }

            // Focus toggle (only when detail is shown)
            KeyCode::Tab if self.show_detail => {
                self.focus = match self.focus {
                    Focus::List => Focus::Detail,
                    Focus::Detail => Focus::List,
                };
                if self.focus == Focus::Detail {
                    self.detail_state.reset();
                }
            }

            // Pane resizing (only when detail is shown)
            KeyCode::Char('<') if self.show_detail => {
                self.split_percent = self
                    .split_percent
                    .saturating_sub(5)
                    .clamp(MIN_SPLIT_PERCENT, MAX_SPLIT_PERCENT);
            }
            KeyCode::Char('>') if self.show_detail => {
                self.split_percent = self
                    .split_percent
                    .saturating_add(5)
                    .clamp(MIN_SPLIT_PERCENT, MAX_SPLIT_PERCENT);
            }

            // Search
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.search_input = TextArea::default();
            }

            // Clear filter (when list focused or no detail)
            KeyCode::Esc if self.focus == Focus::List => {
                self.search_input = TextArea::default();
            }

            // Add new bead
            KeyCode::Char('a') => {
                self.input_mode = InputMode::Creating;
                self.editing_bead_id = None;
                self.create_modal.open();
            }

            // Edit selected bead
            KeyCode::Char('e') if self.focus == Focus::Detail => {
                // Clone the bead to avoid borrow issues
                if let Some(bead) = self.get_selected_bead().cloned() {
                    self.input_mode = InputMode::Editing;
                    self.editing_bead_id = Some(bead.id.clone());
                    self.create_modal.open_with_bead(&bead);
                }
            }

            // Theme
            KeyCode::Char('t') => {
                self.theme_idx = (self.theme_idx + 1) % THEMES.len();
            }

            // Toggle labels in list view
            KeyCode::Char('L') => {
                self.show_labels = !self.show_labels;
            }

            // Refresh
            KeyCode::Char('r') => {
                self.refresh()?;
            }

            // Help
            KeyCode::Char('?') => {
                self.show_help = true;
            }

            // Close/reopen bead (only when detail pane is focused)
            KeyCode::Char('x') if self.focus == Focus::Detail => {
                if let Some(bead) = self.get_selected_bead() {
                    if bead.status == BeadStatus::Closed {
                        // Reopen the bead
                        self.input_mode = InputMode::ReopeningBead;
                        self.reason_input = TextArea::default();
                    } else {
                        // Close the bead
                        self.input_mode = InputMode::ClosingBead;
                        self.reason_input = TextArea::default();
                    }
                }
            }

            // Toggle deferred/open for selected bead (detail pane only)
            KeyCode::Char('D') if self.focus == Focus::Detail => {
                self.toggle_deferred()?;
            }

            // 'c' key - context dependent:
            // - List focused: toggle closed visibility
            // - Detail focused: add comment
            KeyCode::Char('c') if self.focus == Focus::List => {
                self.hide_closed = !self.hide_closed;
                // Clamp selection to new filtered length
                let len = self.filtered_len();
                if let Some(idx) = self.list_state.selected()
                    && idx >= len
                    && len > 0
                {
                    self.list_state.select(Some(len - 1));
                }
            }
            KeyCode::Char('c') if self.focus == Focus::Detail => {
                // Only allow comments if we have a selected bead
                if self.get_selected_bead().is_some() {
                    self.input_mode = InputMode::AddingComment;
                    self.comment_input = TextArea::default();
                }
            }

            _ => {}
        }

        Ok(())
    }

    /// Handle pasted text (bracketed paste mode)
    fn handle_paste(&mut self, text: &str) -> Result<()> {
        // Help overlay consumes the next interaction
        if self.show_help {
            self.show_help = false;
            return Ok(());
        }

        match self.input_mode {
            InputMode::Search => {
                let old_len = self.search_input.lines().join("\n").len();
                let single_line = text
                    .lines()
                    .map(str::trim_end)
                    .collect::<Vec<_>>()
                    .join(" ");
                let _ = self.search_input.insert_str(single_line);
                if self.search_input.lines().join("\n").len() != old_len {
                    self.list_state.first();
                }
            }
            InputMode::Creating | InputMode::Editing => {
                self.create_modal.handle_paste(text);
            }
            InputMode::ClosingBead | InputMode::ReopeningBead => {
                let _ = self.reason_input.insert_str(text);
            }
            InputMode::AddingComment => {
                let _ = self.comment_input.insert_str(text);
            }
            InputMode::Normal => {}
        }

        Ok(())
    }

    /// Scroll up by n lines
    fn scroll_up(&mut self, n: usize) {
        let len = self.filtered_len();
        if len == 0 {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let new_pos = current.saturating_sub(n);
        self.list_state.select(Some(new_pos));
    }

    /// Scroll down by n lines
    fn scroll_down(&mut self, n: usize) {
        let len = self.filtered_len();
        if len == 0 {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let new_pos = (current + n).min(len.saturating_sub(1));
        self.list_state.select(Some(new_pos));
    }

    /// Handle a mouse event
    fn handle_mouse(&mut self, mouse: MouseEvent) -> Result<()> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let x = mouse.column;
                let y = mouse.row;

                // Click/drag the divider between list and detail panes.
                if self.is_on_split_handle(x, y) {
                    self.split_resize_active = true;
                    self.update_split_from_mouse(x);
                    return Ok(());
                }

                self.split_resize_active = false;

                // Check which pane was clicked
                if self.list_area.contains((x, y).into()) {
                    // Calculate which item was clicked
                    let inner_y = y.saturating_sub(self.list_area.y + 1); // +1 for border
                    let idx = inner_y as usize;
                    if idx < self.filtered_len() {
                        self.list_state.select(Some(idx));
                        // Open detail pane on click
                        self.show_detail = true;
                        self.focus = Focus::Detail;
                        self.detail_state.reset();
                    }
                } else if self.detail_area.contains((x, y).into()) {
                    self.focus = Focus::Detail;
                }
            }
            MouseEventKind::Drag(MouseButton::Left) if self.split_resize_active => {
                self.update_split_from_mouse(mouse.column);
            }
            MouseEventKind::Up(_) => {
                self.split_resize_active = false;
            }
            MouseEventKind::ScrollUp => match self.focus {
                Focus::List => self.list_state.previous(self.filtered_len()),
                Focus::Detail => self.detail_state.scroll_up(3),
            },
            MouseEventKind::ScrollDown => match self.focus {
                Focus::List => self.list_state.next(self.filtered_len()),
                Focus::Detail => self.detail_state.scroll_down(3),
            },
            _ => {}
        }
        Ok(())
    }

    /// Returns true when both panes are visible and can be resized.
    fn is_split_resizable(&self) -> bool {
        self.list_area.width > 0 && self.detail_area.width > 0
    }

    /// Check whether the mouse is over the split border between list and detail panes.
    fn is_on_split_handle(&self, x: u16, y: u16) -> bool {
        if !self.is_split_resizable() {
            return false;
        }

        let top = self.list_area.y.min(self.detail_area.y);
        let bottom = self
            .list_area
            .y
            .saturating_add(self.list_area.height)
            .max(self.detail_area.y.saturating_add(self.detail_area.height));
        if y < top || y >= bottom {
            return false;
        }

        let list_right = self
            .list_area
            .x
            .saturating_add(self.list_area.width.saturating_sub(1));
        let detail_left = self.detail_area.x;

        x == list_right || x == detail_left
    }

    /// Update split percentage from a mouse X position.
    fn update_split_from_mouse(&mut self, x: u16) {
        if !self.is_split_resizable() {
            return;
        }

        let content_left = self.list_area.x;
        let total_width = self.list_area.width.saturating_add(self.detail_area.width);
        if total_width == 0 {
            return;
        }

        let content_right = content_left.saturating_add(total_width.saturating_sub(1));
        let clamped_x = x.clamp(content_left, content_right);

        // +1 so dragging on the current right edge preserves current split more naturally.
        let left_width = clamped_x.saturating_sub(content_left).saturating_add(1);
        let raw_percent = ((u32::from(left_width) * 100) / u32::from(total_width)) as u16;

        self.split_percent = raw_percent.clamp(MIN_SPLIT_PERCENT, MAX_SPLIT_PERCENT);
    }

    /// Get the currently selected bead
    fn get_selected_bead(&self) -> Option<&Bead> {
        let idx = self.list_state.selected()?;
        let tree_order = build_tree_order(&self.beads, self.hide_closed, self.filter().as_deref());
        tree_order.get(idx).map(|(bead, _)| *bead)
    }

    /// Toggle selected bead between open and deferred
    fn toggle_deferred(&mut self) -> Result<()> {
        if let Some(bead) = self.get_selected_bead() {
            let id = bead.id.clone();
            let next_status = match bead.status {
                BeadStatus::Open => Some("deferred"),
                BeadStatus::Deferred => Some("open"),
                _ => None,
            };

            if let Some(status) = next_status {
                BrCli::update_status(&id, status)?;
                self.refresh()?;
            }
        }

        Ok(())
    }

    /// Close the selected bead with a reason
    fn close_bead(&mut self) -> Result<()> {
        if let Some(bead) = self.get_selected_bead() {
            let id = bead.id.clone();
            let reason = self.reason_input.lines().join("\n");
            let reason_opt = if reason.is_empty() {
                None
            } else {
                Some(reason)
            };
            BrCli::close(&id, reason_opt.as_deref())?;
            self.refresh()?;
        }
        Ok(())
    }

    /// Reopen the selected bead with a reason
    fn reopen_bead(&mut self) -> Result<()> {
        if let Some(bead) = self.get_selected_bead() {
            let id = bead.id.clone();
            let reason = self.reason_input.lines().join("\n");
            let reason_opt = if reason.is_empty() {
                None
            } else {
                Some(reason)
            };
            // Use update_status to set back to open and add a comment with the reason
            BrCli::update_status(&id, "open")?;
            if let Some(r) = reason_opt {
                // Add the reason as a comment
                let _ = BrCli::add_comment(&id, &format!("Reopened: {}", r));
            }
            self.refresh()?;
        }
        Ok(())
    }

    /// Add a comment to the selected bead
    fn add_comment(&mut self) -> Result<()> {
        if let Some(bead) = self.get_selected_bead() {
            let id = bead.id.clone();
            let comment_text = self.comment_input.lines().join("\n");

            // Don't add empty comments
            if comment_text.trim().is_empty() {
                return Ok(());
            }

            BrCli::add_comment(&id, &comment_text)?;
            self.refresh()?;
        }
        Ok(())
    }

    /// Create a new bead from the create modal
    fn create_bead(&mut self) -> Result<()> {
        let title = self.create_modal.get_title().to_string();
        if title.is_empty() {
            return Ok(());
        }

        let description = self.create_modal.get_description().map(|s| s.to_string());
        let bead_type = self.create_modal.bead_type;
        let priority = self.create_modal.priority;
        let labels = self.create_modal.get_labels();

        // Create the bead
        let id = BrCli::create(&title, bead_type, priority, description.as_deref(), None)?;

        // Add labels if any
        if !labels.is_empty() && !id.is_empty() {
            for label in &labels {
                let _ = BrCli::add_label(&id, label);
            }
        }

        self.refresh()?;

        // Select the newly created bead (should be near the top after refresh)
        self.list_state.first();

        Ok(())
    }

    /// Update an existing bead from the create modal
    fn update_bead(&mut self) -> Result<()> {
        // Get the bead ID we're editing
        let id = match &self.editing_bead_id {
            Some(id) => id.clone(),
            None => return Ok(()), // Safety: shouldn't happen
        };

        // Find the original bead to compare
        let original = self.beads.iter().find(|b| b.id == id);
        if original.is_none() {
            return Ok(()); // Bead not found, nothing to update
        }
        let original = original.unwrap();

        // Get current values from modal
        let new_title = self.create_modal.get_title();
        let new_description = self.create_modal.get_description();
        let new_type = self.create_modal.bead_type;
        let new_priority = self.create_modal.priority;
        let new_labels: std::collections::HashSet<String> =
            self.create_modal.get_labels().into_iter().collect();
        let old_labels: std::collections::HashSet<String> =
            original.labels.iter().cloned().collect();

        // Build update command with only changed fields
        let mut updates_needed = false;

        // Check title
        if new_title != original.title {
            BrCli::update_field(&id, "title", &new_title)?;
            updates_needed = true;
        }

        // Check description
        let old_desc = original.description.as_deref().unwrap_or("");
        let new_desc_str = new_description.as_deref().unwrap_or("");
        if new_desc_str != old_desc {
            BrCli::update_field(&id, "description", new_desc_str)?;
            updates_needed = true;
        }

        // Check type
        if new_type != original.bead_type {
            BrCli::update_field(&id, "type", &new_type.to_string())?;
            updates_needed = true;
        }

        // Check priority
        if new_priority != original.priority {
            BrCli::update_field(&id, "priority", &new_priority.to_string())?;
            updates_needed = true;
        }

        // Handle labels: add new ones, remove old ones
        let labels_to_add: Vec<_> = new_labels.difference(&old_labels).collect();
        let labels_to_remove: Vec<_> = old_labels.difference(&new_labels).collect();

        for label in labels_to_add {
            BrCli::add_label(&id, label)?;
            updates_needed = true;
        }

        for label in labels_to_remove {
            BrCli::remove_label(&id, label)?;
            updates_needed = true;
        }

        // Refresh if we made any changes
        if updates_needed {
            self.refresh()?;
        }

        Ok(())
    }
}

/// Setup the terminal
fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    // Check if we have a TTY
    if !io::stdout().is_terminal() {
        anyhow::bail!("bu requires a terminal (TTY) to run. Cannot run in a pipe or background.");
    }

    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )
    .context("Failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend).context("Failed to create terminal")?;
    Ok(terminal)
}

/// Restore the terminal
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        DisableBracketedPaste
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Run the application
pub async fn run(db_path: PathBuf, refresh_secs: u64) -> Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = App::new(db_path, refresh_secs)?;

    let result = run_loop(&mut terminal, &mut app).await;

    restore_terminal(&mut terminal)?;

    result
}

/// Suspend the process (Ctrl+Z behavior)
fn suspend(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    // Restore terminal to normal state before suspending
    restore_terminal(terminal)?;

    // Send SIGTSTP to ourselves to suspend
    signal::kill(Pid::this(), Signal::SIGTSTP)?;

    // When we resume (after fg), re-setup the terminal
    // Note: setup_terminal creates a new terminal, but we need to reinitialize the existing one
    enable_raw_mode().context("Failed to enable raw mode after resume")?;
    execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )
    .context("Failed to enter alternate screen after resume")?;
    terminal.clear()?;

    Ok(())
}

async fn run_loop(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    let tick_rate = Duration::from_millis(100);

    loop {
        // Get values before drawing to avoid borrow issues
        let theme = app.theme().clone();
        let focus = app.focus;
        let split_percent = app.split_percent;
        let filter = app.filter().map(|s| s.to_string());
        let show_help = app.show_help;
        let hide_closed = app.hide_closed;
        let show_labels = app.show_labels;
        let show_detail = app.show_detail;
        let input_mode = app.input_mode;
        let search_text = app.search_input.lines().join("\n").to_string();
        let search_cursor = app.search_input.cursor().1; // Column position only
        let reason_text = app.reason_input.lines().join("\n").to_string();
        let reason_cursor = app.reason_input.cursor().1; // Column position only
        let comment_text = app.comment_input.lines().join("\n").to_string();
        let comment_cursor = app.comment_input.cursor().1; // Column position only

        // Draw
        terminal.draw(|frame| {
            let (list_area, detail_area) = render_layout(
                frame,
                &app.beads,
                &mut app.list_state,
                &mut app.detail_state,
                &theme,
                focus,
                split_percent,
                filter.as_deref(),
                show_help,
                hide_closed,
                show_labels,
                show_detail,
                input_mode,
                &search_text,
                search_cursor,
                &app.create_modal,
                &reason_text,
                reason_cursor,
                &comment_text,
                comment_cursor,
            );
            // Store areas for mouse handling
            app.list_area = list_area;
            app.detail_area = detail_area;
        })?;

        // Handle events
        if let Some(event) = event::poll_event(tick_rate)? {
            match event {
                Event::Key(key) => match app.handle_key(key) {
                    Ok(()) => {}
                    Err(e) if e.to_string() == "__SUSPEND__" => {
                        suspend(terminal)?;
                    }
                    Err(e) => return Err(e),
                },
                Event::Mouse(mouse) => {
                    app.handle_mouse(mouse)?;
                }
                Event::Paste(text) => {
                    app.handle_paste(&text)?;
                }
                _ => {}
            }
        }

        // Auto-refresh
        if app.refresh_interval.as_secs() > 0 && app.last_refresh.elapsed() >= app.refresh_interval
        {
            let _ = app.refresh();
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
