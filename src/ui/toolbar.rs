use iced::widget::{button, container, row, text, Row};
use iced::{Alignment, Element, Length};

use crate::app::Message;
use crate::i18n::Texts;
use crate::theme;

pub fn view(texts: &Texts, dark_mode: bool) -> Element<'static, Message> {
    let p = theme::palette(dark_mode);

    let toolbar: Row<'static, Message> = row![
        toolbar_button(texts.settings, Message::OpenSettings, dark_mode),
        toolbar_button("FTP", Message::RefreshStructure, dark_mode),
        toolbar_button(texts.new_connection, Message::OpenNewDialog, dark_mode),
        toolbar_button(texts.ping_all, Message::PingAll, dark_mode),
    ]
    .spacing(6)
    .padding(6)
    .align_y(Alignment::Center);

    container(toolbar)
        .width(Length::Fill)
        .style(move |_t: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(p.bg_secondary)),
            border: iced::Border {
                color: p.border,
                width: 1.0,
                radius: theme::CORNER_RADIUS.into(),
            },
            ..Default::default()
        })
        .into()
}

fn toolbar_button(label: &'static str, msg: Message, dark_mode: bool) -> Element<'static, Message> {
    let p = theme::palette(dark_mode);

    button(
        text(label)
            .size(12)
            .color(p.text_primary),
    )
    .on_press(msg)
    .padding([4, 10])
    .style(move |_t: &iced::Theme, status: button::Status| {
        let bg = match status {
            button::Status::Hovered => p.bg_hover,
            button::Status::Pressed => p.bg_active,
            _ => p.bg_tertiary,
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: p.text_primary,
            border: iced::Border {
                color: p.border,
                width: 1.0,
                radius: theme::CORNER_RADIUS.into(),
            },
            ..Default::default()
        }
    })
    .into()
}

