use iced::Color;

pub struct ThemeColors;

impl ThemeColors {
    pub const BG: Color = Color::from_rgb(0.06, 0.06, 0.08);
    pub const FG: Color = Color::from_rgb(0.85, 0.85, 0.90);
    pub const FG_MUTED: Color = Color::from_rgb(0.55, 0.55, 0.60);
    pub const FG_DIM: Color = Color::from_rgb(0.35, 0.35, 0.40);
    pub const SURFACE: Color = Color::from_rgba(0.85, 0.85, 0.90, 0.08);
    pub const BORDER: Color = Color::from_rgba(0.85, 0.85, 0.90, 0.12);
    pub const HOVER: Color = Color::from_rgba(0.85, 0.85, 0.90, 0.06);
    pub const ACCENT: Color = Color::from_rgb(0.55, 0.50, 1.0);
    pub const WARNING: Color = Color::from_rgb(1.0, 0.8, 0.2);
}
