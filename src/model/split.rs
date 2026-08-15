use crate::RgbColor;

use std::ops::Range;

use super::{SettingsPageId, SettingsPageSplitItemId, SettingsPageSplitSourceId};

mod delivery;
mod work;

pub use delivery::{SettingsPageSplitDelivery, SettingsPageSplitDeliveryError};
pub use work::{
    MAX_PAGE_SPLIT_ACTIVE_PAGES, MAX_PAGE_SPLIT_WORK_ITEMS, SettingsPageSplitWork,
    SettingsPageSplitWorkReceiver,
};

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
    logical_position: usize,
    item_id: SettingsPageSplitItemId,
    label: String,
    subtext: Option<String>,
    preview_style: Option<SettingsPageSplitItemPreviewStyle>,
}

impl SettingsPageSplitItem {
    /// Creates a selectable split-list item.
    pub fn new(
        logical_position: usize,
        item_id: impl Into<SettingsPageSplitItemId>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            logical_position,
            item_id: item_id.into(),
            label: label.into(),
            subtext: None,
            preview_style: None,
        }
    }

    /// Returns a copy of this item with secondary text.
    pub fn with_subtext(mut self, subtext: impl Into<String>) -> Self {
        let subtext = subtext.into();
        self.subtext = (!subtext.is_empty()).then_some(subtext);
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

    /// Returns this fragment's exact logical position in its source.
    pub fn logical_position(&self) -> usize {
        self.logical_position
    }

    /// Returns optional preview styling for this item.
    pub fn preview_style(&self) -> Option<&SettingsPageSplitItemPreviewStyle> {
        self.preview_style.as_ref()
    }
}

/// Complete identity of one coherent paged split-list source revision.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SettingsPageSplitSourceKey {
    source_id: SettingsPageSplitSourceId,
    generation: u64,
    revision: u64,
}

impl SettingsPageSplitSourceKey {
    /// Creates a complete source key.
    pub fn new(
        source_id: impl Into<SettingsPageSplitSourceId>,
        generation: u64,
        revision: u64,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            generation,
            revision,
        }
    }

    /// Returns the source's stable identity.
    pub fn source_id(&self) -> &SettingsPageSplitSourceId {
        &self.source_id
    }
    /// Returns the source generation.
    pub fn generation(&self) -> u64 {
        self.generation
    }
    /// Returns the revision within the generation.
    pub fn revision(&self) -> u64 {
        self.revision
    }
}

/// Compact host-selected item identity and last known logical position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsPageSplitSelection {
    item_id: SettingsPageSplitItemId,
    logical_position: usize,
}

impl SettingsPageSplitSelection {
    /// Creates a compact selected-item descriptor.
    pub fn new(item_id: impl Into<SettingsPageSplitItemId>, logical_position: usize) -> Self {
        Self {
            item_id: item_id.into(),
            logical_position,
        }
    }
    /// Returns the selected stable item identity.
    pub fn item_id(&self) -> &SettingsPageSplitItemId {
        &self.item_id
    }
    /// Returns the selected item's last known logical position.
    pub fn logical_position(&self) -> usize {
        self.logical_position
    }
}

/// Data-only descriptor for a bounded, host-owned page-local split source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsPageSplitSource {
    key: SettingsPageSplitSourceKey,
    logical_item_count: usize,
    max_page_items: usize,
    max_page_decoded_bytes: usize,
    selected: Option<SettingsPageSplitSelection>,
}

impl SettingsPageSplitSource {
    /// Creates a paged source descriptor with hard per-result limits.
    pub fn new(
        key: SettingsPageSplitSourceKey,
        logical_item_count: usize,
        max_page_items: usize,
        max_page_decoded_bytes: usize,
    ) -> Self {
        Self {
            key,
            logical_item_count,
            max_page_items,
            max_page_decoded_bytes,
            selected: None,
        }
    }
    /// Returns a copy with a compact host-selected item.
    pub fn with_selected(mut self, selected: SettingsPageSplitSelection) -> Self {
        self.selected = Some(selected);
        self
    }
    /// Returns the complete source key.
    pub fn key(&self) -> &SettingsPageSplitSourceKey {
        &self.key
    }
    /// Returns the logical item count without materializing any items.
    pub fn logical_item_count(&self) -> usize {
        self.logical_item_count
    }
    /// Returns the hard item limit for one page result.
    pub fn max_page_items(&self) -> usize {
        self.max_page_items
    }
    /// Returns the hard decoded UTF-8 byte limit for one page result.
    pub fn max_page_decoded_bytes(&self) -> usize {
        self.max_page_decoded_bytes
    }
    /// Returns the compact selected-item descriptor.
    pub fn selected(&self) -> Option<&SettingsPageSplitSelection> {
        self.selected.as_ref()
    }
}

/// Identity echoed by one split page request and result.
///
/// Identities emitted by the pager are nonzero, monotonic, and never reused within that pager
/// lifetime. Arbitrary identities created by callers do not acquire that issuance guarantee.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SettingsPageSplitRequestId(u64);

impl SettingsPageSplitRequestId {
    /// Creates an exact identity for host echo, transport reconstruction, or contract testing.
    pub fn new(value: u64) -> Self {
        Self(value)
    }
    /// Returns the numeric request identity.
    pub fn get(self) -> u64 {
        self.0
    }
}

/// Exact bounded logical range request emitted to the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsPageSplitPageRequest {
    page_id: SettingsPageId,
    source_key: SettingsPageSplitSourceKey,
    request_id: SettingsPageSplitRequestId,
    range: Range<usize>,
    focus_probe: Option<SettingsPageSplitFocusProbe>,
}

impl SettingsPageSplitPageRequest {
    /// Creates exact request facts for transport and contract testing.
    pub fn new(
        page_id: SettingsPageId,
        source_key: SettingsPageSplitSourceKey,
        request_id: SettingsPageSplitRequestId,
        range: Range<usize>,
    ) -> Self {
        Self {
            page_id,
            source_key,
            request_id,
            range,
            focus_probe: None,
        }
    }
    pub(crate) fn with_focus_probe(mut self, probe: SettingsPageSplitFocusProbe) -> Self {
        self.focus_probe = Some(probe);
        self
    }
    /// Returns the owning settings page.
    pub fn page_id(&self) -> &SettingsPageId {
        &self.page_id
    }
    /// Returns the complete source key.
    pub fn source_key(&self) -> &SettingsPageSplitSourceKey {
        &self.source_key
    }
    /// Returns the unique request identity.
    pub fn request_id(&self) -> SettingsPageSplitRequestId {
        self.request_id
    }
    /// Returns the exact requested logical range.
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
    /// Returns the bounded prior-focus identity probe attached to this request.
    ///
    /// The pager attaches this only to the first exact page request after a same-page,
    /// same-source-identity key replacement while a realized focus identity is pending. The host
    /// must resolve it across the complete replacement source without materializing that source.
    pub fn focus_probe(&self) -> Option<&SettingsPageSplitFocusProbe> {
        self.focus_probe.as_ref()
    }
}

/// Stable prior-focus identity to resolve against a replacement source key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsPageSplitFocusProbe {
    item_id: SettingsPageSplitItemId,
}

impl SettingsPageSplitFocusProbe {
    /// Creates an exact stable-identity focus probe.
    pub fn new(item_id: impl Into<SettingsPageSplitItemId>) -> Self {
        Self {
            item_id: item_id.into(),
        }
    }
    /// Returns the stable item identity being resolved.
    pub fn item_id(&self) -> &SettingsPageSplitItemId {
        &self.item_id
    }
}

/// Exact compact resolution of a prior focus identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsPageSplitFocusResolution {
    /// The probed identity exists at this exact position in the requested source key.
    Found(usize),
    /// The probed identity no longer exists anywhere in the requested source key.
    Removed,
}

/// Bounded typed reason that one requested split range is unavailable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsPageSplitPageFailure {
    /// The host could not make the exact range available.
    Unavailable(String),
}

impl SettingsPageSplitPageFailure {
    /// Returns the bounded host-supplied presentation message.
    pub fn message(&self) -> &str {
        match self {
            Self::Unavailable(message) => message,
        }
    }
}

/// Terminal payload for one exact split page request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsPageSplitPageOutcome {
    /// Exact contiguous item fragments for the requested range.
    Ready(Vec<SettingsPageSplitItem>),
    /// The exact range is unavailable for a typed host-owned reason.
    Failed(SettingsPageSplitPageFailure),
    /// The host cancelled the exact request without publishing content.
    Cancelled,
}

/// Exact keyed result delivered by the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsPageSplitPageResult {
    request: SettingsPageSplitPageRequest,
    logical_item_count: usize,
    outcome: SettingsPageSplitPageOutcome,
    focus_resolution: Option<SettingsPageSplitFocusResolution>,
}

impl SettingsPageSplitPageResult {
    /// Creates a ready exact page result for every position in the repeated request range.
    pub fn ready(
        request: SettingsPageSplitPageRequest,
        logical_item_count: usize,
        items: Vec<SettingsPageSplitItem>,
    ) -> Self {
        Self {
            request,
            logical_item_count,
            outcome: SettingsPageSplitPageOutcome::Ready(items),
            focus_resolution: None,
        }
    }
    /// Creates a typed failed exact page result without interpreting the range as empty.
    pub fn failed(
        request: SettingsPageSplitPageRequest,
        logical_item_count: usize,
        failure: SettingsPageSplitPageFailure,
    ) -> Self {
        Self {
            request,
            logical_item_count,
            outcome: SettingsPageSplitPageOutcome::Failed(failure),
            focus_resolution: None,
        }
    }
    /// Creates a host-cancelled exact page result without publishing page content.
    pub fn cancelled(request: SettingsPageSplitPageRequest, logical_item_count: usize) -> Self {
        Self {
            request,
            logical_item_count,
            outcome: SettingsPageSplitPageOutcome::Cancelled,
            focus_resolution: None,
        }
    }
    /// Returns a copy carrying the exact resolution required by the page's focus probe.
    ///
    /// Supplying this on an unprobed request, or omitting it on a probed request, rejects the
    /// complete result before visible state changes.
    pub fn with_focus_resolution(mut self, resolution: SettingsPageSplitFocusResolution) -> Self {
        self.focus_resolution = Some(resolution);
        self
    }
    /// Returns the repeated exact request facts.
    pub fn request(&self) -> &SettingsPageSplitPageRequest {
        &self.request
    }
    /// Returns the repeated logical item count.
    pub fn logical_item_count(&self) -> usize {
        self.logical_item_count
    }
    /// Returns the terminal outcome.
    pub fn outcome(&self) -> &SettingsPageSplitPageOutcome {
        &self.outcome
    }
    /// Returns the optional exact prior-focus resolution.
    pub fn focus_resolution(&self) -> Option<&SettingsPageSplitFocusResolution> {
        self.focus_resolution.as_ref()
    }
}
