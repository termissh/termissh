use iced::widget::{button, column, container, progress_bar, row, scrollable, text, text_input, Column};
use iced::{Alignment, Element, Length};

use crate::app::{LocalSystemInfo, Message};
use crate::config::{AppTheme, Host};
use crate::i18n::Texts;
use crate::icons::{self, IconName};
use crate::theme;
use std::collections::HashMap;

pub fn view(
    texts: &Texts,
    hosts: &[Host],
    search_query: &str,
    selected_host: Option<usize>,
    ping_results: &HashMap<usize, Option<u128>>,
    system_info: &LocalSystemInfo,
    _structure: &[String],
    theme: AppTheme,
    lc: theme::LayoutConfig,
    borders_on: bool,
    local_only: bool,
) -> Element<'static, Message> {
    let p = theme::palette(theme);
    let cr = lc.corner_radius;
    let bw = if borders_on { 1.0 } else { 0.0 };

    let search_icon = icons::icon(IconName::Search, 12.0, p.text_muted);
    let search = row![
        search_icon,
        text_input(texts.search_placeholder, search_query)
            .on_input(Message::SearchInput)
            .padding([6, 8])
            .size(11)
            .style(move |_t: &iced::Theme, status: text_input::Status| text_input::Style {
                background: iced::Background::Color(iced::Color::TRANSPARENT),
                border: iced::Border {
                    color: iced::Color::TRANSPARENT,
                    width: 0.0,
                    radius: cr.into(),
                },
                icon: p.text_muted,
                placeholder: p.text_muted,
                value: p.text_primary,
                selection: p.accent,
            }),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .padding([0, 6]);

    let search_box = container(search)
        .width(Length::Fill)
        .style(move |_t: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(p.bg_primary)),
            border: iced::Border {
                color: p.border,
                width: bw,
                radius: cr.into(),
            },
            ..Default::default()
        });

    let query_lower = search_query.to_lowercase();
    let filtered_hosts: Vec<(usize, &Host)> = hosts
        .iter()
        .enumerate()
        .filter(|(_, h)| {
            if query_lower.is_empty() {
                return true;
            }
            h.alias.to_lowercase().contains(&query_lower)
                || h.hostname.to_lowercase().contains(&query_lower)
                || h.username.to_lowercase().contains(&query_lower)
        })
        .collect();

    let mut host_list = Column::new().spacing(1);
    for (idx, host) in &filtered_hosts {
        let is_selected = selected_host == Some(*idx);
        let is_synced = host.id.is_some();
        let dot_icon = if is_synced {
            IconName::CircleDot
        } else {
            IconName::Circle
        };
        let dot_color = if is_synced { p.success } else { p.text_muted };

        let ping_text = match ping_results.get(idx) {
            Some(Some(ms)) if *ms < 100 => text(format!("{}ms", ms)).size(9).color(p.success),
            Some(Some(ms)) if *ms < 300 => text(format!("{}ms", ms)).size(9).color(p.warning),
            Some(Some(ms)) => text(format!("{}ms", ms)).size(9).color(p.danger),
            Some(None) => text("×").size(9).color(p.danger),
            None => text("").size(9),
        };

        let alias = host.alias.clone();
        let host_info = format!("{}@{}", host.username, host.hostname);
        let i = *idx;

        let local_badge: Option<Element<'static, Message>> = if local_only && host.id.is_none() {
            Some(icons::icon(IconName::Power, 9.0, p.text_muted).into())
        } else {
            None
        };

        let host_btn = button(
            row![
                icons::icon(dot_icon, 8.0, dot_color),
                column![
                    text(alias).size(11).color(p.text_primary),
                    text(host_info).size(9).color(p.text_muted),
                ]
                .spacing(1),
                local_badge.unwrap_or_else(|| iced::widget::horizontal_space().into()),
                iced::widget::horizontal_space(),
                ping_text,
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .on_press(Message::ConnectToHost(i))
        .width(Length::Fill)
        .padding([5, 8])
        .style(move |_t: &iced::Theme, status: button::Status| {
            let bg = if is_selected {
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
                    radius: cr.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        });

        host_list = host_list.push(host_btn);
    }

    let context_buttons: Element<'static, Message> = if let Some(sel) = selected_host {
        row![
            action_icon_button(IconName::Pencil, Message::OpenEditDialog(sel), false, theme, cr),
            action_icon_button(IconName::Trash, Message::OpenDeleteConfirm(sel), true, theme, cr),
        ]
        .spacing(4)
        .into()
    } else {
        row![].into()
    };

    let cpu_pct = system_info.cpu_usage;
    let ram_pct = system_info.memory_usage;
    let dsk_pct = system_info.disk_usage_percent;

    let sys_monitor = column![
        row![
            text("CPU").size(9).color(p.text_muted).width(Length::Fixed(26.0)),
            progress_bar(0.0..=100.0, cpu_pct).height(3).width(Length::Fill),
            text(format!("{:.0}%", cpu_pct)).size(9).color(p.text_muted).width(Length::Fixed(28.0)),
        ]
        .spacing(4)
        .align_y(Alignment::Center),
        row![
            text("RAM").size(9).color(p.text_muted).width(Length::Fixed(26.0)),
            progress_bar(0.0..=100.0, ram_pct).height(3).width(Length::Fill),
            text(format!("{:.0}%", ram_pct)).size(9).color(p.text_muted).width(Length::Fixed(28.0)),
        ]
        .spacing(4)
        .align_y(Alignment::Center),
        row![
            text("DSK").size(9).color(p.text_muted).width(Length::Fixed(26.0)),
            progress_bar(0.0..=100.0, dsk_pct).height(3).width(Length::Fill),
            text(format!("{:.0}%", dsk_pct)).size(9).color(p.text_muted).width(Length::Fixed(28.0)),
        ]
        .spacing(4)
        .align_y(Alignment::Center),
    ]
    .spacing(4);

    let local_strip: Element<'static, Message> = if local_only {
        container(
            row![
                icons::icon(IconName::Power, 10.0, p.success),
                text("local storage only").size(9).color(p.text_muted),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .padding([2, 6])
        .width(Length::Fill)
        .style(move |_t: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(p.bg_tertiary)),
            border: iced::Border::default(),
            ..Default::default()
        })
        .into()
    } else {
        Column::<Message>::new().into()
    };

    let sidebar_content = column![
        action_row(theme, cr, texts),
        local_strip,
        search_box,
        scrollable(host_list)
            .height(Length::Fill)
            .style(hidden_scrollbar_style),
        context_buttons,
        container(iced::widget::horizontal_rule(1))
            .padding([4, 0]),
        sys_monitor,
    ]
    .spacing(6)
    .padding(8);

    container(sidebar_content)
        .width(Length::Fixed(lc.sidebar_width))
        .height(Length::Fill)
        .style(move |_t: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(p.bg_secondary)),
            border: iced::Border {
                color: p.border,
                width: bw,
                radius: cr.into(),
            },
            ..Default::default()
        })
        .into()
}

fn action_icon_button(
    icon: IconName,
    msg: Message,
    danger: bool,
    theme: AppTheme,
    cr: f32,
) -> Element<'static, Message> {
    let p = theme::palette(theme);
    let icon_color = if danger { p.danger } else { p.text_muted };

    button(
        container(icons::icon(icon, 12.0, icon_color))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_press(msg)
    .padding(0)
    .width(Length::Fill)
    .height(Length::Fixed(22.0))
    .style(move |_t: &iced::Theme, status: button::Status| {
        let hovered = matches!(status, button::Status::Hovered);
        button::Style {
            background: Some(iced::Background::Color(if hovered && danger {
                p.danger
            } else if hovered {
                p.bg_hover
            } else {
                p.bg_tertiary
            })),
            text_color: if hovered && danger { p.bg_primary } else { icon_color },
            border: iced::Border {
                color: p.border,
                width: 1.0,
                radius: cr.into(),
            },
            ..Default::default()
        }
    })
    .into()
}

fn hidden_scrollbar_style(theme: &iced::Theme, status: scrollable::Status) -> scrollable::Style {
    let mut style = scrollable::default(theme, status);
    let invisible_rail = scrollable::Rail {
        background: None,
        border: iced::Border {
            color: iced::Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        scroller: scrollable::Scroller {
            color: iced::Color::TRANSPARENT,
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
        },
    };
    style.vertical_rail = invisible_rail;
    style.horizontal_rail = invisible_rail;
    style.gap = None;
    style
}

fn action_row(theme: AppTheme, cr: f32, _texts: &Texts) -> Element<'static, Message> {
    let items: Vec<(IconName, Message)> = vec![
        (IconName::Plus,           Message::OpenNewDialog),
        (IconName::Wifi,           Message::PingAll),
        (IconName::SquareTerminal, Message::OpenCustomCommands),
        (IconName::Folder,         Message::FtpToggle),
        (IconName::Settings,       Message::OpenSettings),
    ];

    let mut r = row![].spacing(2).align_y(Alignment::Center);
    for (icon_name, msg) in items {
        r = r.push(action_chip(icon_name, msg, theme, cr));
    }

    container(r)
        .width(Length::Fill)
        .padding([2, 0])
        .into()
}

fn action_chip(
    icon: IconName,
    msg: Message,
    theme: AppTheme,
    cr: f32,
) -> Element<'static, Message> {
    let p = theme::palette(theme);
    button(
        container(icons::icon(icon, 14.0, p.text_muted))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_press(msg)
    .padding(0)
    .width(Length::Fill)
    .height(Length::Fixed(30.0))
    .style(move |_t: &iced::Theme, status: button::Status| {
        let hovered = matches!(status, button::Status::Hovered);
        button::Style {
            background: Some(iced::Background::Color(if hovered {
                p.bg_hover
            } else {
                p.bg_tertiary
            })),
            text_color: if hovered { p.text_primary } else { p.text_secondary },
            border: iced::Border {
                color: p.border,
                width: 0.0,
                radius: cr.into(),
            },
            ..Default::default()
        }
    })
    .into()
}
