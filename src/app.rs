//! Application state and main loop

use std::io::{self, Stdout};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::data::{Bead, BeadStatus, BeadStore, BrCli};
use crate::event;
use crate::ui::layout::Focus;
use crate::ui::{render_layout, BeadListState, Theme, THEMES};

/// Application state
pub struct App {
    /// Path to the beads database
    db_path: PathBuf,
    /// All loaded beads
    beads: Vec<Bead>,
    /// List widget state
    list_state: BeadListState,
    /// Current theme index
    theme_idx: usize,
    /// Current focus
    focus: Focus,
    /// Split percentage (left pane width)
    split_percent: u16,
    /// Current search filter
    filter: Option<String>,
    /// Is search input active
    searching: bool,
    /// Show help overlay
    show_help: bool,
    /// Should the app quit
    should_quit: bool,
    /// Refresh interval
    refresh_interval: Duration,
    /// Last refresh time
    last_refresh: Instant,
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
            theme_idx: 0,
            focus: Focus::List,
            split_percent: 40,
            filter: None,
            searching: false,
            show_help: false,
            should_quit: false,
            refresh_interval: Duration::from_secs(refresh_secs),
            last_refresh: Instant::now(),
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

    /// Get filtered beads count
    fn filtered_len(&self) -> usize {
        self.beads
            .iter()
            .filter(|b| {
                self.filter
                    .as_ref()
                    .map(|f| b.title.to_lowercase().contains(&f.to_lowercase()))
                    .unwrap_or(true)
            })
            .count()
    }

    /// Handle a key event
    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        // Help overlay takes precedence
        if self.show_help {
            self.show_help = false;
            return Ok(());
        }

        // Search mode handling
        if self.searching {
            match key.code {
                KeyCode::Esc => {
                    self.searching = false;
                    self.filter = None;
                }
                KeyCode::Enter => {
                    self.searching = false;
                }
                KeyCode::Backspace => {
                    if let Some(ref mut f) = self.filter {
                        f.pop();
                        if f.is_empty() {
                            self.filter = None;
                        }
                    }
                }
                KeyCode::Char(c) => {
                    self.filter.get_or_insert_with(String::new).push(c);
                    // Reset selection when filter changes
                    self.list_state.first();
                }
                _ => {}
            }
            return Ok(());
        }

        // Normal mode
        match key.code {
            // Quit
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }

            // Navigation
            KeyCode::Up | KeyCode::Char('k') => {
                self.list_state.previous(self.filtered_len());
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.list_state.next(self.filtered_len());
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.list_state.first();
            }
            KeyCode::End => {
                self.list_state.last(self.filtered_len());
            }
            KeyCode::Char('G') => {
                self.list_state.last(self.filtered_len());
            }

            // Focus
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::List => Focus::Detail,
                    Focus::Detail => Focus::List,
                };
            }

            // Pane resizing
            KeyCode::Char('<') | KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::SHIFT) || key.code == KeyCode::Char('<') => {
                self.split_percent = self.split_percent.saturating_sub(5).max(20);
            }
            KeyCode::Char('>') | KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::SHIFT) || key.code == KeyCode::Char('>') => {
                self.split_percent = (self.split_percent + 5).min(80);
            }

            // Search
            KeyCode::Char('/') => {
                self.searching = true;
                self.filter = Some(String::new());
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

            // Status change
            KeyCode::Char('s') => {
                self.cycle_status()?;
            }

            _ => {}
        }

        Ok(())
    }

    /// Cycle the status of the selected bead
    fn cycle_status(&mut self) -> Result<()> {
        if let Some(idx) = self.list_state.selected() {
            let filtered: Vec<_> = self.beads
                .iter()
                .filter(|b| {
                    self.filter
                        .as_ref()
                        .map(|f| b.title.to_lowercase().contains(&f.to_lowercase()))
                        .unwrap_or(true)
                })
                .collect();

            if let Some(bead) = filtered.get(idx) {
                let new_status = match bead.status {
                    BeadStatus::Open => "in_progress",
                    BeadStatus::InProgress => "closed",
                    BeadStatus::Blocked => "open",
                    BeadStatus::Closed => "open",
                };
                BrCli::update_status(&bead.id, new_status)?;
                self.refresh()?;
            }
        }
        Ok(())
    }
}

/// Setup the terminal
fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
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

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
) -> Result<()> {
    let tick_rate = Duration::from_millis(100);

    loop {
        // Get values before drawing to avoid borrow issues
        let theme = app.theme().clone();
        let focus = app.focus;
        let split_percent = app.split_percent;
        let filter = app.filter.clone();
        let show_help = app.show_help;

        // Draw
        terminal.draw(|frame| {
            render_layout(
                frame,
                &app.beads,
                &mut app.list_state,
                &theme,
                focus,
                split_percent,
                filter.as_deref(),
                show_help,
            );
        })?;

        // Handle events
        if let Some(event) = event::poll_event(tick_rate)? {
            if let Event::Key(key) = event {
                app.handle_key(key)?;
            }
        }

        // Auto-refresh
        if app.refresh_interval.as_secs() > 0
            && app.last_refresh.elapsed() >= app.refresh_interval
        {
            let _ = app.refresh();
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
