use iced::widget::{button, container, horizontal_space, row, text};
use iced::{Alignment, Element, Length};

use crate::app::Message;
use crate::config::AppTheme;
use crate::i18n::Texts;
use crate::theme;

pub fn view(texts: &Texts, theme: AppTheme, lc: theme::LayoutConfig) -> Element<'static, Message> {
    let p = theme::palette(theme);
    let cr = lc.corner_radius;

    let toolbar = row![
        toolbar_button("+ New", Message::OpenNewDialog, theme, cr),
        toolbar_button("Ping", Message::PingAll, theme, cr),
        horizontal_space(),
        toolbar_button("FTP", Message::FtpToggle, theme, cr),
        toolbar_button(texts.settings, Message::OpenSettings, theme, cr),
    ]
    .spacing(4)
    .padding([4, 8])
    .align_y(Alignment::Center);

    container(toolbar)
        .width(Length::Fill)
        .style(move |_t: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(p.bg_secondary)),
            border: iced::Border {
                color: p.border,
                width: 1.0,
                radius: cr.into(),
            },
            ..Default::default()
        })
        .into()
}

fn toolbar_button(label: &'static str, msg: Message, theme: AppTheme, cr: f32) -> Element<'static, Message> {
    let p = theme::palette(theme);

    button(
        text(label)
            .size(11)
            .color(p.text_primary),
    )
    .on_press(msg)
    .padding([3, 10])
    .style(move |_t: &iced::Theme, status: button::Status| {
        let bg = match status {
            button::Status::Hovered => p.bg_hover,
            button::Status::Pressed => p.bg_active,
            _ => iced::Color::TRANSPARENT,
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: match status {
                button::Status::Hovered | button::Status::Pressed => p.text_primary,
                _ => p.text_secondary,
            },
            border: iced::Border {
                radius: cr.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    })
    .into()
}
