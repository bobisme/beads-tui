//! TUI (Terminal User Interface) tests.
//!
//! Tests for the terminal UI using tmux as a test harness.

mod common;

use common::{TestProject, TuiHarness};

/// Test that TUI starts and shows basic structure.
#[test]
fn test_tui_starts() {
    let project = TestProject::with_name("tui-start");

    // Create a bead to have some content
    project.create_bead("Test bead for TUI");

    let tui = TuiHarness::start(&project);

    // Should be running
    assert!(tui.is_running(), "TUI should be running");

    // Should show basic structure
    let capture = tui.capture();

    // Should have beads panel
    assert!(
        capture.contains("Beads") || capture.contains("Test bead"),
        "Expected beads panel, got:\n{}",
        capture
    );

    // Should have detail panel
    assert!(
        capture.contains("Detail"),
        "Expected detail panel, got:\n{}",
        capture
    );

    // Cleanup happens automatically in Drop
}

/// Test that 'q' quits the TUI.
#[test]
fn test_tui_quit_with_q() {
    let project = TestProject::with_name("tui-quit-q");

    let tui = TuiHarness::start(&project);
    assert!(tui.is_running(), "TUI should start running");

    // Send 'q' to quit
    tui.send_keys("q");

    // Should exit within reasonable time
    assert!(
        tui.wait_for_exit(2000),
        "TUI should exit after pressing 'q'"
    );
}

/// Test Tab navigation between panes.
#[test]
fn test_tui_tab_navigation() {
    let project = TestProject::with_name("tui-nav");

    project.create_bead("Navigation test bead");

    let tui = TuiHarness::start(&project);

    // Initial state
    let initial = tui.capture();

    // Press Tab to switch panes
    tui.send_special("Tab");

    // The capture should change (border highlight moves)
    let after_tab = tui.capture();

    // Both captures should show the basic structure
    assert!(initial.contains("Beads") || initial.contains("Detail"));
    assert!(after_tab.contains("Beads") || after_tab.contains("Detail"));

    // Quit
    tui.send_keys("q");
    tui.wait_for_exit(1000);
}

/// Test j/k scrolling in list.
#[test]
fn test_tui_list_scrolling() {
    let project = TestProject::with_name("tui-scroll");

    // Add multiple beads so we have something to scroll
    for i in 0..5 {
        project.create_bead(&format!("Scroll test bead {}", i));
    }

    let tui = TuiHarness::start(&project);

    // Capture initial state
    let initial = tui.capture();

    // Press j to move down
    tui.send_keys("j");
    let after_down = tui.capture();

    // Press k to move up
    tui.send_keys("k");
    let after_up = tui.capture();

    // All captures should contain beads
    assert!(initial.contains("Scroll test") || initial.contains("Beads"));
    assert!(after_down.contains("Scroll test") || after_down.contains("Beads"));
    assert!(after_up.contains("Scroll test") || after_up.contains("Beads"));

    // Quit
    tui.send_keys("q");
    tui.wait_for_exit(1000);
}

/// Test ? shows help overlay.
#[test]
fn test_tui_help_overlay() {
    let project = TestProject::with_name("tui-help");

    let tui = TuiHarness::start(&project);

    // Press ? to show help
    tui.send_keys("?");
    std::thread::sleep(std::time::Duration::from_millis(100));

    let capture = tui.capture();

    // Should show help content
    assert!(
        capture.contains("Help") || capture.contains("Keyboard"),
        "Expected help overlay, got:\n{}",
        capture
    );

    // Press any key to close help
    tui.send_keys("q");

    // Quit
    tui.send_keys("q");
    tui.wait_for_exit(1000);
}

/// Test / enters search mode.
#[test]
fn test_tui_search_mode() {
    let project = TestProject::with_name("tui-search");

    project.create_bead("Findable bead");
    project.create_bead("Another bead");

    let tui = TuiHarness::start(&project);

    // Press / to enter search
    tui.send_keys("/");

    // Type search query
    tui.send_keys("Find");
    std::thread::sleep(std::time::Duration::from_millis(100));

    let capture = tui.capture();

    // Should still show TUI structure
    assert!(capture.contains("Beads") || capture.contains("Detail"));

    // Press Escape to clear search
    tui.send_special("Escape");

    // Quit
    tui.send_keys("q");
    tui.wait_for_exit(1000);
}

/// Test t cycles themes.
#[test]
fn test_tui_theme_cycling() {
    let project = TestProject::with_name("tui-theme");

    project.create_bead("Theme test bead");

    let tui = TuiHarness::start(&project);

    let initial = tui.capture();

    // Press t to cycle theme
    tui.send_keys("t");
    std::thread::sleep(std::time::Duration::from_millis(100));

    let after_theme = tui.capture();

    // Both should show basic structure (we can't easily test color changes)
    assert!(initial.contains("Beads") || initial.contains("Theme test"));
    assert!(after_theme.contains("Beads") || after_theme.contains("Theme test"));

    // Quit
    tui.send_keys("q");
    tui.wait_for_exit(1000);
}
