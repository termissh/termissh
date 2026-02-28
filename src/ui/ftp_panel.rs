use iced::widget::{button, column, container, row, scrollable, text, text_input, Column};
use iced::{Alignment, Element, Length};

use crate::app::{FtpLayout, FtpState, FtpStatus, Message};
use crate::config::AppTheme;
use crate::ftp;
use crate::theme;

pub fn view(state: &FtpState, theme: AppTheme, lc: theme::LayoutConfig) -> Element<'static, Message> {
    let p = theme::palette(theme);
    let cr = lc.corner_radius;
    let is_right = state.layout == FtpLayout::Right;
    let in_search = state.search_results.is_some() || state.searching;

    // ── Search bar ────────────────────────────────────────────────────
    let search_val = state.search_query.clone();
    let clear_or_search: Element<'static, Message> = if in_search {
        nav_btn("Clr", Message::FtpClearSearch, true, p, cr)
    } else {
        nav_btn("Srch", Message::FtpSearchSubmit, !search_val.trim().is_empty(), p, cr)
    };

    let search_bar = row![
        text_input("Search files...", &search_val)
            .on_input(Message::FtpSearchQueryChanged)
            .on_submit(Message::FtpSearchSubmit)
            .padding([3, 6])
            .size(11)
            .width(Length::Fill)
            .style(move |_t: &iced::Theme, status: text_input::Status| text_input::Style {
                background: iced::Background::Color(p.bg_primary),
                border: iced::Border {
                    color: match status {
                        text_input::Status::Focused => p.border_focused,
                        _ => p.border,
                    },
                    width: 1.0,
                    radius: cr.into(),
                },
                icon: p.text_muted,
                placeholder: p.text_muted,
                value: p.text_primary,
                selection: p.accent,
            }),
        clear_or_search,
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    // ── Path navigation header ────────────────────────────────────────
    let path_display = state.current_path.clone();
    let parent = ftp::parent_path(&path_display);
    let can_go_up = path_display != "/" && !in_search;
    let can_root = path_display != "/" && !in_search;

    let header = row![
        text("SFTP").size(10).color(p.accent),
        text("  ").size(10),
        text(path_display.clone()).size(10).color(p.text_secondary),
        iced::widget::horizontal_space(),
        nav_btn("Up", Message::FtpNavigate(parent), can_go_up, p, cr),
        nav_btn("/root", Message::FtpNavigate("/".to_string()), can_root, p, cr),
        nav_btn("Refresh", Message::FtpRefresh, !in_search, p, cr),
        nav_btn("Upload", Message::FtpPickUploadFile, !in_search, p, cr),
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    // ── Notification bar ──────────────────────────────────────────────
    let notification: Element<'static, Message> = match &state.notification {
        Some((msg, is_err)) => {
            let color = if *is_err { p.danger } else { p.success };
            container(text(msg.clone()).size(10).color(color))
                .padding([2, 8])
                .width(Length::Fill)
                .into()
        }
        None => iced::widget::Space::new(0.0, 0.0).into(),
    };

    // ── File / search result list ─────────────────────────────────────
    let file_list: Column<'static, Message> = if state.searching {
        column![text("  Searching...").size(11).color(p.text_muted)]
    } else if let Some(ref results) = state.search_results {
        if results.is_empty() {
            column![text("  No results found").size(10).color(p.text_muted)]
        } else {
            let mut col = Column::new().spacing(0);
            for entry in results {
                col = col.push(search_result_row(entry, p, cr));
            }
            col
        }
    } else if state.loading {
        column![text("  Loading...").size(11).color(p.text_muted)]
    } else if let FtpStatus::Error(ref err) = state.status {
        column![text(format!("  ⚠ {}", err)).size(10).color(p.danger)]
    } else if state.entries.is_empty() {
        column![text("  (empty directory)").size(10).color(p.text_muted)]
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
            column![header, search_bar].spacing(6)
        )
        .width(Length::Fill)
        .padding([4, 8])
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
    let prefix = if is_dir { "▸ " } else { "  " };

    button(
        row![
            text(format!("{}{}", prefix, name))
                .size(11)
                .color(name_color)
                .width(Length::Fill),
            text(size_str)
                .size(10)
                .color(p.text_muted)
                .width(Length::Fixed(60.0)),
        ]
        .spacing(8)
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
    let prefix = if is_dir { "▸ " } else { "  " };

    // Show parent dir as subtitle
    let parent_dir = std::path::Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    button(
        column![
            text(format!("{}{}", prefix, name))
                .size(11)
                .color(name_color),
            text(parent_dir)
                .size(9)
                .color(p.text_muted),
        ]
        .spacing(1),
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

fn nav_btn(
    label: &'static str,
    msg: Message,
    enabled: bool,
    p: crate::theme::Palette,
    cr: f32,
) -> Element<'static, Message> {
    let color = if enabled { p.text_secondary } else { p.text_muted };
    let mut btn = button(text(label).size(10).color(color)).padding([2, 8]).style(
        move |_: &iced::Theme, status: button::Status| button::Style {
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
        },
    );
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
