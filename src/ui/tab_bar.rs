use iced::widget::{button, container, horizontal_space, row, text, Row};
use iced::{Alignment, Element, Length};

use crate::app::{Message, TerminalTab};
use crate::theme;

pub fn view(tabs: &[TerminalTab], active_tab: Option<usize>, dark_mode: bool) -> Element<'static, Message> {
    let p = theme::palette(dark_mode);
    let mut tab_row: Row<'static, Message> = Row::new().spacing(2).padding([2, 8]);

    for (idx, tab) in tabs.iter().enumerate() {
        let is_active = active_tab == Some(idx);
        let label = tab.label.clone();

        let tab_btn = button(
            row![
                text(label)
                    .size(11)
                    .color(if is_active {
                        p.text_primary
                    } else {
                        p.text_secondary
                    }),
                button(
                    text("x").size(9).color(p.text_muted)
                )
                .on_press(Message::CloseTab(idx))
                .padding([1, 4])
                .style(move |_t: &iced::Theme, status: button::Status| button::Style {
                    background: Some(iced::Background::Color(match status {
                        button::Status::Hovered => p.danger,
                        _ => iced::Color::TRANSPARENT,
                    })),
                    text_color: p.text_muted,
                    border: iced::Border::default(),
                    ..Default::default()
                }),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .on_press(Message::SwitchTab(idx))
        .padding([4, 12])
        .style(move |_t: &iced::Theme, status: button::Status| {
            let bg = if is_active {
                p.bg_active
            } else {
                match status {
                    button::Status::Hovered => p.bg_hover,
                    _ => p.bg_tertiary,
                }
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: p.text_primary,
                border: iced::Border {
                    color: if is_active {
                        p.accent
                    } else {
                        p.border
                    },
                    width: 1.0,
                    radius: theme::CORNER_RADIUS.into(),
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
                radius: theme::CORNER_RADIUS.into(),
            },
            ..Default::default()
        })
        .into()
}

