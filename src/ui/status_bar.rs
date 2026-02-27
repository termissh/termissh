use iced::widget::{container, horizontal_space, row, text};
use iced::{Alignment, Element, Length};

use crate::app::Message;
use crate::config::Language;
use crate::i18n::Texts;
use crate::theme;

pub fn view(
    texts: &Texts,
    has_api_key: bool,
    language: Language,
    dark_mode: bool,
) -> Element<'static, Message> {
    let p = theme::palette(dark_mode);

    let sync_text = if has_api_key {
        text(format!("* {}", texts.sync_status_connected))
            .size(10)
            .color(p.success)
    } else {
        text(format!("o {}", texts.sync_status_local))
            .size(10)
            .color(p.text_muted)
    };

    let lang_text = match language {
        Language::Turkish => "TR",
        Language::English => "EN",
    };

    let bar = row![
        text(" TermiSSH").size(11).color(p.text_primary),
        text(" v0.2.0").size(10).color(p.text_muted),
        horizontal_space(),
        sync_text,
        text("  |  ").size(10).color(p.text_muted),
        text(lang_text).size(10).color(p.text_secondary),
        text("  ").size(10),
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    container(bar)
        .width(Length::Fill)
        .padding([4, 8])
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

