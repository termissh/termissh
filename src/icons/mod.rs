use iced::widget::svg::{self, Handle, Svg};
use iced::{Color, Element, Length};

use crate::app::Message;

#[derive(Debug, Clone, Copy)]
pub enum IconName {
    Plus,
    Settings,
    Wifi,
    Terminal,
    Folder,
    FolderOpen,
    Close,
    Minimize,
    Maximize,
    Server,
    Pencil,
    Trash,
    Search,
    Send,
    ChevronUp,
    ChevronDown,
    ChevronRight,
    ChevronLeft,
    Command,
    Hash,
    History,
    Play,
    Cable,
    CircleDot,
    Circle,
    CircleX,
    SquareAsterisk,
    SquareTerminal,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Check,
    TriangleAlert,
    Info,
    Copy,
    Type,
    Languages,
    LayoutGrid,
    Moon,
    Sun,
    Power,
}

impl IconName {
    fn raw(self) -> &'static [u8] {
        match self {
            Self::Plus            => include_bytes!("svg/plus.svg"),
            Self::Settings        => include_bytes!("svg/settings.svg"),
            Self::Wifi            => include_bytes!("svg/wifi.svg"),
            Self::Terminal        => include_bytes!("svg/terminal.svg"),
            Self::Folder          => include_bytes!("svg/folder.svg"),
            Self::FolderOpen      => include_bytes!("svg/folder-open.svg"),
            Self::Close           => include_bytes!("svg/x.svg"),
            Self::Minimize        => include_bytes!("svg/minus.svg"),
            Self::Maximize        => include_bytes!("svg/square.svg"),
            Self::Server          => include_bytes!("svg/server.svg"),
            Self::Pencil          => include_bytes!("svg/pencil.svg"),
            Self::Trash           => include_bytes!("svg/trash-2.svg"),
            Self::Search          => include_bytes!("svg/search.svg"),
            Self::Send            => include_bytes!("svg/send.svg"),
            Self::ChevronUp       => include_bytes!("svg/chevron-up.svg"),
            Self::ChevronDown     => include_bytes!("svg/chevron-down.svg"),
            Self::ChevronRight    => include_bytes!("svg/chevron-right.svg"),
            Self::ChevronLeft     => include_bytes!("svg/chevron-left.svg"),
            Self::Command         => include_bytes!("svg/command.svg"),
            Self::Hash            => include_bytes!("svg/hash.svg"),
            Self::History         => include_bytes!("svg/history.svg"),
            Self::Play            => include_bytes!("svg/play.svg"),
            Self::Cable           => include_bytes!("svg/cable.svg"),
            Self::CircleDot       => include_bytes!("svg/circle-dot.svg"),
            Self::Circle          => include_bytes!("svg/circle.svg"),
            Self::CircleX         => include_bytes!("svg/circle-x.svg"),
            Self::SquareAsterisk  => include_bytes!("svg/square-asterisk.svg"),
            Self::SquareTerminal  => include_bytes!("svg/square-terminal.svg"),
            Self::ArrowRight      => include_bytes!("svg/arrow-right.svg"),
            Self::ArrowUp         => include_bytes!("svg/arrow-up.svg"),
            Self::ArrowDown       => include_bytes!("svg/arrow-down.svg"),
            Self::Check           => include_bytes!("svg/check.svg"),
            Self::TriangleAlert   => include_bytes!("svg/triangle-alert.svg"),
            Self::Info            => include_bytes!("svg/info.svg"),
            Self::Copy            => include_bytes!("svg/copy.svg"),
            Self::Type            => include_bytes!("svg/type.svg"),
            Self::Languages       => include_bytes!("svg/languages.svg"),
            Self::LayoutGrid      => include_bytes!("svg/layout-grid.svg"),
            Self::Moon            => include_bytes!("svg/moon.svg"),
            Self::Sun             => include_bytes!("svg/sun.svg"),
            Self::Power           => include_bytes!("svg/power.svg"),
        }
    }

    fn handle(self) -> Handle {
        Handle::from_memory(self.raw())
    }

    pub fn raw_svg(self) -> Svg<'static> {
        Svg::new(self.handle())
    }
}

pub fn icon(name: IconName, size: f32, color: Color) -> Element<'static, Message> {
    name.raw_svg()
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(move |_t: &iced::Theme, _s: svg::Status| svg::Style { color: Some(color) })
        .into()
}
