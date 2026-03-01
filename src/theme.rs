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
        // New presets
        LayoutPreset::Zeta => LayoutConfig {
            // Wide sidebar, card-like spacious panels
            corner_radius: 8.0,
            panel_gap: 6.0,
            sidebar_width: 240.0,
            container_padding: 12,
            element_padding: 10,
            spacing: 6.0,
        },
        LayoutPreset::Orion => LayoutConfig {
            // Terminal-first: narrow sidebar, max space for terminal
            corner_radius: 5.0,
            panel_gap: 3.0,
            sidebar_width: 150.0,
            container_padding: 4,
            element_padding: 4,
            spacing: 3.0,
        },
        LayoutPreset::Aria => LayoutConfig {
            // Balanced, symmetrical, clean
            corner_radius: 10.0,
            panel_gap: 6.0,
            sidebar_width: 210.0,
            container_padding: 10,
            element_padding: 10,
            spacing: 6.0,
        },
        LayoutPreset::Dawn => LayoutConfig {
            // Extra rounded, airy, bubble-like
            corner_radius: 20.0,
            panel_gap: 10.0,
            sidebar_width: 215.0,
            container_padding: 16,
            element_padding: 14,
            spacing: 10.0,
        },
        LayoutPreset::Flux => LayoutConfig {
            corner_radius: 12.0,
            panel_gap: 12.0,
            sidebar_width: 205.0,
            container_padding: 12,
            element_padding: 8,
            spacing: 8.0,
        },
        LayoutPreset::NoRound => LayoutConfig {
            corner_radius: 0.0,
            panel_gap: 0.0,
            sidebar_width: 205.0,
            container_padding: 0,
            element_padding: 0,
            spacing: 0.0,
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
            // Deep Neutral Dark – Soft Contrast, Premium Feel
            bg_primary:     Color::from_rgb8(9,  11,  15),   // Daha derin ama siyah değil
            bg_secondary:   Color::from_rgb8(14, 17, 22),
            bg_tertiary:    Color::from_rgb8(20, 24, 30),
            bg_hover:       Color::from_rgb8(28, 34, 42),
            bg_active:      Color::from_rgb8(40, 65, 105),   // Patlamayan koyu mavi

            text_primary:   Color::from_rgb8(228, 232, 240),
            text_secondary: Color::from_rgb8(160, 168, 178),
            text_muted:     Color::from_rgb8(105, 115, 125),

            accent:         Color::from_rgb8(102, 168, 255), // Daha soft mavi
            accent_hover:   Color::from_rgb8(125, 185, 255),

            success:        Color::from_rgb8(46, 160, 67),   // Daha natural green
            warning:        Color::from_rgb8(201, 140, 0),
            danger:         Color::from_rgb8(218, 54, 51),

            border:         Color::from_rgb8(32, 38, 46),
            border_focused: Color::from_rgb8(102, 168, 255),
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

        AppTheme::CyberPunk => Palette {
            // Neon magenta / cyan on deep dark purple
            bg_primary:     Color::from_rgb8(8,   4,   20),
            bg_secondary:   Color::from_rgb8(14,  8,   32),
            bg_tertiary:    Color::from_rgb8(24,  14,  50),
            bg_hover:       Color::from_rgb8(40,  20,  72),
            bg_active:      Color::from_rgb8(72,  0,   120),
            text_primary:   Color::from_rgb8(240, 228, 255),
            text_secondary: Color::from_rgb8(190, 140, 255),
            text_muted:     Color::from_rgb8(120, 70,  170),
            accent:         Color::from_rgb8(255, 20,  200),
            accent_hover:   Color::from_rgb8(255, 90,  220),
            success:        Color::from_rgb8(0,   255, 180),
            warning:        Color::from_rgb8(255, 210, 0),
            danger:         Color::from_rgb8(255, 40,  80),
            border:         Color::from_rgb8(80,  10,  140),
            border_focused: Color::from_rgb8(255, 20,  200),
        },

        AppTheme::Mocha => Palette {
            // Catppuccin Mocha inspired – warm, purple-tinted dark
            bg_primary:     Color::from_rgb8(30,  30,  46),
            bg_secondary:   Color::from_rgb8(36,  36,  54),
            bg_tertiary:    Color::from_rgb8(49,  50,  68),
            bg_hover:       Color::from_rgb8(58,  60,  82),
            bg_active:      Color::from_rgb8(69,  71,  96),
            text_primary:   Color::from_rgb8(205, 214, 244),
            text_secondary: Color::from_rgb8(166, 173, 200),
            text_muted:     Color::from_rgb8(108, 112, 134),
            accent:         Color::from_rgb8(203, 166, 247),
            accent_hover:   Color::from_rgb8(216, 190, 250),
            success:        Color::from_rgb8(166, 227, 161),
            warning:        Color::from_rgb8(249, 226, 175),
            danger:         Color::from_rgb8(243, 139, 168),
            border:         Color::from_rgb8(88,  91,  112),
            border_focused: Color::from_rgb8(203, 166, 247),
        },

        AppTheme::Ocean => Palette {
            // Deep ocean – dark teal & azure
            bg_primary:     Color::from_rgb8(4,   18,  36),
            bg_secondary:   Color::from_rgb8(6,   28,  52),
            bg_tertiary:    Color::from_rgb8(10,  42,  70),
            bg_hover:       Color::from_rgb8(14,  58,  92),
            bg_active:      Color::from_rgb8(18,  78, 118),
            text_primary:   Color::from_rgb8(196, 224, 242),
            text_secondary: Color::from_rgb8(128, 178, 210),
            text_muted:     Color::from_rgb8(68,  118, 158),
            accent:         Color::from_rgb8(0,   178, 202),
            accent_hover:   Color::from_rgb8(30,  198, 222),
            success:        Color::from_rgb8(58,  198, 148),
            warning:        Color::from_rgb8(198, 168, 78),
            danger:         Color::from_rgb8(198, 78,  88),
            border:         Color::from_rgb8(14,  58,  98),
            border_focused: Color::from_rgb8(0,   178, 202),
        },

        AppTheme::Forest => Palette {
            // Earthy woodland greens
            bg_primary:     Color::from_rgb8(8,   16,  8),
            bg_secondary:   Color::from_rgb8(16,  26,  16),
            bg_tertiary:    Color::from_rgb8(26,  40,  26),
            bg_hover:       Color::from_rgb8(38,  58,  36),
            bg_active:      Color::from_rgb8(54,  82,  52),
            text_primary:   Color::from_rgb8(210, 222, 188),
            text_secondary: Color::from_rgb8(158, 180, 136),
            text_muted:     Color::from_rgb8(108, 128, 88),
            accent:         Color::from_rgb8(100, 182, 88),
            accent_hover:   Color::from_rgb8(122, 202, 108),
            success:        Color::from_rgb8(118, 192, 98),
            warning:        Color::from_rgb8(198, 180, 88),
            danger:         Color::from_rgb8(188, 88,  78),
            border:         Color::from_rgb8(50,  72,  48),
            border_focused: Color::from_rgb8(100, 182, 88),
        },

        AppTheme::Gruvbox => Palette {
            // Warm retro dark — classic Gruvbox hard
            bg_primary:     Color::from_rgb8(29,  32,  33),
            bg_secondary:   Color::from_rgb8(40,  40,  40),
            bg_tertiary:    Color::from_rgb8(50,  48,  47),
            bg_hover:       Color::from_rgb8(60,  56,  54),
            bg_active:      Color::from_rgb8(80,  73,  69),
            text_primary:   Color::from_rgb8(235, 219, 178),
            text_secondary: Color::from_rgb8(189, 174, 147),
            text_muted:     Color::from_rgb8(124, 111, 100),
            accent:         Color::from_rgb8(250, 189, 47),
            accent_hover:   Color::from_rgb8(253, 200, 80),
            success:        Color::from_rgb8(152, 151, 26),
            warning:        Color::from_rgb8(214, 93,  14),
            danger:         Color::from_rgb8(204, 36,  29),
            border:         Color::from_rgb8(72,  65,  54),
            border_focused: Color::from_rgb8(250, 189, 47),
        },

        AppTheme::TokyoNight => Palette {
            // Tokyo Night — deep blue-purple dark
            bg_primary:     Color::from_rgb8(26,  27,  38),
            bg_secondary:   Color::from_rgb8(31,  35,  53),
            bg_tertiary:    Color::from_rgb8(41,  46,  66),
            bg_hover:       Color::from_rgb8(52,  59,  88),
            bg_active:      Color::from_rgb8(65,  72, 104),
            text_primary:   Color::from_rgb8(192, 202, 245),
            text_secondary: Color::from_rgb8(131, 148, 191),
            text_muted:     Color::from_rgb8(82,  95, 138),
            accent:         Color::from_rgb8(122, 162, 247),
            accent_hover:   Color::from_rgb8(148, 182, 255),
            success:        Color::from_rgb8(158, 206, 106),
            warning:        Color::from_rgb8(224, 175, 104),
            danger:         Color::from_rgb8(247, 118, 142),
            border:         Color::from_rgb8(54,  59,  88),
            border_focused: Color::from_rgb8(122, 162, 247),
        },

        AppTheme::OneDark => Palette {
            // Atom One Dark — balanced blue-grey
            bg_primary:     Color::from_rgb8(24,  26,  31),
            bg_secondary:   Color::from_rgb8(30,  33,  39),
            bg_tertiary:    Color::from_rgb8(38,  42,  50),
            bg_hover:       Color::from_rgb8(50,  55,  66),
            bg_active:      Color::from_rgb8(65,  71,  87),
            text_primary:   Color::from_rgb8(171, 178, 191),
            text_secondary: Color::from_rgb8(130, 137, 151),
            text_muted:     Color::from_rgb8(90,  96,  109),
            accent:         Color::from_rgb8(97,  175, 239),
            accent_hover:   Color::from_rgb8(120, 190, 248),
            success:        Color::from_rgb8(152, 195, 121),
            warning:        Color::from_rgb8(229, 192, 123),
            danger:         Color::from_rgb8(224, 108, 117),
            border:         Color::from_rgb8(56,  62,  75),
            border_focused: Color::from_rgb8(97,  175, 239),
        },

        AppTheme::Ayu => Palette {
            // Ayu Dark — burnt orange accent, dark charcoal
            bg_primary:     Color::from_rgb8(13,  16,  23),
            bg_secondary:   Color::from_rgb8(15,  20,  30),
            bg_tertiary:    Color::from_rgb8(22,  30,  45),
            bg_hover:       Color::from_rgb8(30,  44,  62),
            bg_active:      Color::from_rgb8(42,  60,  84),
            text_primary:   Color::from_rgb8(203, 204, 198),
            text_secondary: Color::from_rgb8(147, 152, 160),
            text_muted:     Color::from_rgb8(90,  96,  108),
            accent:         Color::from_rgb8(255, 154, 63),
            accent_hover:   Color::from_rgb8(255, 175, 96),
            success:        Color::from_rgb8(149, 199, 89),
            warning:        Color::from_rgb8(230, 177, 62),
            danger:         Color::from_rgb8(240, 91,  88),
            border:         Color::from_rgb8(36,  52,  72),
            border_focused: Color::from_rgb8(255, 154, 63),
        },

        AppTheme::Rosepine => Palette {
            // Rosé Pine — dusty rose on deep plum
            bg_primary:     Color::from_rgb8(25,  23,  36),
            bg_secondary:   Color::from_rgb8(31,  29,  46),
            bg_tertiary:    Color::from_rgb8(38,  35,  58),
            bg_hover:       Color::from_rgb8(50,  47,  76),
            bg_active:      Color::from_rgb8(68,  64, 104),
            text_primary:   Color::from_rgb8(224, 222, 244),
            text_secondary: Color::from_rgb8(144, 140, 170),
            text_muted:     Color::from_rgb8(110, 106, 134),
            accent:         Color::from_rgb8(235, 188, 186),
            accent_hover:   Color::from_rgb8(244, 206, 204),
            success:        Color::from_rgb8(156, 207, 216),
            warning:        Color::from_rgb8(246, 193, 119),
            danger:         Color::from_rgb8(235, 111, 146),
            border:         Color::from_rgb8(68,  64,  98),
            border_focused: Color::from_rgb8(196, 167, 231),
        },

        AppTheme::Kanagawa => Palette {
            // Kanagawa — Japanese wave blues & greens
            bg_primary:     Color::from_rgb8(22,  22,  30),
            bg_secondary:   Color::from_rgb8(28,  28,  38),
            bg_tertiary:    Color::from_rgb8(38,  38,  54),
            bg_hover:       Color::from_rgb8(50,  50,  70),
            bg_active:      Color::from_rgb8(66,  66,  94),
            text_primary:   Color::from_rgb8(220, 215, 186),
            text_secondary: Color::from_rgb8(150, 160, 162),
            text_muted:     Color::from_rgb8(95,  105, 108),
            accent:         Color::from_rgb8(126, 156, 216),
            accent_hover:   Color::from_rgb8(152, 178, 232),
            success:        Color::from_rgb8(106, 153, 85),
            warning:        Color::from_rgb8(192, 153, 86),
            danger:         Color::from_rgb8(192, 87,  78),
            border:         Color::from_rgb8(60,  60,  82),
            border_focused: Color::from_rgb8(126, 156, 216),
        },

        AppTheme::Everforest => Palette {
            // Everforest — muted sage green & warm ivory
            bg_primary:     Color::from_rgb8(29,  32,  30),
            bg_secondary:   Color::from_rgb8(36,  41,  37),
            bg_tertiary:    Color::from_rgb8(48,  56,  48),
            bg_hover:       Color::from_rgb8(60,  68,  60),
            bg_active:      Color::from_rgb8(76,  88,  76),
            text_primary:   Color::from_rgb8(211, 198, 170),
            text_secondary: Color::from_rgb8(157, 157, 139),
            text_muted:     Color::from_rgb8(115, 121, 97),
            accent:         Color::from_rgb8(131, 165, 152),
            accent_hover:   Color::from_rgb8(152, 187, 174),
            success:        Color::from_rgb8(167, 192, 128),
            warning:        Color::from_rgb8(219, 188, 127),
            danger:         Color::from_rgb8(230, 126, 128),
            border:         Color::from_rgb8(68,  80,  68),
            border_focused: Color::from_rgb8(131, 165, 152),
        },

        AppTheme::Midnight => Palette {
            // Midnight — pure deep blue-black, electric blue accent
            bg_primary:     Color::from_rgb8(4,   6,   18),
            bg_secondary:   Color::from_rgb8(8,   12,  30),
            bg_tertiary:    Color::from_rgb8(14,  20,  46),
            bg_hover:       Color::from_rgb8(20,  30,  64),
            bg_active:      Color::from_rgb8(28,  44,  88),
            text_primary:   Color::from_rgb8(200, 212, 240),
            text_secondary: Color::from_rgb8(130, 150, 196),
            text_muted:     Color::from_rgb8(74,  92,  142),
            accent:         Color::from_rgb8(60,  140, 255),
            accent_hover:   Color::from_rgb8(90,  165, 255),
            success:        Color::from_rgb8(60,  200, 140),
            warning:        Color::from_rgb8(200, 170, 60),
            danger:         Color::from_rgb8(220, 60,  80),
            border:         Color::from_rgb8(24,  36,  72),
            border_focused: Color::from_rgb8(60,  140, 255),
        },
    }
}
