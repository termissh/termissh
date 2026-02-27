use iced::widget::{button, column, container, row, text, text_input, Column};
use iced::{Element, Length};

use crate::app::Message;
use crate::config::Language;
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
    pub dark_mode: bool,
    pub language: Language,
}

#[derive(Debug, Clone)]
pub enum DialogState {
    NewConnection(ConnectionForm),
    EditConnection(usize, ConnectionForm),
    Settings(SettingsForm),
    ConfirmDelete(usize),
}

pub fn view_dialog(texts: &Texts, state: &DialogState, dark_mode: bool) -> Element<'static, Message> {
    let p = theme::palette(dark_mode);

    let dialog_content: Element<'static, Message> = match state {
        DialogState::NewConnection(form) | DialogState::EditConnection(_, form) => {
            let title = match state {
                DialogState::NewConnection(_) => texts.new_server,
                _ => texts.edit_server,
            };

            let form_clone = form.clone();

            let form_fields = column![
                text(title).size(16).color(p.text_primary),
                labeled_input(texts.alias, &form_clone.alias, |v| {
                    Message::DialogFieldChanged("alias".to_string(), v)
                }, dark_mode),
                labeled_input(texts.hostname, &form_clone.hostname, |v| {
                    Message::DialogFieldChanged("hostname".to_string(), v)
                }, dark_mode),
                labeled_input(texts.port, &form_clone.port, |v| {
                    Message::DialogFieldChanged("port".to_string(), v)
                }, dark_mode),
                labeled_input(texts.username, &form_clone.username, |v| {
                    Message::DialogFieldChanged("username".to_string(), v)
                }, dark_mode),
                labeled_input(texts.password, &form_clone.password, |v| {
                    Message::DialogFieldChanged("password".to_string(), v)
                }, dark_mode),
                row![
                    dialog_button(texts.cancel, Message::CloseDialog, false, dark_mode),
                    dialog_button(texts.save, Message::SaveDialog, true, dark_mode),
                ]
                .spacing(8),
            ]
            .spacing(12)
            .width(Length::Fixed(350.0));

            form_fields.into()
        }
        DialogState::Settings(form) => {
            let form_clone = form.clone();
            column![
                text(texts.api_key_settings).size(16).color(p.text_primary),
                labeled_input(texts.api_key, &form_clone.api_key, |v| {
                    Message::DialogFieldChanged("api_key".to_string(), v)
                }, dark_mode),
                labeled_input(texts.api_url, &form_clone.api_url, |v| {
                    Message::DialogFieldChanged("api_url".to_string(), v)
                }, dark_mode),
                text("Theme").size(11).color(p.text_secondary),
                row![
                    select_button(
                        "Light",
                        !form_clone.dark_mode,
                        Message::SettingsThemeChanged(false),
                        dark_mode,
                    ),
                    select_button(
                        "Dark",
                        form_clone.dark_mode,
                        Message::SettingsThemeChanged(true),
                        dark_mode,
                    ),
                ]
                .spacing(8),
                text("Language").size(11).color(p.text_secondary),
                row![
                    select_button(
                        "TR",
                        matches!(form_clone.language, Language::Turkish),
                        Message::SettingsLanguageChanged(Language::Turkish),
                        dark_mode,
                    ),
                    select_button(
                        "EN",
                        matches!(form_clone.language, Language::English),
                        Message::SettingsLanguageChanged(Language::English),
                        dark_mode,
                    ),
                ]
                .spacing(8),
                row![
                    dialog_button(texts.cancel, Message::CloseDialog, false, dark_mode),
                    dialog_button(texts.save, Message::SaveSettings, true, dark_mode),
                ]
                .spacing(8),
            ]
            .spacing(12)
            .width(Length::Fixed(400.0))
            .into()
        }
        DialogState::ConfirmDelete(idx) => {
            let idx = *idx;
            column![
                text(texts.delete_confirm).size(14).color(p.text_primary),
                row![
                    dialog_button(texts.cancel, Message::CloseDialog, false, dark_mode),
                    dialog_button(texts.delete, Message::ConfirmDelete(idx), false, dark_mode),
                ]
                .spacing(8),
            ]
            .spacing(16)
            .width(Length::Fixed(350.0))
            .into()
        }
    };

    let card = container(
        container(dialog_content)
            .padding(24)
            .style(move |_t: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(p.bg_secondary)),
                border: iced::Border {
                    color: p.border,
                    width: 1.0,
                    radius: theme::CORNER_RADIUS.into(),
                },
                ..Default::default()
            }),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(move |_t: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgba(
            0.0, 0.0, 0.0, 0.55,
        ))),
        ..Default::default()
    });

    card.into()
}

fn labeled_input<'a>(
    label: &'static str,
    value: &str,
    on_input: impl Fn(String) -> Message + 'static,
    dark_mode: bool,
) -> Column<'a, Message> {
    let p = theme::palette(dark_mode);
    let value_owned = value.to_string();

    column![
        text(label).size(11).color(p.text_secondary),
        text_input("", &value_owned)
            .on_input(on_input)
            .padding(8)
            .size(13)
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
            }),
    ]
    .spacing(4)
}

fn dialog_button(
    label: &'static str,
    msg: Message,
    primary: bool,
    dark_mode: bool,
) -> Element<'static, Message> {
    let p = theme::palette(dark_mode);

    button(text(label).size(12).color(p.text_primary))
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
                    radius: theme::CORNER_RADIUS.into(),
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
    dark_mode: bool,
) -> Element<'static, Message> {
    let p = theme::palette(dark_mode);

    button(text(label).size(12).color(p.text_primary))
        .on_press(msg)
        .padding([6, 12])
        .style(move |_t: &iced::Theme, status: button::Status| {
            let bg = if selected {
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
                    radius: theme::CORNER_RADIUS.into(),
                },
                ..Default::default()
            }
        })
        .into()
}

