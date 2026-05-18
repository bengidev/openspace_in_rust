use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Column, Row, button, container, mouse_area, text};
use iced::{Element, Event, Length, Point, Size, Subscription, Task, Theme, mouse};
use openspace_core::app_command::AppCommand;
use openspace_core::session::{SessionDescriptor, SessionMode};
use openspace_theme::theme::OpenSpaceTheme;
use openspace_theme::tokens::*;
use std::sync::Arc;

use crate::app_router::AppRouter;
use crate::audit::NoopAuditSink;
use crate::center_surface::center_surface;
use crate::command_palette::{CommandPaletteOverlay, CommandRegistry, PaletteMessage};
use openspace_chat::ChatCommands;
use openspace_core::audit::AuditSink;
use openspace_core::command_palette::CommandDescriptorProvider;
use openspace_editor::EditorCommands;
use openspace_terminal::TerminalCommands;

const TOP_BAR_HEIGHT: f32 = 48.0;
const STATUS_BAR_HEIGHT: f32 = 28.0;
const SEPARATOR_SIZE: f32 = 1.0;
const MIN_PANEL_WIDTH: f32 = 100.0;
const MIN_CENTER_WIDTH: f32 = 200.0;
const HIT_MARGIN: f32 = 3.0;

/// Minimum total window width so the layout does not collapse.
/// 2 panel + center + 2 separator
const MIN_WINDOW_WIDTH: f32 = MIN_PANEL_WIDTH * 2.0 + MIN_CENTER_WIDTH + 2.0 * SEPARATOR_SIZE;

pub struct AppShell {
    /// Preferred width = user intent (from drag resize). Not clamped.
    preferred_left_width: f32,
    preferred_right_width: f32,
    /// Actual width = preferred clamped to available space.
    left_width: f32,
    right_width: f32,
    window_size: Size,
    mouse_position: Option<Point>,
    drag: Option<DragState>,
    router: AppRouter,
    theme: OpenSpaceTheme,
    theme_mode: ThemeMode,
    command_registry: CommandRegistry,
    palette: CommandPaletteOverlay,
    audit_sink: Arc<dyn AuditSink>,
}

impl std::fmt::Debug for AppShell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppShell")
            .field("preferred_left_width", &self.preferred_left_width)
            .field("preferred_right_width", &self.preferred_right_width)
            .field("left_width", &self.left_width)
            .field("right_width", &self.right_width)
            .field("window_size", &self.window_size)
            .field("mouse_position", &self.mouse_position)
            .field("drag", &self.drag)
            .field("router", &self.router)
            .field("theme", &self.theme)
            .field("theme_mode", &self.theme_mode)
            .field("command_registry", &self.command_registry)
            .field("palette", &self.palette)
            .field("audit_sink", &"<dyn AuditSink>")
            .finish()
    }
}

#[derive(Debug, Clone)]
enum DragState {
    Left { start_x: f32, start_width: f32 },
    Right { start_x: f32, start_width: f32 },
}

#[derive(Debug, Clone)]
pub enum Message {
    EventOccurred(Event),
    AppCommand(AppCommand),
    ToggleTheme,
    Palette(PaletteMessage),
}

impl From<PaletteMessage> for Message {
    fn from(msg: PaletteMessage) -> Self {
        Message::Palette(msg)
    }
}

impl Default for AppShell {
    fn default() -> Self {
        let preferred = 220.0;
        let mut registry = CommandRegistry::new();
        registry.merge(TerminalCommands::command_descriptors());
        registry.merge(ChatCommands::command_descriptors());
        registry.merge(EditorCommands::command_descriptors());
        for (a, b) in registry.detect_conflicts() {
            tracing::warn!(
                "shortcut conflict: {:?} and {:?} share same shortcut in same context",
                a,
                b
            );
        }
        Self {
            preferred_left_width: preferred,
            preferred_right_width: preferred,
            left_width: preferred,
            right_width: preferred,
            window_size: Size::new(1280.0, 800.0),
            mouse_position: None,
            drag: None,
            router: AppRouter::new(),
            theme: OpenSpaceTheme::dark(),
            theme_mode: ThemeMode::Dark,
            command_registry: registry,
            palette: CommandPaletteOverlay::default(),
            audit_sink: Arc::new(NoopAuditSink),
        }
    }
}

impl AppShell {
    /// Returns the currently wired audit sink. Features will call this
    /// when they begin emitting audit records.
    pub fn audit_sink(&self) -> &Arc<dyn AuditSink> {
        &self.audit_sink
    }

    /// Constructs an [`AppShell`] that opens with the given theme
    /// mode. Used by the onboarding router so a theme toggle inside
    /// the welcome window carries through to the home shell on
    /// transition.
    pub fn with_theme_mode(mode: ThemeMode) -> Self {
        Self {
            theme: OpenSpaceTheme::from_mode(mode),
            theme_mode: mode,
            ..Self::default()
        }
    }

    /// Returns the active theme mode. Used by the router so the
    /// welcome stage can keep in sync when home toggles the theme.
    pub fn theme_mode(&self) -> ThemeMode {
        self.theme_mode
    }
}

pub fn run() -> iced::Result {
    let window = iced::window::Settings {
        size: iced::Size::new(1280.0, 800.0),
        position: iced::window::Position::Centered,
        min_size: Some(iced::Size::new(
            MIN_WINDOW_WIDTH,
            TOP_BAR_HEIGHT + STATUS_BAR_HEIGHT + SEPARATOR_SIZE * 2.0 + 100.0,
        )),
        transparent: true,
        ..iced::window::Settings::default()
    };

    iced::application(AppShell::default, update, view)
        .title("OpenSpace")
        .theme(|shell: &AppShell| shell.theme.to_iced_theme())
        .style(|shell, _theme| iced::theme::Style {
            background_color: shell.theme.background(BackgroundToken::Primary),
            text_color: shell.theme.foreground(ForegroundToken::Primary),
        })
        .window(window)
        .subscription(subscription)
        .run()
}

fn subscription(_state: &AppShell) -> Subscription<Message> {
    iced::event::listen().map(Message::EventOccurred)
}

/// Public adapter for the routing layer.
///
/// Exposes the same subscription used by [`run`] so the
/// onboarding router can splice it into its own subscription tree
/// while the welcome window is dismissed and home is mounted.
pub fn shell_subscription(state: &AppShell) -> Subscription<Message> {
    subscription(state)
}

/// Clamp panel width to the available space from the current window width.
fn clamp_panel(preferred: f32, window_width: f32, other_panel: f32) -> f32 {
    let max = window_width - other_panel - MIN_CENTER_WIDTH - 2.0 * SEPARATOR_SIZE;
    preferred.clamp(MIN_PANEL_WIDTH, max.max(MIN_PANEL_WIDTH))
}

/// Public adapter exposing the shell's update function.
pub fn shell_update(state: &mut AppShell, message: Message) -> Task<Message> {
    update(state, message)
}

/// Public adapter exposing the shell's view function.
pub fn shell_view(state: &AppShell) -> Element<'_, Message> {
    view(state)
}

fn update(state: &mut AppShell, message: Message) -> Task<Message> {
    match message {
        Message::Palette(pmsg) => {
            let all = state.command_registry.all();
            let all_owned: Vec<_> = all.into_iter().cloned().collect();
            state.palette.update(pmsg, &all_owned);
            return Task::none();
        }
        Message::ToggleTheme => {
            state.theme_mode = match state.theme_mode {
                ThemeMode::Dark => ThemeMode::Light,
                ThemeMode::Light => ThemeMode::Dark,
            };
            state.theme = OpenSpaceTheme::from_mode(state.theme_mode);
        }
        Message::AppCommand(cmd) => {
            let events = state.router.apply(cmd);
            for event in events {
                tracing::debug!(?event, "app_event");
            }
        }
        Message::EventOccurred(event) => {
            if let Some(pmsg) = state.palette.handle_event(&event) {
                let all = state.command_registry.all();
                let all_owned: Vec<_> = all.into_iter().cloned().collect();
                state.palette.update(pmsg, &all_owned);
                return Task::none();
            }
            match event {
                Event::Window(iced::window::Event::Resized(size)) => {
                    // Guard: ignore transient invalid dimensions from macOS
                    // fullscreen transition. Sometimes winit sends 0x0 or a
                    // value too small before the final size.
                    if size.width < MIN_WINDOW_WIDTH
                        || size.height
                            < TOP_BAR_HEIGHT + STATUS_BAR_HEIGHT + SEPARATOR_SIZE * 2.0 + 10.0
                    {
                        return Task::none();
                    }
                    state.window_size = size;
                    state.left_width = clamp_panel(
                        state.preferred_left_width,
                        state.window_size.width,
                        state.right_width,
                    );
                    state.right_width = clamp_panel(
                        state.preferred_right_width,
                        state.window_size.width,
                        state.left_width,
                    );
                }
                Event::Mouse(mouse_event) => match mouse_event {
                    iced::mouse::Event::CursorMoved { position } => {
                        state.mouse_position = Some(position);
                        if let Some(drag) = &state.drag {
                            match drag {
                                DragState::Left {
                                    start_x,
                                    start_width,
                                } => {
                                    let delta = position.x - start_x;
                                    let new_preferred = (start_width + delta).max(MIN_PANEL_WIDTH);
                                    state.preferred_left_width = new_preferred;
                                    state.left_width = clamp_panel(
                                        state.preferred_left_width,
                                        state.window_size.width,
                                        state.right_width,
                                    );
                                }
                                DragState::Right {
                                    start_x,
                                    start_width,
                                } => {
                                    let delta = position.x - start_x;
                                    let new_preferred = (start_width - delta).max(MIN_PANEL_WIDTH);
                                    state.preferred_right_width = new_preferred;
                                    state.right_width = clamp_panel(
                                        state.preferred_right_width,
                                        state.window_size.width,
                                        state.left_width,
                                    );
                                }
                            }
                        }
                    }
                    iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
                        if let Some(pos) = state.mouse_position {
                            if is_over_left_sep(pos, state.left_width, state.window_size.height) {
                                state.drag = Some(DragState::Left {
                                    start_x: pos.x,
                                    start_width: state.left_width,
                                });
                            } else if is_over_right_sep(
                                pos,
                                state.window_size.width,
                                state.right_width,
                                state.window_size.height,
                            ) {
                                state.drag = Some(DragState::Right {
                                    start_x: pos.x,
                                    start_width: state.right_width,
                                });
                            }
                        }
                    }
                    iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left) => {
                        state.drag = None;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
    Task::none()
}

fn view(state: &AppShell) -> Element<'_, Message> {
    let theme = state.theme;

    let top_bar = top_bar_view(state);
    let status_bar = region_container(
        "StatusBar",
        Length::Fill,
        Length::Fixed(STATUS_BAR_HEIGHT),
        theme,
    );
    let left_panel = region_container(
        "LeftPanel",
        Length::Fixed(state.left_width),
        Length::Fill,
        theme,
    );
    let right_panel = region_container(
        "RightPanel",
        Length::Fixed(state.right_width),
        Length::Fill,
        theme,
    );

    let center_width =
        state.window_size.width - state.left_width - state.right_width - 2.0 * SEPARATOR_SIZE;
    let center_width = center_width.max(MIN_CENTER_WIDTH);
    let center_view: Element<'_, Message> = container(center_surface(
        state.router.active_session().map(|s| &s.mode),
        theme,
    ))
    .width(Length::Fixed(center_width))
    .height(Length::Fill)
    .into();

    let is_hovering_left = state
        .mouse_position
        .map(|p| is_over_left_sep(p, state.left_width, state.window_size.height))
        .unwrap_or(false);
    let is_hovering_right = state
        .mouse_position
        .map(|p| {
            is_over_right_sep(
                p,
                state.window_size.width,
                state.right_width,
                state.window_size.height,
            )
        })
        .unwrap_or(false);

    let is_dragging_left = matches!(state.drag, Some(DragState::Left { .. }));
    let is_dragging_right = matches!(state.drag, Some(DragState::Right { .. }));

    let left_sep = resize_handle(is_hovering_left || is_dragging_left, theme);
    let right_sep = resize_handle(is_hovering_right || is_dragging_right, theme);

    let middle = Row::new()
        .push(left_panel)
        .push(left_sep)
        .push(center_view)
        .push(right_sep)
        .push(right_panel)
        .height(Length::Fill)
        .width(Length::Fill);

    let main_ui = Column::new()
        .push(top_bar)
        .push(horizontal_separator(theme))
        .push(middle)
        .push(horizontal_separator(theme))
        .push(status_bar)
        .height(Length::Fill)
        .width(Length::Fill);

    if state.palette.visible {
        let overlay: Element<'_, Message> = state.palette.view(theme);
        iced::widget::stack![main_ui, overlay].into()
    } else {
        main_ui.into()
    }
}

fn top_bar_view<'a>(state: &'a AppShell) -> Element<'a, Message> {
    let theme = state.theme;

    let mut row = Row::new()
        .height(Length::Fill)
        .align_y(Vertical::Center)
        .padding([0, 12]);

    row = row.push(
        text("OpenSpace")
            .size(16)
            .style(move |_theme: &Theme| text::Style {
                color: Some(theme.foreground(ForegroundToken::Primary)),
            }),
    );

    if let Some(session) = state.router.active_session() {
        let spacer: Element<'_, Message> = container(iced::widget::Space::new())
            .width(Length::Fill)
            .height(Length::Shrink)
            .into();
        row = row.push(spacer);

        for mode in [
            SessionMode::Terminal,
            SessionMode::Chat,
            SessionMode::Editor,
        ] {
            let label = format!("{:?}", mode);
            let is_active = session.mode == mode;
            let btn = button(
                text(label)
                    .size(13)
                    .style(move |_theme: &Theme| text::Style {
                        color: Some(if is_active {
                            theme.foreground(ForegroundToken::Primary)
                        } else {
                            theme.foreground(ForegroundToken::Primary)
                        }),
                    }),
            )
            .padding([8, 16])
            .on_press(Message::AppCommand(AppCommand::SwitchMode {
                session_id: session.id,
                mode,
            }))
            .style(
                move |_theme: &Theme, status: iced::widget::button::Status| {
                    let mut base = button::Style {
                        background: if is_active {
                            Some(iced::Background::Color(
                                theme.foreground(ForegroundToken::Accent),
                            ))
                        } else {
                            Some(iced::Background::Color(
                                theme.background(BackgroundToken::Tertiary),
                            ))
                        },
                        text_color: if is_active {
                            theme.foreground(ForegroundToken::Primary)
                        } else {
                            theme.foreground(ForegroundToken::Primary)
                        },
                        border: iced::Border {
                            radius: 4.0.into(),
                            width: if is_active { 0.0 } else { 1.0 },
                            color: if is_active {
                                iced::Color::TRANSPARENT
                            } else {
                                theme.border(BorderToken::Default)
                            },
                        },
                        ..Default::default()
                    };
                    if !is_active {
                        base = match status {
                            iced::widget::button::Status::Hovered => button::Style {
                                background: Some(iced::Background::Color(
                                    theme.background(BackgroundToken::Elevated),
                                )),
                                ..base
                            },
                            iced::widget::button::Status::Pressed => button::Style {
                                background: Some(iced::Background::Color(
                                    theme.background(BackgroundToken::Secondary),
                                )),
                                ..base
                            },
                            _ => base,
                        };
                    }
                    base
                },
            );
            row = row.push(btn);

            // small gap between mode buttons
            if mode != SessionMode::Editor {
                let gap: Element<'_, Message> = container(iced::widget::Space::new())
                    .width(Length::Fixed(4.0))
                    .height(Length::Shrink)
                    .into();
                row = row.push(gap);
            }
        }
    } else {
        let spacer: Element<'_, Message> = container(iced::widget::Space::new())
            .width(Length::Fill)
            .height(Length::Shrink)
            .into();
        row = row.push(spacer);
        let new_btn =
            button(
                text("New Session")
                    .size(13)
                    .style(move |_theme: &Theme| text::Style {
                        color: Some(theme.background(BackgroundToken::Primary)),
                    }),
            )
            .padding([8, 16])
            .on_press(Message::AppCommand(AppCommand::CreateSession {
                project_folder: std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from(".")),
                descriptor: SessionDescriptor::new("Untitled"),
            }))
            .style(move |_theme: &Theme, _status| button::Style {
                background: Some(iced::Background::Color(
                    theme.foreground(ForegroundToken::Primary),
                )),
                text_color: theme.background(BackgroundToken::Primary),
                border: iced::Border {
                    radius: 4.0.into(),
                    width: 0.0,
                    color: iced::Color::TRANSPARENT,
                },
                ..Default::default()
            });
        row = row.push(new_btn);
    }

    // Theme toggle — always at the far right
    let toggle_label = match state.theme_mode {
        ThemeMode::Dark => "\u{2600}",   // sun
        ThemeMode::Light => "\u{1F319}", // moon
    };

    // gap before toggle so it doesn't crowd the preceding button
    let toggle_gap: Element<'_, Message> = container(iced::widget::Space::new())
        .width(Length::Fixed(8.0))
        .height(Length::Shrink)
        .into();
    row = row.push(toggle_gap);

    let toggle_btn = button(
        text(toggle_label)
            .size(13)
            .style(move |_theme: &Theme| text::Style {
                color: Some(theme.foreground(ForegroundToken::Primary)),
            }),
    )
    .padding([8, 16])
    .on_press(Message::ToggleTheme)
    .style(
        move |_theme: &Theme, status: iced::widget::button::Status| {
            let mut base = button::Style {
                background: Some(iced::Background::Color(
                    theme.background(BackgroundToken::Tertiary),
                )),
                text_color: theme.foreground(ForegroundToken::Primary),
                border: iced::Border {
                    radius: 4.0.into(),
                    width: 1.0,
                    color: theme.border(BorderToken::Default),
                },
                ..Default::default()
            };
            base = match status {
                iced::widget::button::Status::Hovered => button::Style {
                    background: Some(iced::Background::Color(
                        theme.background(BackgroundToken::Elevated),
                    )),
                    ..base
                },
                iced::widget::button::Status::Pressed => button::Style {
                    background: Some(iced::Background::Color(
                        theme.background(BackgroundToken::Secondary),
                    )),
                    ..base
                },
                _ => base,
            };
            base
        },
    );
    row = row.push(toggle_btn);

    container(row)
        .width(Length::Fill)
        .height(Length::Fixed(TOP_BAR_HEIGHT))
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(
                theme.background(BackgroundToken::Primary),
            )),
            ..Default::default()
        })
        .into()
}

fn resize_handle<'a>(is_active: bool, theme: OpenSpaceTheme) -> Element<'a, Message> {
    let color = if is_active {
        theme.foreground(ForegroundToken::Accent)
    } else {
        theme.border(BorderToken::Default)
    };
    mouse_area(
        container(iced::widget::Space::new())
            .width(Length::Fixed(SEPARATOR_SIZE))
            .height(Length::Fill)
            .style(move |_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(color)),
                ..Default::default()
            }),
    )
    .interaction(mouse::Interaction::ResizingHorizontally)
    .into()
}

fn region_container<'a>(
    label: &'a str,
    width: impl Into<Length>,
    height: impl Into<Length>,
    theme: OpenSpaceTheme,
) -> iced::widget::Container<'a, Message> {
    container(text(label).size(14).center())
        .width(width)
        .height(height)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(
                theme.background(BackgroundToken::Primary),
            )),
            text_color: Some(theme.foreground(ForegroundToken::Muted)),
            ..Default::default()
        })
}

fn horizontal_separator<'a>(theme: OpenSpaceTheme) -> iced::widget::Container<'a, Message> {
    container(iced::widget::Space::new())
        .width(Length::Fill)
        .height(Length::Fixed(SEPARATOR_SIZE))
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(theme.border(BorderToken::Default))),
            ..Default::default()
        })
}

fn is_over_left_sep(point: Point, left_width: f32, window_height: f32) -> bool {
    let x = left_width + SEPARATOR_SIZE / 2.0;
    let y_min = TOP_BAR_HEIGHT + SEPARATOR_SIZE;
    let y_max = window_height - STATUS_BAR_HEIGHT - SEPARATOR_SIZE;
    point.x >= x - HIT_MARGIN && point.x <= x + HIT_MARGIN && point.y >= y_min && point.y <= y_max
}

fn is_over_right_sep(
    point: Point,
    window_width: f32,
    right_width: f32,
    window_height: f32,
) -> bool {
    let x = window_width - right_width - SEPARATOR_SIZE / 2.0;
    let y_min = TOP_BAR_HEIGHT + SEPARATOR_SIZE;
    let y_max = window_height - STATUS_BAR_HEIGHT - SEPARATOR_SIZE;
    point.x >= x - HIT_MARGIN && point.x <= x + HIT_MARGIN && point.y >= y_min && point.y <= y_max
}
