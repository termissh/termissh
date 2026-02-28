use iced::Color;

use crate::config::{AppTheme, LayoutPreset};

pub const SIDEBAR_WIDTH: f32 = 200.0;
pub const PANEL_GAP: f32 = 5.0;
pub const CORNER_RADIUS: f32 = 6.0;

#[derive(Clone, Copy, Debug)]
pub struct LayoutConfig {
    pub corner_radius: f32,
    pub panel_gap: f32,
    pub sidebar_width: f32,
    pub container_padding: u16,
    pub element_padding: u16,
    pub spacing: f32,
}

pub fn layout(preset: LayoutPreset) -> LayoutConfig {
    match preset {
        LayoutPreset::Vega => LayoutConfig {
            corner_radius: 6.0,
            panel_gap: 5.0,
            sidebar_width: 200.0,
            container_padding: 8,
            element_padding: 8,
            spacing: 5.0,
        },
        LayoutPreset::Nova => LayoutConfig {
            corner_radius: 4.0,
            panel_gap: 3.0,
            sidebar_width: 178.0,
            container_padding: 5,
            element_padding: 5,
            spacing: 3.0,
        },
        LayoutPreset::Maia => LayoutConfig {
            corner_radius: 14.0,
            panel_gap: 8.0,
            sidebar_width: 220.0,
            container_padding: 14,
            element_padding: 12,
            spacing: 8.0,
        },
        LayoutPreset::Lyra => LayoutConfig {
            corner_radius: 0.0,
            panel_gap: 4.0,
            sidebar_width: 190.0,
            container_padding: 8,
            element_padding: 8,
            spacing: 4.0,
        },
        LayoutPreset::Mira => LayoutConfig {
            corner_radius: 2.0,
            panel_gap: 2.0,
            sidebar_width: 160.0,
            container_padding: 3,
            element_padding: 3,
            spacing: 2.0,
        },
    }
}

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

pub fn palette(theme: AppTheme) -> Palette {
    match theme {
        AppTheme::Dark => Palette {
            // GitHub-inspired clean dark
            bg_primary:     Color::from_rgb8(13,  17,  23),
            bg_secondary:   Color::from_rgb8(22,  27,  34),
            bg_tertiary:    Color::from_rgb8(33,  38,  45),
            bg_hover:       Color::from_rgb8(48,  54,  61),
            bg_active:      Color::from_rgb8(33,  81, 140),
            text_primary:   Color::from_rgb8(230, 237, 243),
            text_secondary: Color::from_rgb8(139, 148, 158),
            text_muted:     Color::from_rgb8(80,  88,  96),
            accent:         Color::from_rgb8(88,  166, 255),
            accent_hover:   Color::from_rgb8(121, 184, 255),
            success:        Color::from_rgb8(63,  185, 80),
            warning:        Color::from_rgb8(210, 153, 34),
            danger:         Color::from_rgb8(248, 81,  73),
            border:         Color::from_rgb8(48,  54,  61),
            border_focused: Color::from_rgb8(88,  166, 255),
        },

        AppTheme::Light => Palette {
            // Clean neutral light
            bg_primary:     Color::from_rgb8(255, 255, 255),
            bg_secondary:   Color::from_rgb8(246, 248, 250),
            bg_tertiary:    Color::from_rgb8(234, 238, 242),
            bg_hover:       Color::from_rgb8(216, 222, 228),
            bg_active:      Color::from_rgb8(197, 220, 247),
            text_primary:   Color::from_rgb8(31,  35,  40),
            text_secondary: Color::from_rgb8(101, 109, 118),
            text_muted:     Color::from_rgb8(145, 152, 161),
            accent:         Color::from_rgb8(9,   105, 218),
            accent_hover:   Color::from_rgb8(31,  122, 228),
            success:        Color::from_rgb8(31,  136, 61),
            warning:        Color::from_rgb8(154, 103, 0),
            danger:         Color::from_rgb8(207, 34,  46),
            border:         Color::from_rgb8(209, 217, 228),
            border_focused: Color::from_rgb8(9,   105, 218),
        },

        AppTheme::Dracula => Palette {
            // Classic Dracula
            bg_primary:     Color::from_rgb8(24,  25,  36),
            bg_secondary:   Color::from_rgb8(40,  42,  54),
            bg_tertiary:    Color::from_rgb8(52,  55,  70),
            bg_hover:       Color::from_rgb8(68,  71,  90),
            bg_active:      Color::from_rgb8(80,  60, 120),
            text_primary:   Color::from_rgb8(248, 248, 242),
            text_secondary: Color::from_rgb8(189, 147, 249),
            text_muted:     Color::from_rgb8(98,  114, 164),
            accent:         Color::from_rgb8(189, 147, 249),
            accent_hover:   Color::from_rgb8(203, 166, 247),
            success:        Color::from_rgb8(80,  250, 123),
            warning:        Color::from_rgb8(241, 250, 140),
            danger:         Color::from_rgb8(255, 85,  85),
            border:         Color::from_rgb8(68,  71,  90),
            border_focused: Color::from_rgb8(189, 147, 249),
        },

        AppTheme::Nord => Palette {
            // Arctic Nord
            bg_primary:     Color::from_rgb8(36,  41,  51),
            bg_secondary:   Color::from_rgb8(46,  52,  64),
            bg_tertiary:    Color::from_rgb8(59,  66,  82),
            bg_hover:       Color::from_rgb8(67,  76,  94),
            bg_active:      Color::from_rgb8(76,  86, 106),
            text_primary:   Color::from_rgb8(236, 239, 244),
            text_secondary: Color::from_rgb8(216, 222, 233),
            text_muted:     Color::from_rgb8(129, 161, 193),
            accent:         Color::from_rgb8(136, 192, 208),
            accent_hover:   Color::from_rgb8(143, 199, 216),
            success:        Color::from_rgb8(163, 190, 140),
            warning:        Color::from_rgb8(235, 203, 139),
            danger:         Color::from_rgb8(191, 97,  106),
            border:         Color::from_rgb8(67,  76,  94),
            border_focused: Color::from_rgb8(136, 192, 208),
        },

        AppTheme::Solarized => Palette {
            // Solarized Dark
            bg_primary:     Color::from_rgb8(0,   43,  54),
            bg_secondary:   Color::from_rgb8(7,   54,  66),
            bg_tertiary:    Color::from_rgb8(0,   50,  62),
            bg_hover:       Color::from_rgb8(20,  72,  86),
            bg_active:      Color::from_rgb8(0,   75,  95),
            text_primary:   Color::from_rgb8(147, 161, 161),
            text_secondary: Color::from_rgb8(101, 123, 131),
            text_muted:     Color::from_rgb8(88,  110, 117),
            accent:         Color::from_rgb8(38,  139, 210),
            accent_hover:   Color::from_rgb8(63,  153, 218),
            success:        Color::from_rgb8(133, 153, 0),
            warning:        Color::from_rgb8(181, 137, 0),
            danger:         Color::from_rgb8(220, 50,  47),
            border:         Color::from_rgb8(7,   54,  66),
            border_focused: Color::from_rgb8(38,  139, 210),
        },

        AppTheme::MonoDark => Palette {
            // Pure monochrome dark
            bg_primary:     Color::from_rgb8(0,   0,   0),
            bg_secondary:   Color::from_rgb8(14,  14,  14),
            bg_tertiary:    Color::from_rgb8(24,  24,  24),
            bg_hover:       Color::from_rgb8(36,  36,  36),
            bg_active:      Color::from_rgb8(52,  52,  52),
            text_primary:   Color::from_rgb8(255, 255, 255),
            text_secondary: Color::from_rgb8(180, 180, 180),
            text_muted:     Color::from_rgb8(100, 100, 100),
            accent:         Color::from_rgb8(220, 220, 220),
            accent_hover:   Color::from_rgb8(255, 255, 255),
            success:        Color::from_rgb8(160, 160, 160),
            warning:        Color::from_rgb8(200, 200, 200),
            danger:         Color::from_rgb8(130, 130, 130),
            border:         Color::from_rgb8(42,  42,  42),
            border_focused: Color::from_rgb8(180, 180, 180),
        },

        AppTheme::MonoLight => Palette {
            // Pure monochrome light
            bg_primary:     Color::from_rgb8(255, 255, 255),
            bg_secondary:   Color::from_rgb8(242, 242, 242),
            bg_tertiary:    Color::from_rgb8(226, 226, 226),
            bg_hover:       Color::from_rgb8(208, 208, 208),
            bg_active:      Color::from_rgb8(188, 188, 188),
            text_primary:   Color::from_rgb8(0,   0,   0),
            text_secondary: Color::from_rgb8(75,  75,  75),
            text_muted:     Color::from_rgb8(150, 150, 150),
            accent:         Color::from_rgb8(30,  30,  30),
            accent_hover:   Color::from_rgb8(0,   0,   0),
            success:        Color::from_rgb8(70,  70,  70),
            warning:        Color::from_rgb8(100, 100, 100),
            danger:         Color::from_rgb8(50,  50,  50),
            border:         Color::from_rgb8(200, 200, 200),
            border_focused: Color::from_rgb8(30,  30,  30),
        },

        AppTheme::Haki => Palette {
            // Olive / military green
            bg_primary:     Color::from_rgb8(16,  18,  12),
            bg_secondary:   Color::from_rgb8(24,  27,  18),
            bg_tertiary:    Color::from_rgb8(34,  38,  26),
            bg_hover:       Color::from_rgb8(48,  54,  36),
            bg_active:      Color::from_rgb8(68,  78,  50),
            text_primary:   Color::from_rgb8(206, 198, 168),
            text_secondary: Color::from_rgb8(152, 144, 114),
            text_muted:     Color::from_rgb8(106, 100, 76),
            accent:         Color::from_rgb8(122, 150, 82),
            accent_hover:   Color::from_rgb8(142, 172, 98),
            success:        Color::from_rgb8(128, 160, 86),
            warning:        Color::from_rgb8(192, 160, 88),
            danger:         Color::from_rgb8(182, 98,  86),
            border:         Color::from_rgb8(64,  72,  48),
            border_focused: Color::from_rgb8(142, 172, 98),
        },

        AppTheme::SoftRose => Palette {
            // Dusty rose / warm pink
            bg_primary:     Color::from_rgb8(20,  14,  18),
            bg_secondary:   Color::from_rgb8(30,  20,  26),
            bg_tertiary:    Color::from_rgb8(44,  30,  38),
            bg_hover:       Color::from_rgb8(60,  42,  54),
            bg_active:      Color::from_rgb8(84,  58,  74),
            text_primary:   Color::from_rgb8(240, 218, 228),
            text_secondary: Color::from_rgb8(180, 138, 158),
            text_muted:     Color::from_rgb8(128, 90,  108),
            accent:         Color::from_rgb8(218, 126, 158),
            accent_hover:   Color::from_rgb8(234, 148, 174),
            success:        Color::from_rgb8(158, 192, 130),
            warning:        Color::from_rgb8(212, 174, 110),
            danger:         Color::from_rgb8(224, 100, 110),
            border:         Color::from_rgb8(72,  48,  62),
            border_focused: Color::from_rgb8(218, 126, 158),
        },

        AppTheme::SoftSky => Palette {
            // Soft cyan / sky blue
            bg_primary:     Color::from_rgb8(10,  18,  28),
            bg_secondary:   Color::from_rgb8(16,  28,  44),
            bg_tertiary:    Color::from_rgb8(24,  42,  62),
            bg_hover:       Color::from_rgb8(36,  60,  86),
            bg_active:      Color::from_rgb8(50,  84, 116),
            text_primary:   Color::from_rgb8(208, 228, 244),
            text_secondary: Color::from_rgb8(138, 180, 212),
            text_muted:     Color::from_rgb8(86,  130, 166),
            accent:         Color::from_rgb8(82,  198, 230),
            accent_hover:   Color::from_rgb8(108, 212, 240),
            success:        Color::from_rgb8(96,  200, 158),
            warning:        Color::from_rgb8(210, 178, 98),
            danger:         Color::from_rgb8(212, 108, 108),
            border:         Color::from_rgb8(38,  64,  92),
            border_focused: Color::from_rgb8(82,  198, 230),
        },
    }
}
