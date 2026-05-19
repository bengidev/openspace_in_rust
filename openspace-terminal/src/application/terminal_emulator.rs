//! ANSI terminal emulator state machine.
//!
//! Pure logic — no I/O, no PTY, no Iced. Owns the primary grid,
//! scrollback buffer, cursor position, and current SGR attributes.
//! Feed raw PTY bytes via [`Emulator::feed`]; read immutable state
//! via [`Emulator::snapshot`].
//!
//! Part of issue #39.

use vte::{Params, Parser, Perform};

use crate::domain::{Cell, CellAttrs, CursorPos, GridSize, Snapshot};

// ── Emulator ──────────────────────────────────────────────────────────────────

/// ANSI terminal emulator state machine.
pub struct Emulator {
    size: GridSize,
    grid: Vec<Vec<Cell>>,
    scrollback: Vec<Vec<Cell>>,
    cursor: CursorPos,
    attrs: CellAttrs,
    parser: Parser,
}

impl Emulator {
    /// Create a new emulator with a blank grid of the given size.
    pub fn new(size: GridSize) -> Self {
        let grid = blank_grid(size);
        Self {
            size,
            grid,
            scrollback: Vec::new(),
            cursor: CursorPos::default(),
            attrs: CellAttrs::default(),
            parser: Parser::new(),
        }
    }

    /// Current grid dimensions.
    pub fn size(&self) -> GridSize {
        self.size
    }

    /// Current cursor position (zero-based).
    pub fn cursor(&self) -> CursorPos {
        self.cursor
    }

    /// Number of lines in the scrollback buffer.
    pub fn scrollback_len(&self) -> usize {
        self.scrollback.len()
    }

    /// Feed raw bytes from the PTY. Parses ANSI escapes and mutates
    /// internal state. Unsupported sequences are silently ignored.
    pub fn feed(&mut self, bytes: &[u8]) {
        // vte 0.13 advance() takes one byte at a time.
        // We need to temporarily take ownership of the parser to satisfy
        // the borrow checker (Perform is impl'd on Self which also owns parser).
        let mut parser = std::mem::replace(&mut self.parser, Parser::new());
        for &byte in bytes {
            parser.advance(self, byte);
        }
        self.parser = parser;
    }

    /// Snapshot of current emulator state for UI consumption.
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            size: self.size,
            cursor: self.cursor,
            grid: self.grid.clone(),
            scrollback: self.scrollback.clone(),
        }
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    /// Scroll the primary grid up by one line: move the top row into
    /// scrollback and append a blank row at the bottom.
    fn scroll_up(&mut self) {
        let blank = blank_row(self.size.cols);
        let top = std::mem::replace(&mut self.grid[0], blank.clone());
        self.scrollback.push(top);
        self.grid.remove(0);
        self.grid.push(blank);
    }

    /// Write `ch` at the current cursor position with current attrs,
    /// then advance the cursor. Wraps and scrolls as needed.
    fn print_char(&mut self, ch: char) {
        let rows = self.size.rows;
        let cols = self.size.cols;

        // Write the character.
        self.grid[self.cursor.row][self.cursor.col] = Cell {
            ch,
            attrs: self.attrs,
        };

        // Advance column; wrap if needed.
        self.cursor.col += 1;
        if self.cursor.col >= cols {
            self.cursor.col = 0;
            self.cursor.row += 1;
        }

        // Scroll if we've gone past the last row.
        if self.cursor.row >= rows {
            self.scroll_up();
            self.cursor.row = rows - 1;
        }
    }
}

// ── Perform impl ──────────────────────────────────────────────────────────────

impl Perform for Emulator {
    fn print(&mut self, c: char) {
        self.print_char(c);
    }

    fn execute(&mut self, byte: u8) {
        let cols = self.size.cols;
        let rows = self.size.rows;
        match byte {
            // LF — line feed
            0x0A => {
                self.cursor.row += 1;
                if self.cursor.row >= rows {
                    self.scroll_up();
                    self.cursor.row = rows - 1;
                }
            }
            // CR — carriage return
            0x0D => {
                self.cursor.col = 0;
            }
            // BS — backspace
            0x08 => {
                self.cursor.col = self.cursor.col.saturating_sub(1);
            }
            // HT — horizontal tab (next multiple of 8, clamped)
            0x09 => {
                let next = (self.cursor.col / 8 + 1) * 8;
                self.cursor.col = next.min(cols - 1);
            }
            // All other C0/C1 bytes silently ignored.
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {
        // Stub — extended in commits 3 and 4.
    }
}

// ── Grid helpers ──────────────────────────────────────────────────────────────

fn blank_row(cols: usize) -> Vec<Cell> {
    vec![Cell::blank(); cols]
}

fn blank_grid(size: GridSize) -> Vec<Vec<Cell>> {
    (0..size.rows).map(|_| blank_row(size.cols)).collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Cell, CellAttrs, Color, CursorPos, GridSize};

    fn make(rows: usize, cols: usize) -> Emulator {
        Emulator::new(GridSize::new(rows, cols))
    }

    // Helper: feed a string as UTF-8 bytes.
    fn feed_str(em: &mut Emulator, s: &str) {
        em.feed(s.as_bytes());
    }

    #[test]
    fn emulator_new_blank_snapshot_matches_size() {
        let em = make(5, 10);
        let snap = em.snapshot();
        assert_eq!(snap.size, GridSize::new(5, 10));
        assert_eq!(snap.grid.len(), 5);
        for row in &snap.grid {
            assert_eq!(row.len(), 10);
            for cell in row {
                assert_eq!(*cell, Cell::blank());
            }
        }
        assert_eq!(snap.scrollback.len(), 0);
        assert_eq!(snap.cursor, CursorPos { row: 0, col: 0 });
    }

    #[test]
    fn emulator_print_ascii_writes_to_grid() {
        let mut em = make(5, 10);
        feed_str(&mut em, "A");
        let snap = em.snapshot();
        assert_eq!(snap.grid[0][0].ch, 'A');
        assert_eq!(snap.cursor, CursorPos { row: 0, col: 1 });
    }

    #[test]
    fn emulator_print_wraps_at_end_of_row() {
        let mut em = make(5, 4);
        feed_str(&mut em, "ABCDE"); // 5 chars, cols=4 → wrap after 4th
        let snap = em.snapshot();
        assert_eq!(snap.grid[0][0].ch, 'A');
        assert_eq!(snap.grid[0][1].ch, 'B');
        assert_eq!(snap.grid[0][2].ch, 'C');
        assert_eq!(snap.grid[0][3].ch, 'D');
        assert_eq!(snap.grid[1][0].ch, 'E');
        assert_eq!(snap.cursor, CursorPos { row: 1, col: 1 });
    }

    #[test]
    fn emulator_lf_advances_row() {
        let mut em = make(5, 10);
        feed_str(&mut em, "A");
        em.feed(&[0x0A]); // LF
        let snap = em.snapshot();
        assert_eq!(snap.cursor, CursorPos { row: 1, col: 1 });
    }

    #[test]
    fn emulator_cr_resets_column() {
        let mut em = make(5, 10);
        feed_str(&mut em, "ABC");
        em.feed(&[0x0D]); // CR
        let snap = em.snapshot();
        assert_eq!(snap.cursor.col, 0);
        assert_eq!(snap.cursor.row, 0);
    }

    #[test]
    fn emulator_bs_decrements_column_saturating() {
        let mut em = make(5, 10);
        feed_str(&mut em, "AB");
        em.feed(&[0x08]); // BS
        assert_eq!(em.cursor().col, 1);
        em.feed(&[0x08]); // BS again
        assert_eq!(em.cursor().col, 0);
        em.feed(&[0x08]); // BS at col 0 — saturates
        assert_eq!(em.cursor().col, 0);
    }

    #[test]
    fn emulator_ht_advances_to_next_tab_stop() {
        let mut em = make(5, 40);
        // col 0 → tab → col 8
        em.feed(&[0x09]);
        assert_eq!(em.cursor().col, 8);
        // col 8 → tab → col 16
        em.feed(&[0x09]);
        assert_eq!(em.cursor().col, 16);
    }

    #[test]
    fn emulator_print_past_last_row_scrolls_into_scrollback() {
        // 3 rows, 4 cols. Fill 3 rows then add one more line.
        let mut em = make(3, 4);
        feed_str(&mut em, "AAAA"); // row 0 full, wraps to row 1 col 0
        feed_str(&mut em, "BBBB"); // row 1 full, wraps to row 2 col 0
        feed_str(&mut em, "CCCC"); // row 2 full, wraps → scroll
        // After scroll: row 0 (AAAA) pushed to scrollback
        assert_eq!(em.scrollback_len(), 1);
        let snap = em.snapshot();
        // scrollback[0] should be the AAAA row
        assert_eq!(snap.scrollback[0][0].ch, 'A');
        // grid[0] is now BBBB
        assert_eq!(snap.grid[0][0].ch, 'B');
    }

    #[test]
    fn emulator_snapshot_is_cloneable_and_independent() {
        let mut em = make(3, 5);
        feed_str(&mut em, "Hello");
        let snap1 = em.snapshot();
        let mut snap2 = snap1.clone();
        snap2.grid[0][0] = Cell {
            ch: 'X',
            attrs: CellAttrs {
                fg: Color::Red,
                bg: Color::Default,
                bold: false,
            },
        };
        // Original snapshot unchanged
        assert_eq!(snap1.grid[0][0].ch, 'H');
        assert_ne!(snap1.grid[0][0], snap2.grid[0][0]);
    }
}
