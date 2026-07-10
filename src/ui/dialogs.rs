use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input, Column, Row};
use iced::{Alignment, Element, Length};

use crate::app::Message;
use crate::config::{AppTheme, CustomCommand, Language, LayoutPreset};
use crate::icons::{self, IconName};
use crate::i18n::Texts;
use crate::theme;

#[derive(Debug, Clone)]
pub struct ConnectionForm {
    pub alias: String,
    pub hostname: String,
    pub port: String,
    pub username: String,
    pub password: String,
}

impl Default for ConnectionForm {
    fn default() -> Self {
        Self {
            alias: String::new(),
            hostname: String::new(),
            port: "22".to_string(),
            username: String::new(),
            password: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SettingsForm {
    pub api_key: String,
    pub api_url: String,
    pub theme: AppTheme,
    pub language: Language,
    pub layout: LayoutPreset,
    pub terminal_font_size: f32,
    pub show_borders: bool,
    pub suggestions_enabled: bool,
    pub auto_reconnect: bool,
    pub reconnect_interval_secs: u32,
    pub local_storage_only: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CustomCommandsForm {
    pub commands: Vec<CustomCommand>,
    pub new_trigger: String,
    pub new_script: String,
    pub new_description: String,
}

#[derive(Debug, Clone)]
pub enum DialogState {
    NewConnection(ConnectionForm),
    EditConnection(usize, ConnectionForm),
    Settings(SettingsForm),
    ConfirmDelete(usize),
    CustomCommands(CustomCommandsForm),
}

pub fn view_dialog(
    texts: &Texts,
    state: &DialogState,
    theme: AppTheme,
    lc: theme::LayoutConfig,
    borders_on: bool,
) -> Element<'static, Message> {
    let p = theme::palette(theme);
    let cr = lc.corner_radius;

    let (title, title_icon, dialog_width, body) = match state {
        DialogState::NewConnection(form) => (
            texts.new_server,
            IconName::Plus,
            420.0_f32,
            connection_form_body(texts, form, theme, cr, borders_on),
        ),
        DialogState::EditConnection(_, form) => (
            texts.edit_server,
            IconName::Pencil,
            420.0_f32,
            connection_form_body(texts, form, theme, cr, borders_on),
        ),
        DialogState::Settings(form) => (
            "Settings",
            IconName::Settings,
            480.0_f32,
            settings_body(texts, form, theme, cr, borders_on),
        ),
        DialogState::ConfirmDelete(_) => (
            texts.delete_confirm,
            IconName::Trash,
            360.0_f32,
            confirm_delete_body(texts, state, theme, cr),
        ),
        DialogState::CustomCommands(form) => (
            "Custom Commands",
            IconName::Command,
            520.0_f32,
            custom_commands_body(texts, form, theme, cr, borders_on),
        ),
    };

    let body = body;

    let header = modal_header(title, title_icon, p, cr);
    let footer = modal_footer(texts, state, theme, cr);

    let dialog_inner = column![header, body, footer]
        .spacing(0)
        .width(Length::Fixed(dialog_width));

    let card = container(dialog_inner).padding(20).style(move |_t: &iced::Theme| {
        // Frosted-glass feel: slightly translucent bg + accent border + soft deep shadow
        container::Style {
            background: Some(iced::Background::Color(iced::Color {
                a: 0.97,
                ..p.bg_secondary
            })),
            border: iced::Border {
                color: iced::Color {
                    a: 0.6,
                    ..p.border_focused
                },
                width: 1.0,
                radius: 12.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.55),
                offset: iced::Vector::new(0.0, 16.0),
                blur_radius: 40.0,
            },
            ..Default::default()
        }
    });

    // Scrim with deeper, slightly warm-black tint to imply frosted glass behind the modal
    let scrim = container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(move |_t: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.04, 0.05, 0.07, 0.62,
            ))),
            ..Default::default()
        });

    scrim.into()
}

// ── Header (title row + close button) ──────────────────────────────────
fn modal_header(title: &'static str, icon: IconName, p: theme::Palette, cr: f32) -> Element<'static, Message> {
    let title_text = text(title).size(14).color(p.text_primary).font(iced::Font {
        family: iced::font::Family::Name("Segoe UI"),
        weight: iced::font::Weight::Semibold,
        ..iced::Font::DEFAULT
    });

    let title_row = row![
        icons::icon(icon, 16.0, p.accent),
        title_text,
    ]
    .spacing(9)
    .align_y(Alignment::Center);

    let close = button(
        container(icons::icon(IconName::Close, 12.0, p.text_secondary))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_press(Message::CloseDialog)
    .padding(0)
    .width(Length::Fixed(24.0))
    .height(Length::Fixed(24.0))
    .style(move |_t: &iced::Theme, status: button::Status| {
        let hovered = matches!(status, button::Status::Hovered);
        button::Style {
            background: Some(iced::Background::Color(if hovered {
                p.bg_hover
            } else {
                iced::Color::TRANSPARENT
            })),
            text_color: if hovered { p.text_primary } else { p.text_secondary },
            border: iced::Border {
                color: if hovered { p.border_focused } else { p.border },
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    });

    container(
        row![title_row, iced::widget::horizontal_space(), close]
            .align_y(Alignment::Center)
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .padding(iced::Padding { top: 0.0, right: 4.0, bottom: 10.0, left: 4.0 })
    .style(move |_t: &iced::Theme| container::Style {
        background: None,
        border: iced::Border {
            color: p.border,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}

// ── Footer (Cancel / Save row) ────────────────────────────────────────
fn modal_footer(
    texts: &Texts,
    state: &DialogState,
    theme: AppTheme,
    cr: f32,
) -> Element<'static, Message> {
    let p = theme::palette(theme);
    let (save_msg, save_label) = match state {
        DialogState::NewConnection(_) | DialogState::EditConnection(_, _) => {
            (Message::SaveDialog, texts.save)
        }
        DialogState::Settings(_) => (Message::SaveSettings, texts.save),
        DialogState::ConfirmDelete(idx) => (Message::ConfirmDelete(*idx), texts.delete),
        DialogState::CustomCommands(_) => (Message::SaveCustomCommands, texts.save),
    };

    let cancel = button(text(texts.cancel).size(11).color(p.text_secondary))
        .on_press(Message::CloseDialog)
        .padding([7, 18])
        .style(move |_t: &iced::Theme, status: button::Status| {
            button::Style {
                background: Some(iced::Background::Color(match status {
                    button::Status::Hovered => p.bg_hover,
                    _ => iced::Color::TRANSPARENT,
                })),
                text_color: p.text_secondary,
                border: iced::Border {
                    color: p.border,
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        });

    let save = button(
        row![
            icons::icon(IconName::Check, 12.0, p.text_primary),
            text(save_label).size(11).color(p.text_primary),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .on_press(save_msg)
    .padding([7, 18])
    .style(move |_t: &iced::Theme, status: button::Status| {
        let bg = match status {
            button::Status::Hovered => p.accent_hover,
            _ => p.accent,
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: p.text_primary,
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        }
    });

    let footer = row![iced::widget::horizontal_space(), cancel, save]
        .spacing(6)
        .align_y(Alignment::Center)
        .width(Length::Fill);

    container(footer)
        .width(Length::Fill)
        .padding(iced::Padding { top: 10.0, right: 4.0, bottom: 0.0, left: 4.0 })
        .style(move |_t: &iced::Theme| container::Style {
            background: None,
            ..Default::default()
        })
        .into()
}

// ── Section card (icon + title + body) ─────────────────────────────────
fn section_card(
    icon: IconName,
    title: &'static str,
    body: Element<'static, Message>,
    p: theme::Palette,
) -> Element<'static, Message> {
    let header_row = row![
        icons::icon(icon, 12.0, p.accent),
        text(title).size(11).color(p.text_secondary).font(iced::Font {
            family: iced::font::Family::Name("Segoe UI"),
            weight: iced::font::Weight::Semibold,
            ..iced::Font::DEFAULT
        }),
    ]
    .spacing(7)
    .align_y(Alignment::Center);

    let inner = column![header_row, body]
        .spacing(7)
        .width(Length::Fill)
        .padding([10, 12]);

    container(inner)
        .width(Length::Fill)
        .style(move |_t: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(p.bg_tertiary)),
            border: iced::Border {
                color: iced::Color {
                    a: 0.5,
                    ..p.border
                },
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        })
        .into()
}

// ── Body builders ─────────────────────────────────────────────────────

fn connection_form_body(
    _texts: &Texts,
    form: &ConnectionForm,
    theme: AppTheme,
    cr: f32,
    _borders_on: bool,
) -> Element<'static, Message> {
    let p = theme::palette(theme);
    let form_clone = form.clone();

    let identity_card = section_card(
        IconName::Hash,
        "Identity",
        column![
            labeled_input("Alias", &form_clone.alias, |v| {
                Message::DialogFieldChanged("alias".to_string(), v)
            }, theme, cr),
            labeled_input("Username", &form_clone.username, |v| {
                Message::DialogFieldChanged("username".to_string(), v)
            }, theme, cr),
        ]
        .spacing(6)
        .width(Length::Fill)
        .into(),
        p,
    );

    let server_card = section_card(
        IconName::Server,
        "Server",
        column![
            labeled_input("Hostname", &form_clone.hostname, |v| {
                Message::DialogFieldChanged("hostname".to_string(), v)
            }, theme, cr),
            labeled_input("Port", &form_clone.port, |v| {
                Message::DialogFieldChanged("port".to_string(), v)
            }, theme, cr),
        ]
        .spacing(6)
        .width(Length::Fill)
        .into(),
        p,
    );

    let auth_card = section_card(
        IconName::SquareAsterisk,
        "Authentication",
        column![labeled_input("Password", &form_clone.password, |v| {
            Message::DialogFieldChanged("password".to_string(), v)
        }, theme, cr),]
        .spacing(4)
        .width(Length::Fill)
        .into(),
        p,
    );

    column![identity_card, server_card, auth_card]
        .spacing(7)
        .width(Length::Fill)
        .into()
}

fn settings_body(
    _texts: &Texts,
    form: &SettingsForm,
    theme: AppTheme,
    cr: f32,
    _borders_on: bool,
) -> Element<'static, Message> {
    let p = theme::palette(theme);
    let form_clone = form.clone();
    let font_size = form_clone.terminal_font_size;
    let borders_on = form_clone.show_borders;
    let suggestions_on = form_clone.suggestions_enabled;
    let auto_reconnect = form_clone.auto_reconnect;
    let reconnect_interval = form_clone.reconnect_interval_secs;
    let local_only = form_clone.local_storage_only;

    let theme_picker = pick_list(
        AppTheme::all(),
        Some(form_clone.theme),
        Message::SettingsThemeChanged,
    )
    .width(Length::Fill)
    .style(move |_t: &iced::Theme, status: pick_list::Status| pick_list::Style {
        text_color: p.text_primary,
        placeholder_color: p.text_muted,
        handle_color: p.accent,
        background: iced::Background::Color(p.bg_secondary),
        border: iced::Border {
            color: match status {
                pick_list::Status::Hovered | pick_list::Status::Opened => p.border_focused,
                _ => p.border,
            },
            width: 1.0,
            radius: 6.0.into(),
        },
    });

    let layout_picker = pick_list(
        LayoutPreset::all(),
        Some(form_clone.layout),
        Message::SettingsLayoutChanged,
    )
    .width(Length::Fill)
    .style(move |_t: &iced::Theme, status: pick_list::Status| pick_list::Style {
        text_color: p.text_primary,
        placeholder_color: p.text_muted,
        handle_color: p.accent,
        background: iced::Background::Color(p.bg_secondary),
        border: iced::Border {
            color: match status {
                pick_list::Status::Hovered | pick_list::Status::Opened => p.border_focused,
                _ => p.border,
            },
            width: 1.0,
            radius: 6.0.into(),
        },
    });

    let lang_row = row![
        select_button("Türkçe", matches!(form_clone.language, Language::Turkish),
            Message::SettingsLanguageChanged(Language::Turkish), theme, cr),
        select_button("English", matches!(form_clone.language, Language::English),
            Message::SettingsLanguageChanged(Language::English), theme, cr),
    ]
    .spacing(8);

    // ── Sections ──
    let api_card = section_card(
        IconName::Server,
        "API Sync",
        column![
            labeled_input("API Key", &form_clone.api_key, |v| {
                Message::DialogFieldChanged("api_key".to_string(), v)
            }, theme, cr),
            labeled_input("API URL", &form_clone.api_url, |v| {
                Message::DialogFieldChanged("api_url".to_string(), v)
            }, theme, cr),
        ]
        .spacing(6)
        .width(Length::Fill)
        .into(),
        p,
    );

    let appearance_card = section_card(
        IconName::Sun,
        "Appearance",
        column![
            column![text("Theme").size(10).color(p.text_muted), theme_picker].spacing(4).width(Length::Fill),
            column![text("Layout").size(10).color(p.text_muted), layout_picker].spacing(4).width(Length::Fill),
            column![text("Language").size(10).color(p.text_muted), lang_row].spacing(4).width(Length::Fill),
        ]
        .spacing(7)
        .width(Length::Fill)
        .into(),
        p,
    );

    let terminal_card = section_card(
        IconName::Terminal,
        "Terminal",
        column![
            column![text("Default Font Size").size(10).color(p.text_muted),
                row![
                    select_button("A−", false, Message::SettingsFontSizeChanged(font_size - 1.0), theme, cr),
                    container(text(format!("{:.0}px", font_size)).size(11).color(p.text_primary))
                        .padding([5, 12])
                        .style(move |_: &iced::Theme| container::Style {
                            background: Some(iced::Background::Color(p.bg_secondary)),
                            border: iced::Border { color: p.border, width: 1.0, radius: 6.0.into() },
                            ..Default::default()
                        }),
                    select_button("A+", false, Message::SettingsFontSizeChanged(font_size + 1.0), theme, cr),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            ]
            .spacing(4)
            .width(Length::Fill),
            column![text("Panel Borders").size(10).color(p.text_muted),
                row![
                    select_button("Bordered", borders_on, Message::SettingsShowBordersChanged(true), theme, cr),
                    select_button("Borderless", !borders_on, Message::SettingsShowBordersChanged(false), theme, cr),
                ]
                .spacing(6),
            ]
            .spacing(4)
            .width(Length::Fill),
        ]
        .spacing(7)
        .width(Length::Fill)
        .into(),
        p,
    );

    let suggestions_card = section_card(
        IconName::Search,
        "Suggestions",
        column![column![text("Command Suggestions").size(10).color(p.text_muted),
            row![
                select_button("Enabled", suggestions_on, Message::SettingsSuggestionsChanged(true), theme, cr),
                select_button("Disabled", !suggestions_on, Message::SettingsSuggestionsChanged(false), theme, cr),
            ]
            .spacing(6),
        ]
        .spacing(4)
        .width(Length::Fill),]
        .spacing(4)
        .width(Length::Fill)
        .into(),
        p,
    );

    let connection_card = section_card(
        IconName::Cable,
        "Connection",
        column![
            column![text("Auto-Reconnect on Drop").size(10).color(p.text_muted),
                row![
                    select_button("Enabled", auto_reconnect, Message::SettingsAutoReconnectChanged(true), theme, cr),
                    select_button("Disabled", !auto_reconnect, Message::SettingsAutoReconnectChanged(false), theme, cr),
                ]
                .spacing(6),
            ]
            .spacing(4)
            .width(Length::Fill),
            column![text("Reconnect Interval").size(10).color(p.text_muted),
                row![
                    select_button("3s",  reconnect_interval == 3,  Message::SettingsReconnectIntervalChanged(3),  theme, cr),
                    select_button("5s",  reconnect_interval == 5,  Message::SettingsReconnectIntervalChanged(5),  theme, cr),
                    select_button("10s", reconnect_interval == 10, Message::SettingsReconnectIntervalChanged(10), theme, cr),
                    select_button("30s", reconnect_interval == 30, Message::SettingsReconnectIntervalChanged(30), theme, cr),
                ]
                .spacing(6),
            ]
            .spacing(4)
            .width(Length::Fill),
        ]
        .spacing(7)
        .width(Length::Fill)
        .into(),
        p,
    );

    let storage_card = section_card(
        IconName::Power,
        "Storage",
        column![
            text("Keep All SSH Info Locally")
                .size(11)
                .color(p.text_primary),
            text("When enabled, hosts and credentials are stored only on this device and never synced to the API.")
                .size(10)
                .color(p.text_muted),
            row![
                select_button("Local Only", local_only, Message::SettingsLocalStorageChanged(true), theme, cr),
                select_button("Allow Sync", !local_only, Message::SettingsLocalStorageChanged(false), theme, cr),
            ]
            .spacing(6),
        ]
        .spacing(6)
        .width(Length::Fill)
        .into(),
        p,
    );

    column![
        api_card,
        appearance_card,
        terminal_card,
        suggestions_card,
        connection_card,
        storage_card,
    ]
    .spacing(10)
    .width(Length::Fill)
    .into()
}

fn confirm_delete_body(
    _texts: &Texts,
    _state: &DialogState,
    _theme: AppTheme,
    _cr: f32,
) -> Element<'static, Message> {
    // The actual content is just a small subtitle, body is minimal.
    Element::from(
        container(text(""))
            .width(Length::Fill)
            .height(Length::Fixed(8.0)),
    )
}

fn custom_commands_body(
    _texts: &Texts,
    form: &CustomCommandsForm,
    theme: AppTheme,
    cr: f32,
    _borders_on: bool,
) -> Element<'static, Message> {
    let p = theme::palette(theme);
    let form_clone = form.clone();

    let mut list_col = Column::new().spacing(4);
    if form_clone.commands.is_empty() {
        list_col = list_col.push(
            row![
                icons::icon(IconName::Info, 11.0, p.text_muted),
                text("No custom commands yet — add one below.")
                    .size(10)
                    .color(p.text_muted),
            ]
            .spacing(6)
            .align_y(Alignment::Center)
            .padding([4, 4]),
        );
    }
    for (idx, cmd) in form_clone.commands.iter().enumerate() {
        let trigger_label = cmd.trigger.clone();
        let desc_label = if cmd.description.is_empty() {
            cmd.script.chars().take(40).collect::<String>()
        } else {
            cmd.description.clone()
        };
        let row_content = row![
            icons::icon(IconName::SquareAsterisk, 10.0, p.accent),
            text(trigger_label).size(11).color(p.accent).width(Length::Fixed(90.0)),
            text(desc_label).size(10).color(p.text_muted).width(Length::Fill),
            button(
                container(icons::icon(IconName::Close, 9.0, p.text_muted))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .on_press(Message::DeleteCustomCommand(idx))
            .padding(0)
            .width(Length::Fixed(20.0))
            .height(Length::Fixed(20.0))
            .style(move |_t: &iced::Theme, s: button::Status| button::Style {
                background: Some(iced::Background::Color(match s {
                    button::Status::Hovered => p.danger,
                    _ => iced::Color::TRANSPARENT,
                })),
                text_color: if matches!(s, button::Status::Hovered) { p.bg_primary } else { p.text_muted },
                border: iced::Border {
                    color: p.border,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }),
        ]
        .spacing(6)
        .align_y(Alignment::Center);
        list_col = list_col.push(
            container(row_content)
                .padding([4, 8])
                .width(Length::Fill)
                .style(move |_t: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(p.bg_tertiary)),
                    border: iced::Border {
                        color: p.border,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                }),
        );
    }

    let list_scroll: Element<'static, Message> = scrollable(list_col).height(Length::Fixed(150.0)).into();

    let add_form = section_card(
        IconName::Plus,
        "Add Custom Command",
        column![
            labeled_input(
                "Trigger (e.g. -runtest)",
                &form_clone.new_trigger,
                |v| Message::DialogFieldChanged("trigger".to_string(), v),
                theme, cr,
            ),
            labeled_input(
                "Script (e.g. cd /app && npm test)",
                &form_clone.new_script,
                |v| Message::DialogFieldChanged("script".to_string(), v),
                theme, cr,
            ),
            labeled_input(
                "Description (optional)",
                &form_clone.new_description,
                |v| Message::DialogFieldChanged("description".to_string(), v),
                theme, cr,
            ),
            button(
                row![
                    icons::icon(IconName::Plus, 11.0, p.text_primary),
                    text("Add Command").size(11).color(p.text_primary),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .on_press(Message::AddCustomCommand)
            .padding([6, 14])
            .style(move |_t: &iced::Theme, s: button::Status| button::Style {
                background: Some(iced::Background::Color(match s {
                    button::Status::Hovered => p.accent_hover,
                    _ => p.accent,
                })),
                text_color: p.text_primary,
                border: iced::Border {
                    color: iced::Color::TRANSPARENT,
                    width: 0.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }),
        ]
        .spacing(6)
        .width(Length::Fill)
        .into(),
        p,
    );

    let list_section = section_card(
        IconName::Command,
        "Your Commands",
        list_scroll,
        p,
    );

    column![list_section, add_form]
        .spacing(7)
        .width(Length::Fill)
        .into()
}

// ── Form helpers ─────────────────────────────────────────────────────

fn labeled_input(
    label: &'static str,
    value: &str,
    on_input: impl Fn(String) -> Message + 'static,
    theme: AppTheme,
    cr: f32,
) -> Column<'static, Message> {
    let p = theme::palette(theme);
    let value_owned = value.to_string();

    column![
        text(label).size(10).color(p.text_muted),
        text_input("", &value_owned)
            .on_input(on_input)
            .padding(5)
            .size(11)
            .style(move |_t: &iced::Theme, status: text_input::Status| text_input::Style {
                background: iced::Background::Color(p.bg_secondary),
                border: iced::Border {
                    color: match status {
                        text_input::Status::Focused => p.accent,
                        _ => p.border,
                    },
                    width: 1.0,
                    radius: 6.0.into(),
                },
                icon: p.text_muted,
                placeholder: p.text_muted,
                value: p.text_primary,
                selection: p.accent,
            }),
    ]
    .spacing(4)
}

fn dialog_button(
    label: &'static str,
    msg: Message,
    primary: bool,
    theme: AppTheme,
    cr: f32,
) -> Element<'static, Message> {
    let p = theme::palette(theme);

    button(text(label).size(11).color(p.text_primary))
        .on_press(msg)
        .padding([6, 16])
        .style(move |_t: &iced::Theme, status: button::Status| {
            let bg = if primary {
                match status {
                    button::Status::Hovered => p.accent_hover,
                    _ => p.accent,
                }
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
                    color: p.border,
                    width: 1.0,
                    radius: cr.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

fn select_button(
    label: &'static str,
    selected: bool,
    msg: Message,
    theme: AppTheme,
    cr: f32,
) -> Element<'static, Message> {
    let p = theme::palette(theme);

    button(text(label).size(11).color(p.text_primary))
        .on_press(msg)
        .padding([5, 12])
        .style(move |_t: &iced::Theme, status: button::Status| {
            let bg = if selected {
                match status {
                    button::Status::Hovered => p.accent_hover,
                    _ => p.accent,
                }
            } else {
                match status {
                    button::Status::Hovered => p.bg_hover,
                    _ => p.bg_secondary,
                }
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: p.text_primary,
                border: iced::Border {
                    color: if selected { iced::Color::TRANSPARENT } else { p.border },
                    width: 1.0,
                    radius: 6.0.into(),
                },
                ..Default::default()
            }
        })
        .into()
}
