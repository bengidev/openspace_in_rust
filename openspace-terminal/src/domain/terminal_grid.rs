//! ANSI grid domain types.
//!
//! Pure value types describing the visual state of a terminal grid.
//! No I/O, no Iced, no async — these are plain data shapes consumed
//! by the emulator (application layer) and the UI presenter.

// ── Color ────────────────────────────────────────────────────────────────────

/// A terminal foreground or background color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Color {
    /// Terminal default color (inherits from theme / profile).
    #[default]
    Default,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    /// 256-color palette index (0–255).
    Indexed(u8),
    /// 24-bit RGB color.
    Rgb(u8, u8, u8),
}

// ── CellAttrs ────────────────────────────────────────────────────────────────

/// Visual attributes applied to a single grid cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CellAttrs {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
}

impl CellAttrs {
    /// Returns the SGR-reset attribute set (all fields at their defaults).
    pub const fn reset() -> Self {
        Self {
            fg: Color::Default,
            bg: Color::Default,
            bold: false,
        }
    }
}

// ── Cell ─────────────────────────────────────────────────────────────────────

/// A single character cell in the terminal grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub attrs: CellAttrs,
}

impl Cell {
    /// A blank cell: space character with default attributes.
    pub const fn blank() -> Self {
        Self {
            ch: ' ',
            attrs: CellAttrs::reset(),
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::blank()
    }
}

// ── CursorPos ────────────────────────────────────────────────────────────────

/// Zero-based cursor position within the primary screen grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CursorPos {
    pub row: usize,
    pub col: usize,
}

// ── GridSize ─────────────────────────────────────────────────────────────────

/// Dimensions of the terminal grid in character cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridSize {
    pub rows: usize,
    pub cols: usize,
}

impl GridSize {
    pub const fn new(rows: usize, cols: usize) -> Self {
        Self { rows, cols }
    }
}

impl Default for GridSize {
    /// 80×24 matches the historical VT100 default and the `PtySize`
    /// default in `terminal_types.rs`.
    fn default() -> Self {
        Self::new(24, 80)
    }
}

// ── Snapshot ─────────────────────────────────────────────────────────────────

/// Immutable snapshot of emulator state. Cheap to clone for UI thread consumption.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub size: GridSize,
    pub cursor: CursorPos,
    /// Primary screen grid: `grid[row][col]`.
    pub grid: Vec<Vec<Cell>>,
    /// Lines that have scrolled off the top of the primary screen.
    pub scrollback: Vec<Vec<Cell>>,
}

impl Snapshot {
    /// Constructs a blank snapshot for the given size.
    ///
    /// Every cell is [`Cell::blank()`], scrollback is empty, and the
    /// cursor sits at the origin (0, 0).
    pub fn blank(size: GridSize) -> Self {
        let grid = (0..size.rows)
            .map(|_| vec![Cell::blank(); size.cols])
            .collect();
        Self {
            size,
            cursor: CursorPos::default(),
            grid,
            scrollback: Vec::new(),
        }
    }

    /// Returns a reference to the cell at `(row, col)`, or `None` if
    /// the coordinates are out of bounds.
    pub fn cell_at(&self, row: usize, col: usize) -> Option<&Cell> {
        self.grid.get(row)?.get(col)
    }

    /// Number of lines currently in the scrollback buffer.
    pub fn scrollback_len(&self) -> usize {
        self.scrollback.len()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_default_is_default_variant() {
        assert_eq!(Color::default(), Color::Default);
    }

    #[test]
    fn cell_blank_is_space_with_default_attrs() {
        let cell = Cell::blank();
        assert_eq!(cell.ch, ' ');
        assert_eq!(cell.attrs, CellAttrs::default());
    }

    #[test]
    fn cell_attrs_reset_equals_default() {
        assert_eq!(CellAttrs::reset(), CellAttrs::default());
    }

    #[test]
    fn grid_size_default_is_24x80() {
        let size = GridSize::default();
        assert_eq!(size.rows, 24);
        assert_eq!(size.cols, 80);
    }

    #[test]
    fn snapshot_blank_fills_grid_with_blank_cells() {
        let size = GridSize::new(3, 5);
        let snap = Snapshot::blank(size);
        assert_eq!(snap.grid.len(), 3);
        for row in &snap.grid {
            assert_eq!(row.len(), 5);
            for cell in row {
                assert_eq!(*cell, Cell::blank());
            }
        }
    }

    #[test]
    fn snapshot_cell_at_returns_none_for_out_of_bounds() {
        let snap = Snapshot::blank(GridSize::new(2, 2));
        assert!(snap.cell_at(2, 0).is_none());
        assert!(snap.cell_at(0, 2).is_none());
        assert!(snap.cell_at(10, 10).is_none());
    }

    #[test]
    fn snapshot_cell_at_returns_some_for_inbounds() {
        let snap = Snapshot::blank(GridSize::new(4, 8));
        assert_eq!(snap.cell_at(0, 0), Some(&Cell::blank()));
        assert_eq!(snap.cell_at(3, 7), Some(&Cell::blank()));
    }

    #[test]
    fn snapshot_scrollback_len_is_zero_for_blank() {
        let snap = Snapshot::blank(GridSize::default());
        assert_eq!(snap.scrollback_len(), 0);
    }

    #[test]
    fn snapshot_is_cloneable() {
        let original = Snapshot::blank(GridSize::new(2, 2));
        let mut cloned = original.clone();
        // Mutate the clone's grid.
        cloned.grid[0][0] = Cell {
            ch: 'X',
            attrs: CellAttrs {
                fg: Color::Red,
                bg: Color::Default,
                bold: true,
            },
        };
        // Original must be unchanged.
        assert_eq!(original.grid[0][0], Cell::blank());
        assert_ne!(original.grid[0][0], cloned.grid[0][0]);
    }
}
