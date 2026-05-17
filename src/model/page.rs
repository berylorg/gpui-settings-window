use super::{SettingsPageAction, SettingsPageId, SettingsPageSplit, SettingsRow};

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
    local_split: Option<SettingsPageSplit>,
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
            local_split: None,
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

    /// Returns a copy of this page with a page-local leading split list.
    pub fn with_local_split(mut self, split: SettingsPageSplit) -> Self {
        self.local_split = Some(split);
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

    /// Returns the optional page-local leading split list.
    pub fn local_split(&self) -> Option<&SettingsPageSplit> {
        self.local_split.as_ref()
    }

    /// Returns the host-supplied modified presentation state.
    pub fn is_modified(&self) -> bool {
        self.modified
    }
}
