mod api;
mod app;
mod config;
mod i18n;
mod terminal;
mod theme;
mod ui;

use app::App;
use iced::Font;

fn ui_font() -> Font {
    Font {
        family: iced::font::Family::Name("Segoe UI"),
        weight: iced::font::Weight::Light,
        ..Font::DEFAULT
    }
}

fn app_icon() -> Option<iced::window::Icon> {
    iced::window::icon::from_file_data(include_bytes!("icons/mini-icon.png"), None).ok()
}

fn main() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .window(iced::window::Settings {
            icon: app_icon(),
            ..Default::default()
        })
        .default_font(ui_font())
        .theme(App::theme)
        .subscription(App::subscription)
        .run_with(App::new)
}
