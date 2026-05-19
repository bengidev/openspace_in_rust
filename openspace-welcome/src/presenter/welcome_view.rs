//! Welcome window view.
//!
//! The welcome window is a deliberately minimal one-pager that
//! introduces OpenSpace as a desktop AI assistant. It is *not* a
//! multi-page onboarding flow: every piece of context lives on a
//! single page so users who already know what OpenSpace is can
//! clear it in a single keystroke (`Enter` triggers the primary
//! CTA, `Esc` triggers Skip).
//!
//! Visual identity: retro-futuristic / terminal. Phosphor-orange
//! accent, monospace stack, scanline + hatch overlays, a pixel
//! particle orb that mirrors the iOS app's identity.
//!
//! State + reducer logic lives in [`crate::application`];
//! this module only describes how the state is rendered.

use iced::alignment::{Horizontal, Vertical};
use iced::widget::canvas::Canvas;
use iced::widget::{button, column, container, row, text, Column, MouseArea, Row, Space};
use iced::Element;
use iced::Length;
use iced::Theme;

use openspace_theme::theme::OpenSpaceTheme;
use openspace_theme::tokens::{BackgroundToken, BorderToken, ForegroundToken, ThemeMode};

use crate::application::welcome_dynamics::dynamics_for_progress;
use crate::application::welcome_messages::WelcomeMessage;
use crate::application::welcome_state::WelcomeState;
use crate::presenter::welcome_orb::AsciiOrbProgram;

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

    // ASCII orb canvas — the centerpiece. We give it a fixed
    // logical height so the layout is stable regardless of the
    // window size. Speed and zoom are driven by the integrated
    // `displayed_*` fields so press-and-hold ramps the galaxy up
    // smoothly and release lets it decay back down.
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
    // border equal to half the diameter so the rectangle reads as
    // a circle without needing the canvas.
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

#[cfg(test)]
mod tests {
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
