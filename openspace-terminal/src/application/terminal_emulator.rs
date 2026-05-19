//! ANSI terminal emulator state machine.
//!
//! Pure logic — no I/O, no PTY, no Iced. Owns the primary grid,
//! scrollback buffer, cursor position, and current SGR attributes.
//! Feed raw PTY bytes via [`Emulator::feed`]; read immutable state
//! via [`Emulator::snapshot`].
//!
//! Part of issue #39.

use vte::{Params, Parser, Perform};

use crate::domain::{Cell, CellAttrs, Color, CursorPos, GridSize, Snapshot};

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

    fn csi_dispatch(
        &mut self,
        params: &Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        // Helper: first param value, defaulting to `default` if absent or zero.
        let p0 = |default: usize| -> usize {
            params
                .iter()
                .next()
                .and_then(|s| s.first().copied())
                .map(|v| if v == 0 { default } else { v as usize })
                .unwrap_or(default)
        };

        let rows = self.size.rows;
        let cols = self.size.cols;

        match action {
            // SGR — Select Graphic Rendition
            'm' => apply_sgr(&mut self.attrs, params),

            // CUU — cursor up
            'A' => {
                let n = p0(1);
                self.cursor.row = self.cursor.row.saturating_sub(n);
            }
            // CUD — cursor down
            'B' => {
                let n = p0(1);
                self.cursor.row = (self.cursor.row + n).min(rows - 1);
            }
            // CUF — cursor forward
            'C' => {
                let n = p0(1);
                self.cursor.col = (self.cursor.col + n).min(cols - 1);
            }
            // CUB — cursor back
            'D' => {
                let n = p0(1);
                self.cursor.col = self.cursor.col.saturating_sub(n);
            }
            // CUP / HVP — set cursor position (1-based params)
            'H' | 'f' => {
                let mut iter = params.iter();
                let row = iter
                    .next()
                    .and_then(|s| s.first().copied())
                    .map(|v| if v == 0 { 1 } else { v as usize })
                    .unwrap_or(1);
                let col = iter
                    .next()
                    .and_then(|s| s.first().copied())
                    .map(|v| if v == 0 { 1 } else { v as usize })
                    .unwrap_or(1);
                self.cursor.row = (row - 1).min(rows - 1);
                self.cursor.col = (col - 1).min(cols - 1);
            }
            // CHA — cursor horizontal absolute (1-based)
            'G' => {
                let n = p0(1);
                self.cursor.col = (n - 1).min(cols - 1);
            }
            // ED — erase in display
            'J' => {
                let n = params
                    .iter()
                    .next()
                    .and_then(|s| s.first().copied())
                    .unwrap_or(0);
                match n {
                    // 0 or absent: cursor to end of grid
                    0 => {
                        let (r, c) = (self.cursor.row, self.cursor.col);
                        for col in c..cols {
                            self.grid[r][col] = Cell::blank();
                        }
                        for row in (r + 1)..rows {
                            for col in 0..cols {
                                self.grid[row][col] = Cell::blank();
                            }
                        }
                    }
                    // 1: start of grid to cursor (inclusive)
                    1 => {
                        let (r, c) = (self.cursor.row, self.cursor.col);
                        for row in 0..r {
                            for col in 0..cols {
                                self.grid[row][col] = Cell::blank();
                            }
                        }
                        for col in 0..=c {
                            self.grid[r][col] = Cell::blank();
                        }
                    }
                    // 2: erase entire grid (cursor unchanged)
                    2 => {
                        for row in 0..rows {
                            for col in 0..cols {
                                self.grid[row][col] = Cell::blank();
                            }
                        }
                    }
                    // 3: erase entire grid AND clear scrollback
                    3 => {
                        for row in 0..rows {
                            for col in 0..cols {
                                self.grid[row][col] = Cell::blank();
                            }
                        }
                        self.scrollback.clear();
                    }
                    _ => {}
                }
            }
            // EL — erase in line
            'K' => {
                let n = params
                    .iter()
                    .next()
                    .and_then(|s| s.first().copied())
                    .unwrap_or(0);
                let (r, c) = (self.cursor.row, self.cursor.col);
                match n {
                    // 0 or absent: cursor to end of line
                    0 => {
                        for col in c..cols {
                            self.grid[r][col] = Cell::blank();
                        }
                    }
                    // 1: start of line to cursor (inclusive)
                    1 => {
                        for col in 0..=c {
                            self.grid[r][col] = Cell::blank();
                        }
                    }
                    // 2: entire line
                    2 => {
                        for col in 0..cols {
                            self.grid[r][col] = Cell::blank();
                        }
                    }
                    _ => {}
                }
            }
            // All other CSI actions silently ignored.
            _ => {}
        }
    }
}

// ── SGR handler ───────────────────────────────────────────────────────────────

/// Apply SGR (Select Graphic Rendition) params to `attrs`.
///
/// Each item from `params.iter()` is a `&[u16]` sub-param slice.
/// For most codes it is a single element. For 256-color / RGB the
/// colour type code (38 or 48) and its arguments arrive as separate
/// params (e.g. `[38]`, `[5]`, `[n]`).
fn apply_sgr(attrs: &mut CellAttrs, params: &Params) {
    let mut iter = params.iter().peekable();

    // Empty params list means SGR 0 (reset).
    if iter.peek().is_none() {
        *attrs = CellAttrs::default();
        return;
    }

    while let Some(param) = iter.next() {
        let code = param.first().copied().unwrap_or(0);
        match code {
            // Reset
            0 => *attrs = CellAttrs::default(),
            // Bold on / off
            1 => attrs.bold = true,
            22 => attrs.bold = false,
            // Foreground basic colors (30–37)
            30 => attrs.fg = Color::Black,
            31 => attrs.fg = Color::Red,
            32 => attrs.fg = Color::Green,
            33 => attrs.fg = Color::Yellow,
            34 => attrs.fg = Color::Blue,
            35 => attrs.fg = Color::Magenta,
            36 => attrs.fg = Color::Cyan,
            37 => attrs.fg = Color::White,
            39 => attrs.fg = Color::Default,
            // Background basic colors (40–47)
            40 => attrs.bg = Color::Black,
            41 => attrs.bg = Color::Red,
            42 => attrs.bg = Color::Green,
            43 => attrs.bg = Color::Yellow,
            44 => attrs.bg = Color::Blue,
            45 => attrs.bg = Color::Magenta,
            46 => attrs.bg = Color::Cyan,
            47 => attrs.bg = Color::White,
            49 => attrs.bg = Color::Default,
            // Bright foreground (90–97)
            90 => attrs.fg = Color::BrightBlack,
            91 => attrs.fg = Color::BrightRed,
            92 => attrs.fg = Color::BrightGreen,
            93 => attrs.fg = Color::BrightYellow,
            94 => attrs.fg = Color::BrightBlue,
            95 => attrs.fg = Color::BrightMagenta,
            96 => attrs.fg = Color::BrightCyan,
            97 => attrs.fg = Color::BrightWhite,
            // Bright background (100–107)
            100 => attrs.bg = Color::BrightBlack,
            101 => attrs.bg = Color::BrightRed,
            102 => attrs.bg = Color::BrightGreen,
            103 => attrs.bg = Color::BrightYellow,
            104 => attrs.bg = Color::BrightBlue,
            105 => attrs.bg = Color::BrightMagenta,
            106 => attrs.bg = Color::BrightCyan,
            107 => attrs.bg = Color::BrightWhite,
            // Extended color: 38 or 48 followed by 5;n or 2;r;g;b
            38 | 48 => {
                let is_fg = code == 38;
                // Peek at next param for the sub-type (5 = indexed, 2 = RGB).
                if let Some(sub) = iter.next() {
                    let sub_code = sub.first().copied().unwrap_or(0);
                    match sub_code {
                        5 => {
                            // 256-color indexed
                            if let Some(idx_param) = iter.next() {
                                let idx = idx_param.first().copied().unwrap_or(0) as u8;
                                if is_fg {
                                    attrs.fg = Color::Indexed(idx);
                                } else {
                                    attrs.bg = Color::Indexed(idx);
                                }
                            }
                        }
                        2 => {
                            // 24-bit RGB
                            let r = iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                            let g = iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                            let b = iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as u8;
                            if is_fg {
                                attrs.fg = Color::Rgb(r, g, b);
                            } else {
                                attrs.bg = Color::Rgb(r, g, b);
                            }
                        }
                        // Unknown sub-type — ignore.
                        _ => {}
                    }
                }
            }
            // Unknown SGR code — silently ignore.
            _ => {}
        }
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

    // ── SGR tests ─────────────────────────────────────────────────────────────

    // Helper: build a CSI SGR byte sequence from a list of numeric params.
    // e.g. sgr(&[1]) → b"\x1b[1m", sgr(&[38,5,200]) → b"\x1b[38;5;200m"
    fn sgr_bytes(codes: &[u16]) -> Vec<u8> {
        let parts: Vec<String> = codes.iter().map(|n| n.to_string()).collect();
        format!("\x1b[{}m", parts.join(";")).into_bytes()
    }

    #[test]
    fn emulator_sgr_bold_sets_bold_attr() {
        let mut em = make(5, 10);
        em.feed(&sgr_bytes(&[1]));
        em.feed(b"A");
        assert!(em.snapshot().grid[0][0].attrs.bold);
    }

    #[test]
    fn emulator_sgr_reset_clears_bold() {
        let mut em = make(5, 10);
        em.feed(&sgr_bytes(&[1])); // bold on
        em.feed(&sgr_bytes(&[0])); // reset
        em.feed(b"A");
        assert!(!em.snapshot().grid[0][0].attrs.bold);
    }

    #[test]
    fn emulator_sgr_fg_30_sets_black() {
        let mut em = make(5, 10);
        em.feed(&sgr_bytes(&[30]));
        em.feed(b"A");
        assert_eq!(em.snapshot().grid[0][0].attrs.fg, Color::Black);
    }

    #[test]
    fn emulator_sgr_fg_31_sets_red() {
        let mut em = make(5, 10);
        em.feed(&sgr_bytes(&[31]));
        em.feed(b"A");
        assert_eq!(em.snapshot().grid[0][0].attrs.fg, Color::Red);
    }

    #[test]
    fn emulator_sgr_bg_44_sets_blue() {
        let mut em = make(5, 10);
        em.feed(&sgr_bytes(&[44]));
        em.feed(b"A");
        assert_eq!(em.snapshot().grid[0][0].attrs.bg, Color::Blue);
    }

    #[test]
    fn emulator_sgr_fg_91_sets_bright_red() {
        let mut em = make(5, 10);
        em.feed(&sgr_bytes(&[91]));
        em.feed(b"A");
        assert_eq!(em.snapshot().grid[0][0].attrs.fg, Color::BrightRed);
    }

    #[test]
    fn emulator_sgr_fg_indexed_38_5_sets_indexed() {
        let mut em = make(5, 10);
        em.feed(&sgr_bytes(&[38, 5, 200]));
        em.feed(b"A");
        assert_eq!(em.snapshot().grid[0][0].attrs.fg, Color::Indexed(200));
    }

    #[test]
    fn emulator_sgr_fg_rgb_38_2_sets_rgb() {
        let mut em = make(5, 10);
        em.feed(&sgr_bytes(&[38, 2, 10, 20, 30]));
        em.feed(b"A");
        assert_eq!(em.snapshot().grid[0][0].attrs.fg, Color::Rgb(10, 20, 30));
    }

    #[test]
    fn emulator_sgr_default_param_resets() {
        let mut em = make(5, 10);
        em.feed(&sgr_bytes(&[1])); // bold on
        em.feed(&sgr_bytes(&[31])); // fg red
        // Empty SGR (ESC[m) should reset
        em.feed(b"\x1b[m");
        em.feed(b"A");
        let cell = em.snapshot().grid[0][0];
        assert!(!cell.attrs.bold);
        assert_eq!(cell.attrs.fg, Color::Default);
    }

    // ── Cursor move + erase tests ─────────────────────────────────────────────

    #[test]
    fn emulator_cup_sets_cursor_position() {
        let mut em = make(10, 20);
        em.feed(b"\x1b[5;10H"); // row 5, col 10 (1-based) → (4, 9) zero-based
        assert_eq!(em.cursor(), CursorPos { row: 4, col: 9 });
    }

    #[test]
    fn emulator_cup_clamps_to_grid() {
        let mut em = make(5, 10);
        em.feed(b"\x1b[99;99H");
        assert_eq!(em.cursor(), CursorPos { row: 4, col: 9 });
    }

    #[test]
    fn emulator_cuu_cud_cuf_cub_move_cursor() {
        let mut em = make(10, 20);
        // Start at (5, 10)
        em.feed(b"\x1b[6;11H");
        assert_eq!(em.cursor(), CursorPos { row: 5, col: 10 });
        // Up 2
        em.feed(b"\x1b[2A");
        assert_eq!(em.cursor(), CursorPos { row: 3, col: 10 });
        // Down 1
        em.feed(b"\x1b[1B");
        assert_eq!(em.cursor(), CursorPos { row: 4, col: 10 });
        // Forward 3
        em.feed(b"\x1b[3C");
        assert_eq!(em.cursor(), CursorPos { row: 4, col: 13 });
        // Back 5
        em.feed(b"\x1b[5D");
        assert_eq!(em.cursor(), CursorPos { row: 4, col: 8 });
    }

    #[test]
    fn emulator_cha_sets_column() {
        let mut em = make(5, 20);
        em.feed(b"\x1b[10G"); // col 10 (1-based) → 9 zero-based
        assert_eq!(em.cursor().col, 9);
    }

    #[test]
    fn emulator_ed_0_erases_from_cursor() {
        let mut em = make(3, 5);
        feed_str(&mut em, "AAAAA");
        feed_str(&mut em, "BBBBB");
        feed_str(&mut em, "CCCCC");
        // After 3 full rows in a 3-row grid: AAAAA scrolled off,
        // grid = [BBBBB, CCCCC, blank], cursor at (2, 0).
        // Move to (1, 2) and erase forward.
        em.feed(b"\x1b[2;3H"); // row 2, col 3 (1-based) → (1, 2)
        em.feed(b"\x1b[0J");
        let snap = em.snapshot();
        // (1,0) and (1,1) should still be 'C' (CCCCC row)
        assert_eq!(snap.grid[1][0].ch, 'C');
        assert_eq!(snap.grid[1][1].ch, 'C');
        // (1,2) onward and all of row 2 should be blank
        assert_eq!(snap.grid[1][2], Cell::blank());
        assert_eq!(snap.grid[2][0], Cell::blank());
    }

    #[test]
    fn emulator_ed_2_clears_entire_grid() {
        let mut em = make(3, 5);
        feed_str(&mut em, "Hello");
        em.feed(b"\x1b[2J");
        let snap = em.snapshot();
        for row in &snap.grid {
            for cell in row {
                assert_eq!(*cell, Cell::blank());
            }
        }
        // Cursor unchanged (still wherever it was)
    }

    #[test]
    fn emulator_ed_3_clears_grid_and_scrollback() {
        let mut em = make(2, 4);
        // Fill enough to push lines into scrollback
        feed_str(&mut em, "AAAA");
        feed_str(&mut em, "BBBB");
        feed_str(&mut em, "CCCC");
        assert!(em.scrollback_len() > 0);
        em.feed(b"\x1b[3J");
        assert_eq!(em.scrollback_len(), 0);
        let snap = em.snapshot();
        for row in &snap.grid {
            for cell in row {
                assert_eq!(*cell, Cell::blank());
            }
        }
    }

    #[test]
    fn emulator_el_0_erases_to_eol() {
        let mut em = make(3, 5);
        feed_str(&mut em, "ABCDE");
        em.feed(b"\x1b[1;3H"); // move to (0, 2)
        em.feed(b"\x1b[0K");
        let snap = em.snapshot();
        assert_eq!(snap.grid[0][0].ch, 'A');
        assert_eq!(snap.grid[0][1].ch, 'B');
        assert_eq!(snap.grid[0][2], Cell::blank());
        assert_eq!(snap.grid[0][3], Cell::blank());
        assert_eq!(snap.grid[0][4], Cell::blank());
    }

    #[test]
    fn emulator_el_1_erases_from_bol() {
        let mut em = make(3, 5);
        feed_str(&mut em, "ABCDE");
        em.feed(b"\x1b[1;4H"); // move to (0, 3)
        em.feed(b"\x1b[1K");
        let snap = em.snapshot();
        assert_eq!(snap.grid[0][0], Cell::blank());
        assert_eq!(snap.grid[0][1], Cell::blank());
        assert_eq!(snap.grid[0][2], Cell::blank());
        assert_eq!(snap.grid[0][3], Cell::blank());
        assert_eq!(snap.grid[0][4].ch, 'E');
    }

    #[test]
    fn emulator_el_2_erases_entire_line() {
        let mut em = make(3, 5);
        feed_str(&mut em, "ABCDE");
        em.feed(b"\x1b[1;3H"); // move to (0, 2)
        em.feed(b"\x1b[2K");
        let snap = em.snapshot();
        for cell in &snap.grid[0] {
            assert_eq!(*cell, Cell::blank());
        }
    }

    #[test]
    fn emulator_unsupported_csi_is_silently_ignored() {
        let mut em = make(3, 10);
        feed_str(&mut em, "hello");
        let before = em.snapshot();
        // ESC[?25h (show cursor — private mode, not implemented)
        em.feed(b"\x1b[?25h");
        let after = em.snapshot();
        // Grid and cursor must be unchanged
        assert_eq!(before.grid, after.grid);
        assert_eq!(before.cursor, after.cursor);
    }
}
