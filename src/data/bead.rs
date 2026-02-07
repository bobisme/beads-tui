//! Bead data structures

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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
    /// Get the display icon for this status (simple unicode, no emojis)
    pub fn icon(&self) -> &'static str {
        match self {
            BeadStatus::Open => "\u{25cb}",       // ○ (open circle)
            BeadStatus::InProgress => "\u{25cf}", // ● (filled circle)
            BeadStatus::Blocked => "\u{25a0}",    // ■ (filled square - blocked)
            BeadStatus::Closed => "\u{2713}",     // ✓ (check mark)
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

impl BeadType {
    /// Get the outline (open/blocked-open) icon for this type
    pub fn icon_outline(&self) -> &'static str {
        match self {
            BeadType::Task => "\u{25b7}",    // ▷ right triangle outline
            BeadType::Bug => "\u{2298}",     // ⊘ circled slash
            BeadType::Feature => "\u{2606}", // ☆ star outline
            BeadType::Epic => "\u{25c7}",    // ◇ diamond outline
            BeadType::Story => "\u{2630}",   // ☰ trigram
        }
    }

    /// Get the filled (in-progress/blocked-in-progress) icon for this type
    pub fn icon_filled(&self) -> &'static str {
        match self {
            BeadType::Task => "\u{25b6}",    // ▶ right triangle filled
            BeadType::Bug => "\u{25cf}",     // ● filled circle
            BeadType::Feature => "\u{2605}", // ★ star filled
            BeadType::Epic => "\u{25c6}",    // ◆ diamond filled
            BeadType::Story => "\u{25e4}",   // ▤ square with lines
        }
    }

    /// Get the closed (done) icon for this type
    pub fn icon_closed(&self) -> &'static str {
        match self {
            BeadType::Task => "\u{25b6}",    // ▶ right triangle filled
            BeadType::Bug => "\u{25cf}",     // ● filled circle
            BeadType::Feature => "\u{2605}", // ★ star filled
            BeadType::Epic => "\u{25c6}",    // ◆ diamond filled
            BeadType::Story => "\u{25a0}",   // ■ filled square
        }
    }

    /// Get the appropriate icon based on status
    /// - Open: outline
    /// - InProgress: filled
    /// - Blocked: outline (color will be red)
    /// - Closed: closed variant (filled, grayed)
    pub fn icon_for_status(&self, status: &BeadStatus) -> &'static str {
        match status {
            BeadStatus::Open => self.icon_outline(),
            BeadStatus::InProgress => self.icon_filled(),
            BeadStatus::Blocked => self.icon_outline(), // Outline but red
            BeadStatus::Closed => self.icon_closed(),
        }
    }
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

/// A comment on a bead
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// Comment author
    pub author: String,
    /// Comment text
    pub text: String,
    /// When the comment was created
    pub created_at: Option<DateTime<Utc>>,
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
    /// Comments on this bead
    pub comments: Vec<Comment>,
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

    /// Check if this bead has the "deferred" label
    pub fn is_deferred(&self) -> bool {
        self.labels.iter().any(|l| l == "deferred")
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
            comments: Vec::new(),
        }
    }
}

/// Build a tree-ordered list of beads with their depths.
/// Non-closed beads are arranged hierarchically, closed beads are flat at the end.
/// Returns Vec of (bead reference, depth).
pub fn build_tree_order<'a>(
    beads: &'a [Bead],
    hide_closed: bool,
    filter: Option<&str>,
) -> Vec<(&'a Bead, usize)> {
    // Filter beads first
    let filtered: Vec<&Bead> = beads
        .iter()
        .filter(|b| {
            // Apply hide_closed filter
            if hide_closed && b.status == BeadStatus::Closed {
                return false;
            }
            // Apply text filter (matches title or ID)
            filter
                .map(|f| {
                    let f_lower = f.to_lowercase();
                    b.title.to_lowercase().contains(&f_lower)
                        || b.id.to_lowercase().contains(&f_lower)
                })
                .unwrap_or(true)
        })
        .collect();

    // Separate closed and non-closed
    let (closed, non_closed): (Vec<_>, Vec<_>) = filtered
        .into_iter()
        .partition(|b| b.status == BeadStatus::Closed);

    // Build parent -> children map for non-closed beads
    let non_closed_ids: HashSet<&str> = non_closed.iter().map(|b| b.id.as_str()).collect();
    let mut children_map: HashMap<&str, Vec<&Bead>> = HashMap::new();
    let mut has_parent: HashSet<&str> = HashSet::new();

    for bead in &non_closed {
        // Parent-child relationships
        for parent_id in &bead.parent_ids {
            if non_closed_ids.contains(parent_id.as_str()) {
                children_map
                    .entry(parent_id.as_str())
                    .or_default()
                    .push(bead);
                has_parent.insert(bead.id.as_str());
            }
        }
        // Blocked-by relationships: if A is blocked by B, show A under B
        for blocker_id in &bead.blocked_by {
            if non_closed_ids.contains(blocker_id.as_str()) {
                children_map
                    .entry(blocker_id.as_str())
                    .or_default()
                    .push(bead);
                has_parent.insert(bead.id.as_str());
            }
        }
    }

    // Find roots (beads with no parent in the set)
    let mut roots: Vec<&Bead> = non_closed
        .iter()
        .filter(|b| !has_parent.contains(b.id.as_str()))
        .copied()
        .collect();

    // Sort roots: non-deferred first (by priority, then title), deferred last (by priority, then title)
    roots.sort_by(|a, b| {
        a.is_deferred()
            .cmp(&b.is_deferred())
            .then(a.priority.cmp(&b.priority))
            .then(a.title.cmp(&b.title))
    });

    // DFS to build ordered list with depths
    let mut result: Vec<(&Bead, usize)> = Vec::new();
    let mut stack: Vec<(&Bead, usize)> = roots.into_iter().map(|b| (b, 0)).rev().collect();
    let mut visited: HashSet<&str> = HashSet::new();

    while let Some((bead, depth)) = stack.pop() {
        // Skip if already visited (can happen with multiple dependency types)
        if visited.contains(bead.id.as_str()) {
            continue;
        }
        visited.insert(bead.id.as_str());
        result.push((bead, depth));

        // Add children in reverse order (so they come out in correct order)
        if let Some(children) = children_map.get(bead.id.as_str()) {
            let mut sorted_children = children.clone();
            sorted_children.sort_by(|a, b| {
                b.priority.cmp(&a.priority).then(b.title.cmp(&a.title)) // Reverse for stack
            });
            for child in sorted_children {
                stack.push((child, depth + 1));
            }
        }
    }

    // Add closed beads flat at the end (depth 0)
    // They're already sorted by closed_at from sqlite.rs, so just append in order
    for bead in closed {
        result.push((bead, 0));
    }

    result
}
