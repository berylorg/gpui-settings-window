use crate::RgbColor;

use super::SettingsPageSplitItemId;

/// Optional preview styling for an item in a page-local split list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SettingsPageSplitItemPreviewStyle {
    font_family: Option<String>,
    font_size: Option<u16>,
    font_weight: Option<u16>,
    foreground: Option<RgbColor>,
    background: Option<RgbColor>,
    border: Option<RgbColor>,
}

impl SettingsPageSplitItemPreviewStyle {
    /// Returns a copy of this preview style with a font family hint.
    pub fn with_font_family(mut self, font_family: impl Into<String>) -> Self {
        let font_family = font_family.into();
        self.font_family = (!font_family.is_empty()).then_some(font_family);
        self
    }

    /// Returns a copy of this preview style with a font size hint in pixels.
    pub fn with_font_size(mut self, font_size: u16) -> Self {
        self.font_size = Some(font_size);
        self
    }

    /// Returns a copy of this preview style with a font weight hint.
    pub fn with_font_weight(mut self, font_weight: u16) -> Self {
        self.font_weight = Some(font_weight);
        self
    }

    /// Returns a copy of this preview style with a foreground color hint.
    pub fn with_foreground(mut self, foreground: RgbColor) -> Self {
        self.foreground = Some(foreground);
        self
    }

    /// Returns a copy of this preview style with a background color hint.
    pub fn with_background(mut self, background: RgbColor) -> Self {
        self.background = Some(background);
        self
    }

    /// Returns a copy of this preview style with a border color hint.
    pub fn with_border(mut self, border: RgbColor) -> Self {
        self.border = Some(border);
        self
    }

    /// Returns the optional font family hint.
    pub fn font_family(&self) -> Option<&str> {
        self.font_family.as_deref()
    }

    /// Returns the optional font size hint in pixels.
    pub fn font_size(&self) -> Option<u16> {
        self.font_size
    }

    /// Returns the optional font weight hint.
    pub fn font_weight(&self) -> Option<u16> {
        self.font_weight
    }

    /// Returns the optional foreground color hint.
    pub fn foreground(&self) -> Option<RgbColor> {
        self.foreground
    }

    /// Returns the optional background color hint.
    pub fn background(&self) -> Option<RgbColor> {
        self.background
    }

    /// Returns the optional border color hint.
    pub fn border(&self) -> Option<RgbColor> {
        self.border
    }
}

/// One selectable item in a page-local leading split list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsPageSplitItem {
    item_id: SettingsPageSplitItemId,
    label: String,
    subtext: Option<String>,
    selected: bool,
    preview_style: Option<SettingsPageSplitItemPreviewStyle>,
}

impl SettingsPageSplitItem {
    /// Creates a selectable split-list item.
    pub fn new(item_id: impl Into<SettingsPageSplitItemId>, label: impl Into<String>) -> Self {
        Self {
            item_id: item_id.into(),
            label: label.into(),
            subtext: None,
            selected: false,
            preview_style: None,
        }
    }

    /// Returns a copy of this item with secondary text.
    pub fn with_subtext(mut self, subtext: impl Into<String>) -> Self {
        let subtext = subtext.into();
        self.subtext = (!subtext.is_empty()).then_some(subtext);
        self
    }

    /// Returns a copy of this item with host-supplied selected state.
    pub fn with_selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Returns a copy of this item with optional preview styling.
    pub fn with_preview_style(mut self, preview_style: SettingsPageSplitItemPreviewStyle) -> Self {
        self.preview_style = Some(preview_style);
        self
    }

    /// Returns the stable item identifier.
    pub fn item_id(&self) -> &SettingsPageSplitItemId {
        &self.item_id
    }

    /// Returns the item label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the optional item subtext.
    pub fn subtext(&self) -> Option<&str> {
        self.subtext.as_deref()
    }

    /// Returns whether the host model marks this item selected.
    pub fn is_selected(&self) -> bool {
        self.selected
    }

    /// Returns optional preview styling for this item.
    pub fn preview_style(&self) -> Option<&SettingsPageSplitItemPreviewStyle> {
        self.preview_style.as_ref()
    }
}

/// Page-local leading list rendered beside a page's detail rows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SettingsPageSplit {
    items: Vec<SettingsPageSplitItem>,
}

impl SettingsPageSplit {
    /// Creates an empty page-local split list.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Returns a copy of this split list with one appended item.
    pub fn with_item(mut self, item: SettingsPageSplitItem) -> Self {
        self.items.push(item);
        self
    }

    /// Returns ordered split-list items.
    pub fn items(&self) -> &[SettingsPageSplitItem] {
        &self.items
    }

    /// Returns the host-selected item, if one is marked selected.
    pub fn selected_item(&self) -> Option<&SettingsPageSplitItem> {
        self.items.iter().find(|item| item.is_selected())
    }
}
