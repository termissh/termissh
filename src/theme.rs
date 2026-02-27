use iced::Color;

pub const SIDEBAR_WIDTH: f32 = 290.0;
pub const PANEL_GAP: f32 = 10.0;
pub const CORNER_RADIUS: f32 = 2.0;

#[derive(Clone, Copy)]
pub struct Palette {
    pub bg_primary: Color,
    pub bg_secondary: Color,
    pub bg_tertiary: Color,
    pub bg_hover: Color,
    pub bg_active: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub border: Color,
    pub border_focused: Color,
}

pub fn palette(dark_mode: bool) -> Palette {
    if dark_mode {
        Palette {
            bg_primary: Color::from_rgb8(24, 25, 21),
            bg_secondary: Color::from_rgb8(31, 33, 27),
            bg_tertiary: Color::from_rgb8(40, 43, 35),
            bg_hover: Color::from_rgb8(52, 57, 45),
            bg_active: Color::from_rgb8(66, 77, 51),
            text_primary: Color::from_rgb8(232, 227, 214),
            text_secondary: Color::from_rgb8(176, 170, 150),
            text_muted: Color::from_rgb8(134, 130, 115),
            accent: Color::from_rgb8(136, 150, 88),
            accent_hover: Color::from_rgb8(154, 170, 101),
            success: Color::from_rgb8(145, 171, 95),
            warning: Color::from_rgb8(201, 172, 104),
            danger: Color::from_rgb8(190, 116, 104),
            border: Color::from_rgb8(79, 83, 71),
            border_focused: Color::from_rgb8(154, 170, 101),
        }
    } else {
        Palette {
            bg_primary: Color::from_rgb8(246, 243, 235),
            bg_secondary: Color::from_rgb8(239, 233, 220),
            bg_tertiary: Color::from_rgb8(229, 220, 201),
            bg_hover: Color::from_rgb8(216, 209, 186),
            bg_active: Color::from_rgb8(191, 197, 157),
            text_primary: Color::from_rgb8(33, 34, 27),
            text_secondary: Color::from_rgb8(78, 76, 63),
            text_muted: Color::from_rgb8(117, 113, 95),
            accent: Color::from_rgb8(129, 139, 88),
            accent_hover: Color::from_rgb8(146, 157, 102),
            success: Color::from_rgb8(114, 141, 74),
            warning: Color::from_rgb8(173, 143, 78),
            danger: Color::from_rgb8(170, 98, 86),
            border: Color::from_rgb8(177, 170, 151),
            border_focused: Color::from_rgb8(129, 139, 88),
        }
    }
}
