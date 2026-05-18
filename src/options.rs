use std::{fmt, sync::Arc};

use gpui::AnyElement;
use gpui_text_input::TextInputOptions;

use crate::{RgbColor, SettingsPageCustomBodyId, SettingsWindowTheme};

const DEFAULT_WINDOW_WIDTH: f32 = 800.0;
const DEFAULT_WINDOW_HEIGHT: f32 = 520.0;

/// App-neutral renderer for host-owned page custom body regions.
#[derive(Clone)]
pub struct SettingsPageBodyRenderer(Arc<dyn Fn(&SettingsPageCustomBodyId) -> Option<AnyElement>>);

impl SettingsPageBodyRenderer {
    /// Creates a page body renderer from a host callback.
    pub fn new(render: impl Fn(&SettingsPageCustomBodyId) -> Option<AnyElement> + 'static) -> Self {
        Self(Arc::new(render))
    }

    pub(crate) fn render(&self, body_id: &SettingsPageCustomBodyId) -> Option<AnyElement> {
        (self.0)(body_id)
    }
}

impl fmt::Debug for SettingsPageBodyRenderer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SettingsPageBodyRenderer")
            .finish_non_exhaustive()
    }
}

impl PartialEq for SettingsPageBodyRenderer {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

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
    page_body_renderer: Option<SettingsPageBodyRenderer>,
}

impl SettingsWindowOptions {
    /// Creates settings-window options with a custom title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            width: DEFAULT_WINDOW_WIDTH,
            height: DEFAULT_WINDOW_HEIGHT,
            min_width: DEFAULT_WINDOW_WIDTH,
            min_height: DEFAULT_WINDOW_HEIGHT,
            saved_color_swatches: Vec::new(),
            visual_theme: SettingsWindowTheme::default(),
            text_input_undo_byte_limit: TextInputOptions::DEFAULT_UNDO_BYTE_LIMIT,
            page_body_renderer: None,
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

    /// Returns a copy with a host renderer for page-owned custom body regions.
    pub fn with_page_body_renderer(mut self, renderer: SettingsPageBodyRenderer) -> Self {
        self.page_body_renderer = Some(renderer);
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

    /// Returns the optional host renderer for page custom body regions.
    pub fn page_body_renderer(&self) -> Option<&SettingsPageBodyRenderer> {
        self.page_body_renderer.as_ref()
    }
}

impl Default for SettingsWindowOptions {
    fn default() -> Self {
        Self::new("Settings")
    }
}
