use iced::widget::{button, column, container, row, scrollable, text, text_input, Column, Row};
use iced::{Alignment, Element, Length};

use crate::app::{FtpLayout, FtpState, FtpStatus, Message};
use crate::config::AppTheme;
use crate::ftp;
use crate::icons::{self, IconName};
use crate::theme;

pub fn view(state: &FtpState, theme: AppTheme, lc: theme::LayoutConfig, borders_on: bool) -> Element<'static, Message> {
    let p = theme::palette(theme);
    let cr = lc.corner_radius;
    let is_right = state.layout == FtpLayout::Right;
    let in_search = state.search_results.is_some() || state.searching;

    // ── Search bar ────────────────────────────────────────────────────
    let search_val = state.search_query.clone();
    let search_btn: Element<'static, Message> = if in_search {
        icon_nav_btn(IconName::Close, Message::FtpClearSearch, true, p, cr, 10.0)
    } else {
        icon_nav_btn(IconName::Search, Message::FtpSearchSubmit, !search_val.trim().is_empty(), p, cr, 10.0)
    };

    let search_bar = row![
        container(icons::icon(IconName::Search, 10.0, p.text_muted))
            .padding([0, 1])
            .center_y(Length::Fill),
        text_input("Search files...", &search_val)
            .on_input(Message::FtpSearchQueryChanged)
            .on_submit(Message::FtpSearchSubmit)
            .padding([1, 4])
            .size(10)
            .width(Length::Fill)
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
        search_btn,
    ]
    .spacing(1)
    .align_y(Alignment::Center)
    .height(Length::Fixed(22.0));

    // ── Path navigation header ────────────────────────────────────────
    let path_display = state.current_path.clone();
    let parent = ftp::parent_path(&path_display);
    let can_go_up = path_display != "/" && !in_search;
    let can_root = path_display != "/" && !in_search;

    let header = row![
        row![
            icons::icon(IconName::Folder, 11.0, p.accent),
            text("SFTP").size(10).color(p.accent),
        ]
        .spacing(5)
        .align_y(Alignment::Center),
        text(path_display.clone())
            .size(10)
            .color(p.text_secondary)
            .width(Length::Fill)
            .wrapping(iced::widget::text::Wrapping::None),
        icon_nav_btn(IconName::ChevronUp, Message::FtpNavigate(parent), can_go_up, p, cr, 11.0),
        icon_nav_btn(IconName::Folder, Message::FtpNavigate("/".to_string()), can_root, p, cr, 11.0),
        icon_nav_btn(IconName::History, Message::FtpRefresh, !in_search, p, cr, 11.0),
        icon_nav_btn(IconName::ArrowUp, Message::FtpPickUploadFile, !in_search, p, cr, 11.0),
    ]
    .spacing(3)
    .align_y(Alignment::Center);

    // ── Notification bar ──────────────────────────────────────────────
    let notification: Element<'static, Message> = match &state.notification {
        Some((msg, is_err)) => {
            let color = if *is_err { p.danger } else { p.success };
            let notif_icon = if *is_err { IconName::TriangleAlert } else { IconName::Check };
            container(
                row![
                    icons::icon(notif_icon, 10.0, color),
                    text(msg.clone()).size(10).color(color),
                ]
                .spacing(5)
                .align_y(Alignment::Center)
            )
            .padding([3, 8])
            .width(Length::Fill)
            .into()
        }
        None => Column::<Message>::new().into(),
    };

    // ── File / search result list ─────────────────────────────────────
    let file_list: Column<'static, Message> = if state.searching {
        column![status_row(IconName::Search, "Searching...".to_string(), p.text_muted)]
    } else if let Some(ref results) = state.search_results {
        if results.is_empty() {
            column![status_row(IconName::Info, "No results found".to_string(), p.text_muted)]
        } else {
            let mut col = Column::new().spacing(0);
            for entry in results {
                col = col.push(search_result_row(entry, p, cr));
            }
            col
        }
    } else if state.loading {
        column![status_row(IconName::History, "Loading...".to_string(), p.text_muted)]
    } else if let FtpStatus::Error(ref err) = state.status {
        column![
            row![
                icons::icon(IconName::TriangleAlert, 11.0, p.danger),
                text(err.clone()).size(10).color(p.danger),
            ]
            .spacing(5)
            .align_y(Alignment::Center)
            .padding([6, 12]),
        ]
    } else if state.entries.is_empty() {
        column![status_row(IconName::FolderOpen, "Empty directory".to_string(), p.text_muted)]
    } else {
        let mut col = Column::new().spacing(0);
        for entry in &state.entries {
            col = col.push(entry_row(entry, p, cr));
        }
        col
    };

    let body = container(
        scrollable(file_list)
            .height(Length::Fill)
            .style(invisible_scrollbar),
    )
    .height(Length::Fill)
    .width(Length::Fill);

    let panel = column![
        container(
            column![header, search_bar].spacing(2)
        )
        .width(Length::Fill)
        .padding([3, 6])
        .style(move |_: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(p.bg_tertiary)),
            border: iced::Border {
                color: p.border,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        }),
        notification,
        body,
    ]
    .spacing(0);

    // Size depends on layout
    let (width, height) = if is_right {
        (Length::Fixed(320.0), Length::Fill)
    } else {
        (Length::Fill, Length::Fixed(260.0))
    };

    container(panel)
        .width(width)
        .height(height)
        .style(move |_: &iced::Theme| container::Style {
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

fn status_row(icon: IconName, msg: String, color: iced::Color) -> Row<'static, Message> {
    row![
        icons::icon(icon, 11.0, color),
        text(msg).size(10).color(color),
    ]
    .spacing(5)
    .align_y(Alignment::Center)
    .padding([6, 12])
}

fn entry_row(entry: &crate::ftp::FtpEntry, p: crate::theme::Palette, cr: f32) -> Element<'static, Message> {
    let name = entry.name.clone();
    let path = entry.path.clone();
    let is_dir = entry.is_dir;
    let size_str = if is_dir {
        "DIR".to_string()
    } else {
        ftp::format_size(entry.size)
    };

    let msg = if is_dir {
        Message::FtpNavigate(path.clone())
    } else {
        Message::FtpEntryClick(path.clone())
    };

    let name_color = if is_dir { p.accent } else { p.text_primary };
    let type_icon = if is_dir { IconName::Folder } else { IconName::Type };
    let type_color = if is_dir { p.accent } else { p.text_muted };

    button(
        row![
            icons::icon(type_icon, 12.0, type_color),
            text(name)
                .size(11)
                .color(name_color)
                .width(Length::Fill),
            text(size_str)
                .size(10)
                .color(p.text_muted)
                .width(Length::Fixed(60.0)),
        ]
        .spacing(7)
        .align_y(Alignment::Center),
    )
    .on_press(msg)
    .width(Length::Fill)
    .padding([3, 10])
    .style(move |_: &iced::Theme, status: button::Status| button::Style {
        background: Some(iced::Background::Color(match status {
            button::Status::Hovered => p.bg_hover,
            _ => iced::Color::TRANSPARENT,
        })),
        text_color: p.text_primary,
        border: iced::Border {
            radius: cr.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

fn search_result_row(
    entry: &crate::ftp::FtpEntry,
    p: crate::theme::Palette,
    cr: f32,
) -> Element<'static, Message> {
    let name = entry.name.clone();
    let path = entry.path.clone();
    let is_dir = entry.is_dir;

    let msg = if is_dir {
        Message::FtpNavigate(path.clone())
    } else {
        Message::FtpEntryClick(path.clone())
    };

    let name_color = if is_dir { p.accent } else { p.text_primary };
    let type_icon = if is_dir { IconName::Folder } else { IconName::Type };
    let type_color = if is_dir { p.accent } else { p.text_muted };

    // Show parent dir as subtitle
    let parent_dir = std::path::Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    button(
        row![
            icons::icon(type_icon, 12.0, type_color),
            column![
                text(name).size(11).color(name_color),
                row![
                    icons::icon(IconName::Folder, 8.0, p.text_muted),
                    text(parent_dir).size(9).color(p.text_muted),
                ]
                .spacing(3)
                .align_y(Alignment::Center),
            ]
            .spacing(1)
            .width(Length::Fill),
        ]
        .spacing(7)
        .align_y(Alignment::Center),
    )
    .on_press(msg)
    .width(Length::Fill)
    .padding([3, 10])
    .style(move |_: &iced::Theme, status: button::Status| button::Style {
        background: Some(iced::Background::Color(match status {
            button::Status::Hovered => p.bg_hover,
            _ => iced::Color::TRANSPARENT,
        })),
        text_color: p.text_primary,
        border: iced::Border {
            radius: cr.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

fn icon_nav_btn(
    icon: IconName,
    msg: Message,
    enabled: bool,
    p: crate::theme::Palette,
    cr: f32,
    icon_size: f32,
) -> Element<'static, Message> {
    let color = if enabled { p.text_secondary } else { p.text_muted };
    let mut btn = button(
        container(icons::icon(icon, icon_size, color))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .padding(0)
    .width(Length::Fixed(22.0))
    .height(Length::Fixed(20.0))
    .style(move |_: &iced::Theme, status: button::Status| button::Style {
        background: Some(iced::Background::Color(match status {
            button::Status::Hovered if enabled => p.bg_hover,
            _ => iced::Color::TRANSPARENT,
        })),
        text_color: color,
        border: iced::Border {
            radius: cr.into(),
            ..Default::default()
        },
        ..Default::default()
    });
    if enabled {
        btn = btn.on_press(msg);
    }
    btn.into()
}

fn invisible_scrollbar(theme: &iced::Theme, status: scrollable::Status) -> scrollable::Style {
    let mut s = scrollable::default(theme, status);
    let rail = scrollable::Rail {
        background: None,
        border: iced::Border::default(),
        scroller: scrollable::Scroller {
            color: iced::Color::TRANSPARENT,
            border: iced::Border::default(),
        },
    };
    s.vertical_rail = rail;
    s.horizontal_rail = rail;
    s.gap = None;
    s
}
