use super::{
    SettingsPageAction, SettingsPageCustomBodyId, SettingsPageId, SettingsPageSplitSource,
    SettingsRow,
};

/// App-neutral page body layout selected by the host page model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsPageBodyLayout {
    /// The page body renders the page's detail rows as one scrollable surface.
    DetailRows,
    /// The page body reserves a leading custom region stacked above detail rows.
    StackedCustom,
}

/// App-neutral descriptor for a page-owned custom body region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsPageCustomBody {
    body_id: SettingsPageCustomBodyId,
    height_px: u16,
}

impl SettingsPageCustomBody {
    /// Creates a custom page body descriptor with a stable identifier and
    /// fixed logical-pixel height.
    pub fn new(body_id: impl Into<SettingsPageCustomBodyId>, height_px: u16) -> Self {
        Self {
            body_id: body_id.into(),
            height_px,
        }
    }

    /// Returns the stable custom body identifier.
    pub fn body_id(&self) -> &SettingsPageCustomBodyId {
        &self.body_id
    }

    /// Returns the fixed logical-pixel height reserved for this body region.
    pub fn height_px(&self) -> u16 {
        self.height_px
    }
}

/// One breadcrumb segment for a right-pane page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsBreadcrumbSegment {
    label: String,
    target_page_id: Option<SettingsPageId>,
}

impl SettingsBreadcrumbSegment {
    /// Creates a non-navigable breadcrumb segment.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            target_page_id: None,
        }
    }

    /// Creates a breadcrumb segment that may request navigation.
    pub fn linked(label: impl Into<String>, target_page_id: impl Into<SettingsPageId>) -> Self {
        Self {
            label: label.into(),
            target_page_id: Some(target_page_id.into()),
        }
    }

    /// Returns the breadcrumb display label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the optional navigation target.
    pub fn target_page_id(&self) -> Option<&SettingsPageId> {
        self.target_page_id.as_ref()
    }
}

/// One right-pane settings page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsPage {
    page_id: SettingsPageId,
    title: String,
    breadcrumb_path: Vec<SettingsBreadcrumbSegment>,
    back_target_page_id: Option<SettingsPageId>,
    rows: Vec<SettingsRow>,
    actions: Vec<SettingsPageAction>,
    paged_split_source: Option<SettingsPageSplitSource>,
    stacked_custom_body: Option<SettingsPageCustomBody>,
    modified: bool,
}

impl SettingsPage {
    /// Creates an empty settings page.
    pub fn new(page_id: impl Into<SettingsPageId>, title: impl Into<String>) -> Self {
        Self {
            page_id: page_id.into(),
            title: title.into(),
            breadcrumb_path: Vec::new(),
            back_target_page_id: None,
            rows: Vec::new(),
            actions: Vec::new(),
            paged_split_source: None,
            stacked_custom_body: None,
            modified: false,
        }
    }

    /// Returns a copy of this page with one appended row.
    pub fn with_row(mut self, row: SettingsRow) -> Self {
        self.rows.push(row);
        self
    }

    /// Returns a copy of this page with one appended page-level action.
    pub fn with_action(mut self, action: SettingsPageAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Returns a copy with a bounded paged page-local split source.
    pub fn with_paged_split_source(mut self, source: SettingsPageSplitSource) -> Self {
        self.paged_split_source = Some(source);
        self
    }

    /// Returns a copy of this page with a stacked custom body region above the
    /// ordinary detail rows.
    pub fn with_stacked_custom_body(mut self, body: SettingsPageCustomBody) -> Self {
        self.stacked_custom_body = Some(body);
        self
    }

    /// Returns a copy of this page with one appended breadcrumb segment.
    pub fn with_breadcrumb_segment(mut self, segment: SettingsBreadcrumbSegment) -> Self {
        self.breadcrumb_path.push(segment);
        self
    }

    /// Returns a copy of this page with a back navigation target.
    pub fn with_back_target(mut self, page_id: impl Into<SettingsPageId>) -> Self {
        self.back_target_page_id = Some(page_id.into());
        self
    }

    /// Returns a copy of this page with host-supplied modified presentation state.
    pub fn with_modified(mut self, modified: bool) -> Self {
        self.modified = modified;
        self
    }

    /// Returns the stable page identifier.
    pub fn page_id(&self) -> &SettingsPageId {
        &self.page_id
    }

    /// Returns the page title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns ordered breadcrumb metadata.
    pub fn breadcrumb_path(&self) -> &[SettingsBreadcrumbSegment] {
        &self.breadcrumb_path
    }

    /// Returns the optional back target.
    pub fn back_target_page_id(&self) -> Option<&SettingsPageId> {
        self.back_target_page_id.as_ref()
    }

    /// Returns ordered page rows.
    pub fn rows(&self) -> &[SettingsRow] {
        &self.rows
    }

    pub(crate) fn rows_mut(&mut self) -> &mut [SettingsRow] {
        &mut self.rows
    }

    /// Returns ordered page-level actions.
    pub fn actions(&self) -> &[SettingsPageAction] {
        &self.actions
    }

    /// Returns the optional bounded paged page-local split source.
    pub fn paged_split_source(&self) -> Option<&SettingsPageSplitSource> {
        self.paged_split_source.as_ref()
    }

    /// Returns the stacked custom body descriptor when this page has one.
    pub fn stacked_custom_body(&self) -> Option<&SettingsPageCustomBody> {
        if self.paged_split_source.is_some() {
            None
        } else {
            self.stacked_custom_body.as_ref()
        }
    }

    /// Returns the app-neutral body layout requested by the host page model.
    pub fn body_layout(&self) -> SettingsPageBodyLayout {
        match self.stacked_custom_body() {
            Some(_) => SettingsPageBodyLayout::StackedCustom,
            None => SettingsPageBodyLayout::DetailRows,
        }
    }

    /// Returns the host-supplied modified presentation state.
    pub fn is_modified(&self) -> bool {
        self.modified
    }
}
