use iced::widget::{button, column, container, horizontal_space, row, text, Column, Row};
use iced::{Alignment, Element, Length};

use crate::app::{Message, TerminalTab};
use crate::config::AppTheme;
use crate::icons::{self, IconName};
use crate::theme;

pub fn view(
    tabs: &[TerminalTab],
    active_tab: Option<usize>,
    theme: AppTheme,
    lc: theme::LayoutConfig,
    borders_on: bool,
) -> Element<'static, Message> {
    let p = theme::palette(theme);
    let cr = lc.corner_radius;

    let mut tab_strip: Row<'static, Message> = Row::new().spacing(2).align_y(Alignment::End);

    for (idx, tab) in tabs.iter().enumerate() {
        let is_active = active_tab == Some(idx);
        let label = tab.label.clone();
        let connected = tab.connected;
        let dot_icon = if connected { IconName::CircleDot } else { IconName::Circle };

        // Close button — fixed 16x16, icon centered, aligned with tab content
        let close_btn = button(
            container(icons::icon(IconName::Close, 10.0, p.text_muted))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press(Message::CloseTab(idx))
        .padding(0)
        .width(Length::Fixed(16.0))
        .height(Length::Fixed(16.0))
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
        });

        // 2px active-tab underline. Transparent for inactive so the column
        // keeps a consistent height — every tab aligns at the same baseline.
        let indicator_color = if is_active { p.accent } else { iced::Color::TRANSPARENT };
        let indicator = container(horizontal_space())
            .width(Length::Fill)
            .height(Length::Fixed(2.0))
            .style(move |_: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(indicator_color)),
                ..Default::default()
            });

        // Indicator inside the button keeps its natural width (matches row above)
        let tab_btn = button(
            column![
                row![
                    icons::icon(dot_icon, 8.0, if connected { p.success } else { p.text_muted }),
                    text(label)
                        .size(11)
                        .color(if is_active { p.text_primary } else { p.text_secondary }),
                    close_btn,
                ]
                .spacing(5)
                .align_y(Alignment::Center)
                .height(Length::Fixed(22.0))
                .padding([0, 4]),
                indicator,
            ]
            .spacing(0)
            .align_x(Alignment::Center),
        )
        .on_press(Message::SwitchTab(idx))
        .padding([3, 8])
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
                border: iced::Border::default(),
                ..Default::default()
            }
        });

        tab_strip = tab_strip.push(tab_btn);
    }

    let mut tab_row: Row<'static, Message> = Row::new()
        .spacing(0)
        .padding([2, 6])
        .align_y(Alignment::End);
    tab_row = tab_row.push(tab_strip);
    tab_row = tab_row.push(horizontal_space());

    container(tab_row)
        .width(Length::Fill)
        .style(move |_t: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(p.bg_primary)),
            border: theme::border(p.border, 1.0, 0.0, borders_on),
            ..Default::default()
        })
        .into()
}
