use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input, Column};
use iced::{Element, Length};

use crate::app::{Message, SecurityFinding, SecuritySeverity};
use crate::config::{AppTheme, CustomCommand, Language, LayoutPreset};
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
    SecurityAudit(Vec<SecurityFinding>),
}

pub fn view_dialog(texts: &Texts, state: &DialogState, theme: AppTheme, lc: theme::LayoutConfig) -> Element<'static, Message> {
    let p = theme::palette(theme);
    let cr = lc.corner_radius;

    let dialog_content: Element<'static, Message> = match state {
        DialogState::NewConnection(form) | DialogState::EditConnection(_, form) => {
            let title = match state {
                DialogState::NewConnection(_) => texts.new_server,
                _ => texts.edit_server,
            };
            let form_clone = form.clone();
            column![
                text(title).size(16).color(p.text_primary),
                labeled_input(texts.alias, &form_clone.alias, |v| {
                    Message::DialogFieldChanged("alias".to_string(), v)
                }, theme, cr),
                labeled_input(texts.hostname, &form_clone.hostname, |v| {
                    Message::DialogFieldChanged("hostname".to_string(), v)
                }, theme, cr),
                labeled_input(texts.port, &form_clone.port, |v| {
                    Message::DialogFieldChanged("port".to_string(), v)
                }, theme, cr),
                labeled_input(texts.username, &form_clone.username, |v| {
                    Message::DialogFieldChanged("username".to_string(), v)
                }, theme, cr),
                labeled_input(texts.password, &form_clone.password, |v| {
                    Message::DialogFieldChanged("password".to_string(), v)
                }, theme, cr),
                row![
                    dialog_button(texts.cancel, Message::CloseDialog, false, theme, cr),
                    dialog_button(texts.save, Message::SaveDialog, true, theme, cr),
                ]
                .spacing(8),
            ]
            .spacing(12)
            .width(Length::Fixed(350.0))
            .into()
        }

        DialogState::Settings(form) => {
            let form_clone = form.clone();
            let current_theme = form_clone.theme;
            let current_layout = form_clone.layout;

            let theme_picker = pick_list(
                AppTheme::all(),
                Some(current_theme),
                Message::SettingsThemeChanged,
            )
            .width(Length::Fill)
            .style(move |_t: &iced::Theme, status: pick_list::Status| pick_list::Style {
                text_color: p.text_primary,
                placeholder_color: p.text_muted,
                handle_color: p.accent,
                background: iced::Background::Color(p.bg_tertiary),
                border: iced::Border {
                    color: match status {
                        pick_list::Status::Hovered | pick_list::Status::Opened => p.border_focused,
                        _ => p.border,
                    },
                    width: 1.0,
                    radius: cr.into(),
                },
            });

            let layout_picker = pick_list(
                LayoutPreset::all(),
                Some(current_layout),
                Message::SettingsLayoutChanged,
            )
            .width(Length::Fill)
            .style(move |_t: &iced::Theme, status: pick_list::Status| pick_list::Style {
                text_color: p.text_primary,
                placeholder_color: p.text_muted,
                handle_color: p.accent,
                background: iced::Background::Color(p.bg_tertiary),
                border: iced::Border {
                    color: match status {
                        pick_list::Status::Hovered | pick_list::Status::Opened => p.border_focused,
                        _ => p.border,
                    },
                    width: 1.0,
                    radius: cr.into(),
                },
            });

            column![
                text(texts.api_key_settings).size(16).color(p.text_primary),
                labeled_input(texts.api_key, &form_clone.api_key, |v| {
                    Message::DialogFieldChanged("api_key".to_string(), v)
                }, theme, cr),
                labeled_input(texts.api_url, &form_clone.api_url, |v| {
                    Message::DialogFieldChanged("api_url".to_string(), v)
                }, theme, cr),
                column![
                    text("Theme").size(11).color(p.text_secondary),
                    theme_picker,
                ].spacing(4),
                column![
                    text("Layout").size(11).color(p.text_secondary),
                    layout_picker,
                ].spacing(4),
                text("Language").size(11).color(p.text_secondary),
                row![
                    select_button("TR", matches!(form_clone.language, Language::Turkish),
                        Message::SettingsLanguageChanged(Language::Turkish), theme, cr),
                    select_button("EN", matches!(form_clone.language, Language::English),
                        Message::SettingsLanguageChanged(Language::English), theme, cr),
                ]
                .spacing(8),
                row![
                    dialog_button(texts.cancel, Message::CloseDialog, false, theme, cr),
                    dialog_button(texts.save, Message::SaveSettings, true, theme, cr),
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
                    dialog_button(texts.cancel, Message::CloseDialog, false, theme, cr),
                    dialog_button(texts.delete, Message::ConfirmDelete(idx), false, theme, cr),
                ]
                .spacing(8),
            ]
            .spacing(16)
            .width(Length::Fixed(350.0))
            .into()
        }

        DialogState::CustomCommands(form) => {
            let form_clone = form.clone();

            // Existing commands list
            let mut list_col = Column::new().spacing(4);
            if form_clone.commands.is_empty() {
                list_col = list_col.push(
                    text("No custom commands yet. Add one below.")
                        .size(11)
                        .color(p.text_muted),
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
                    text(trigger_label).size(11).color(p.accent).width(Length::Fixed(90.0)),
                    text(desc_label).size(10).color(p.text_muted).width(Length::Fill),
                    button(text("✕").size(10).color(p.danger))
                        .on_press(Message::DeleteCustomCommand(idx))
                        .padding([1, 6])
                        .style(move |_t: &iced::Theme, s: button::Status| button::Style {
                            background: Some(iced::Background::Color(match s {
                                button::Status::Hovered => p.bg_hover,
                                _ => iced::Color::TRANSPARENT,
                            })),
                            text_color: p.danger,
                            border: iced::Border {
                                color: p.border,
                                width: 1.0,
                                radius: cr.into(),
                            },
                            ..Default::default()
                        }),
                ]
                .spacing(6)
                .align_y(iced::Alignment::Center);
                list_col = list_col.push(
                    container(row_content)
                        .padding([3, 6])
                        .width(Length::Fill)
                        .style(move |_t: &iced::Theme| container::Style {
                            background: Some(iced::Background::Color(p.bg_tertiary)),
                            border: iced::Border {
                                color: p.border,
                                width: 1.0,
                                radius: cr.into(),
                            },
                            ..Default::default()
                        }),
                );
            }

            // Scrollable list
            let list_scroll = scrollable(list_col)
                .height(Length::Fixed(180.0));

            // Add new command form
            let add_form = column![
                text("Add Custom Command").size(12).color(p.text_secondary),
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
                button(text("+ Add").size(11).color(p.text_primary))
                    .on_press(Message::AddCustomCommand)
                    .padding([4, 14])
                    .style(move |_t: &iced::Theme, s: button::Status| button::Style {
                        background: Some(iced::Background::Color(match s {
                            button::Status::Hovered => p.accent_hover,
                            _ => p.accent,
                        })),
                        text_color: p.text_primary,
                        border: iced::Border {
                            color: p.border,
                            width: 1.0,
                            radius: cr.into(),
                        },
                        ..Default::default()
                    }),
            ]
            .spacing(8);

            column![
                text("Custom Commands (Aliases)").size(16).color(p.text_primary),
                text("Type a trigger (e.g. -runtest) in the terminal and press Enter to execute the script.")
                    .size(10)
                    .color(p.text_muted),
                list_scroll,
                add_form,
                row![
                    dialog_button(texts.cancel, Message::CloseDialog, false, theme, cr),
                    dialog_button(texts.save, Message::SaveCustomCommands, true, theme, cr),
                ]
                .spacing(8),
            ]
            .spacing(12)
            .width(Length::Fixed(480.0))
            .into()
        }

        DialogState::SecurityAudit(findings) => {
            let findings_clone = findings.clone();
            let mut findings_col = Column::new().spacing(6);

            for finding in &findings_clone {
                let sev_color = match finding.severity {
                    SecuritySeverity::Critical => iced::Color::from_rgb8(220, 38, 38),
                    SecuritySeverity::High     => iced::Color::from_rgb8(234, 88, 12),
                    SecuritySeverity::Medium   => iced::Color::from_rgb8(202, 138, 4),
                    SecuritySeverity::Low      => iced::Color::from_rgb8(37, 99, 235),
                    SecuritySeverity::Info     => p.text_muted,
                };
                let badge_text = format!(
                    "{}  {}",
                    finding.severity.label(),
                    finding.category
                );
                let finding_row = column![
                    text(badge_text).size(9).color(sev_color),
                    text(finding.message.clone()).size(11).color(p.text_primary),
                ]
                .spacing(2);

                findings_col = findings_col.push(
                    container(finding_row)
                        .padding([6, 8])
                        .width(Length::Fill)
                        .style(move |_t: &iced::Theme| container::Style {
                            background: Some(iced::Background::Color(p.bg_tertiary)),
                            border: iced::Border {
                                color: sev_color,
                                width: 1.0,
                                radius: cr.into(),
                            },
                            ..Default::default()
                        }),
                );
            }

            let count_critical = findings_clone
                .iter()
                .filter(|f| f.severity == SecuritySeverity::Critical || f.severity == SecuritySeverity::High)
                .count();
            let summary = if count_critical == 0 {
                "No high-severity issues found.".to_string()
            } else {
                format!("{} high/critical issue(s) require attention.", count_critical)
            };
            let summary_color = if count_critical == 0 { p.success } else { p.danger };

            column![
                text("Security Audit").size(16).color(p.text_primary),
                text(summary).size(11).color(summary_color),
                scrollable(findings_col).height(Length::Fixed(340.0)),
                dialog_button(texts.cancel, Message::CloseDialog, false, theme, cr),
            ]
            .spacing(12)
            .width(Length::Fixed(500.0))
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
                    radius: cr.into(),
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
            0.0, 0.0, 0.0, 0.6,
        ))),
        ..Default::default()
    });

    card.into()
}

fn labeled_input<'a>(
    label: &'static str,
    value: &str,
    on_input: impl Fn(String) -> Message + 'static,
    theme: AppTheme,
    cr: f32,
) -> Column<'a, Message> {
    let p = theme::palette(theme);
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
                    radius: cr.into(),
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
                    radius: cr.into(),
                },
                ..Default::default()
            }
        })
        .into()
}
