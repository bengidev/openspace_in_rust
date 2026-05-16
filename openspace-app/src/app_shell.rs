use iced::alignment::{Horizontal, Vertical};
use iced::widget::{container, mouse_area, text, Column, Row};
use iced::{mouse, Element, Event, Length, Point, Size, Subscription, Task, Theme};
use openspace_theme::theme_styles::ThemeColors;

const TOP_BAR_HEIGHT: f32 = 48.0;
const STATUS_BAR_HEIGHT: f32 = 28.0;
const SEPARATOR_SIZE: f32 = 1.0;
const MIN_PANEL_WIDTH: f32 = 100.0;
const MIN_CENTER_WIDTH: f32 = 200.0;
const HIT_MARGIN: f32 = 3.0;

/// Minimum total window width agar layout tidak collapse.
/// 2 panel + center + 2 separator
const MIN_WINDOW_WIDTH: f32 = MIN_PANEL_WIDTH * 2.0 + MIN_CENTER_WIDTH + 2.0 * SEPARATOR_SIZE;

#[derive(Debug)]
pub struct AppShell {
    /// Preferred width = user intent (dari drag resize). Tidak di-clamp.
    preferred_left_width: f32,
    preferred_right_width: f32,
    /// Actual width = preferred di-clamp ke available space.
    left_width: f32,
    right_width: f32,
    window_size: Size,
    mouse_position: Option<Point>,
    drag: Option<DragState>,
}

#[derive(Debug, Clone)]
enum DragState {
    Left { start_x: f32, start_width: f32 },
    Right { start_x: f32, start_width: f32 },
}

#[derive(Debug, Clone)]
pub enum Message {
    EventOccurred(Event),
}

impl Default for AppShell {
    fn default() -> Self {
        let preferred = 220.0;
        Self {
            preferred_left_width: preferred,
            preferred_right_width: preferred,
            left_width: preferred,
            right_width: preferred,
            window_size: Size::new(1280.0, 800.0),
            mouse_position: None,
            drag: None,
        }
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
        .theme(Theme::Dark)
        .style(|_state, _theme| iced::theme::Style {
            background_color: ThemeColors::BG,
            text_color: ThemeColors::FG,
        })
        .window(window)
        .subscription(subscription)
        .run()
}

fn subscription(_state: &AppShell) -> Subscription<Message> {
    iced::event::listen().map(Message::EventOccurred)
}

/// Clamp panel width ke available space dari window width saat ini.
fn clamp_panel(preferred: f32, window_width: f32, other_panel: f32) -> f32 {
    let max = window_width - other_panel - MIN_CENTER_WIDTH - 2.0 * SEPARATOR_SIZE;
    preferred.clamp(MIN_PANEL_WIDTH, max.max(MIN_PANEL_WIDTH))
}

fn update(state: &mut AppShell, message: Message) -> Task<Message> {
    match message {
        Message::EventOccurred(event) => match event {
            Event::Window(iced::window::Event::Resized(size)) => {
                // Guard: abaikan dimensi sementara tidak wajar dari macOS
                // fullscreen transition. Kadang winit kirim 0x0 atau nilai
                // terlalu kecil sebelum ukuran final.
                if size.width < MIN_WINDOW_WIDTH || size.height < TOP_BAR_HEIGHT + STATUS_BAR_HEIGHT + SEPARATOR_SIZE * 2.0 + 10.0 {
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
                            DragState::Left { start_x, start_width } => {
                                let delta = position.x - start_x;
                                let new_preferred = (start_width + delta).max(MIN_PANEL_WIDTH);
                                state.preferred_left_width = new_preferred;
                                state.left_width = clamp_panel(
                                    state.preferred_left_width,
                                    state.window_size.width,
                                    state.right_width,
                                );
                            }
                            DragState::Right { start_x, start_width } => {
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
        },
    }
    Task::none()
}

fn view(state: &AppShell) -> Element<'_, Message> {
    let top_bar = region_container("TopBar", Length::Fill, Length::Fixed(TOP_BAR_HEIGHT));
    let status_bar = region_container("StatusBar", Length::Fill, Length::Fixed(STATUS_BAR_HEIGHT));
    let left_panel = region_container("LeftPanel", Length::Fixed(state.left_width), Length::Fill);
    let right_panel = region_container("RightPanel", Length::Fixed(state.right_width), Length::Fill);

    let center_width = state.window_size.width
        - state.left_width
        - state.right_width
        - 2.0 * SEPARATOR_SIZE;
    let center_width = center_width.max(MIN_CENTER_WIDTH);
    let center_surface = region_container("CenterSurface", Length::Fixed(center_width), Length::Fill);

    let is_hovering_left = state
        .mouse_position
        .map(|p| is_over_left_sep(p, state.left_width, state.window_size.height))
        .unwrap_or(false);
    let is_hovering_right = state
        .mouse_position
        .map(|p| is_over_right_sep(p, state.window_size.width, state.right_width, state.window_size.height))
        .unwrap_or(false);

    let is_dragging_left = matches!(state.drag, Some(DragState::Left { .. }));
    let is_dragging_right = matches!(state.drag, Some(DragState::Right { .. }));

    let left_sep = resize_handle(is_hovering_left || is_dragging_left);
    let right_sep = resize_handle(is_hovering_right || is_dragging_right);

    let middle = Row::new()
        .push(left_panel)
        .push(left_sep)
        .push(center_surface)
        .push(right_sep)
        .push(right_panel)
        .height(Length::Fill)
        .width(Length::Fill);

    Column::new()
        .push(top_bar)
        .push(horizontal_separator())
        .push(middle)
        .push(horizontal_separator())
        .push(status_bar)
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

fn resize_handle<'a>(is_active: bool) -> Element<'a, Message> {
    let color = if is_active {
        ThemeColors::ACCENT
    } else {
        ThemeColors::BORDER
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
) -> iced::widget::Container<'a, Message> {
    container(text(label).size(14).center())
        .width(width)
        .height(height)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(ThemeColors::BG)),
            text_color: Some(ThemeColors::FG_MUTED),
            ..Default::default()
        })
}

fn horizontal_separator<'a>() -> iced::widget::Container<'a, Message> {
    container(iced::widget::Space::new())
        .width(Length::Fill)
        .height(Length::Fixed(SEPARATOR_SIZE))
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(ThemeColors::BORDER)),
            ..Default::default()
        })
}

fn is_over_left_sep(point: Point, left_width: f32, window_height: f32) -> bool {
    let x = left_width + SEPARATOR_SIZE / 2.0;
    let y_min = TOP_BAR_HEIGHT + SEPARATOR_SIZE;
    let y_max = window_height - STATUS_BAR_HEIGHT - SEPARATOR_SIZE;
    point.x >= x - HIT_MARGIN
        && point.x <= x + HIT_MARGIN
        && point.y >= y_min
        && point.y <= y_max
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
    point.x >= x - HIT_MARGIN
        && point.x <= x + HIT_MARGIN
        && point.y >= y_min
        && point.y <= y_max
}
