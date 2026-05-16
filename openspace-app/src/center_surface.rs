use iced::alignment::{Horizontal, Vertical};
use iced::widget::{container, text};
use iced::{Element, Length, Theme};
use openspace_core::session::SessionMode;
use openspace_theme::theme_styles::ThemeColors;

pub fn center_surface<'a, Message: 'a>(mode: Option<&SessionMode>) -> Element<'a, Message> {
    let content: Element<'a, Message> = match mode {
        Some(SessionMode::Terminal) => terminal_workspace_view(),
        Some(SessionMode::Chat) => chat_workflow_view(),
        Some(SessionMode::Editor) => editor_workspace_view(),
        None => empty_center_view(),
    };
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(ThemeColors::BG)),
            ..Default::default()
        })
        .into()
}

fn terminal_workspace_view<'a, Message: 'a>() -> Element<'a, Message> {
    placeholder_view("TerminalWorkspaceView")
}

fn chat_workflow_view<'a, Message: 'a>() -> Element<'a, Message> {
    placeholder_view("ChatWorkflowView")
}

fn editor_workspace_view<'a, Message: 'a>() -> Element<'a, Message> {
    placeholder_view("EditorWorkspaceView")
}

fn empty_center_view<'a, Message: 'a>() -> Element<'a, Message> {
    placeholder_view("No Active Session")
}

fn placeholder_view<'a, Message: 'a>(label: &'static str) -> Element<'a, Message> {
    container(
        text(label)
            .size(18)
            .style(|_theme: &Theme| text::Style {
                color: Some(ThemeColors::FG_MUTED),
            }),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center)
    .into()
}
