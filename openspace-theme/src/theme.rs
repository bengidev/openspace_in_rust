use iced::Color;

use crate::tokens::*;

/// Resolved semantic theme for OpenSpace.
///
/// Holds concrete [`Color`] values for every semantic token.
/// Use the category methods (`background`, `foreground`, `border`,
/// `status`, `mode_accent`) to look up colours in a type-safe way.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OpenSpaceTheme {
    // Backgrounds
    pub background_primary: Color,
    pub background_secondary: Color,
    pub background_tertiary: Color,
    pub background_elevated: Color,

    // Foregrounds
    pub foreground_primary: Color,
    pub foreground_secondary: Color,
    pub foreground_muted: Color,
    pub foreground_accent: Color,

    // Borders
    pub border_subtle: Color,
    pub border_default: Color,
    pub border_strong: Color,

    // Status
    pub status_success: Color,
    pub status_warning: Color,
    pub status_error: Color,
    pub status_info: Color,

    // Mode accents
    pub mode_terminal: Color,
    pub mode_chat: Color,
    pub mode_editor: Color,
}

impl OpenSpaceTheme {
    // ------------------------------------------------------------------
    // Token resolution
    // ------------------------------------------------------------------

    /// Resolve a [`BackgroundToken`].
    pub fn background(&self, token: BackgroundToken) -> Color {
        match token {
            BackgroundToken::Primary => self.background_primary,
            BackgroundToken::Secondary => self.background_secondary,
            BackgroundToken::Tertiary => self.background_tertiary,
            BackgroundToken::Elevated => self.background_elevated,
        }
    }

    /// Resolve a [`ForegroundToken`].
    pub fn foreground(&self, token: ForegroundToken) -> Color {
        match token {
            ForegroundToken::Primary => self.foreground_primary,
            ForegroundToken::Secondary => self.foreground_secondary,
            ForegroundToken::Muted => self.foreground_muted,
            ForegroundToken::Accent => self.foreground_accent,
        }
    }

    /// Resolve a [`BorderToken`].
    pub fn border(&self, token: BorderToken) -> Color {
        match token {
            BorderToken::Subtle => self.border_subtle,
            BorderToken::Default => self.border_default,
            BorderToken::Strong => self.border_strong,
        }
    }

    /// Resolve a [`StatusToken`].
    pub fn status(&self, token: StatusToken) -> Color {
        match token {
            StatusToken::Success => self.status_success,
            StatusToken::Warning => self.status_warning,
            StatusToken::Error => self.status_error,
            StatusToken::Info => self.status_info,
        }
    }

    /// Resolve a [`ModeAccentToken`].
    pub fn mode_accent(&self, token: ModeAccentToken) -> Color {
        match token {
            ModeAccentToken::Terminal => self.mode_terminal,
            ModeAccentToken::Chat => self.mode_chat,
            ModeAccentToken::Editor => self.mode_editor,
        }
    }

    // ------------------------------------------------------------------
    // Iced integration
    // ------------------------------------------------------------------

    /// Build an [`iced::Theme`] from the resolved tokens.
    ///
    /// Maps the semantic tokens onto Iced’s built-in [`Palette`] so
    /// standard widgets pick up the correct base / text / primary /
    /// success / warning / danger colours automatically.
    pub fn to_iced_theme(&self) -> iced::Theme {
        let palette = iced::theme::Palette {
            background: self.background_primary,
            text: self.foreground_primary,
            primary: self.foreground_accent,
            success: self.status_success,
            warning: self.status_warning,
            danger: self.status_error,
        };
        iced::Theme::custom(self.mode_name(), palette)
    }

    // ------------------------------------------------------------------
    // Palette constructors
    // ------------------------------------------------------------------

    /// Dark palette optimised for long developer sessions.
    ///
    /// * High-contrast code/content (light grey text on near-black).
    /// * Muted chrome (subdued secondary surfaces).
    /// * Clear semantic status colours (green / yellow / red / blue).
    /// * No glass / frosted effects.
    pub const fn dark() -> Self {
        Self {
            // Backgrounds — near-black, stepped greys
            background_primary: Color::from_rgb(0.008, 0.008, 0.008), // #020202
            background_secondary: Color::from_rgb(0.063, 0.063, 0.063), // #101010
            background_tertiary: Color::from_rgb(0.039, 0.039, 0.039), // #0a0a0a
            background_elevated: Color::from_rgb(0.086, 0.086, 0.086), // #161616

            // Foregrounds — warm grey scale
            foreground_primary: Color::from_rgb(0.933, 0.933, 0.933), // #eeeeee
            foreground_secondary: Color::from_rgb(0.643, 0.616, 0.604), // #a49d9a
            foreground_muted: Color::from_rgb(0.541, 0.514, 0.502),   // #8a8380
            foreground_accent: Color::from_rgb(0.937, 0.435, 0.180),  // #ef6f2e

            // Borders — low-chroma greys
            border_subtle: Color::from_rgb(0.141, 0.133, 0.129), // #242221
            border_default: Color::from_rgb(0.239, 0.227, 0.224), // #3d3a39
            border_strong: Color::from_rgb(0.302, 0.286, 0.278), // #4d4947

            // Status — vivid but not neon
            status_success: Color::from_rgb(0.298, 0.686, 0.314), // #4caf50
            status_warning: Color::from_rgb(1.0, 0.8, 0.2),       // #ffcc33
            status_error: Color::from_rgb(0.957, 0.263, 0.212),   // #f44336
            status_info: Color::from_rgb(0.129, 0.588, 0.953),    // #2196f3

            // Mode accents — tie to semantic colours
            mode_terminal: Color::from_rgb(0.298, 0.686, 0.314), // success green
            mode_chat: Color::from_rgb(0.937, 0.435, 0.180),     // accent orange
            mode_editor: Color::from_rgb(0.129, 0.588, 0.953),   // info blue
        }
    }

    /// Light palette for bright environments.
    ///
    /// * Soft off-white backgrounds to reduce eye strain.
    /// * Dark grey text for strong readability.
    /// * Same accent orange to preserve brand identity.
    /// * Slightly softened status colours for light surfaces.
    pub const fn light() -> Self {
        Self {
            // Backgrounds — off-white, stepped
            background_primary: Color::from_rgb(0.98, 0.98, 0.98), // #fafafa
            background_secondary: Color::from_rgb(0.941, 0.941, 0.941), // #f0f0f0
            background_tertiary: Color::from_rgb(0.91, 0.91, 0.91), // #e8e8e8
            background_elevated: Color::from_rgb(1.0, 1.0, 1.0),   // #ffffff

            // Foregrounds — dark grey scale
            foreground_primary: Color::from_rgb(0.102, 0.102, 0.102), // #1a1a1a
            foreground_secondary: Color::from_rgb(0.4, 0.4, 0.4),     // #666666
            foreground_muted: Color::from_rgb(0.6, 0.6, 0.6),         // #999999
            foreground_accent: Color::from_rgb(0.937, 0.435, 0.180),  // #ef6f2e

            // Borders — light greys
            border_subtle: Color::from_rgb(0.933, 0.933, 0.933), // #eeeeee
            border_default: Color::from_rgb(0.816, 0.816, 0.816), // #d0d0d0
            border_strong: Color::from_rgb(0.69, 0.69, 0.69),    // #b0b0b0

            // Status — softened for light backgrounds
            status_success: Color::from_rgb(0.263, 0.627, 0.278), // #43a047
            status_warning: Color::from_rgb(1.0, 0.655, 0.149),   // #ffa726
            status_error: Color::from_rgb(0.898, 0.224, 0.208),   // #e53935
            status_info: Color::from_rgb(0.118, 0.533, 0.898),    // #1e88e5

            // Mode accents
            mode_terminal: Color::from_rgb(0.263, 0.627, 0.278), // success green
            mode_chat: Color::from_rgb(0.937, 0.435, 0.180),     // accent orange
            mode_editor: Color::from_rgb(0.118, 0.533, 0.898),   // info blue
        }
    }

    /// Convenience: build from a [`ThemeMode`].
    pub const fn from_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::dark(),
            ThemeMode::Light => Self::light(),
        }
    }

    /// Returns the name used for the iced custom theme.
    pub const fn mode_name(&self) -> &'static str {
        // We compare background_primary as a heuristic since the struct
        // does not store the mode variant directly.
        if self.background_primary.r < 0.5 {
            "OpenSpace Dark"
        } else {
            "OpenSpace Light"
        }
    }
}

impl Default for OpenSpaceTheme {
    fn default() -> Self {
        Self::dark()
    }
}

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_valid_color(c: Color) {
        assert!(
            c.r.is_finite() && c.g.is_finite() && c.b.is_finite() && c.a.is_finite(),
            "color components must be finite, got {:?}",
            c
        );
        assert!(
            (0.0..=1.0).contains(&c.r)
                && (0.0..=1.0).contains(&c.g)
                && (0.0..=1.0).contains(&c.b)
                && (0.0..=1.0).contains(&c.a),
            "color components must be in [0,1], got {:?}",
            c
        );
    }

    #[test]
    fn dark_palette_resolves_all_tokens() {
        let theme = OpenSpaceTheme::dark();

        // Background tokens
        assert_valid_color(theme.background(BackgroundToken::Primary));
        assert_valid_color(theme.background(BackgroundToken::Secondary));
        assert_valid_color(theme.background(BackgroundToken::Tertiary));
        assert_valid_color(theme.background(BackgroundToken::Elevated));

        // Foreground tokens
        assert_valid_color(theme.foreground(ForegroundToken::Primary));
        assert_valid_color(theme.foreground(ForegroundToken::Secondary));
        assert_valid_color(theme.foreground(ForegroundToken::Muted));
        assert_valid_color(theme.foreground(ForegroundToken::Accent));

        // Border tokens
        assert_valid_color(theme.border(BorderToken::Subtle));
        assert_valid_color(theme.border(BorderToken::Default));
        assert_valid_color(theme.border(BorderToken::Strong));

        // Status tokens
        assert_valid_color(theme.status(StatusToken::Success));
        assert_valid_color(theme.status(StatusToken::Warning));
        assert_valid_color(theme.status(StatusToken::Error));
        assert_valid_color(theme.status(StatusToken::Info));

        // Mode-accent tokens
        assert_valid_color(theme.mode_accent(ModeAccentToken::Terminal));
        assert_valid_color(theme.mode_accent(ModeAccentToken::Chat));
        assert_valid_color(theme.mode_accent(ModeAccentToken::Editor));
    }

    #[test]
    fn to_iced_theme_produces_valid_custom_theme() {
        let theme = OpenSpaceTheme::dark();
        let iced = theme.to_iced_theme();

        let palette = iced.palette();
        assert_valid_color(palette.background);
        assert_valid_color(palette.text);
        assert_valid_color(palette.primary);
        assert_valid_color(palette.success);
        assert_valid_color(palette.warning);
        assert_valid_color(palette.danger);

        // Verify the semantic intent is preserved
        assert_eq!(palette.background, theme.background_primary);
        assert_eq!(palette.text, theme.foreground_primary);
        assert_eq!(palette.primary, theme.foreground_accent);
        assert_eq!(palette.success, theme.status_success);
        assert_eq!(palette.warning, theme.status_warning);
        assert_eq!(palette.danger, theme.status_error);
    }

    #[test]
    fn light_palette_resolves_all_tokens() {
        let theme = OpenSpaceTheme::light();

        assert_valid_color(theme.background(BackgroundToken::Primary));
        assert_valid_color(theme.background(BackgroundToken::Secondary));
        assert_valid_color(theme.background(BackgroundToken::Tertiary));
        assert_valid_color(theme.background(BackgroundToken::Elevated));

        assert_valid_color(theme.foreground(ForegroundToken::Primary));
        assert_valid_color(theme.foreground(ForegroundToken::Secondary));
        assert_valid_color(theme.foreground(ForegroundToken::Muted));
        assert_valid_color(theme.foreground(ForegroundToken::Accent));

        assert_valid_color(theme.border(BorderToken::Subtle));
        assert_valid_color(theme.border(BorderToken::Default));
        assert_valid_color(theme.border(BorderToken::Strong));

        assert_valid_color(theme.status(StatusToken::Success));
        assert_valid_color(theme.status(StatusToken::Warning));
        assert_valid_color(theme.status(StatusToken::Error));
        assert_valid_color(theme.status(StatusToken::Info));

        assert_valid_color(theme.mode_accent(ModeAccentToken::Terminal));
        assert_valid_color(theme.mode_accent(ModeAccentToken::Chat));
        assert_valid_color(theme.mode_accent(ModeAccentToken::Editor));
    }

    #[test]
    fn from_mode_matches_constructors() {
        assert_eq!(
            OpenSpaceTheme::from_mode(ThemeMode::Dark),
            OpenSpaceTheme::dark()
        );
        assert_eq!(
            OpenSpaceTheme::from_mode(ThemeMode::Light),
            OpenSpaceTheme::light()
        );
    }

    #[test]
    fn mode_name_heuristic_works() {
        assert_eq!(OpenSpaceTheme::dark().mode_name(), "OpenSpace Dark");
        assert_eq!(OpenSpaceTheme::light().mode_name(), "OpenSpace Light");
    }
}
