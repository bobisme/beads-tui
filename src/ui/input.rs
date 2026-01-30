//! Text input widget with terminal-style navigation
//!
//! Supports:
//! - Ctrl+A: Move to beginning of line
//! - Ctrl+E: Move to end of line
//! - Ctrl+W: Delete word backward
//! - Ctrl+U: Delete to beginning of line
//! - Ctrl+K: Delete to end of line
//! - Ctrl+B / Left: Move cursor left
//! - Ctrl+F / Right: Move cursor right
//! - Alt+B: Move word backward
//! - Alt+F: Move word forward
//! - Backspace: Delete char before cursor
//! - Delete: Delete char at cursor

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// A text input with cursor position and terminal-style navigation
#[derive(Debug, Clone, Default)]
pub struct TextInput {
    /// The current text
    text: String,
    /// Cursor position (byte index)
    cursor: usize,
}

impl TextInput {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn with_text(text: impl Into<String>) -> Self {
        let text = text.into();
        let cursor = text.len();
        Self { text, cursor }
    }

    /// Get the current text
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get the cursor position
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Clear the input
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }

    /// Handle a key event, returns true if the event was handled
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);

        match key.code {
            // Character input
            KeyCode::Char(c) if !ctrl && !alt => {
                self.insert_char(c);
                true
            }

            // Ctrl+A: Beginning of line (current line in multiline)
            KeyCode::Char('a') if ctrl => {
                self.move_to_line_start();
                true
            }

            // Ctrl+E: End of line (current line in multiline)
            KeyCode::Char('e') if ctrl => {
                self.move_to_line_end();
                true
            }

            // Up arrow: Move to previous line (multiline support)
            KeyCode::Up => {
                self.move_up();
                true
            }

            // Down arrow: Move to next line (multiline support)
            KeyCode::Down => {
                self.move_down();
                true
            }

            // Ctrl+B or Left: Move left
            KeyCode::Char('b') if ctrl => {
                self.move_left();
                true
            }
            KeyCode::Left => {
                self.move_left();
                true
            }

            // Ctrl+F or Right: Move right
            KeyCode::Char('f') if ctrl => {
                self.move_right();
                true
            }
            KeyCode::Right => {
                self.move_right();
                true
            }

            // Alt+B: Move word backward
            KeyCode::Char('b') if alt => {
                self.move_word_backward();
                true
            }

            // Alt+F: Move word forward
            KeyCode::Char('f') if alt => {
                self.move_word_forward();
                true
            }

            // Ctrl+W: Delete word backward
            KeyCode::Char('w') if ctrl => {
                self.delete_word_backward();
                true
            }

            // Ctrl+U: Delete to beginning
            KeyCode::Char('u') if ctrl => {
                self.text = self.text[self.cursor..].to_string();
                self.cursor = 0;
                true
            }

            // Ctrl+K: Delete to end
            KeyCode::Char('k') if ctrl => {
                self.text.truncate(self.cursor);
                true
            }

            // Backspace: Delete char before cursor
            KeyCode::Backspace => {
                self.delete_char_backward();
                true
            }

            // Delete: Delete char at cursor
            KeyCode::Delete => {
                self.delete_char_forward();
                true
            }

            // Home: Beginning of line
            KeyCode::Home => {
                self.cursor = 0;
                true
            }

            // End: End of line
            KeyCode::End => {
                self.cursor = self.text.len();
                true
            }

            _ => false,
        }
    }

    fn insert_char(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    fn move_left(&mut self) {
        if self.cursor > 0 {
            // Move to previous char boundary
            let mut new_cursor = self.cursor - 1;
            while new_cursor > 0 && !self.text.is_char_boundary(new_cursor) {
                new_cursor -= 1;
            }
            self.cursor = new_cursor;
        }
    }

    fn move_right(&mut self) {
        if self.cursor < self.text.len() {
            // Move to next char boundary
            let mut new_cursor = self.cursor + 1;
            while new_cursor < self.text.len() && !self.text.is_char_boundary(new_cursor) {
                new_cursor += 1;
            }
            self.cursor = new_cursor;
        }
    }

    fn move_word_backward(&mut self) {
        if self.cursor == 0 {
            return;
        }

        // Skip whitespace
        let mut pos = self.cursor;
        while pos > 0 {
            let prev = self.prev_char_boundary(pos);
            if !self.text[prev..pos]
                .chars()
                .next()
                .map(|c| c.is_whitespace())
                .unwrap_or(false)
            {
                break;
            }
            pos = prev;
        }

        // Skip word chars
        while pos > 0 {
            let prev = self.prev_char_boundary(pos);
            if self.text[prev..pos]
                .chars()
                .next()
                .map(|c| c.is_whitespace())
                .unwrap_or(true)
            {
                break;
            }
            pos = prev;
        }

        self.cursor = pos;
    }

    fn move_word_forward(&mut self) {
        let len = self.text.len();
        if self.cursor >= len {
            return;
        }

        // Skip current word
        let mut pos = self.cursor;
        while pos < len {
            let next = self.next_char_boundary(pos);
            if self.text[pos..next]
                .chars()
                .next()
                .map(|c| c.is_whitespace())
                .unwrap_or(true)
            {
                break;
            }
            pos = next;
        }

        // Skip whitespace
        while pos < len {
            let next = self.next_char_boundary(pos);
            if !self.text[pos..next]
                .chars()
                .next()
                .map(|c| c.is_whitespace())
                .unwrap_or(false)
            {
                break;
            }
            pos = next;
        }

        self.cursor = pos;
    }

    fn delete_char_backward(&mut self) {
        if self.cursor > 0 {
            let prev = self.prev_char_boundary(self.cursor);
            self.text.drain(prev..self.cursor);
            self.cursor = prev;
        }
    }

    fn delete_char_forward(&mut self) {
        if self.cursor < self.text.len() {
            let next = self.next_char_boundary(self.cursor);
            self.text.drain(self.cursor..next);
        }
    }

    fn delete_word_backward(&mut self) {
        if self.cursor == 0 {
            return;
        }

        let start = self.cursor;

        // Skip whitespace
        while self.cursor > 0 {
            let prev = self.prev_char_boundary(self.cursor);
            if !self.text[prev..self.cursor]
                .chars()
                .next()
                .map(|c| c.is_whitespace())
                .unwrap_or(false)
            {
                break;
            }
            self.cursor = prev;
        }

        // Skip word chars
        while self.cursor > 0 {
            let prev = self.prev_char_boundary(self.cursor);
            if self.text[prev..self.cursor]
                .chars()
                .next()
                .map(|c| c.is_whitespace())
                .unwrap_or(true)
            {
                break;
            }
            self.cursor = prev;
        }

        self.text.drain(self.cursor..start);
    }

    /// Move cursor to start of current line
    fn move_to_line_start(&mut self) {
        // Find previous newline or start of text
        let text_before = &self.text[..self.cursor];
        if let Some(pos) = text_before.rfind('\n') {
            self.cursor = pos + 1; // After the newline
        } else {
            self.cursor = 0; // No newline found, go to start
        }
    }

    /// Move cursor to end of current line
    fn move_to_line_end(&mut self) {
        // Find next newline or end of text
        let text_after = &self.text[self.cursor..];
        if let Some(pos) = text_after.find('\n') {
            self.cursor += pos; // Before the newline
        } else {
            self.cursor = self.text.len(); // No newline found, go to end
        }
    }

    /// Move cursor up one line (multiline navigation)
    fn move_up(&mut self) {
        if self.cursor == 0 {
            return;
        }

        // Find start of current line and column position (in chars)
        let (line_start, col_chars) = self.get_line_start_and_col_chars();

        if line_start == 0 {
            return; // Already on first line
        }

        // Find start of previous line
        let text_before_line = &self.text[..line_start.saturating_sub(1)]; // -1 to skip the \n
        let prev_line_start = text_before_line.rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0);

        // Find end of previous line (before the \n)
        let prev_line_end = line_start.saturating_sub(1);
        let prev_line = &self.text[prev_line_start..prev_line_end];
        let prev_line_chars = prev_line.chars().count();

        // Move to same column (in chars) or end of line if shorter
        let target_col = col_chars.min(prev_line_chars);
        self.cursor = self.byte_offset_for_char_col(prev_line_start, target_col);
    }

    /// Move cursor down one line (multiline navigation)
    fn move_down(&mut self) {
        let text_len = self.text.len();
        if self.cursor >= text_len {
            return;
        }

        // Find start of current line and column position (in chars)
        let (line_start, col_chars) = self.get_line_start_and_col_chars();

        // Find end of current line (next newline or end of text)
        let text_from_line_start = &self.text[line_start..];
        let line_end = if let Some(pos) = text_from_line_start.find('\n') {
            line_start + pos
        } else {
            return; // Already on last line
        };

        // Find start of next line
        let next_line_start = line_end + 1;
        if next_line_start >= text_len {
            return; // No next line
        }

        // Find end of next line
        let text_from_next = &self.text[next_line_start..];
        let next_line_end = text_from_next.find('\n')
            .map(|pos| next_line_start + pos)
            .unwrap_or(text_len);
        let next_line = &self.text[next_line_start..next_line_end];
        let next_line_chars = next_line.chars().count();

        // Move to same column (in chars) or end of line if shorter
        let target_col = col_chars.min(next_line_chars);
        self.cursor = self.byte_offset_for_char_col(next_line_start, target_col);
    }

    /// Get the start position (in bytes) and column (in chars) of the current line
    fn get_line_start_and_col_chars(&self) -> (usize, usize) {
        let text_before = &self.text[..self.cursor];
        let line_start = text_before.rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0);

        // Count characters from line start to cursor
        let line_text = &self.text[line_start..self.cursor];
        let col_chars = line_text.chars().count();

        (line_start, col_chars)
    }

    /// Given a line start byte offset and a character column, return the byte offset for that position
    fn byte_offset_for_char_col(&self, line_start: usize, char_col: usize) -> usize {
        let text_from_start = &self.text[line_start..];

        // Find the byte offset that corresponds to char_col characters
        let mut byte_offset = 0;

        for (chars_seen, ch) in text_from_start.chars().enumerate() {
            if chars_seen >= char_col {
                break;
            }
            if ch == '\n' {
                break; // Don't go past newline
            }
            byte_offset += ch.len_utf8();
        }

        line_start + byte_offset
    }

    fn prev_char_boundary(&self, pos: usize) -> usize {
        let mut new_pos = pos.saturating_sub(1);
        while new_pos > 0 && !self.text.is_char_boundary(new_pos) {
            new_pos -= 1;
        }
        new_pos
    }

    fn next_char_boundary(&self, pos: usize) -> usize {
        let mut new_pos = pos + 1;
        while new_pos < self.text.len() && !self.text.is_char_boundary(new_pos) {
            new_pos += 1;
        }
        new_pos.min(self.text.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    fn alt(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT)
    }

    #[test]
    fn test_insert() {
        let mut input = TextInput::new();
        input.handle_key(key(KeyCode::Char('h')));
        input.handle_key(key(KeyCode::Char('i')));
        assert_eq!(input.text(), "hi");
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn test_ctrl_a_e() {
        let mut input = TextInput::with_text("hello");
        assert_eq!(input.cursor(), 5);

        input.handle_key(ctrl('a'));
        assert_eq!(input.cursor(), 0);

        input.handle_key(ctrl('e'));
        assert_eq!(input.cursor(), 5);
    }

    #[test]
    fn test_ctrl_w() {
        let mut input = TextInput::with_text("hello world");
        input.handle_key(ctrl('w'));
        assert_eq!(input.text(), "hello ");
    }

    #[test]
    fn test_backspace() {
        let mut input = TextInput::with_text("hello");
        input.handle_key(key(KeyCode::Backspace));
        assert_eq!(input.text(), "hell");
    }

    #[test]
    fn test_move_word() {
        let mut input = TextInput::with_text("hello world");
        input.cursor = 0;

        input.handle_key(alt('f'));
        assert_eq!(input.cursor(), 6); // after "hello "

        input.handle_key(alt('b'));
        assert_eq!(input.cursor(), 0);
    }
}
