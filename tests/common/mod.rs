//! Common test utilities for integration tests.
//!
//! Provides a harness for TUI testing using tmux.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use tempfile::TempDir;

static PROJECT_COUNTER: AtomicUsize = AtomicUsize::new(0);
static TUI_SESSION_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Get the path to the bu binary.
pub fn bu_bin() -> PathBuf {
    // Try release first, fall back to debug
    let release = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/release/bu");
    if release.exists() {
        return release;
    }

    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/debug/bu")
}

/// A test project with a temporary .beads directory.
pub struct TestProject {
    pub dir: TempDir,
    pub path: PathBuf,
}

impl TestProject {
    /// Create a new test project with beads initialized.
    pub fn new() -> Self {
        let count = PROJECT_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = TempDir::with_prefix(&format!("bu-test-{}-", count))
            .expect("Failed to create temp dir");
        let path = dir.path().to_path_buf();

        let project = Self { dir, path };

        // Initialize beads using br
        let output = Command::new("br")
            .arg("init")
            .current_dir(&project.path)
            .output()
            .expect("Failed to run br init");

        assert!(
            output.status.success(),
            "Failed to init beads: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        project
    }

    /// Create a new test project with a custom name (for debugging).
    pub fn with_name(name: &str) -> Self {
        let dir =
            TempDir::with_prefix(&format!("bu-{}-", name)).expect("Failed to create temp dir");
        let path = dir.path().to_path_buf();

        let project = Self { dir, path };

        let output = Command::new("br")
            .arg("init")
            .current_dir(&project.path)
            .output()
            .expect("Failed to run br init");

        assert!(
            output.status.success(),
            "Failed to init beads: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        project
    }

    /// Create a bead and return its ID.
    pub fn create_bead(&self, title: &str) -> String {
        let output = Command::new("br")
            .args(["create", "--title", title])
            .current_dir(&self.path)
            .output()
            .expect("Failed to run br create");

        assert!(
            output.status.success(),
            "Failed to create bead: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // Parse ID from output like "Created: bd-abc123"
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .find_map(|line| {
                line.split_whitespace()
                    .find(|w| w.starts_with("bd-"))
                    .map(|s| s.trim_end_matches([',', ':', '.']).to_string())
            })
            .unwrap_or_default()
    }

    /// Get path to the project directory.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Wrapper around Command output with helper methods.
pub struct BuOutput {
    pub status: std::process::ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl From<Output> for BuOutput {
    fn from(output: Output) -> Self {
        Self {
            status: output.status,
            stdout: output.stdout,
            stderr: output.stderr,
        }
    }
}

impl BuOutput {
    pub fn success(&self) -> bool {
        self.status.success()
    }

    pub fn stdout_str(&self) -> String {
        String::from_utf8_lossy(&self.stdout).to_string()
    }

    pub fn stderr_str(&self) -> String {
        String::from_utf8_lossy(&self.stderr).to_string()
    }
}

/// TUI test harness using tmux.
pub struct TuiHarness {
    session_name: String,
    #[allow(dead_code)]
    project_path: PathBuf,
}

impl TuiHarness {
    /// Start the TUI in a tmux session.
    pub fn start(project: &TestProject) -> Self {
        let count = TUI_SESSION_COUNTER.fetch_add(1, Ordering::SeqCst);
        let session_name = format!("bu-tui-{}-{}", std::process::id(), count);
        let bin = bu_bin();

        // Start tmux session with TUI
        let status = Command::new("tmux")
            .args([
                "new-session",
                "-d",
                "-s",
                &session_name,
                "-x",
                "100",
                "-y",
                "30",
                &bin.display().to_string(),
            ])
            .current_dir(&project.path)
            .status()
            .expect("Failed to start tmux");

        assert!(status.success(), "Failed to start tmux session");

        // Give TUI time to initialize
        std::thread::sleep(std::time::Duration::from_millis(500));

        Self {
            session_name,
            project_path: project.path.clone(),
        }
    }

    /// Capture the current pane content.
    pub fn capture(&self) -> String {
        let output = Command::new("tmux")
            .args(["capture-pane", "-t", &self.session_name, "-p"])
            .output()
            .expect("Failed to capture tmux pane");

        String::from_utf8_lossy(&output.stdout).to_string()
    }

    /// Send keys to the TUI.
    pub fn send_keys(&self, keys: &str) {
        let status = Command::new("tmux")
            .args(["send-keys", "-t", &self.session_name, keys])
            .status()
            .expect("Failed to send keys");

        assert!(status.success(), "Failed to send keys to tmux");

        // Small delay for TUI to process
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    /// Send a special key (Tab, Enter, Escape, etc).
    pub fn send_special(&self, key: &str) {
        self.send_keys(key);
    }

    /// Check if the session is still running.
    pub fn is_running(&self) -> bool {
        let output = Command::new("tmux")
            .args(["has-session", "-t", &self.session_name])
            .status();

        output.map(|s| s.success()).unwrap_or(false)
    }

    /// Wait for TUI to exit.
    pub fn wait_for_exit(&self, timeout_ms: u64) -> bool {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        while start.elapsed() < timeout {
            if !self.is_running() {
                return true;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        false
    }

    /// Assert that capture contains a string.
    #[allow(dead_code)]
    pub fn assert_contains(&self, needle: &str) {
        let content = self.capture();
        assert!(
            content.contains(needle),
            "Expected TUI to contain '{}', got:\n{}",
            needle,
            content
        );
    }

    /// Kill the tmux session (cleanup).
    pub fn kill(&self) {
        let _ = Command::new("tmux")
            .args(["kill-session", "-t", &self.session_name])
            .status();
    }
}

impl Drop for TuiHarness {
    fn drop(&mut self) {
        self.kill();
    }
}
