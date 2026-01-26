//! Application state and main loop

use std::io::{self, IsTerminal, Stdout};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossterm::{
    event::{
        DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

use crate::data::{build_tree_order, Bead, BeadStatus, BeadStore, BrCli};
use crate::event;
use crate::ui::layout::Focus;
use crate::ui::{
    render_layout, BeadListState, CreateModal, DetailState, ModalAction, TextInput, Theme, THEMES,
};

/// Input mode for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Search,
    Creating,
    ClosingBead,
    ReopeningBead,
}

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
    search_input: TextInput,
    /// Create modal state
    create_modal: CreateModal,
    /// Reason input for closing/reopening beads
    reason_input: TextInput,
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
            search_input: TextInput::new(),
            create_modal: CreateModal::new(),
            reason_input: TextInput::new(),
            show_help: false,
            hide_closed: true, // Start with closed beads hidden
            show_detail: false, // Start with only list visible
            should_quit: false,
            refresh_interval: Duration::from_secs(refresh_secs),
            last_refresh: Instant::now(),
            list_area: Rect::default(),
            detail_area: Rect::default(),
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
    fn filter(&self) -> Option<&str> {
        if !self.search_input.is_empty() {
            Some(self.search_input.text())
        } else {
            None
        }
    }

    /// Get filtered beads count (uses tree order for consistency)
    fn filtered_len(&self) -> usize {
        build_tree_order(&self.beads, self.hide_closed, self.filter()).len()
    }

    /// Handle a key event
    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

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
                        self.search_input.clear();
                    }
                    KeyCode::Enter => {
                        self.input_mode = InputMode::Normal;
                        // Keep the filter text in search_input
                    }
                    _ => {
                        let old_len = self.search_input.text().len();
                        self.search_input.handle_key(key);
                        // Reset selection when filter changes
                        if self.search_input.text().len() != old_len {
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
                    }
                    ModalAction::Cancelled => {
                        self.input_mode = InputMode::Normal;
                    }
                    ModalAction::None => {}
                }
                return Ok(());
            }
            InputMode::ClosingBead => {
                match key.code {
                    KeyCode::Esc => {
                        self.input_mode = InputMode::Normal;
                        self.reason_input.clear();
                    }
                    KeyCode::Enter => {
                        self.close_bead()?;
                        self.input_mode = InputMode::Normal;
                        self.reason_input.clear();
                    }
                    _ => {
                        self.reason_input.handle_key(key);
                    }
                }
                return Ok(());
            }
            InputMode::ReopeningBead => {
                match key.code {
                    KeyCode::Esc => {
                        self.input_mode = InputMode::Normal;
                        self.reason_input.clear();
                    }
                    KeyCode::Enter => {
                        self.reopen_bead()?;
                        self.input_mode = InputMode::Normal;
                        self.reason_input.clear();
                    }
                    _ => {
                        self.reason_input.handle_key(key);
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
            KeyCode::End => {
                self.list_state.last(self.filtered_len());
            }
            KeyCode::Char('G') => {
                self.list_state.last(self.filtered_len());
            }

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
            KeyCode::Char('<') | KeyCode::Char('H') if self.show_detail => {
                self.split_percent = self.split_percent.saturating_sub(5).max(20);
            }
            KeyCode::Char('>') | KeyCode::Char('L') if self.show_detail => {
                self.split_percent = (self.split_percent + 5).min(80);
            }

            // Search
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.search_input.clear();
            }

            // Clear filter (when list focused or no detail)
            KeyCode::Esc if self.focus == Focus::List => {
                self.search_input.clear();
            }

            // Add new bead
            KeyCode::Char('a') => {
                self.input_mode = InputMode::Creating;
                self.create_modal.open();
            }

            // Theme
            KeyCode::Char('t') => {
                self.theme_idx = (self.theme_idx + 1) % THEMES.len();
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
                        self.reason_input.clear();
                    } else {
                        // Close the bead
                        self.input_mode = InputMode::ClosingBead;
                        self.reason_input.clear();
                    }
                }
            }

            // Toggle closed visibility
            KeyCode::Char('c') => {
                self.hide_closed = !self.hide_closed;
                // Clamp selection to new filtered length
                let len = self.filtered_len();
                if let Some(idx) = self.list_state.selected() {
                    if idx >= len && len > 0 {
                        self.list_state.select(Some(len - 1));
                    }
                }
            }

            _ => {}
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

    /// Get the currently selected bead
    fn get_selected_bead(&self) -> Option<&Bead> {
        let idx = self.list_state.selected()?;
        let tree_order = build_tree_order(&self.beads, self.hide_closed, self.filter());
        tree_order.get(idx).map(|(bead, _)| *bead)
    }

    /// Close the selected bead with a reason
    fn close_bead(&mut self) -> Result<()> {
        if let Some(bead) = self.get_selected_bead() {
            let id = bead.id.clone();
            let reason = self.reason_input.text();
            let reason_opt = if reason.is_empty() {
                None
            } else {
                Some(reason)
            };
            BrCli::close(&id, reason_opt)?;
            self.refresh()?;
        }
        Ok(())
    }

    /// Reopen the selected bead with a reason
    fn reopen_bead(&mut self) -> Result<()> {
        if let Some(bead) = self.get_selected_bead() {
            let id = bead.id.clone();
            let reason = self.reason_input.text();
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
}

/// Setup the terminal
fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    // Check if we have a TTY
    if !io::stdout().is_terminal() {
        anyhow::bail!("bu requires a terminal (TTY) to run. Cannot run in a pipe or background.");
    }

    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
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
        DisableMouseCapture
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
        EnableMouseCapture
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
        let show_detail = app.show_detail;
        let input_mode = app.input_mode;
        let search_text = app.search_input.text().to_string();
        let search_cursor = app.search_input.cursor();
        let reason_text = app.reason_input.text().to_string();
        let reason_cursor = app.reason_input.cursor();

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
                show_detail,
                input_mode,
                &search_text,
                search_cursor,
                &app.create_modal,
                &reason_text,
                reason_cursor,
            );
            // Store areas for mouse handling
            app.list_area = list_area;
            app.detail_area = detail_area;
        })?;

        // Handle events
        if let Some(event) = event::poll_event(tick_rate)? {
            match event {
                Event::Key(key) => {
                    match app.handle_key(key) {
                        Ok(()) => {}
                        Err(e) if e.to_string() == "__SUSPEND__" => {
                            suspend(terminal)?;
                        }
                        Err(e) => return Err(e),
                    }
                }
                Event::Mouse(mouse) => {
                    app.handle_mouse(mouse)?;
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
