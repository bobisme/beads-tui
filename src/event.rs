//! Event handling for beads-tui

#![allow(dead_code)]

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

/// Poll for an event with a timeout
pub fn poll_event(timeout: Duration) -> Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

/// Check if a key event is a quit command
pub fn is_quit(key: &KeyEvent) -> bool {
    matches!(
        key,
        KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            ..
        } | KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }
    )
}

/// Check if a key is navigation up
pub fn is_up(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Up | KeyCode::Char('k'))
}

/// Check if a key is navigation down
pub fn is_down(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Down | KeyCode::Char('j'))
}

/// Check if a key is go to first
pub fn is_first(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Home | KeyCode::Char('g'))
}

/// Check if a key is go to last
pub fn is_last(key: &KeyEvent) -> bool {
    matches!(
        key,
        KeyEvent {
            code: KeyCode::End,
            ..
        } | KeyEvent {
            code: KeyCode::Char('G'),
            modifiers: KeyModifiers::SHIFT,
            ..
        }
    )
}
