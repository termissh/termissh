use iced::widget::{button, container, horizontal_space, mouse_area, row, text};
use iced::{Alignment, Element, Length};

use crate::app::Message;
use crate::config::AppTheme;
use crate::icons::{self, IconName};
use crate::theme;

pub fn view(theme: AppTheme, borders_on: bool) -> Element<'static, Message> {
    let p = theme::palette(theme);

    let mark = icons::icon(IconName::SquareTerminal, 13.0, p.accent);
    let brand = text("TermiSSH")
        .size(11)
        .font(iced::Font {
            family: iced::font::Family::Name("Segoe UI"),
            weight: iced::font::Weight::Semibold,
            ..iced::Font::DEFAULT
        })
        .color(p.text_secondary);

    let left = row![mark, brand]
        .spacing(7)
        .align_y(Alignment::Center)
        .padding([0, 12]);

    let drag: Element<'static, Message> = mouse_area(
        container(horizontal_space().width(Length::Fill).height(Length::Fixed(22.0)))
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fixed(34.0)),
    )
    .on_press(Message::WindowDrag)
    .interaction(iced::mouse::Interaction::Grab)
    .into();

    let minimize = window_button(
        IconName::Minimize,
        Message::WindowMinimize,
        p.text_muted,
        p.bg_hover,
        13.0,
    );
    let maximize = window_button(
        IconName::Maximize,
        Message::WindowToggleMaximize,
        p.text_muted,
        p.bg_hover,
        12.0,
    );
    let close = close_button(
        IconName::Close,
        Message::WindowClose,
        p.text_secondary,
    );

    let controls = row![minimize, maximize, close]
        .spacing(0)
        .align_y(Alignment::Center);

    let bar = row![left, drag, controls]
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fixed(34.0));

    container(bar)
        .width(Length::Fill)
        .height(Length::Fixed(34.0))
        .style(move |_t: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(p.bg_primary)),
            border: iced::Border {
                color: p.border,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn window_button(
    icon: IconName,
    msg: Message,
    base_color: iced::Color,
    hover_bg: iced::Color,
    icon_size: f32,
) -> Element<'static, Message> {
    button(
        container(icons::icon(icon, icon_size, base_color))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_press(msg)
    .padding(0)
    .width(Length::Fixed(44.0))
    .height(Length::Fill)
    .style(move |_t: &iced::Theme, status: button::Status| {
        let hovered = matches!(status, button::Status::Hovered);
        button::Style {
            background: Some(iced::Background::Color(if hovered {
                hover_bg
            } else {
                iced::Color::TRANSPARENT
            })),
            text_color: base_color,
            border: iced::Border::default(),
            ..Default::default()
        }
    })
    .into()
}

fn close_button(
    icon: IconName,
    msg: Message,
    base_color: iced::Color,
) -> Element<'static, Message> {
    let hover_bg = iced::Color::from_rgb(0.86, 0.27, 0.27);
    let hover_fg = iced::Color::WHITE;
    button(
        container(icons::icon(icon, 13.0, base_color))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_press(msg)
    .padding(0)
    .width(Length::Fixed(46.0))
    .height(Length::Fill)
    .style(move |_t: &iced::Theme, status: button::Status| {
        let hovered = matches!(status, button::Status::Hovered);
        button::Style {
            background: Some(iced::Background::Color(if hovered {
                hover_bg
            } else {
                iced::Color::TRANSPARENT
            })),
            text_color: if hovered { hover_fg } else { base_color },
            border: iced::Border::default(),
            ..Default::default()
        }
    })
    .into()
}
