use gpui_text_input::TextInputOptions;

use crate::{RgbColor, SettingsWindowTheme};

/// Options used when creating a settings window.
#[derive(Debug, Clone, PartialEq)]
pub struct SettingsWindowOptions {
    title: String,
    width: f32,
    height: f32,
    min_width: f32,
    min_height: f32,
    saved_color_swatches: Vec<RgbColor>,
    visual_theme: SettingsWindowTheme,
    text_input_undo_byte_limit: usize,
}

impl SettingsWindowOptions {
    /// Creates settings-window options with a custom title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            width: 1200.0,
            height: 860.0,
            min_width: 720.0,
            min_height: 520.0,
            saved_color_swatches: Vec::new(),
            visual_theme: SettingsWindowTheme::default(),
            text_input_undo_byte_limit: TextInputOptions::DEFAULT_UNDO_BYTE_LIMIT,
        }
    }

    /// Returns a copy with a custom initial window size in pixels.
    pub fn with_window_size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Returns a copy with a custom minimum window size in pixels.
    pub fn with_min_window_size(mut self, width: f32, height: f32) -> Self {
        self.min_width = width;
        self.min_height = height;
        self
    }

    /// Returns a copy with saved swatches offered by color picker popups.
    pub fn with_saved_color_swatches(mut self, colors: impl IntoIterator<Item = RgbColor>) -> Self {
        self.saved_color_swatches = colors.into_iter().collect();
        self
    }

    /// Returns a copy with a host-provided app-neutral visual theme.
    pub fn with_visual_theme(mut self, theme: SettingsWindowTheme) -> Self {
        self.visual_theme = theme;
        self
    }

    /// Returns a copy with a custom per-stack text-input undo byte limit.
    pub fn with_text_input_undo_byte_limit(mut self, byte_limit: usize) -> Self {
        self.text_input_undo_byte_limit = byte_limit;
        self
    }

    /// Returns the window title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the initial window size in pixels.
    pub fn window_size(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    /// Returns the minimum window size in pixels.
    pub fn min_window_size(&self) -> (f32, f32) {
        (self.min_width, self.min_height)
    }

    /// Returns host-provided saved color swatches.
    pub fn saved_color_swatches(&self) -> &[RgbColor] {
        &self.saved_color_swatches
    }

    /// Returns the visual theme used by settings-window presentation.
    pub fn visual_theme(&self) -> &SettingsWindowTheme {
        &self.visual_theme
    }

    /// Returns the per-stack text-input undo byte limit.
    pub fn text_input_undo_byte_limit(&self) -> usize {
        self.text_input_undo_byte_limit
    }
}

impl Default for SettingsWindowOptions {
    fn default() -> Self {
        Self::new("Settings")
    }
}
