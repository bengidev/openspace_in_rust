//! Welcome window — shown exactly once on first launch.
//!
//! The welcome window is a deliberately minimal one-pager that
//! introduces OpenSpace as a desktop AI assistant. It is *not* a
//! multi-page onboarding flow: every piece of context lives on a
//! single page so users who already know what OpenSpace is can clear
//! it in a single keystroke (`Enter` triggers the primary CTA, `Esc`
//! triggers Skip).
//!
//! Visual identity: retro-futuristic / terminal. Phosphor-orange
//! accent, monospace stack, scanline + hatch overlays, a pixel
//! particle orb that mirrors the iOS app's identity.

use std::sync::Arc;
use std::time::{Duration, Instant};

use iced::alignment::{Horizontal, Vertical};
use iced::widget::canvas::Canvas;
use iced::widget::{button, column, container, row, text, Column, MouseArea, Row, Space};
use iced::Length;
use iced::Theme;
use iced::{Element, Subscription};

use openspace_theme::theme::OpenSpaceTheme;
use openspace_theme::tokens::{BackgroundToken, BorderToken, ForegroundToken, ThemeMode};

use super::ascii_orb::{dynamics_for_progress, AsciiOrbProgram};
use super::persistence::{WelcomePersistence, WelcomePersistenceError};

// ---------------------------------------------------------------------------
// State + messages
// ---------------------------------------------------------------------------

/// State of the welcome page.
///
/// The animation timeline is captured by `(started_at, now)`. Both
/// are monotonic `Instant`s so the orb canvas can run a pure draw
/// based on elapsed seconds without ever calling `Instant::now()` in
/// the render path.
///
/// The hold-to-zoom interaction is driven by three fields:
///
/// * `is_holding` — set when the user presses on the orb, cleared
///   when they release. Sticky between ticks.
/// * `hold_progress` ∈ `[0, 1]` — integrated each `Tick` using a
///   real elapsed `dt`. Grows toward `1.0` while held, decays
///   toward `0.0` while released. Frame-rate independent.
/// * `displayed_speed` / `displayed_zoom` — the canvas inputs
///   derived from `hold_progress` via `dynamics_for_progress`.
///
/// Decoupling the boolean intent (`is_holding`) from the integrated
/// progress lets the `Tick` reducer stay pure: every transition
/// produces the same trajectory regardless of how often `Tick`
/// fires, and tests can drive the timeline with synthetic
/// `Instant`s.
pub struct WelcomeState {
    pub theme: OpenSpaceTheme,
    pub theme_mode: ThemeMode,
    pub started_at: Instant,
    pub now: Instant,
    pub persistence: Arc<dyn WelcomePersistence>,
    /// Whether the user is currently holding down the mouse on the
    /// orb. Toggled by `OrbPressed` / `OrbReleased`.
    pub is_holding: bool,
    /// Integrated hold progress in `[0, 1]`. `0.0` is the rest
    /// state; `1.0` is the final form.
    pub hold_progress: f32,
    /// Speed multiplier handed to the canvas, derived from
    /// `hold_progress`.
    pub displayed_speed: f32,
    /// Zoom factor handed to the canvas, derived from
    /// `hold_progress`.
    pub displayed_zoom: f32,
}

impl std::fmt::Debug for WelcomeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WelcomeState")
            .field("theme_mode", &self.theme_mode)
            .field("started_at", &self.started_at)
            .field("now", &self.now)
            .field("is_holding", &self.is_holding)
            .field("hold_progress", &self.hold_progress)
            .field("displayed_speed", &self.displayed_speed)
            .field("displayed_zoom", &self.displayed_zoom)
            .field("persistence", &"<dyn WelcomePersistence>")
            .finish()
    }
}

impl WelcomeState {
    pub fn new(persistence: Arc<dyn WelcomePersistence>, theme_mode: ThemeMode) -> Self {
        let now = Instant::now();
        let (initial_speed, initial_zoom) = dynamics_for_progress(0.0);
        Self {
            theme: OpenSpaceTheme::from_mode(theme_mode),
            theme_mode,
            started_at: now,
            now,
            persistence,
            is_holding: false,
            hold_progress: 0.0,
            displayed_speed: initial_speed,
            displayed_zoom: initial_zoom,
        }
    }
}

/// Messages produced by the welcome view.
///
/// `EnterPressed` and `Skipped` are both terminal — they tell the
/// parent router to mark the welcome flag and transition to the
/// home shell. The router is the single place that owns transition
/// logic; the welcome view does not own it.
#[derive(Debug, Clone)]
pub enum WelcomeMessage {
    Tick(Instant),
    ToggleTheme,
    /// User started holding the mouse button down on the orb.
    /// Begins the zoom-in / speed-up ramp.
    OrbPressed,
    /// User released the mouse button. Begins the decay back to the
    /// rest state.
    OrbReleased,
    EnterPressed,
    Skipped,
}

/// Outcome the parent router needs after dispatching a message.
///
/// The router uses this to decide whether to mark the persistence
/// flag and route to home. Keeping the decision out of the welcome
/// state lets us test routing logic without filesystem I/O.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WelcomeOutcome {
    /// State updated locally; no transition.
    None,
    /// User toggled the theme; the parent should mirror that change
    /// into any shared theme state it owns.
    ThemeToggled(ThemeMode),
    /// User accepted the welcome window; the router should mark the
    /// persistence flag and transition to the home shell.
    Completed,
    /// User skipped. Treated identically to Completed for routing,
    /// but a separate variant lets the audit/event sink distinguish
    /// the two behaviours later if we want to.
    Skipped,
}

impl WelcomeState {
    /// Apply a message and report what the parent should do next.
    ///
    /// Persistence side-effects (`mark_completed`) live in the
    /// router, not here, so unit tests do not need a real filesystem
    /// to drive the welcome flow.
    pub fn update(&mut self, message: WelcomeMessage) -> WelcomeOutcome {
        match message {
            WelcomeMessage::Tick(now) => {
                // Compute dt against the previous tick *before*
                // overwriting `self.now`, so each integration step
                // uses the elapsed wall-clock between ticks rather
                // than assuming a fixed 33ms cadence. Saturating
                // subtraction protects against clock skew.
                let dt = now.saturating_duration_since(self.now).as_secs_f32();
                self.now = now;
                self.advance_orb_progress(dt);
                WelcomeOutcome::None
            }
            WelcomeMessage::ToggleTheme => {
                self.theme_mode = match self.theme_mode {
                    ThemeMode::Dark => ThemeMode::Light,
                    ThemeMode::Light => ThemeMode::Dark,
                };
                self.theme = OpenSpaceTheme::from_mode(self.theme_mode);
                WelcomeOutcome::ThemeToggled(self.theme_mode)
            }
            WelcomeMessage::OrbPressed => {
                self.is_holding = true;
                WelcomeOutcome::None
            }
            WelcomeMessage::OrbReleased => {
                self.is_holding = false;
                WelcomeOutcome::None
            }
            WelcomeMessage::EnterPressed => WelcomeOutcome::Completed,
            WelcomeMessage::Skipped => WelcomeOutcome::Skipped,
        }
    }

    /// Integrate the hold-progress trajectory by `dt` seconds and
    /// re-derive the displayed speed/zoom from it.
    ///
    /// While held, progress ramps toward `1.0` at `HOLD_RAMP_PER_SEC`
    /// (≈1.7s to fill from rest). On release it decays back toward
    /// `0.0` at `RELEASE_RAMP_PER_SEC` (≈1.1s to drain). Linear
    /// integration keeps the math obvious; the perceptual easing is
    /// supplied by `dynamics_for_progress`, which is free to apply
    /// any non-linear curve later without touching this loop.
    fn advance_orb_progress(&mut self, dt: f32) {
        const HOLD_RAMP_PER_SEC: f32 = 0.6;
        const RELEASE_RAMP_PER_SEC: f32 = 0.9;

        let dt = dt.clamp(0.0, 0.25);
        let delta = if self.is_holding {
            HOLD_RAMP_PER_SEC * dt
        } else {
            -RELEASE_RAMP_PER_SEC * dt
        };
        self.hold_progress = (self.hold_progress + delta).clamp(0.0, 1.0);

        let (speed, zoom) = dynamics_for_progress(self.hold_progress);
        self.displayed_speed = speed;
        self.displayed_zoom = zoom;
    }

    /// Subscription driving the orb animation. Targets ~30 fps which
    /// is enough for the slow shimmer in the reference recording and
    /// avoids burning a render-thread core.
    pub fn subscription(&self) -> Subscription<WelcomeMessage> {
        iced::time::every(Duration::from_millis(33)).map(WelcomeMessage::Tick)
    }
}

/// Marks the welcome window as completed in persistence, returning
/// any error so the caller can surface it. Kept on the state struct
/// so the router can call it after dispatching the outcome.
pub fn mark_completed(
    persistence: &Arc<dyn WelcomePersistence>,
) -> Result<(), WelcomePersistenceError> {
    persistence.mark_completed()
}

// ---------------------------------------------------------------------------
// View
// ---------------------------------------------------------------------------

/// Render the welcome window. The view is intentionally pure so the
/// parent router can wrap it in any element tree it likes.
pub fn view(state: &WelcomeState) -> Element<'_, WelcomeMessage> {
    let theme = state.theme;

    let main = column![
        top_bar(state),
        Space::new().height(Length::Fixed(20.0)),
        hero_block(state),
        Space::new().height(Length::Fixed(28.0)),
        feature_grid(state),
        Space::new().height(Length::Fixed(28.0)),
        cta_row(state),
        Space::new().height(Length::Fixed(80.0)),
        legend_strip(state),
    ]
    .max_width(820)
    .spacing(0)
    .width(Length::Fill);

    let centered = container(main)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([28, 36])
        .align_x(Horizontal::Center)
        .align_y(Vertical::Top)
        .style(move |_t: &Theme| container::Style {
            background: Some(iced::Background::Color(
                theme.background(BackgroundToken::Primary),
            )),
            ..Default::default()
        });

    centered.into()
}

// ---------------------------------------------------------------------------
// Sub-views
// ---------------------------------------------------------------------------

fn top_bar(state: &WelcomeState) -> Element<'_, WelcomeMessage> {
    let theme = state.theme;

    // Tiny phosphor pip + product label, status chip on the right.
    // Mirrors the iOS top bar so the whole brand reads as one
    // family across platforms.
    let lhs = row![
        pip(theme.foreground(ForegroundToken::Accent), 8.0),
        Space::new().width(Length::Fixed(10.0)),
        column![
            text("OPENSPACE")
                .size(13)
                .style(move |_t: &Theme| text::Style {
                    color: Some(theme.foreground(ForegroundToken::Primary)),
                }),
            text("DESKTOP AI ASSISTANCE")
                .size(9)
                .style(move |_t: &Theme| text::Style {
                    color: Some(theme.foreground(ForegroundToken::Muted)),
                }),
        ]
        .spacing(2),
    ]
    .align_y(Vertical::Center);

    let toggle_label = match state.theme_mode {
        ThemeMode::Dark => "[ \u{2600} LIGHT ]",
        ThemeMode::Light => "[ \u{263D} DARK ]",
    };

    let toggle = button(
        text(toggle_label)
            .size(10)
            .style(move |_t: &Theme| text::Style {
                color: Some(theme.foreground(ForegroundToken::Secondary)),
            }),
    )
    .padding([6, 10])
    .on_press(WelcomeMessage::ToggleTheme)
    .style(move |_t: &Theme, status| terminal_chip_style(theme, status, false));

    let status = container(
        row![
            pip(
                theme.status(openspace_theme::tokens::StatusToken::Success),
                6.0
            ),
            Space::new().width(Length::Fixed(8.0)),
            text("ONLINE")
                .size(10)
                .style(move |_t: &Theme| text::Style {
                    color: Some(theme.foreground(ForegroundToken::Secondary)),
                }),
            Space::new().width(Length::Fixed(10.0)),
            text("\u{2502}")
                .size(11)
                .style(move |_t: &Theme| text::Style {
                    color: Some(theme.border(BorderToken::Default)),
                }),
            Space::new().width(Length::Fixed(10.0)),
            text("WELCOME / 01")
                .size(10)
                .style(move |_t: &Theme| text::Style {
                    color: Some(theme.foreground(ForegroundToken::Muted)),
                }),
        ]
        .align_y(Vertical::Center),
    )
    .padding([6, 12])
    .style(move |_t: &Theme| container::Style {
        background: Some(iced::Background::Color(
            theme.background(BackgroundToken::Tertiary),
        )),
        border: iced::Border {
            radius: 4.0.into(),
            width: 1.0,
            color: theme.border(BorderToken::Default),
        },
        ..Default::default()
    });

    row![
        lhs,
        Space::new().width(Length::Fill),
        status,
        Space::new().width(Length::Fixed(10.0)),
        toggle,
    ]
    .align_y(Vertical::Center)
    .width(Length::Fill)
    .into()
}

fn hero_block(state: &WelcomeState) -> Element<'_, WelcomeMessage> {
    let theme = state.theme;

    // ASCII orb canvas — the centerpiece. We give it a fixed logical
    // height so the layout is stable regardless of the window size.
    // Speed and zoom are driven by the integrated `displayed_*`
    // fields so press-and-hold ramps the galaxy up smoothly and
    // release lets it decay back down.
    let orb = Canvas::new(AsciiOrbProgram::with_dynamics(
        theme,
        state.started_at,
        state.now,
        state.displayed_speed,
        state.displayed_zoom,
    ))
    .width(Length::Fill)
    .height(Length::Fixed(220.0));

    // Wrap the canvas in a `MouseArea` so left-button press starts
    // the hold ramp and release starts the decay. Iced 0.14's
    // `Program::update` impl can't reach the host application's
    // message stream cleanly — `MouseArea` is the idiomatic way to
    // turn a presentational widget into a press/release surface.
    // The pointer cursor signals the affordance.
    let interactive_orb = MouseArea::new(orb)
        .on_press(WelcomeMessage::OrbPressed)
        .on_release(WelcomeMessage::OrbReleased)
        .interaction(iced::mouse::Interaction::Pointer);

    // The orb sits directly on the page background — no card, no
    // border — so the particle field bleeds into the surrounding
    // chrome the way the reference recording does.
    let orb_card = container(interactive_orb)
        .width(Length::Fill)
        .height(Length::Fixed(220.0));

    let badge = container(
        row![
            text("\u{25C6}")
                .size(10)
                .style(move |_t: &Theme| text::Style {
                    color: Some(theme.foreground(ForegroundToken::Accent)),
                }),
            Space::new().width(Length::Fixed(8.0)),
            text(boot_sequence_label(state.hold_progress))
                .size(10)
                .style(move |_t: &Theme| text::Style {
                    color: Some(theme.foreground(ForegroundToken::Accent)),
                }),
        ]
        .align_y(Vertical::Center),
    )
    .padding([6, 10])
    .style(move |_t: &Theme| container::Style {
        background: Some(iced::Background::Color(with_alpha(
            theme.foreground(ForegroundToken::Accent),
            if is_dark(state.theme_mode) {
                0.12
            } else {
                0.10
            },
        ))),
        border: iced::Border {
            radius: 4.0.into(),
            width: 1.0,
            color: with_alpha(theme.foreground(ForegroundToken::Accent), 0.32),
        },
        ..Default::default()
    });

    let headline = text("Welcome to OpenSpace.")
        .size(34)
        .style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Primary)),
        });

    let subhead = text(
        "An always-on desktop AI assistant. \
         Wire your projects, terminals and editors into a single \
         operator that helps you ship faster — without leaving \
         the console aesthetic.",
    )
    .size(13)
    .style(move |_t: &Theme| text::Style {
        color: Some(theme.foreground(ForegroundToken::Secondary)),
    });

    column![
        badge,
        Space::new().height(Length::Fixed(10.0)),
        orb_card,
        Space::new().height(Length::Fixed(20.0)),
        headline,
        Space::new().height(Length::Fixed(10.0)),
        subhead,
        Space::new().height(Length::Fixed(20.0)),
    ]
    .spacing(0)
    .width(Length::Fill)
    .into()
}

fn feature_grid(state: &WelcomeState) -> Element<'_, WelcomeMessage> {
    let theme = state.theme;

    let cells = [
        (
            "// 01",
            "TERMINAL",
            "Drive shells from chat. Parallel ptys, audited.",
        ),
        (
            "// 02",
            "EDITOR",
            "LSP-aware. Inline diffs gated by permissions.",
        ),
        (
            "// 03",
            "CHAT",
            "Models route across providers. Sessions stay local.",
        ),
        (
            "// 04",
            "AGENTS",
            "Long-running tasks with guardrails and replay.",
        ),
    ];

    let mut row_widget: Row<'_, WelcomeMessage> = Row::new().spacing(10).width(Length::Fill);
    for (tag, title, body) in cells {
        row_widget = row_widget.push(feature_cell(theme, tag, title, body));
    }
    row_widget.into()
}

fn feature_cell<'a>(
    theme: OpenSpaceTheme,
    tag: &'a str,
    title: &'a str,
    body: &'a str,
) -> Element<'a, WelcomeMessage> {
    let cell = column![
        text(tag).size(9).style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Accent)),
        }),
        Space::new().height(Length::Fixed(6.0)),
        text(title).size(12).style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Primary)),
        }),
        Space::new().height(Length::Fixed(6.0)),
        text(body).size(11).style(move |_t: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Muted)),
        }),
    ]
    .spacing(0);

    container(cell)
        .padding(12)
        .width(Length::FillPortion(1))
        .style(move |_t: &Theme| container::Style {
            background: Some(iced::Background::Color(
                theme.background(BackgroundToken::Secondary),
            )),
            border: iced::Border {
                radius: 4.0.into(),
                width: 1.0,
                color: theme.border(BorderToken::Default),
            },
            ..Default::default()
        })
        .into()
}

fn cta_row(state: &WelcomeState) -> Element<'_, WelcomeMessage> {
    let theme = state.theme;

    let primary = button(
        row![
            text("ENTER OPENSPACE").size(12).style(move |_t: &Theme| {
                text::Style {
                    color: Some(theme.background(BackgroundToken::Primary)),
                }
            }),
            Space::new().width(Length::Fixed(10.0)),
            text("\u{21B5}").size(12).style(move |_t: &Theme| {
                text::Style {
                    color: Some(theme.background(BackgroundToken::Primary)),
                }
            }),
        ]
        .align_y(Vertical::Center),
    )
    .padding([12, 22])
    .on_press(WelcomeMessage::EnterPressed)
    .style(move |_t: &Theme, status| primary_button_style(theme, status));

    let skip = button(
        text("SKIP / ESC")
            .size(11)
            .style(move |_t: &Theme| text::Style {
                color: Some(theme.foreground(ForegroundToken::Secondary)),
            }),
    )
    .padding([12, 18])
    .on_press(WelcomeMessage::Skipped)
    .style(move |_t: &Theme, status| terminal_chip_style(theme, status, true));

    row![
        Space::new().width(Length::Fill),
        skip,
        Space::new().width(Length::Fixed(8.0)),
        primary
    ]
    .align_y(Vertical::Center)
    .into()
}

fn legend_strip(state: &WelcomeState) -> Element<'_, WelcomeMessage> {
    let theme = state.theme;

    container(
        row![
            text("\u{2588}\u{2593}\u{2592}\u{2591}")
                .size(10)
                .style(move |_t: &Theme| text::Style {
                    color: Some(theme.foreground(ForegroundToken::Accent)),
                }),
            Space::new().width(Length::Fixed(10.0)),
            text("// FIRST RUN \u{2014} this window only appears once.")
                .size(10)
                .style(move |_t: &Theme| text::Style {
                    color: Some(theme.foreground(ForegroundToken::Muted)),
                }),
            Space::new().width(Length::Fill),
            text("v0.1 \u{2022} OPENSPACE/RUST")
                .size(10)
                .style(move |_t: &Theme| text::Style {
                    color: Some(theme.foreground(ForegroundToken::Muted)),
                }),
        ]
        .align_y(Vertical::Center),
    )
    .padding([8, 12])
    .style(move |_t: &Theme| container::Style {
        background: Some(iced::Background::Color(
            theme.background(BackgroundToken::Tertiary),
        )),
        border: iced::Border {
            radius: 4.0.into(),
            width: 1.0,
            color: theme.border(BorderToken::Subtle),
        },
        ..Default::default()
    })
    .into()
}

// ---------------------------------------------------------------------------
// Style helpers
// ---------------------------------------------------------------------------

fn pip(color: iced::Color, diameter: f32) -> Element<'static, WelcomeMessage> {
    // Use a tiny container as a phosphor pip. We apply a rounded
    // border equal to half the diameter so the rectangle reads as a
    // circle without needing the canvas.
    container(Space::new())
        .width(Length::Fixed(diameter))
        .height(Length::Fixed(diameter))
        .style(move |_t: &Theme| container::Style {
            background: Some(iced::Background::Color(color)),
            border: iced::Border {
                radius: (diameter * 0.5).into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        })
        .into()
}

fn primary_button_style(theme: OpenSpaceTheme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(iced::Background::Color(
            theme.foreground(ForegroundToken::Accent),
        )),
        text_color: theme.background(BackgroundToken::Primary),
        border: iced::Border {
            radius: 4.0.into(),
            width: 1.0,
            color: theme.foreground(ForegroundToken::Accent),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(iced::Background::Color(with_alpha(
                theme.foreground(ForegroundToken::Accent),
                0.92,
            ))),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(iced::Background::Color(with_alpha(
                theme.foreground(ForegroundToken::Accent),
                0.78,
            ))),
            ..base
        },
        _ => base,
    }
}

fn terminal_chip_style(
    theme: OpenSpaceTheme,
    status: button::Status,
    _accent_text: bool,
) -> button::Style {
    // The chip uses Secondary text in every state today; the
    // `_accent_text` parameter is retained so callers can opt into
    // a future accent variant without a signature change.
    let text_color = theme.foreground(ForegroundToken::Secondary);
    let base = button::Style {
        background: Some(iced::Background::Color(
            theme.background(BackgroundToken::Tertiary),
        )),
        text_color,
        border: iced::Border {
            radius: 4.0.into(),
            width: 1.0,
            color: theme.border(BorderToken::Default),
        },
        ..Default::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(iced::Background::Color(
                theme.background(BackgroundToken::Elevated),
            )),
            border: iced::Border {
                radius: 4.0.into(),
                width: 1.0,
                color: theme.border(BorderToken::Strong),
            },
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(iced::Background::Color(
                theme.background(BackgroundToken::Secondary),
            )),
            ..base
        },
        _ => base,
    }
}

fn with_alpha(color: iced::Color, alpha: f32) -> iced::Color {
    iced::Color {
        a: alpha.clamp(0.0, 1.0),
        ..color
    }
}

fn is_dark(mode: ThemeMode) -> bool {
    matches!(mode, ThemeMode::Dark)
}

/// Format the badge label so users get inline feedback on how
/// charged the orb is.
///
/// At rest (`progress` near `0`) we keep the original "READY" copy
/// so the welcome screen reads identically to the previous design
/// when the user is not interacting. Near the final form we switch
/// the suffix to `FINAL FORM` so the climax is unambiguous.
/// Between those endpoints we surface the speed multiplier as
/// `Nx` (rounded to the nearest integer) so the user can read the
/// ramp progress without staring at the orb.
fn boot_sequence_label(hold_progress: f32) -> String {
    let progress = hold_progress.clamp(0.0, 1.0);
    if progress < 0.05 {
        "BOOT SEQUENCE / READY".to_string()
    } else if progress > 0.95 {
        "BOOT SEQUENCE / FINAL FORM".to_string()
    } else {
        let (speed, _) = dynamics_for_progress(progress);
        format!("BOOT SEQUENCE / {}x", speed.round() as u32)
    }
}

// Suppress unused-import warning if these helpers aren't used in
// every cfg combination.
#[allow(dead_code)]
fn _column_marker(_: &Column<'_, WelcomeMessage>) {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::onboarding_welcome::persistence::InMemoryWelcomePersistence;

    #[test]
    fn enter_pressed_yields_completed_outcome() {
        let mut state =
            WelcomeState::new(Arc::new(InMemoryWelcomePersistence::new()), ThemeMode::Dark);
        let outcome = state.update(WelcomeMessage::EnterPressed);
        assert_eq!(outcome, WelcomeOutcome::Completed);
    }

    #[test]
    fn skipped_yields_skipped_outcome() {
        let mut state =
            WelcomeState::new(Arc::new(InMemoryWelcomePersistence::new()), ThemeMode::Dark);
        let outcome = state.update(WelcomeMessage::Skipped);
        assert_eq!(outcome, WelcomeOutcome::Skipped);
    }

    #[test]
    fn toggle_theme_swaps_mode() {
        let mut state =
            WelcomeState::new(Arc::new(InMemoryWelcomePersistence::new()), ThemeMode::Dark);
        let outcome = state.update(WelcomeMessage::ToggleTheme);
        assert_eq!(outcome, WelcomeOutcome::ThemeToggled(ThemeMode::Light));
        assert_eq!(state.theme_mode, ThemeMode::Light);

        let outcome = state.update(WelcomeMessage::ToggleTheme);
        assert_eq!(outcome, WelcomeOutcome::ThemeToggled(ThemeMode::Dark));
    }

    #[test]
    fn tick_updates_now_without_transition() {
        let mut state =
            WelcomeState::new(Arc::new(InMemoryWelcomePersistence::new()), ThemeMode::Dark);
        let baseline = state.now;
        let later = baseline + Duration::from_millis(120);
        let outcome = state.update(WelcomeMessage::Tick(later));
        assert_eq!(outcome, WelcomeOutcome::None);
        assert!(state.now >= later);
    }

    #[test]
    fn mark_completed_propagates_to_persistence() {
        let store: Arc<dyn WelcomePersistence> = Arc::new(InMemoryWelcomePersistence::new());
        assert!(!store.is_completed());
        super::mark_completed(&store).unwrap();
        assert!(store.is_completed());
    }

    #[test]
    fn fresh_state_starts_at_rest() {
        let state = WelcomeState::new(Arc::new(InMemoryWelcomePersistence::new()), ThemeMode::Dark);
        assert!(!state.is_holding);
        assert_eq!(state.hold_progress, 0.0);
        let (rest_speed, rest_zoom) = dynamics_for_progress(0.0);
        assert!((state.displayed_speed - rest_speed).abs() < 1e-6);
        assert!((state.displayed_zoom - rest_zoom).abs() < 1e-6);
    }

    #[test]
    fn press_then_release_toggles_is_holding_flag() {
        let mut state =
            WelcomeState::new(Arc::new(InMemoryWelcomePersistence::new()), ThemeMode::Dark);

        let outcome = state.update(WelcomeMessage::OrbPressed);
        assert_eq!(outcome, WelcomeOutcome::None);
        assert!(state.is_holding);

        let outcome = state.update(WelcomeMessage::OrbReleased);
        assert_eq!(outcome, WelcomeOutcome::None);
        assert!(!state.is_holding);
    }

    #[test]
    fn holding_for_long_enough_reaches_final_form() {
        let mut state =
            WelcomeState::new(Arc::new(InMemoryWelcomePersistence::new()), ThemeMode::Dark);
        state.update(WelcomeMessage::OrbPressed);

        // 5 seconds of synthetic ticks at 33ms — comfortably above
        // the natural fill time of ~1.7s so the integration must
        // saturate at 1.0 even in the presence of clamping bugs.
        let mut now = state.now;
        for _ in 0..150 {
            now += Duration::from_millis(33);
            state.update(WelcomeMessage::Tick(now));
        }

        let (max_speed, max_zoom) = dynamics_for_progress(1.0);
        assert!((state.hold_progress - 1.0).abs() < 1e-6);
        assert!((state.displayed_speed - max_speed).abs() < 1e-6);
        assert!((state.displayed_zoom - max_zoom).abs() < 1e-6);
    }

    #[test]
    fn releasing_decays_progress_back_to_rest() {
        let mut state =
            WelcomeState::new(Arc::new(InMemoryWelcomePersistence::new()), ThemeMode::Dark);
        state.update(WelcomeMessage::OrbPressed);

        // Charge to the final form, then release.
        let mut now = state.now;
        for _ in 0..150 {
            now += Duration::from_millis(33);
            state.update(WelcomeMessage::Tick(now));
        }
        assert!((state.hold_progress - 1.0).abs() < 1e-6);

        state.update(WelcomeMessage::OrbReleased);
        for _ in 0..150 {
            now += Duration::from_millis(33);
            state.update(WelcomeMessage::Tick(now));
        }

        let (rest_speed, rest_zoom) = dynamics_for_progress(0.0);
        assert!(state.hold_progress.abs() < 1e-6);
        assert!((state.displayed_speed - rest_speed).abs() < 1e-6);
        assert!((state.displayed_zoom - rest_zoom).abs() < 1e-6);
    }

    #[test]
    fn release_mid_ramp_decays_without_first_completing() {
        let mut state =
            WelcomeState::new(Arc::new(InMemoryWelcomePersistence::new()), ThemeMode::Dark);
        state.update(WelcomeMessage::OrbPressed);

        // Hold briefly so we are partway up the ramp but nowhere
        // near the final form.
        let mut now = state.now;
        for _ in 0..15 {
            now += Duration::from_millis(33);
            state.update(WelcomeMessage::Tick(now));
        }
        assert!(state.hold_progress > 0.0);
        assert!(state.hold_progress < 1.0);
        let mid_progress = state.hold_progress;

        // Release mid-ramp — progress should immediately start
        // decaying instead of climbing further.
        state.update(WelcomeMessage::OrbReleased);
        now += Duration::from_millis(33);
        state.update(WelcomeMessage::Tick(now));
        assert!(state.hold_progress < mid_progress);
    }

    #[test]
    fn first_tick_after_press_advances_progress_proportionally_to_dt() {
        let mut state =
            WelcomeState::new(Arc::new(InMemoryWelcomePersistence::new()), ThemeMode::Dark);
        state.update(WelcomeMessage::OrbPressed);

        let baseline = state.hold_progress;
        let now = state.now + Duration::from_millis(100);
        state.update(WelcomeMessage::Tick(now));

        // 100ms at HOLD_RAMP_PER_SEC=0.6 should add ~0.06 progress.
        let delta = state.hold_progress - baseline;
        assert!(
            (delta - 0.06).abs() < 1e-3,
            "expected ~0.06 progress after 100ms hold, got {delta}"
        );
    }

    #[test]
    fn boot_sequence_label_reflects_progress() {
        assert_eq!(super::boot_sequence_label(0.0), "BOOT SEQUENCE / READY");
        assert_eq!(super::boot_sequence_label(0.04), "BOOT SEQUENCE / READY");
        assert_eq!(
            super::boot_sequence_label(1.0),
            "BOOT SEQUENCE / FINAL FORM"
        );
        assert_eq!(
            super::boot_sequence_label(0.96),
            "BOOT SEQUENCE / FINAL FORM"
        );
        // Midway should surface an `Nx` label (the exact integer
        // depends on `dynamics_for_progress`, but it must not be
        // the rest copy).
        let mid = super::boot_sequence_label(0.5);
        assert!(mid.starts_with("BOOT SEQUENCE / ") && mid.ends_with('x'));
    }
}
