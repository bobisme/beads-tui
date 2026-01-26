//! SQLite database reader for beads

#![allow(dead_code)]

use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};
use std::path::Path;

use super::{Bead, BeadStatus, BeadType, DependencyType};

/// A store that reads beads from SQLite
pub struct BeadStore {
    conn: Connection,
}

impl BeadStore {
    /// Open a connection to the beads database
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open_with_flags(
            path.as_ref(),
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .with_context(|| format!("Failed to open database: {:?}", path.as_ref()))?;

        Ok(Self { conn })
    }

    /// Load all beads from the database
    pub fn load_all(&self) -> Result<Vec<Bead>> {
        let mut beads = self.load_beads()?;
        let deps = self.load_dependencies()?;
        let labels = self.load_labels()?;

        // Apply labels to beads
        for bead in &mut beads {
            for (issue_id, label) in &labels {
                if issue_id == &bead.id {
                    bead.labels.push(label.clone());
                }
            }
        }

        // Apply dependencies to beads
        for bead in &mut beads {
            for (from_id, to_id, dep_type) in &deps {
                match dep_type {
                    DependencyType::ParentChild if from_id == &bead.id => {
                        bead.parent_ids.push(to_id.clone());
                    }
                    DependencyType::Blocks if from_id == &bead.id => {
                        bead.blocked_by.push(to_id.clone());
                    }
                    DependencyType::Blocks if to_id == &bead.id => {
                        bead.blocks.push(from_id.clone());
                    }
                    _ => {}
                }
            }
        }

        // Sort by status (open/in_progress first), then by priority
        // For closed beads, sort by closed_at (most recent first)
        beads.sort_by(|a, b| {
            let status_ord = |s: &BeadStatus| match s {
                BeadStatus::InProgress => 0,
                BeadStatus::Open => 1,
                BeadStatus::Blocked => 2,
                BeadStatus::Closed => 3,
            };
            let status_cmp = status_ord(&a.status).cmp(&status_ord(&b.status));

            // If both are closed, sort by closed_at (most recent first)
            if a.status == BeadStatus::Closed && b.status == BeadStatus::Closed {
                // Compare closed_at in reverse (None sorts to end)
                match (&b.closed_at, &a.closed_at) {
                    (Some(b_time), Some(a_time)) => b_time.cmp(a_time),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => a.title.cmp(&b.title),
                }
            } else {
                // For non-closed beads, sort by priority then title
                status_cmp
                    .then(a.priority.cmp(&b.priority))
                    .then(a.title.cmp(&b.title))
            }
        });

        Ok(beads)
    }

    fn load_beads(&self) -> Result<Vec<Bead>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT 
                id,
                title,
                status,
                priority,
                issue_type,
                description,
                created_by,
                assignee,
                created_at,
                updated_at,
                closed_at,
                close_reason
            FROM issues
            WHERE status != 'tombstone' AND deleted_at IS NULL
            "#,
        )?;

        let beads = stmt
            .query_map([], |row| {
                Ok(Bead {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    status: row.get::<_, String>(2)?.parse().unwrap_or(BeadStatus::Open),
                    priority: row.get::<_, i64>(3)? as u8,
                    bead_type: row.get::<_, String>(4)?.parse().unwrap_or(BeadType::Task),
                    description: row.get(5)?,
                    labels: Vec::new(), // Loaded separately from labels table
                    created_by: row.get(6)?,
                    assignee: row.get(7)?,
                    created_at: row
                        .get::<_, Option<String>>(8)?
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc)),
                    updated_at: row
                        .get::<_, Option<String>>(9)?
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc)),
                    closed_at: row
                        .get::<_, Option<String>>(10)?
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc)),
                    close_reason: row.get(11)?,
                    parent_ids: Vec::new(),
                    blocked_by: Vec::new(),
                    blocks: Vec::new(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(beads)
    }

    fn load_dependencies(&self) -> Result<Vec<(String, String, DependencyType)>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT issue_id, depends_on_id, type
            FROM dependencies
            "#,
        )?;

        let deps = stmt
            .query_map([], |row| {
                let dep_type: String = row.get(2)?;
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    dep_type.parse().unwrap_or(DependencyType::Related),
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(deps)
    }

    fn load_labels(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT issue_id, label
            FROM labels
            "#,
        )?;

        let labels = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(labels)
    }

    /// Get a single bead by ID
    pub fn get(&self, id: &str) -> Result<Option<Bead>> {
        let beads = self.load_all()?;
        Ok(beads.into_iter().find(|b| b.id == id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_parsing() {
        assert_eq!("open".parse::<BeadStatus>().unwrap(), BeadStatus::Open);
        assert_eq!(
            "in_progress".parse::<BeadStatus>().unwrap(),
            BeadStatus::InProgress
        );
        assert_eq!(
            "blocked".parse::<BeadStatus>().unwrap(),
            BeadStatus::Blocked
        );
        assert_eq!("closed".parse::<BeadStatus>().unwrap(), BeadStatus::Closed);
    }
}
