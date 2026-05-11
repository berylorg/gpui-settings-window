use crate::RgbColor;

/// App-neutral visual theme for the reusable settings window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsWindowTheme {
    pub window_background: RgbColor,
    pub panel: SettingsSurfaceTheme,
    pub row: SettingsSurfaceTheme,
    pub popup: SettingsSurfaceTheme,
    pub input: SettingsInputTheme,
    pub navigation_button: SettingsButtonTheme,
    pub primary_button: SettingsButtonTheme,
    pub secondary_button: SettingsButtonTheme,
}

/// Shared surface colors for settings-window panels, rows, and popups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsSurfaceTheme {
    pub background: RgbColor,
    pub border: RgbColor,
    pub foreground: RgbColor,
    pub muted_foreground: RgbColor,
}

/// Input colors used by text and color fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsInputTheme {
    pub background: RgbColor,
    pub border: RgbColor,
    pub active_border: RgbColor,
    pub error_border: RgbColor,
    pub foreground: RgbColor,
    pub caret: RgbColor,
    pub selection_background: RgbColor,
}

/// Button colors for all interactive states of one button variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsButtonTheme {
    pub normal: SettingsButtonStateTheme,
    pub hover: SettingsButtonStateTheme,
    pub active: SettingsButtonStateTheme,
    pub disabled: SettingsButtonStateTheme,
}

/// Background, border, and foreground colors for one button state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsButtonStateTheme {
    pub background: RgbColor,
    pub border: RgbColor,
    pub foreground: RgbColor,
}

impl Default for SettingsWindowTheme {
    fn default() -> Self {
        let text = color(0xE7E3D8);
        let muted = color(0x8D959C);
        let panel_border = color(0x31363B);
        let active_border = color(0x49966F);
        Self {
            window_background: color(0x171819),
            panel: SettingsSurfaceTheme::new(color(0x111214), panel_border, text, muted),
            row: SettingsSurfaceTheme::new(color(0x1D2125), panel_border, text, muted),
            popup: SettingsSurfaceTheme::new(color(0x191C1F), color(0x4B535C), text, muted),
            input: SettingsInputTheme {
                background: color(0x090A0B),
                border: panel_border,
                active_border,
                error_border: color(0xD05F5F),
                foreground: text,
                caret: color(0xF3F7F4),
                selection_background: active_border,
            },
            navigation_button: SettingsButtonTheme::secondary(),
            primary_button: SettingsButtonTheme::primary(),
            secondary_button: SettingsButtonTheme::secondary(),
        }
    }
}

impl SettingsSurfaceTheme {
    pub const fn new(
        background: RgbColor,
        border: RgbColor,
        foreground: RgbColor,
        muted_foreground: RgbColor,
    ) -> Self {
        Self {
            background,
            border,
            foreground,
            muted_foreground,
        }
    }
}

impl SettingsButtonTheme {
    fn primary() -> Self {
        Self {
            normal: SettingsButtonStateTheme::new(
                color(0x2D6A4F),
                color(0x49966F),
                color(0xE7E3D8),
            ),
            hover: SettingsButtonStateTheme::new(color(0x347A5A), color(0x55A77C), color(0xF3F7F4)),
            active: SettingsButtonStateTheme::new(
                color(0x24583F),
                color(0x3F805F),
                color(0xF3F7F4),
            ),
            disabled: SettingsButtonStateTheme::new(
                color(0x252A2F),
                color(0x3B434B),
                color(0x8D959C),
            ),
        }
    }

    fn secondary() -> Self {
        Self {
            normal: SettingsButtonStateTheme::new(
                color(0x252A2F),
                color(0x3B434B),
                color(0xE7E3D8),
            ),
            hover: SettingsButtonStateTheme::new(color(0x303740), color(0x4A5560), color(0xF3F7F4)),
            active: SettingsButtonStateTheme::new(
                color(0x1F2429),
                color(0x313942),
                color(0xF3F7F4),
            ),
            disabled: SettingsButtonStateTheme::new(
                color(0x1B1F24),
                color(0x2B3137),
                color(0x8D959C),
            ),
        }
    }
}

impl SettingsButtonStateTheme {
    pub const fn new(background: RgbColor, border: RgbColor, foreground: RgbColor) -> Self {
        Self {
            background,
            border,
            foreground,
        }
    }
}

const fn color(packed: u32) -> RgbColor {
    RgbColor::new(
        ((packed >> 16) & 0xFF) as u8,
        ((packed >> 8) & 0xFF) as u8,
        (packed & 0xFF) as u8,
    )
}
