//! Floating debug overlay rendered in `cfg(debug_assertions)`
//! builds only.
//!
//! Provides two affordances:
//! * a live indicator of the current window size, useful when
//!   tweaking the welcome layout against the reference
//!   screenshots;
//! * a "back to welcome" button that clears the welcome flag and
//!   bounces the stage back to Welcome without restarting the
//!   process.
//!
//! The overlay is positioned in the bottom-right of the window,
//! inside a translucent chip that hugs the terminal aesthetic.

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, container, row, text};
use iced::Length;
use iced::Theme;

use openspace_theme::theme::OpenSpaceTheme;
use openspace_theme::tokens::{BackgroundToken, BorderToken, ForegroundToken, StatusToken};

use crate::application::onboarding_app::OnboardingApp;
use crate::domain::app_messages::Message;
use crate::domain::app_stage::Stage;

pub fn view(state: &OnboardingApp) -> iced::Element<'_, Message> {
    let theme = state.theme();
    let size = state.window_size;

    let stage_label = match state.stage {
        Stage::Welcome(_) => "WELCOME",
        Stage::Home(_) => "HOME",
    };

    // Size + stage indicator chip.
    let indicator = container(
        row![
            pip(theme.status(StatusToken::Warning), 6.0),
            spacer(8.0, true),
            text(format!(
                "DEV \u{2022} {stage} \u{2022} {w}\u{00D7}{h}",
                stage = stage_label,
                w = size.width.round() as i32,
                h = size.height.round() as i32,
            ))
            .size(10)
            .style(move |_t: &Theme| text::Style {
                color: Some(theme.foreground(ForegroundToken::Secondary)),
            }),
        ]
        .align_y(Vertical::Center),
    )
    .padding([6, 10])
    .style(move |_t: &Theme| container::Style {
        background: Some(iced::Background::Color(with_alpha(
            theme.background(BackgroundToken::Tertiary),
            0.92,
        ))),
        border: iced::Border {
            radius: 4.0.into(),
            width: 1.0,
            color: theme.border(BorderToken::Default),
        },
        ..Default::default()
    });

    // "Back to welcome" button — only meaningful while in the
    // home stage. We still render it in welcome (greyed out) so
    // the overlay layout does not jump.
    let in_welcome = matches!(state.stage, Stage::Welcome(_));
    let label = if in_welcome {
        "ON WELCOME"
    } else {
        "BACK TO WELCOME"
    };

    let mut back_button = button(text(label).size(10).style(move |_t: &Theme| text::Style {
        color: Some(if in_welcome {
            theme.foreground(ForegroundToken::Muted)
        } else {
            theme.foreground(ForegroundToken::Accent)
        }),
    }))
    .padding([6, 10])
    .style(move |_t: &Theme, status| dev_button_style(theme, status, in_welcome));

    if !in_welcome {
        back_button = back_button.on_press(Message::DevResetToWelcome);
    }

    let chip_row = row![indicator, spacer(8.0, true), back_button].align_y(Vertical::Center);

    // Position the chip in the bottom-right corner without
    // intercepting clicks elsewhere on the page.
    container(chip_row)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(12)
        .align_x(Horizontal::Right)
        .align_y(Vertical::Bottom)
        .into()
}

fn dev_button_style(
    theme: OpenSpaceTheme,
    status: button::Status,
    disabled: bool,
) -> button::Style {
    let base = button::Style {
        background: Some(iced::Background::Color(with_alpha(
            theme.background(BackgroundToken::Tertiary),
            0.92,
        ))),
        text_color: theme.foreground(ForegroundToken::Accent),
        border: iced::Border {
            radius: 4.0.into(),
            width: 1.0,
            color: theme.border(BorderToken::Default),
        },
        ..Default::default()
    };
    if disabled {
        return base;
    }
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

fn pip(color: iced::Color, diameter: f32) -> iced::Element<'static, Message> {
    container(iced::widget::Space::new())
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

fn spacer(size: f32, horizontal: bool) -> iced::Element<'static, Message> {
    let s = iced::widget::Space::new();
    if horizontal {
        s.width(Length::Fixed(size)).into()
    } else {
        s.height(Length::Fixed(size)).into()
    }
}

fn with_alpha(color: iced::Color, alpha: f32) -> iced::Color {
    iced::Color {
        a: alpha.clamp(0.0, 1.0),
        ..color
    }
}
