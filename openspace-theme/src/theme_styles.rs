use iced::Color;

pub struct ThemeColors;

impl ThemeColors {
    pub const BG: Color = Color::from_rgb(0.008, 0.008, 0.008); // #020202
    pub const BG_SECONDARY: Color = Color::from_rgb(0.063, 0.063, 0.063); // #101010
    pub const SURFACE: Color = Color::from_rgb(0.039, 0.039, 0.039); // #0a0a0a
    pub const ELEVATED_SURFACE: Color = Color::from_rgb(0.086, 0.086, 0.086); // #161616
    pub const FG: Color = Color::from_rgb(0.933, 0.933, 0.933); // #eeeeee
    pub const FG_SECONDARY: Color = Color::from_rgb(0.643, 0.616, 0.604); // #a49d9a
    pub const FG_MUTED: Color = Color::from_rgb(0.541, 0.514, 0.502); // #8a8380
    pub const BORDER: Color = Color::from_rgb(0.239, 0.227, 0.224); // #3d3a39
    pub const BORDER_STRONG: Color = Color::from_rgb(0.302, 0.286, 0.278); // #4d4947
    pub const ACCENT: Color = Color::from_rgb(0.937, 0.435, 0.180); // #ef6f2e
    pub const ACCENT_SOFT: Color = Color::from_rgb(0.933, 0.376, 0.094); // #ee6018
    pub const ACCENT_TEXT: Color = Color::from_rgb(1.0, 1.0, 1.0); // #ffffff
    pub const PRIMARY_FILL: Color = Color::from_rgb(0.933, 0.933, 0.933); // #eeeeee
    pub const PRIMARY_TEXT: Color = Color::from_rgb(0.008, 0.008, 0.008); // #020202
    pub const WARNING: Color = Color::from_rgb(1.0, 0.8, 0.2);
}
