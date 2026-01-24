//! Bead data structures

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Status of a bead
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BeadStatus {
    #[default]
    Open,
    InProgress,
    Blocked,
    Closed,
}

impl BeadStatus {
    /// Get the display icon for this status
    pub fn icon(&self) -> &'static str {
        match self {
            BeadStatus::Open => "\u{25cf}",       // ●
            BeadStatus::InProgress => "\u{25d0}", // ◐
            BeadStatus::Blocked => "\u{26d4}",    // ⛔
            BeadStatus::Closed => "\u{2714}",     // ✔
        }
    }

    /// Get all possible statuses
    pub fn all() -> &'static [BeadStatus] {
        &[
            BeadStatus::Open,
            BeadStatus::InProgress,
            BeadStatus::Blocked,
            BeadStatus::Closed,
        ]
    }
}

impl fmt::Display for BeadStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BeadStatus::Open => write!(f, "open"),
            BeadStatus::InProgress => write!(f, "in_progress"),
            BeadStatus::Blocked => write!(f, "blocked"),
            BeadStatus::Closed => write!(f, "closed"),
        }
    }
}

impl std::str::FromStr for BeadStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(BeadStatus::Open),
            "in_progress" | "in-progress" | "inprogress" => Ok(BeadStatus::InProgress),
            "blocked" => Ok(BeadStatus::Blocked),
            "closed" => Ok(BeadStatus::Closed),
            _ => anyhow::bail!("Unknown status: {}", s),
        }
    }
}

/// Type of a bead
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BeadType {
    #[default]
    Task,
    Bug,
    Feature,
    Epic,
    Story,
}

impl fmt::Display for BeadType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BeadType::Task => write!(f, "task"),
            BeadType::Bug => write!(f, "bug"),
            BeadType::Feature => write!(f, "feature"),
            BeadType::Epic => write!(f, "epic"),
            BeadType::Story => write!(f, "story"),
        }
    }
}

impl std::str::FromStr for BeadType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "task" => Ok(BeadType::Task),
            "bug" => Ok(BeadType::Bug),
            "feature" => Ok(BeadType::Feature),
            "epic" => Ok(BeadType::Epic),
            "story" => Ok(BeadType::Story),
            _ => anyhow::bail!("Unknown type: {}", s),
        }
    }
}

/// Type of dependency between beads
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DependencyType {
    Blocks,
    ParentChild,
    Related,
}

impl fmt::Display for DependencyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DependencyType::Blocks => write!(f, "blocks"),
            DependencyType::ParentChild => write!(f, "parent-child"),
            DependencyType::Related => write!(f, "related"),
        }
    }
}

impl std::str::FromStr for DependencyType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "blocks" => Ok(DependencyType::Blocks),
            "parent-child" | "parent_child" => Ok(DependencyType::ParentChild),
            "related" => Ok(DependencyType::Related),
            _ => anyhow::bail!("Unknown dependency type: {}", s),
        }
    }
}

/// A dependency relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// The bead that depends on another
    pub from_id: String,
    /// The bead being depended on
    pub to_id: String,
    /// Type of dependency
    pub dep_type: DependencyType,
}

/// A bead (issue) in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bead {
    /// Unique identifier (e.g., "bd-abc123")
    pub id: String,
    /// Short title
    pub title: String,
    /// Current status
    pub status: BeadStatus,
    /// Priority (0 = critical, 4 = backlog)
    pub priority: u8,
    /// Type of bead
    pub bead_type: BeadType,
    /// Full description
    pub description: Option<String>,
    /// Labels/tags
    pub labels: Vec<String>,
    /// Who created this bead
    pub created_by: Option<String>,
    /// Who is assigned to this bead
    pub assignee: Option<String>,
    /// When created
    pub created_at: Option<DateTime<Utc>>,
    /// When last updated
    pub updated_at: Option<DateTime<Utc>>,
    /// When closed (if closed)
    pub closed_at: Option<DateTime<Utc>>,
    /// Reason for closing
    pub close_reason: Option<String>,
    /// Parent bead IDs (from parent-child dependencies)
    pub parent_ids: Vec<String>,
    /// Beads that block this one
    pub blocked_by: Vec<String>,
    /// Beads that this one blocks
    pub blocks: Vec<String>,
}

impl Bead {
    /// Check if this bead is blocked by any open beads
    pub fn is_blocked(&self) -> bool {
        self.status == BeadStatus::Blocked || !self.blocked_by.is_empty()
    }

    /// Get a priority label
    pub fn priority_label(&self) -> String {
        format!("P{}", self.priority)
    }
}

impl Default for Bead {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            status: BeadStatus::Open,
            priority: 2,
            bead_type: BeadType::Task,
            description: None,
            labels: Vec::new(),
            created_by: None,
            assignee: None,
            created_at: None,
            updated_at: None,
            closed_at: None,
            close_reason: None,
            parent_ids: Vec::new(),
            blocked_by: Vec::new(),
            blocks: Vec::new(),
        }
    }
}
