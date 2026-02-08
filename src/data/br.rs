//! Wrapper for the br (beads_rust) CLI

#![allow(dead_code)]

use anyhow::{Context, Result};
use std::process::Command;

use super::BeadType;

/// CLI wrapper for the br command
pub struct BrCli;

impl BrCli {
    /// Create a new bead
    pub fn create(
        title: &str,
        bead_type: BeadType,
        priority: u8,
        description: Option<&str>,
        parent_id: Option<&str>,
    ) -> Result<String> {
        let mut cmd = Command::new("br");
        cmd.arg("create")
            .arg(format!("--title={}", title))
            .arg("--type")
            .arg(bead_type.to_string())
            .arg("--priority")
            .arg(priority.to_string());

        if let Some(desc) = description {
            cmd.arg(format!("--description={}", desc));
        }

        let output = cmd
            .output()
            .context("Failed to execute br create command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("br create failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse the created ID from output like "Created: bd-abc123"
        let id = stdout
            .lines()
            .find_map(|line| {
                if line.contains("Created") || line.contains("bd-") {
                    line.split_whitespace()
                        .find(|w| w.starts_with("bd-"))
                        .map(|s| s.trim_end_matches([',', ':', '.']))
                } else {
                    None
                }
            })
            .unwrap_or("")
            .to_string();

        // If we have a parent, add the parent-child dependency
        if let Some(pid) = parent_id
            && !id.is_empty()
        {
            Self::add_dependency(&id, pid, "parent-child")?;
        }

        Ok(id)
    }

    /// Update a bead's status
    pub fn update_status(id: &str, status: &str) -> Result<()> {
        let output = Command::new("br")
            .arg("update")
            .arg(id)
            .arg("--status")
            .arg(status)
            .output()
            .context("Failed to execute br update command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("br update failed: {}", stderr);
        }

        Ok(())
    }

    /// Close a bead
    pub fn close(id: &str, reason: Option<&str>) -> Result<()> {
        let mut cmd = Command::new("br");
        cmd.arg("close").arg(id);

        if let Some(r) = reason {
            cmd.arg(format!("--reason={}", r));
        }

        let output = cmd.output().context("Failed to execute br close command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("br close failed: {}", stderr);
        }

        Ok(())
    }

    /// Add a dependency between beads
    pub fn add_dependency(from_id: &str, to_id: &str, dep_type: &str) -> Result<()> {
        let output = Command::new("br")
            .arg("dep")
            .arg("add")
            .arg(from_id)
            .arg(to_id)
            .arg("--type")
            .arg(dep_type)
            .output()
            .context("Failed to execute br dep add command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("br dep add failed: {}", stderr);
        }

        Ok(())
    }

    /// Update a generic field on a bead (title, description, type, priority)
    pub fn update_field(id: &str, field: &str, value: &str) -> Result<()> {
        let flag = format!("--{}", field);
        let arg = format!("--{}={}", field, value);

        let output = Command::new("br")
            .arg("update")
            .arg(id)
            .arg(arg)
            .output()
            .context(format!("Failed to execute br update {} command", flag))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("br update {} failed: {}", flag, stderr);
        }

        Ok(())
    }

    /// Add a label to a bead
    pub fn add_label(id: &str, label: &str) -> Result<()> {
        let output = Command::new("br")
            .arg("update")
            .arg(id)
            .arg(format!("--add-label={}", label))
            .output()
            .context("Failed to execute br update --add-label command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("br update --add-label failed: {}", stderr);
        }

        Ok(())
    }

    /// Remove a label from a bead
    pub fn remove_label(id: &str, label: &str) -> Result<()> {
        let output = Command::new("br")
            .arg("update")
            .arg(id)
            .arg(format!("--remove-label={}", label))
            .output()
            .context("Failed to execute br update --remove-label command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("br update --remove-label failed: {}", stderr);
        }

        Ok(())
    }

    /// Add a comment to a bead
    pub fn add_comment(id: &str, comment: &str) -> Result<()> {
        let output = Command::new("br")
            .arg("comments")
            .arg("add")
            .arg(id)
            .arg("--")
            .arg(comment)
            .output()
            .context("Failed to execute br comments add command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("br comments add failed: {}", stderr);
        }

        Ok(())
    }

    /// Run `br sync` to rebuild/export state (including SQLite DB)
    pub fn sync() -> Result<()> {
        let output = Command::new("br")
            .arg("sync")
            .output()
            .context("Failed to execute br sync command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("br sync failed: {}", stderr);
        }

        Ok(())
    }

    /// Check if br CLI is available
    pub fn is_available() -> bool {
        Command::new("br")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
