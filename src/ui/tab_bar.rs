use iced::widget::{button, container, horizontal_space, row, text, Row};
use iced::{Alignment, Element, Length};

use crate::app::{Message, TerminalTab};
use crate::config::AppTheme;
use crate::theme;

pub fn view(tabs: &[TerminalTab], active_tab: Option<usize>, theme: AppTheme, lc: theme::LayoutConfig) -> Element<'static, Message> {
    let p = theme::palette(theme);
    let cr = lc.corner_radius;
    let mut tab_row: Row<'static, Message> = Row::new().spacing(2).padding([2, 6]);

    for (idx, tab) in tabs.iter().enumerate() {
        let is_active = active_tab == Some(idx);
        let label = tab.label.clone();
        let connected = tab.connected;

        let dot_color = if connected { p.success } else { p.text_muted };

        let tab_btn = button(
            row![
                text(if connected { "●" } else { "○" })
                    .size(8)
                    .color(dot_color),
                text(label)
                    .size(11)
                    .color(if is_active { p.text_primary } else { p.text_secondary }),
                button(
                    text("×").size(10).color(p.text_muted)
                )
                .on_press(Message::CloseTab(idx))
                .padding([0, 4])
                .style(move |_t: &iced::Theme, status: button::Status| button::Style {
                    background: Some(iced::Background::Color(match status {
                        button::Status::Hovered => p.danger,
                        _ => iced::Color::TRANSPARENT,
                    })),
                    text_color: match status {
                        button::Status::Hovered => p.bg_primary,
                        _ => p.text_muted,
                    },
                    border: iced::Border::default(),
                    ..Default::default()
                }),
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        )
        .on_press(Message::SwitchTab(idx))
        .padding([3, 10])
        .style(move |_t: &iced::Theme, status: button::Status| {
            let bg = if is_active {
                p.bg_active
            } else {
                match status {
                    button::Status::Hovered => p.bg_hover,
                    _ => iced::Color::TRANSPARENT,
                }
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: p.text_primary,
                border: iced::Border {
                    color: if is_active { p.accent } else { p.border },
                    width: if is_active { 1.0 } else { 0.0 },
                    radius: cr.into(),
                },
                ..Default::default()
            }
        });

        tab_row = tab_row.push(tab_btn);
    }

    tab_row = tab_row.push(horizontal_space());

    container(tab_row)
        .width(Length::Fill)
        .style(move |_t: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(p.bg_primary)),
            border: iced::Border {
                color: p.border,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}
