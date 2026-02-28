use iced::widget::{container, horizontal_space, row, text};
use iced::{Alignment, Element, Length};

use crate::app::Message;
use crate::config::{AppTheme, Language};
use crate::i18n::Texts;
use crate::theme;

pub fn view(
    texts: &Texts,
    has_api_key: bool,
    language: Language,
    theme: AppTheme,
    lc: theme::LayoutConfig,
) -> Element<'static, Message> {
    let p = theme::palette(theme);
    let cr = lc.corner_radius;

    let sync_indicator = if has_api_key {
        text(format!("● {}", texts.sync_status_connected))
            .size(10)
            .color(p.success)
    } else {
        text(format!("○ {}", texts.sync_status_local))
            .size(10)
            .color(p.text_muted)
    };

    let lang_text = match language {
        Language::Turkish => "TR",
        Language::English => "EN",
    };

    let bar = row![
        text("© termissh").size(10).color(p.text_muted),
        text("  ·  ").size(10).color(p.border),
        text("Developed by Hacı Mert Gökhan").size(10).color(p.text_muted),
        text("  ·  ").size(10).color(p.border),
        text("termissh.org").size(10).color(p.accent),
        horizontal_space(),
        sync_indicator,
        text("  ·  ").size(10).color(p.border),
        text(lang_text).size(10).color(p.text_muted),
    ]
    .spacing(0)
    .align_y(Alignment::Center);

    container(bar)
        .width(Length::Fill)
        .padding([3, 10])
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
