//! Data layer for beads-tui
//!
//! This module handles reading beads from SQLite and invoking the br CLI
//! for mutations.

mod bead;
mod br;
mod sqlite;

pub use bead::{Bead, BeadStatus, BeadType, DependencyType};
pub use br::BrCli;
pub use sqlite::BeadStore;
