//! Center-surface placeholder views for each session mode.
//!
//! Renders a labelled placeholder per active mode (Terminal /
//! Chat / Editor) plus an empty state when no session is active.
//! Real per-mode content is owned by the corresponding
//! sub-feature crates.

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{container, text};
use iced::{Element, Length, Theme};
use openspace_core::session::SessionMode;
use openspace_theme::theme::OpenSpaceTheme;
use openspace_theme::tokens::*;

pub fn center_surface<'a, Message: 'a>(
    mode: Option<&SessionMode>,
    theme: OpenSpaceTheme,
) -> Element<'a, Message> {
    let content: Element<'a, Message> = match mode {
        Some(SessionMode::Terminal) => terminal_workspace_view(theme),
        Some(SessionMode::Chat) => chat_workflow_view(theme),
        Some(SessionMode::Editor) => editor_workspace_view(theme),
        None => empty_center_view(theme),
    };
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(
                theme.background(BackgroundToken::Primary),
            )),
            ..Default::default()
        })
        .into()
}

fn terminal_workspace_view<'a, Message: 'a>(theme: OpenSpaceTheme) -> Element<'a, Message> {
    placeholder_view("TerminalWorkspaceView", theme)
}

fn chat_workflow_view<'a, Message: 'a>(theme: OpenSpaceTheme) -> Element<'a, Message> {
    placeholder_view("ChatWorkflowView", theme)
}

fn editor_workspace_view<'a, Message: 'a>(theme: OpenSpaceTheme) -> Element<'a, Message> {
    placeholder_view("EditorWorkspaceView", theme)
}

fn empty_center_view<'a, Message: 'a>(theme: OpenSpaceTheme) -> Element<'a, Message> {
    placeholder_view("No Active Session", theme)
}

fn placeholder_view<'a, Message: 'a>(
    label: &'static str,
    theme: OpenSpaceTheme,
) -> Element<'a, Message> {
    container(
        text(label)
            .size(18)
            .style(move |_theme: &Theme| text::Style {
                color: Some(theme.foreground(ForegroundToken::Muted)),
            }),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center)
    .into()
}
