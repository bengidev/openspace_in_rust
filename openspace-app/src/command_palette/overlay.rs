use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Event, Length, Padding, Theme};
use openspace_core::command_palette::{CommandCategory, CommandMetadata};
use openspace_theme::theme::OpenSpaceTheme;
use openspace_theme::tokens::*;

/// State for the command palette overlay.
#[derive(Debug, Clone)]
pub struct CommandPaletteOverlay {
    pub visible: bool,
    pub query: String,
    pub selected_index: usize,
    pub filtered: Vec<CommandMetadata>,
}

impl Default for CommandPaletteOverlay {
    fn default() -> Self {
        Self {
            visible: false,
            query: String::new(),
            selected_index: 0,
            filtered: Vec::new(),
        }
    }
}

/// Messages produced by the palette overlay.
#[derive(Debug, Clone)]
pub enum PaletteMessage {
    Open,
    Close,
    QueryChanged(String),
    SelectNext,
    SelectPrevious,
    Confirm,
    CommandSelected(String),
}

impl CommandPaletteOverlay {
    pub fn open(&mut self) {
        self.visible = true;
        self.query.clear();
        self.selected_index = 0;
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.query.clear();
        self.selected_index = 0;
        self.filtered.clear();
    }

    pub fn update(&mut self, msg: PaletteMessage, all_commands: &[CommandMetadata]) {
        match msg {
            PaletteMessage::Open => self.open(),
            PaletteMessage::Close => self.close(),
            PaletteMessage::QueryChanged(q) => {
                self.query = q;
                self.selected_index = 0;
                self.filtered = crate::command_palette::filter::filter_by_context_and_query(
                    all_commands,
                    None, // context filtering happens upstream in AppShell
                    &self.query,
                )
                .into_iter()
                .cloned()
                .collect();
            }
            PaletteMessage::SelectNext => {
                if !self.filtered.is_empty() {
                    self.selected_index =
                        (self.selected_index + 1).min(self.filtered.len() - 1);
                }
            }
            PaletteMessage::SelectPrevious => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            PaletteMessage::Confirm => {
                if let Some(cmd) = self.filtered.get(self.selected_index) {
                    tracing::info!("command confirmed: {:?}", cmd.id);
                    self.close();
                }
            }
            PaletteMessage::CommandSelected(id) => {
                tracing::info!("command selected: {}", id);
                self.close();
            }
        }
    }

    pub fn view<'a, Message: Clone + From<PaletteMessage> + 'a>(
        &'a self,
        theme: OpenSpaceTheme,
    ) -> Element<'a, Message> {
        let overlay_bg = iced::Color::from_rgba(0.0, 0.0, 0.0, 0.4);

        let input = text_input("Type a command...", &self.query)
            .on_input(|s| Message::from(PaletteMessage::QueryChanged(s)))
            .on_submit(Message::from(PaletteMessage::Confirm))
            .padding(10)
            .width(Length::Fill)
            .style(move |_theme: &Theme, _status: text_input::Status| text_input::Style {
                background: iced::Background::Color(theme.background(BackgroundToken::Tertiary)),
                border: iced::Border {
                    radius: 6.0.into(),
                    width: 1.0,
                    color: theme.border(BorderToken::Default),
                },
                icon: iced::Color::TRANSPARENT,
                placeholder: theme.foreground(ForegroundToken::Muted),
                value: theme.foreground(ForegroundToken::Primary),
                selection: theme.foreground(ForegroundToken::Accent),
            });

        let input_row = row![input]
            .padding(Padding::new(12.0))
            .width(Length::Fill);

        let results: Element<'a, Message> = if self.filtered.is_empty() && !self.query.is_empty() {
            container(
                text("No matches")
                    .size(14)
                    .style(move |_theme: &Theme| text::Style {
                        color: Some(theme.foreground(ForegroundToken::Muted)),
                    }),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
        } else {
            let items: Vec<Element<'a, Message>> = self
                .filtered
                .iter()
                .enumerate()
                .map(|(index, cmd)| {
                    let is_selected = index == self.selected_index;
                    palette_row(cmd, is_selected, theme, index)
                })
                .collect();

            scrollable(column(items).spacing(2).padding(Padding::new(4.0)))
                .height(Length::Fill)
                .style(move |_theme: &Theme, _status: scrollable::Status| scrollable::Style {
                    container: container::Style {
                        background: Some(iced::Background::Color(
                            theme.background(BackgroundToken::Secondary),
                        )),
                        ..Default::default()
                    },
                    vertical_rail: scrollable::Rail {
                        background: Some(iced::Background::Color(
                            theme.background(BackgroundToken::Tertiary),
                        )),
                        border: iced::Border::default(),
                        scroller: scrollable::Scroller {
                            background: iced::Background::Color(
                                theme.foreground(ForegroundToken::Muted),
                            ),
                            border: iced::Border::default(),
                        },
                    },
                    horizontal_rail: scrollable::Rail {
                        background: None,
                        border: iced::Border::default(),
                        scroller: scrollable::Scroller {
                            background: iced::Background::Color(iced::Color::TRANSPARENT),
                            border: iced::Border::default(),
                        },
                    },
                    gap: None,
                    auto_scroll: scrollable::AutoScroll {
                        background: iced::Background::Color(theme.background(BackgroundToken::Elevated)),
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                        icon: theme.foreground(ForegroundToken::Primary),
                    },
                })
                .into()
        };

        let panel = column![input_row, results]
            .width(Length::Fixed(520.0))
            .height(Length::Fixed(360.0))
            .spacing(0);

        let modal = container(panel)
            .width(Length::Fixed(520.0))
            .height(Length::Fixed(360.0))
            .style(move |_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(
                    theme.background(BackgroundToken::Elevated),
                )),
                border: iced::Border {
                    radius: 10.0.into(),
                    width: 1.0,
                    color: theme.border(BorderToken::Default),
                },
                shadow: iced::Shadow {
                    color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                    offset: iced::Vector::new(0.0, 8.0),
                    blur_radius: 24.0,
                },
                ..Default::default()
            });

        // Full-screen translucent backdrop + centered modal
        container(
            container(modal)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Top)
                .padding(iced::Padding::new(0.0).top(80.0)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(overlay_bg)),
            ..Default::default()
        })
        .into()
    }

    pub fn handle_event(&self, event: &Event) -> Option<PaletteMessage> {
        use iced::keyboard;
        if !self.visible {
            if let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event {
                let is_p = matches!(key.as_ref(), keyboard::Key::Character("p"))
                    || matches!(key.as_ref(), keyboard::Key::Character("P"));
                let palette_shortcut = if cfg!(target_os = "macos") {
                    modifiers.command() && modifiers.shift() && !modifiers.alt() && is_p
                } else {
                    modifiers.control() && modifiers.shift() && !modifiers.alt() && is_p
                };
                if palette_shortcut {
                    return Some(PaletteMessage::Open);
                }
            }
            return None;
        }

        match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers: _, .. }) => {
                match key {
                    keyboard::Key::Named(keyboard::key::Named::Escape) => {
                        return Some(PaletteMessage::Close);
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                        return Some(PaletteMessage::SelectNext);
                    }
                    keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                        return Some(PaletteMessage::SelectPrevious);
                    }
                    keyboard::Key::Named(keyboard::key::Named::Enter) => {
                        return Some(PaletteMessage::Confirm);
                    }
                    _ => None,
                }
            }
            Event::Mouse(iced::mouse::Event::ButtonPressed(_)) => Some(PaletteMessage::Close),
            _ => None,
        }
    }
}

fn palette_row<'a, Message: Clone + From<PaletteMessage> + 'a>(
    cmd: &'a CommandMetadata,
    is_selected: bool,
    theme: OpenSpaceTheme,
    _index: usize,
) -> Element<'a, Message> {
    let bg = if is_selected {
        theme.background(BackgroundToken::Tertiary)
    } else {
        theme.background(BackgroundToken::Secondary)
    };

    let category_color = category_color(cmd.category, theme);

    let title = text(&cmd.title)
        .size(14)
        .style(move |_theme: &Theme| text::Style {
            color: Some(theme.foreground(ForegroundToken::Primary)),
        });

    let category_label = text(format!("{:?}", cmd.category))
        .size(11)
        .style(move |_theme: &Theme| text::Style {
            color: Some(category_color),
        });

    let shortcut_text = cmd.shortcut.as_ref().map(|s| format_shortcut(s));

    let shortcut = shortcut_text.map(|label| {
        text(label)
            .size(11)
            .style(move |_theme: &Theme| text::Style {
                color: Some(theme.foreground(ForegroundToken::Muted)),
            })
    });

    let mut row_content = row![title].spacing(8).align_y(Vertical::Center);
    row_content = row_content.push(iced::widget::Space::new().width(Length::Fill));
    row_content = row_content.push(category_label);
    if let Some(sc) = shortcut {
        row_content = row_content.push(sc);
    }

    button(row_content)
        .padding([8, 12])
        .width(Length::Fill)
        .on_press(Message::from(PaletteMessage::CommandSelected(cmd.id.0.clone())))
        .style(move |_theme: &Theme, _status| button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: theme.foreground(ForegroundToken::Primary),
            border: iced::Border {
                radius: 6.0.into(),
                width: 0.0,
                color: iced::Color::TRANSPARENT,
            },
            ..Default::default()
        })
        .into()
}

fn category_color(category: CommandCategory, theme: OpenSpaceTheme) -> iced::Color {
    use openspace_core::command_palette::CommandCategory::*;
    match category {
        Terminal => theme.mode_accent(ModeAccentToken::Terminal),
        Chat => theme.mode_accent(ModeAccentToken::Chat),
        Editor => theme.mode_accent(ModeAccentToken::Editor),
        Navigation => theme.foreground(ForegroundToken::Accent),
        Session => theme.status(StatusToken::Info),
        File => theme.status(StatusToken::Warning),
        Git => theme.status(StatusToken::Success),
        Settings => theme.foreground(ForegroundToken::Secondary),
        Help => theme.foreground(ForegroundToken::Muted),
    }
}

fn format_shortcut(shortcut: &openspace_core::command_palette::KeyboardShortcut) -> String {
    let mut parts = Vec::new();
    if shortcut.modifiers.ctrl {
        parts.push("Ctrl");
    }
    if shortcut.modifiers.shift {
        parts.push("Shift");
    }
    if shortcut.modifiers.alt {
        parts.push("Alt");
    }
    if shortcut.modifiers.meta {
        parts.push("Cmd");
    }
    parts.push(&shortcut.key);
    parts.join("+")
}
