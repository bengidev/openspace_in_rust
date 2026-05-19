/// Application theme mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

/// Background layer tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackgroundToken {
    /// Deepest layer — app chrome, empty panels.
    Primary,
    /// Slightly elevated — sidebars, nav bars.
    Secondary,
    /// Tertiary surfaces — toolbars, input fields.
    Tertiary,
    /// Elevated cards, popovers, overlays.
    Elevated,
}

/// Foreground / text tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ForegroundToken {
    /// Primary content text.
    Primary,
    /// Secondary labels, meta text.
    Secondary,
    /// Placeholders, disabled text.
    Muted,
    /// Accent-colored text (CTAs, links).
    Accent,
}

/// Border tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BorderToken {
    /// Hairline dividers, inactive borders.
    Subtle,
    /// Default borders for inputs and panels.
    Default,
    /// Focus rings, active borders.
    Strong,
}

/// Semantic status tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusToken {
    Success,
    Warning,
    Error,
    Info,
}

/// Mode-accent tokens for distinguishing terminal / chat / editor
/// surfaces at a glance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModeAccentToken {
    Terminal,
    Chat,
    Editor,
}
