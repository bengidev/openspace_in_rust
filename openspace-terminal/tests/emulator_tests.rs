//! Integration tests for the ANSI terminal emulator.
//!
//! Drives Emulator end-to-end with realistic byte sequences.
//! Part of issue #39.

use openspace_terminal::application::Emulator;
use openspace_terminal::domain::{Cell, CellAttrs, Color, CursorPos, GridSize};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make(rows: usize, cols: usize) -> Emulator {
    Emulator::new(GridSize::new(rows, cols))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// "hello\r\nworld" places "hello" at row 0 and "world" at row 1,
/// cursor ends at (1, 5).
#[test]
fn hello_world_two_lines() {
    let mut em = make(10, 20);
    em.feed(b"hello\r\nworld");
    let snap = em.snapshot();

    assert_eq!(snap.grid[0][0].ch, 'h');
    assert_eq!(snap.grid[0][1].ch, 'e');
    assert_eq!(snap.grid[0][2].ch, 'l');
    assert_eq!(snap.grid[0][3].ch, 'l');
    assert_eq!(snap.grid[0][4].ch, 'o');

    assert_eq!(snap.grid[1][0].ch, 'w');
    assert_eq!(snap.grid[1][1].ch, 'o');
    assert_eq!(snap.grid[1][2].ch, 'r');
    assert_eq!(snap.grid[1][3].ch, 'l');
    assert_eq!(snap.grid[1][4].ch, 'd');

    assert_eq!(snap.cursor, CursorPos { row: 1, col: 5 });
}

/// ESC[31m sets fg red on subsequent chars; ESC[0m resets.
#[test]
fn sgr_red_then_reset() {
    let mut em = make(5, 20);
    // Write "RED" in red, then reset and write "X"
    em.feed(b"\x1b[31mRED\x1b[0mX");
    let snap = em.snapshot();

    assert_eq!(snap.grid[0][0].ch, 'R');
    assert_eq!(snap.grid[0][0].attrs.fg, Color::Red);
    assert_eq!(snap.grid[0][1].ch, 'E');
    assert_eq!(snap.grid[0][1].attrs.fg, Color::Red);
    assert_eq!(snap.grid[0][2].ch, 'D');
    assert_eq!(snap.grid[0][2].attrs.fg, Color::Red);

    // After reset, X should have default fg
    assert_eq!(snap.grid[0][3].ch, 'X');
    assert_eq!(snap.grid[0][3].attrs.fg, Color::Default);
    assert!(!snap.grid[0][3].attrs.bold);
}

/// ESC[1;1H places cursor at top-left; write "X"; ESC[2J clears grid;
/// cell (0,0) becomes blank.
#[test]
fn cup_write_then_clear() {
    let mut em = make(5, 20);
    em.feed(b"some text");
    em.feed(b"\x1b[1;1H"); // move to top-left
    em.feed(b"X");
    assert_eq!(em.snapshot().grid[0][0].ch, 'X');

    em.feed(b"\x1b[2J"); // clear entire grid
    let snap = em.snapshot();
    assert_eq!(snap.grid[0][0], Cell::blank());
    // All cells should be blank
    for row in &snap.grid {
        for cell in row {
            assert_eq!(*cell, Cell::blank());
        }
    }
}

/// ESC[H ESC[K erases the first line.
#[test]
fn cup_home_then_erase_line() {
    let mut em = make(5, 10);
    em.feed(b"Hello");
    em.feed(b"\x1b[H"); // move to (0,0)
    em.feed(b"\x1b[K"); // erase to end of line (EL 0)
    let snap = em.snapshot();
    for cell in &snap.grid[0] {
        assert_eq!(*cell, Cell::blank());
    }
    // Row 1 untouched (still blank since we only wrote one line)
    assert_eq!(snap.grid[1][0], Cell::blank());
}

/// A long text run that overflows the grid pushes lines to scrollback.
#[test]
fn overflow_pushes_to_scrollback() {
    let rows = 5;
    let cols = 10;
    let mut em = make(rows, cols);

    // Write enough characters to fill more than `rows` lines.
    // Each line is `cols` chars, so rows+2 lines = (rows+2)*cols chars.
    let line: String = "A".repeat(cols);
    for _ in 0..(rows + 3) {
        em.feed(line.as_bytes());
    }

    assert!(
        em.scrollback_len() > 0,
        "expected scrollback to be non-empty"
    );
}

/// Garbage escape ESC[?1234z is silently ignored; "foo" still prints.
#[test]
fn garbage_escape_ignored_foo_printed() {
    let mut em = make(5, 20);
    // Feed garbage CSI then "foo"
    em.feed(b"\x1b[?1234zfoo");
    let snap = em.snapshot();
    // "foo" should appear starting at (0,0)
    assert_eq!(snap.grid[0][0].ch, 'f');
    assert_eq!(snap.grid[0][1].ch, 'o');
    assert_eq!(snap.grid[0][2].ch, 'o');
}

/// Bold attribute is carried through multiple chars and reset correctly.
#[test]
fn bold_attribute_carried_and_reset() {
    let mut em = make(5, 20);
    em.feed(b"\x1b[1mBOLD\x1b[22mNORMAL");
    let snap = em.snapshot();

    // BOLD chars
    for col in 0..4 {
        assert!(snap.grid[0][col].attrs.bold, "col {col} should be bold");
    }
    // NORMAL chars
    for col in 4..10 {
        assert!(
            !snap.grid[0][col].attrs.bold,
            "col {col} should not be bold"
        );
    }
}

/// 256-color and RGB color sequences work end-to-end.
#[test]
fn extended_colors_end_to_end() {
    let mut em = make(5, 20);
    // 256-color fg
    em.feed(b"\x1b[38;5;200mA");
    // RGB bg
    em.feed(b"\x1b[48;2;10;20;30mB");
    let snap = em.snapshot();

    assert_eq!(snap.grid[0][0].attrs.fg, Color::Indexed(200));
    assert_eq!(snap.grid[0][1].attrs.bg, Color::Rgb(10, 20, 30));
}

/// Cursor movement sequences position the cursor correctly.
#[test]
fn cursor_movement_end_to_end() {
    let mut em = make(10, 20);
    // Place cursor at (5, 10) via CUP
    em.feed(b"\x1b[6;11H");
    assert_eq!(em.cursor(), CursorPos { row: 5, col: 10 });

    // Move up 3, down 1, forward 2, back 4
    em.feed(b"\x1b[3A"); // up 3 → row 2
    em.feed(b"\x1b[1B"); // down 1 → row 3
    em.feed(b"\x1b[2C"); // forward 2 → col 12
    em.feed(b"\x1b[4D"); // back 4 → col 8
    assert_eq!(em.cursor(), CursorPos { row: 3, col: 8 });
}

/// ED 3 clears both grid and scrollback.
#[test]
fn ed3_clears_scrollback() {
    let mut em = make(3, 5);
    // Overflow to build scrollback
    for _ in 0..5 {
        em.feed(b"AAAAA");
    }
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

/// CellAttrs default after SGR reset matches CellAttrs::default().
#[test]
fn sgr_reset_produces_default_attrs() {
    let mut em = make(5, 20);
    em.feed(b"\x1b[1;31;42m"); // bold, red fg, green bg
    em.feed(b"\x1b[0m"); // reset
    em.feed(b"X");
    let cell = em.snapshot().grid[0][0];
    assert_eq!(cell.attrs, CellAttrs::default());
}
