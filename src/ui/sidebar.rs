use iced::widget::{button, column, container, horizontal_rule, progress_bar, row, scrollable, text, text_input, Column};
use iced::{Alignment, Element, Length};

use crate::app::{LocalSystemInfo, Message};
use crate::config::Host;
use crate::i18n::Texts;
use crate::theme;
use std::collections::HashMap;

pub fn view(
    texts: &Texts,
    hosts: &[Host],
    search_query: &str,
    selected_host: Option<usize>,
    ping_results: &HashMap<usize, Option<u128>>,
    system_info: &LocalSystemInfo,
    structure: &[String],
    dark_mode: bool,
) -> Element<'static, Message> {
    let p = theme::palette(dark_mode);

    let search = text_input(texts.search_placeholder, search_query)
        .on_input(Message::SearchInput)
        .padding(8)
        .size(12)
        .style(move |_t: &iced::Theme, status: text_input::Status| text_input::Style {
            background: iced::Background::Color(p.bg_tertiary),
            border: iced::Border {
                color: match status {
                    text_input::Status::Focused => p.border_focused,
                    _ => p.border,
                },
                width: 1.0,
                radius: theme::CORNER_RADIUS.into(),
            },
            icon: p.text_muted,
            placeholder: p.text_muted,
            value: p.text_primary,
            selection: p.accent,
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

    let mut host_list = Column::new().spacing(2);
    for (idx, host) in &filtered_hosts {
        let is_selected = selected_host == Some(*idx);
        let sync_dot = if host.id.is_some() { "*" } else { "o" };
        let sync_color = if host.id.is_some() { p.success } else { p.text_muted };

        let ping_text = match ping_results.get(idx) {
            Some(Some(ms)) if *ms < 100 => text(format!("{}ms", ms)).size(10).color(p.success),
            Some(Some(ms)) if *ms < 300 => text(format!("{}ms", ms)).size(10).color(p.warning),
            Some(Some(ms)) => text(format!("{}ms", ms)).size(10).color(p.danger),
            Some(None) => text("timeout").size(10).color(p.danger),
            None => text("").size(10),
        };

        let alias = host.alias.clone();
        let conn_info = format!("{}@{}:{}", host.username, host.hostname, host.port);
        let i = *idx;

        let host_btn = button(
            row![
                text(sync_dot).size(10).color(sync_color),
                column![
                    text(alias).size(12).color(p.text_primary),
                    text(conn_info).size(10).color(p.text_secondary),
                ]
                .spacing(1),
                iced::widget::horizontal_space(),
                ping_text,
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .on_press(Message::ConnectToHost(i))
        .width(Length::Fill)
        .padding([6, 8])
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
                    radius: theme::CORNER_RADIUS.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        });

        host_list = host_list.push(host_btn);
    }

    let context_buttons: Element<'static, Message> = if let Some(sel) = selected_host {
        row![
            button(text("Edit").size(10).color(p.text_primary))
                .on_press(Message::OpenEditDialog(sel))
                .padding([3, 8])
                .style(move |_t: &iced::Theme, status: button::Status| button::Style {
                    background: Some(iced::Background::Color(match status {
                        button::Status::Hovered => p.bg_hover,
                        _ => p.bg_tertiary,
                    })),
                    text_color: p.text_primary,
                    border: iced::Border {
                        color: p.border,
                        width: 1.0,
                        radius: theme::CORNER_RADIUS.into(),
                    },
                    ..Default::default()
                }),
            button(text("Del").size(10).color(p.danger))
                .on_press(Message::OpenDeleteConfirm(sel))
                .padding([3, 8])
                .style(move |_t: &iced::Theme, status: button::Status| button::Style {
                    background: Some(iced::Background::Color(match status {
                        button::Status::Hovered => p.danger,
                        _ => p.bg_tertiary,
                    })),
                    text_color: if matches!(status, button::Status::Hovered) {
                        p.bg_primary
                    } else {
                        p.danger
                    },
                    border: iced::Border {
                        color: p.border,
                        width: 1.0,
                        radius: theme::CORNER_RADIUS.into(),
                    },
                    ..Default::default()
                }),
        ]
        .spacing(4)
        .into()
    } else {
        row![].into()
    };

    let cpu_label = format!("CPU  {:.0}%", system_info.cpu_usage);
    let ram_label = format!(
        "RAM  {} / {} MB",
        system_info.memory_used_mb, system_info.memory_total_mb
    );
    let disk_label = format!(
        "DSK  {:.1} / {:.1} GB",
        system_info.disk_used_gb, system_info.disk_total_gb
    );

    let sys_monitor = column![
        text(texts.system).size(11).color(p.text_secondary),
        text(cpu_label).size(10).color(p.text_secondary),
        progress_bar(0.0..=100.0, system_info.cpu_usage).height(4),
        text(ram_label).size(10).color(p.text_secondary),
        progress_bar(0.0..=100.0, system_info.memory_usage).height(4),
        text(disk_label).size(10).color(p.text_secondary),
        progress_bar(0.0..=100.0, system_info.disk_usage_percent).height(4),
    ]
    .spacing(3);

    let mut structure_list = Column::new().spacing(2);
    let structure_items: Vec<String> = structure.iter().take(60).cloned().collect();
    if structure_items.is_empty() {
        structure_list = structure_list.push(
            text("No remote structure yet")
                .size(10)
                .color(p.text_muted),
        );
    } else {
        for item in structure_items {
            structure_list = structure_list.push(
                text(item)
                    .size(10)
                    .color(p.text_secondary),
            );
        }
    }

    let structure_panel = column![
        row![
            text("SFTP Structure").size(11).color(p.text_secondary),
            iced::widget::horizontal_space(),
            button(text("Refresh").size(10).color(p.text_primary))
                .on_press(Message::RefreshStructure)
                .padding([2, 8])
                .style(move |_t: &iced::Theme, status: button::Status| button::Style {
                    background: Some(iced::Background::Color(match status {
                        button::Status::Hovered => p.bg_hover,
                        _ => p.bg_tertiary,
                    })),
                    text_color: p.text_primary,
                    border: iced::Border {
                        color: p.border,
                        width: 1.0,
                        radius: theme::CORNER_RADIUS.into(),
                    },
                    ..Default::default()
                }),
        ]
        .align_y(Alignment::Center),
        scrollable(structure_list)
            .height(Length::Fixed(130.0))
            .style(hidden_scrollbar_style),
    ]
    .spacing(6);

    let sidebar_content = column![
        search,
        scrollable(host_list).height(Length::Fill),
        context_buttons,
        horizontal_rule(1),
        sys_monitor,
        horizontal_rule(1),
        structure_panel,
    ]
    .spacing(8)
    .padding(10);

    container(sidebar_content)
        .width(Length::Fixed(theme::SIDEBAR_WIDTH))
        .height(Length::Fill)
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

