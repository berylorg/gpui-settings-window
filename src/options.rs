use std::{fmt, sync::Arc};

use gpui::AnyElement;
use gpui_text_input::TextInputOptions;

use crate::{RgbColor, SettingsPageCustomBodyId, SettingsSavedColorSwatchId, SettingsWindowTheme};

const DEFAULT_WINDOW_WIDTH: f32 = 800.0;
const DEFAULT_WINDOW_HEIGHT: f32 = 520.0;
/// Maximum saved color swatches rendered by one color picker.
pub const MAX_SAVED_COLOR_SWATCHES: usize = 30;

/// Invalid settings-window options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsWindowOptionsError {
    /// The saved-color grid would exceed its fixed render capacity.
    TooManySavedColorSwatches {
        at_least_swatch_count: usize,
        max_swatch_count: usize,
    },
    /// The saved-color collection repeats an identity.
    DuplicateSavedColorSwatchId(SettingsSavedColorSwatchId),
}

/// One host-owned saved color entry offered by picker popups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsSavedColorSwatch {
    swatch_id: SettingsSavedColorSwatchId,
    color: RgbColor,
}

impl SettingsSavedColorSwatch {
    /// Creates a saved swatch with its stable app-neutral identity.
    pub fn new(swatch_id: impl Into<SettingsSavedColorSwatchId>, color: RgbColor) -> Self {
        Self {
            swatch_id: swatch_id.into(),
            color,
        }
    }

    /// Returns the stable identity.
    pub fn swatch_id(&self) -> &SettingsSavedColorSwatchId {
        &self.swatch_id
    }

    /// Returns the swatch color.
    pub fn color(&self) -> RgbColor {
        self.color
    }
}

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
    saved_color_swatches: Vec<SettingsSavedColorSwatch>,
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
    /// Returns options with saved colors, rejecting values above the fixed grid capacity.
    pub fn with_saved_color_swatches(
        mut self,
        swatches: impl IntoIterator<Item = SettingsSavedColorSwatch>,
    ) -> Result<Self, SettingsWindowOptionsError> {
        let swatches: Vec<_> = swatches
            .into_iter()
            .take(MAX_SAVED_COLOR_SWATCHES + 1)
            .collect();
        if swatches.len() > MAX_SAVED_COLOR_SWATCHES {
            return Err(SettingsWindowOptionsError::TooManySavedColorSwatches {
                at_least_swatch_count: swatches.len(),
                max_swatch_count: MAX_SAVED_COLOR_SWATCHES,
            });
        }
        for (index, swatch) in swatches.iter().enumerate() {
            if swatches[..index]
                .iter()
                .any(|existing| existing.swatch_id == swatch.swatch_id)
            {
                return Err(SettingsWindowOptionsError::DuplicateSavedColorSwatchId(
                    swatch.swatch_id.clone(),
                ));
            }
        }
        self.saved_color_swatches = swatches;
        Ok(self)
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
    pub fn saved_color_swatches(&self) -> &[SettingsSavedColorSwatch] {
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
