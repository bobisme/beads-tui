//! beads-tui (bu) - A TUI for viewing and managing beads
//!
//! This application provides a terminal user interface for interacting with
//! beads (issues) stored in a local SQLite database.

mod app;
mod data;
mod event;
mod ui;

use anyhow::Result;
use clap::Parser;
use std::path::{Path, PathBuf};

use crate::data::BrCli;

/// A TUI for viewing and managing beads
#[derive(Parser, Debug)]
#[command(name = "bu", version, about, long_about = None)]
struct Args {
    /// Path to the beads database (default: .beads/beads.db)
    #[arg(short, long)]
    db: Option<PathBuf>,

    /// Refresh interval in seconds (0 to disable auto-refresh)
    #[arg(short, long, default_value = "3")]
    refresh: u64,
}

fn ensure_database_exists(db_path: &Path) -> Result<()> {
    if db_path.exists() {
        return Ok(());
    }

    let in_beads_workspace = db_path
        .parent()
        .map(|p| p.ends_with(".beads") && p.exists())
        .unwrap_or(false);

    if in_beads_workspace {
        eprintln!(
            "Database missing at {:?}. Detected .beads workspace; running 'br sync'...",
            db_path
        );
        BrCli::sync()?;

        if db_path.exists() {
            return Ok(());
        }
    }

    anyhow::bail!(
        "Database not found at {:?}. Run 'br init' (or 'br sync') to initialize a beads workspace.",
        db_path
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Find the database path
    let db_path = args.db.unwrap_or_else(|| PathBuf::from(".beads/beads.db"));

    ensure_database_exists(&db_path)?;

    // Run the application
    app::run(db_path, args.refresh).await
}
