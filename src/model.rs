use std::collections::HashSet;
use std::error::Error;
use std::fmt;

mod actions;
mod element_id;
mod ids;
mod page;
mod row;
mod split;

pub use actions::{
    SettingsActionAvailability, SettingsPageAction, SettingsPageActionPriority, SettingsRowAction,
};
pub(crate) use element_id::element_id_suffix;
pub use ids::{
    SettingsFieldId, SettingsPageActionId, SettingsPageId, SettingsPageSplitItemId,
    SettingsRowActionId, SettingsSectionId,
};
pub use page::{SettingsBreadcrumbSegment, SettingsPage};
pub use row::{
    SettingsChoiceOption, SettingsFieldKind, SettingsRow, SettingsRowDetailField, SettingsRowKind,
};
pub use split::{SettingsPageSplit, SettingsPageSplitItem, SettingsPageSplitItemPreviewStyle};

/// Maximum number of detail rows allowed on one settings page.
pub const MAX_PAGE_DETAIL_ROWS: usize = 32;

/// One left-navigation section with a root page and optional subpages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsSection {
    section_id: SettingsSectionId,
    label: String,
    root_page: SettingsPage,
    subpages: Vec<SettingsPage>,
}

impl SettingsSection {
    /// Creates a settings section whose root page uses the section identifier.
    pub fn new(section_id: impl Into<SettingsSectionId>, label: impl Into<String>) -> Self {
        let section_id = section_id.into();
        let label = label.into();
        let root_page = SettingsPage::new(section_id.as_str().to_owned(), label.clone());
        Self {
            section_id,
            label,
            root_page,
            subpages: Vec::new(),
        }
    }

    /// Returns a copy of this section with a replaced root page.
    pub fn with_root_page(mut self, page: SettingsPage) -> Self {
        self.root_page = page;
        self
    }

    /// Returns a copy of this section with an appended root-page row.
    pub fn with_row(mut self, row: SettingsRow) -> Self {
        let root_page = std::mem::replace(
            &mut self.root_page,
            SettingsPage::new("__temporary_root_page__", ""),
        );
        self.root_page = root_page.with_row(row);
        self
    }

    /// Returns a copy of this section with an appended subpage.
    pub fn with_page(mut self, page: SettingsPage) -> Self {
        self.subpages.push(page);
        self
    }

    /// Returns the section's stable identifier.
    pub fn section_id(&self) -> &SettingsSectionId {
        &self.section_id
    }

    /// Returns the section label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the section root page.
    pub fn root_page(&self) -> &SettingsPage {
        &self.root_page
    }

    /// Returns ordered subpages for this section.
    pub fn subpages(&self) -> &[SettingsPage] {
        &self.subpages
    }

    /// Returns the ordered rows in this section's root page.
    pub fn rows(&self) -> &[SettingsRow] {
        self.root_page.rows()
    }

    /// Returns every page owned by this section, root page first.
    pub fn pages(&self) -> impl Iterator<Item = &SettingsPage> {
        std::iter::once(&self.root_page).chain(self.subpages.iter())
    }
}

/// Top-level presentation model for a settings window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsWindowModel {
    sections: Vec<SettingsSection>,
    selected_section_id: SettingsSectionId,
    selected_page_id: SettingsPageId,
}

impl SettingsWindowModel {
    /// Creates a model and selects the first section's root page.
    pub fn new(sections: Vec<SettingsSection>) -> Result<Self, SettingsWindowError> {
        let selected_section_id = sections
            .first()
            .map(|section| section.section_id.clone())
            .ok_or(SettingsWindowError::EmptySections)?;

        Self::with_selected_section(sections, selected_section_id)
    }

    /// Creates a model and selects a specific section's root page.
    pub fn with_selected_section(
        sections: Vec<SettingsSection>,
        selected_section_id: impl Into<SettingsSectionId>,
    ) -> Result<Self, SettingsWindowError> {
        validate_sections(&sections)?;

        let selected_section_id = selected_section_id.into();
        let Some(section) = sections
            .iter()
            .find(|section| section.section_id == selected_section_id)
        else {
            return Err(SettingsWindowError::MissingSelectedSection(
                selected_section_id,
            ));
        };
        let selected_page_id = section.root_page().page_id().clone();

        Ok(Self {
            sections,
            selected_section_id,
            selected_page_id,
        })
    }

    /// Creates a model and selects a specific page within a specific section.
    pub fn with_selected_page(
        sections: Vec<SettingsSection>,
        selected_section_id: impl Into<SettingsSectionId>,
        selected_page_id: impl Into<SettingsPageId>,
    ) -> Result<Self, SettingsWindowError> {
        validate_sections(&sections)?;

        let selected_section_id = selected_section_id.into();
        let selected_page_id = selected_page_id.into();
        let Some(section) = sections
            .iter()
            .find(|section| section.section_id == selected_section_id)
        else {
            return Err(SettingsWindowError::MissingSelectedSection(
                selected_section_id,
            ));
        };

        if !section
            .pages()
            .any(|page| page.page_id() == &selected_page_id)
        {
            if sections
                .iter()
                .flat_map(SettingsSection::pages)
                .any(|page| page.page_id() == &selected_page_id)
            {
                return Err(SettingsWindowError::SelectedPageOutsideSection {
                    section_id: selected_section_id,
                    page_id: selected_page_id,
                });
            }

            return Err(SettingsWindowError::MissingSelectedPage(selected_page_id));
        }

        Ok(Self {
            sections,
            selected_section_id,
            selected_page_id,
        })
    }

    /// Selects another section and its root page.
    pub fn select_section(
        &mut self,
        section_id: impl Into<SettingsSectionId>,
    ) -> Result<(), SettingsWindowError> {
        let section_id = section_id.into();
        let Some(section) = self
            .sections
            .iter()
            .find(|section| section.section_id == section_id)
        else {
            return Err(SettingsWindowError::MissingSelectedSection(section_id));
        };

        self.selected_page_id = section.root_page().page_id().clone();
        self.selected_section_id = section_id;
        Ok(())
    }

    /// Selects another page and its owning section.
    pub fn select_page(
        &mut self,
        page_id: impl Into<SettingsPageId>,
    ) -> Result<(), SettingsWindowError> {
        let page_id = page_id.into();
        let Some(section) = self
            .sections
            .iter()
            .find(|section| section.pages().any(|page| page.page_id() == &page_id))
        else {
            return Err(SettingsWindowError::MissingSelectedPage(page_id));
        };

        self.selected_section_id = section.section_id().clone();
        self.selected_page_id = page_id;
        Ok(())
    }

    /// Returns all ordered sections.
    pub fn sections(&self) -> &[SettingsSection] {
        &self.sections
    }

    /// Returns all rows across all pages in presentation order.
    pub fn rows(&self) -> impl Iterator<Item = &SettingsRow> {
        self.sections
            .iter()
            .flat_map(SettingsSection::pages)
            .flat_map(SettingsPage::rows)
    }

    /// Returns all editable field rows across all pages in presentation order.
    pub fn field_rows(&self) -> impl Iterator<Item = &SettingsRow> {
        self.rows().filter(|row| row.is_field())
    }

    /// Returns the field kind for a primary or secondary field by identifier.
    pub fn field_kind(&self, field_id: &SettingsFieldId) -> Option<SettingsFieldKind> {
        self.rows().find_map(|row| {
            if row.field_id() == field_id {
                return Some(row.kind());
            }
            row.detail_field()
                .filter(|field| field.field_id() == field_id)
                .map(|field| field.kind())
        })
    }

    /// Returns the presentation value for a primary or secondary field by identifier.
    pub fn field_value(&self, field_id: &SettingsFieldId) -> Option<&str> {
        self.rows().find_map(|row| {
            if row.field_id() == field_id {
                return Some(row.value());
            }
            row.detail_field()
                .filter(|field| field.field_id() == field_id)
                .map(|field| field.value())
        })
    }

    /// Returns the validation message for a primary or secondary field by identifier.
    pub fn field_error(&self, field_id: &SettingsFieldId) -> Option<&str> {
        self.rows().find_map(|row| {
            if row.field_id() == field_id {
                return row.error();
            }
            row.detail_field()
                .filter(|field| field.field_id() == field_id)
                .and_then(|field| field.error())
        })
    }

    /// Returns the choices for a primary or secondary choice field by identifier.
    pub fn field_choices(&self, field_id: &SettingsFieldId) -> Option<&[SettingsChoiceOption]> {
        self.rows().find_map(|row| {
            if row.field_id() == field_id {
                return Some(row.choices());
            }
            row.detail_field()
                .filter(|field| field.field_id() == field_id)
                .map(|field| field.choices())
        })
    }

    /// Returns the currently selected section identifier.
    pub fn selected_section_id(&self) -> &SettingsSectionId {
        &self.selected_section_id
    }

    /// Returns the currently selected page identifier.
    pub fn selected_page_id(&self) -> &SettingsPageId {
        &self.selected_page_id
    }

    /// Returns the currently selected section.
    pub fn selected_section(&self) -> &SettingsSection {
        self.sections
            .iter()
            .find(|section| section.section_id == self.selected_section_id)
            .expect("selected section is validated when the model is created or updated")
    }

    /// Returns the currently selected page.
    pub fn selected_page(&self) -> &SettingsPage {
        self.page(&self.selected_page_id)
            .expect("selected page is validated when the model is created or updated")
    }

    /// Returns the rows for the currently selected page.
    pub fn selected_rows(&self) -> &[SettingsRow] {
        self.selected_page().rows()
    }

    /// Finds a page by its stable identifier.
    pub fn page(&self, page_id: &SettingsPageId) -> Option<&SettingsPage> {
        self.sections
            .iter()
            .flat_map(SettingsSection::pages)
            .find(|page| page.page_id() == page_id)
    }

    /// Finds a row by its field or row identifier.
    pub fn row(&self, field_id: &SettingsFieldId) -> Option<&SettingsRow> {
        self.sections
            .iter()
            .flat_map(SettingsSection::pages)
            .flat_map(SettingsPage::rows)
            .find(|row| row.field_id() == field_id)
    }

    /// Updates a field row value by identifier.
    pub fn set_row_value(
        &mut self,
        field_id: &SettingsFieldId,
        value: impl Into<String>,
    ) -> Result<(), SettingsWindowError> {
        let value = value.into();
        for row in self
            .sections
            .iter_mut()
            .flat_map(|section| {
                std::iter::once(&mut section.root_page).chain(section.subpages.iter_mut())
            })
            .flat_map(SettingsPage::rows_mut)
        {
            if row.field_id() == field_id {
                row.set_value(value);
                return Ok(());
            }
            if let Some(field) = row
                .detail_field_mut()
                .filter(|field| field.field_id() == field_id)
            {
                field.set_value(value);
                return Ok(());
            }
        }

        Err(SettingsWindowError::MissingField(field_id.clone()))
    }
}

/// App-neutral event emitted by the settings window.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SettingsWindowEvent {
    /// The user selected another navigation section.
    SectionSelected {
        /// Selected section identifier.
        section_id: SettingsSectionId,
    },
    /// The user requested navigation to a page.
    PageNavigationRequested {
        /// Target page identifier.
        page_id: SettingsPageId,
    },
    /// The user changed a field value.
    FieldChanged {
        /// Changed field identifier.
        field_id: SettingsFieldId,
        /// New presentation value.
        value: String,
    },
    /// The user requested that the compact color field expand into the picker.
    ColorPickerRequested {
        /// Color field identifier.
        field_id: SettingsFieldId,
    },
    /// The user requested an app-neutral action attached to a row.
    RowActionRequested {
        /// Field or row identifier for the row that owns the action.
        field_id: SettingsFieldId,
        /// Requested action identifier.
        action_id: SettingsRowActionId,
    },
    /// The user requested an app-neutral action attached to a page.
    PageActionRequested {
        /// Page identifier for the page that owns the action.
        page_id: SettingsPageId,
        /// Requested action identifier.
        action_id: SettingsPageActionId,
    },
    /// The user selected an item from a page-local split list.
    PageSplitItemSelected {
        /// Page identifier for the page that owns the local split list.
        page_id: SettingsPageId,
        /// Selected split-list item identifier.
        item_id: SettingsPageSplitItemId,
    },
    /// The user requested accepting current settings values, usually from OK or Enter.
    AcceptRequested,
    /// The user requested that the host apply current settings values.
    ApplyRequested,
    /// The user requested that unapplied edits be dismissed.
    CancelRequested,
    /// The user requested that the settings window close or hide.
    CloseRequested,
}

/// Validation error for a settings presentation model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsWindowError {
    /// The model has no sections.
    EmptySections,
    /// A section has an empty identifier.
    EmptySectionId,
    /// More than one section uses the same identifier.
    DuplicateSectionId(SettingsSectionId),
    /// A page has an empty identifier.
    EmptyPageId,
    /// More than one page uses the same identifier.
    DuplicatePageId(SettingsPageId),
    /// A page has more detail rows than the full-rendered page contract allows.
    TooManyPageRows {
        /// Page that owns the excess detail rows.
        page_id: SettingsPageId,
        /// Actual detail row count.
        row_count: usize,
        /// Maximum allowed detail row count.
        max_row_count: usize,
    },
    /// A field has an empty identifier.
    EmptyFieldId,
    /// More than one row uses the same field identifier.
    DuplicateFieldId(SettingsFieldId),
    /// A row action has an empty identifier.
    EmptyRowActionId {
        /// Field identifier for the row that owns the invalid action.
        field_id: SettingsFieldId,
    },
    /// A choice field has no choices.
    EmptyChoiceOptions {
        /// Field identifier for the invalid choice row.
        field_id: SettingsFieldId,
    },
    /// A choice field option has an empty value.
    EmptyChoiceOptionValue {
        /// Field identifier for the invalid choice row.
        field_id: SettingsFieldId,
    },
    /// More than one choice option on a row uses the same value.
    DuplicateChoiceOptionValue {
        /// Field identifier for the invalid choice row.
        field_id: SettingsFieldId,
        /// Duplicate choice value.
        value: String,
    },
    /// A choice field row value does not match any option value.
    MissingChoiceValue {
        /// Field identifier for the invalid choice row.
        field_id: SettingsFieldId,
        /// Missing selected value.
        value: String,
    },
    /// More than one action on a row uses the same identifier.
    DuplicateRowActionId {
        /// Field identifier for the row that owns the duplicate action.
        field_id: SettingsFieldId,
        /// Duplicate action identifier.
        action_id: SettingsRowActionId,
    },
    /// A page action has an empty identifier.
    EmptyPageActionId {
        /// Page identifier for the page that owns the invalid action.
        page_id: SettingsPageId,
    },
    /// More than one action on a page uses the same identifier.
    DuplicatePageActionId {
        /// Page identifier for the page that owns the duplicate action.
        page_id: SettingsPageId,
        /// Duplicate action identifier.
        action_id: SettingsPageActionId,
    },
    /// A page-local split-list item has an empty identifier.
    EmptyPageSplitItemId {
        /// Page identifier for the page that owns the invalid item.
        page_id: SettingsPageId,
    },
    /// More than one page-local split-list item uses the same identifier on one page.
    DuplicatePageSplitItemId {
        /// Page identifier for the page that owns the duplicate item.
        page_id: SettingsPageId,
        /// Duplicate item identifier.
        item_id: SettingsPageSplitItemId,
    },
    /// More than one page-local split-list item is marked selected on one page.
    MultiplePageSplitItemsSelected {
        /// Page identifier for the page that owns the invalid selected state.
        page_id: SettingsPageId,
    },
    /// A field identifier does not exist in the model.
    MissingField(SettingsFieldId),
    /// The selected section is not present in the model.
    MissingSelectedSection(SettingsSectionId),
    /// The selected page is not present in the model.
    MissingSelectedPage(SettingsPageId),
    /// The selected page belongs to a different section.
    SelectedPageOutsideSection {
        /// Selected section identifier.
        section_id: SettingsSectionId,
        /// Selected page identifier.
        page_id: SettingsPageId,
    },
    /// A navigation row targets a page that does not exist.
    MissingNavigationTargetPage {
        /// Field identifier for the navigation row.
        field_id: SettingsFieldId,
        /// Missing target page.
        target_page_id: SettingsPageId,
    },
    /// A page back target does not exist.
    MissingBackTargetPage {
        /// Page that owns the invalid back target.
        page_id: SettingsPageId,
        /// Missing target page.
        target_page_id: SettingsPageId,
    },
    /// A breadcrumb target does not exist.
    MissingBreadcrumbTargetPage {
        /// Page that owns the invalid breadcrumb.
        page_id: SettingsPageId,
        /// Missing target page.
        target_page_id: SettingsPageId,
    },
}

impl fmt::Display for SettingsWindowError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySections => write!(formatter, "settings model has no sections"),
            Self::EmptySectionId => write!(formatter, "settings section id is empty"),
            Self::DuplicateSectionId(section_id) => {
                write!(formatter, "duplicate settings section id `{section_id}`")
            }
            Self::EmptyPageId => write!(formatter, "settings page id is empty"),
            Self::DuplicatePageId(page_id) => {
                write!(formatter, "duplicate settings page id `{page_id}`")
            }
            Self::TooManyPageRows {
                page_id,
                row_count,
                max_row_count,
            } => {
                write!(
                    formatter,
                    "settings page `{page_id}` has {row_count} detail rows, above the maximum {max_row_count}"
                )
            }
            Self::EmptyFieldId => write!(formatter, "settings field id is empty"),
            Self::DuplicateFieldId(field_id) => {
                write!(formatter, "duplicate settings field id `{field_id}`")
            }
            Self::EmptyRowActionId { field_id } => {
                write!(
                    formatter,
                    "settings row action id is empty for `{field_id}`"
                )
            }
            Self::EmptyChoiceOptions { field_id } => {
                write!(formatter, "settings choice row `{field_id}` has no options")
            }
            Self::EmptyChoiceOptionValue { field_id } => {
                write!(
                    formatter,
                    "settings choice row `{field_id}` has an empty option value"
                )
            }
            Self::DuplicateChoiceOptionValue { field_id, value } => {
                write!(
                    formatter,
                    "duplicate settings choice option value `{value}` for `{field_id}`"
                )
            }
            Self::MissingChoiceValue { field_id, value } => {
                write!(
                    formatter,
                    "settings choice row `{field_id}` selected missing value `{value}`"
                )
            }
            Self::DuplicateRowActionId {
                field_id,
                action_id,
            } => {
                write!(
                    formatter,
                    "duplicate settings row action id `{action_id}` for `{field_id}`"
                )
            }
            Self::EmptyPageActionId { page_id } => {
                write!(
                    formatter,
                    "settings page action id is empty for `{page_id}`"
                )
            }
            Self::DuplicatePageActionId { page_id, action_id } => {
                write!(
                    formatter,
                    "duplicate settings page action id `{action_id}` for `{page_id}`"
                )
            }
            Self::EmptyPageSplitItemId { page_id } => {
                write!(
                    formatter,
                    "settings page split item id is empty for `{page_id}`"
                )
            }
            Self::DuplicatePageSplitItemId { page_id, item_id } => {
                write!(
                    formatter,
                    "duplicate settings page split item id `{item_id}` for `{page_id}`"
                )
            }
            Self::MultiplePageSplitItemsSelected { page_id } => {
                write!(
                    formatter,
                    "settings page `{page_id}` has multiple selected split items"
                )
            }
            Self::MissingField(field_id) => {
                write!(formatter, "settings field `{field_id}` does not exist")
            }
            Self::MissingSelectedSection(section_id) => {
                write!(
                    formatter,
                    "selected settings section `{section_id}` does not exist"
                )
            }
            Self::MissingSelectedPage(page_id) => {
                write!(
                    formatter,
                    "selected settings page `{page_id}` does not exist"
                )
            }
            Self::SelectedPageOutsideSection {
                section_id,
                page_id,
            } => {
                write!(
                    formatter,
                    "selected settings page `{page_id}` is outside section `{section_id}`"
                )
            }
            Self::MissingNavigationTargetPage {
                field_id,
                target_page_id,
            } => {
                write!(
                    formatter,
                    "navigation row `{field_id}` targets missing page `{target_page_id}`"
                )
            }
            Self::MissingBackTargetPage {
                page_id,
                target_page_id,
            } => {
                write!(
                    formatter,
                    "settings page `{page_id}` has missing back target `{target_page_id}`"
                )
            }
            Self::MissingBreadcrumbTargetPage {
                page_id,
                target_page_id,
            } => {
                write!(
                    formatter,
                    "settings page `{page_id}` has missing breadcrumb target `{target_page_id}`"
                )
            }
        }
    }
}

impl Error for SettingsWindowError {}

fn validate_sections(sections: &[SettingsSection]) -> Result<(), SettingsWindowError> {
    if sections.is_empty() {
        return Err(SettingsWindowError::EmptySections);
    }

    let mut section_ids = HashSet::new();
    let mut page_ids = HashSet::new();

    for section in sections {
        if section.section_id.as_str().is_empty() {
            return Err(SettingsWindowError::EmptySectionId);
        }

        if !section_ids.insert(section.section_id.clone()) {
            return Err(SettingsWindowError::DuplicateSectionId(
                section.section_id.clone(),
            ));
        }

        for page in section.pages() {
            if page.page_id().as_str().is_empty() {
                return Err(SettingsWindowError::EmptyPageId);
            }

            if !page_ids.insert(page.page_id().clone()) {
                return Err(SettingsWindowError::DuplicatePageId(page.page_id().clone()));
            }
        }
    }

    let mut field_ids = HashSet::new();
    for section in sections {
        for page in section.pages() {
            validate_page(page, &page_ids, &mut field_ids)?;
        }
    }

    Ok(())
}

fn validate_page(
    page: &SettingsPage,
    page_ids: &HashSet<SettingsPageId>,
    field_ids: &mut HashSet<SettingsFieldId>,
) -> Result<(), SettingsWindowError> {
    if page.rows().len() > MAX_PAGE_DETAIL_ROWS {
        return Err(SettingsWindowError::TooManyPageRows {
            page_id: page.page_id().clone(),
            row_count: page.rows().len(),
            max_row_count: MAX_PAGE_DETAIL_ROWS,
        });
    }

    if let Some(target_page_id) = page.back_target_page_id() {
        if !page_ids.contains(target_page_id) {
            return Err(SettingsWindowError::MissingBackTargetPage {
                page_id: page.page_id().clone(),
                target_page_id: target_page_id.clone(),
            });
        }
    }

    for segment in page.breadcrumb_path() {
        if let Some(target_page_id) = segment.target_page_id() {
            if !page_ids.contains(target_page_id) {
                return Err(SettingsWindowError::MissingBreadcrumbTargetPage {
                    page_id: page.page_id().clone(),
                    target_page_id: target_page_id.clone(),
                });
            }
        }
    }

    let mut page_action_ids = HashSet::new();
    for action in page.actions() {
        if action.action_id().as_str().is_empty() {
            return Err(SettingsWindowError::EmptyPageActionId {
                page_id: page.page_id().clone(),
            });
        }

        if !page_action_ids.insert(action.action_id().clone()) {
            return Err(SettingsWindowError::DuplicatePageActionId {
                page_id: page.page_id().clone(),
                action_id: action.action_id().clone(),
            });
        }
    }

    if let Some(split) = page.local_split() {
        let mut item_ids = HashSet::new();
        let mut selected_count = 0;
        for item in split.items() {
            if item.item_id().as_str().is_empty() {
                return Err(SettingsWindowError::EmptyPageSplitItemId {
                    page_id: page.page_id().clone(),
                });
            }

            if !item_ids.insert(item.item_id().clone()) {
                return Err(SettingsWindowError::DuplicatePageSplitItemId {
                    page_id: page.page_id().clone(),
                    item_id: item.item_id().clone(),
                });
            }

            if item.is_selected() {
                selected_count += 1;
            }
        }

        if selected_count > 1 {
            return Err(SettingsWindowError::MultiplePageSplitItemsSelected {
                page_id: page.page_id().clone(),
            });
        }
    }

    for row in page.rows() {
        validate_row(row, page_ids, field_ids)?;
    }

    Ok(())
}

fn validate_row(
    row: &SettingsRow,
    page_ids: &HashSet<SettingsPageId>,
    field_ids: &mut HashSet<SettingsFieldId>,
) -> Result<(), SettingsWindowError> {
    if row.field_id().as_str().is_empty() {
        return Err(SettingsWindowError::EmptyFieldId);
    }

    if !field_ids.insert(row.field_id().clone()) {
        return Err(SettingsWindowError::DuplicateFieldId(
            row.field_id().clone(),
        ));
    }

    if let Some(target_page_id) = row.navigation_target_page_id() {
        if !page_ids.contains(target_page_id) {
            return Err(SettingsWindowError::MissingNavigationTargetPage {
                field_id: row.field_id().clone(),
                target_page_id: target_page_id.clone(),
            });
        }
    }

    let mut action_ids = HashSet::new();
    for action in row.actions() {
        if action.action_id().as_str().is_empty() {
            return Err(SettingsWindowError::EmptyRowActionId {
                field_id: row.field_id().clone(),
            });
        }

        if !action_ids.insert(action.action_id().clone()) {
            return Err(SettingsWindowError::DuplicateRowActionId {
                field_id: row.field_id().clone(),
                action_id: action.action_id().clone(),
            });
        }
    }

    validate_field_choices(row.field_id(), row.kind(), row.value(), row.choices())?;

    if let Some(field) = row.detail_field() {
        if !row.is_field() {
            return Err(SettingsWindowError::MissingField(field.field_id().clone()));
        }
        if field.field_id().as_str().is_empty() {
            return Err(SettingsWindowError::EmptyFieldId);
        }
        if !field_ids.insert(field.field_id().clone()) {
            return Err(SettingsWindowError::DuplicateFieldId(
                field.field_id().clone(),
            ));
        }
        validate_field_choices(
            field.field_id(),
            field.kind(),
            field.value(),
            field.choices(),
        )?;
    }

    Ok(())
}

fn validate_field_choices(
    field_id: &SettingsFieldId,
    kind: row::SettingsFieldKind,
    value: &str,
    choices: &[row::SettingsChoiceOption],
) -> Result<(), SettingsWindowError> {
    if kind == row::SettingsFieldKind::Choice {
        if choices.is_empty() {
            return Err(SettingsWindowError::EmptyChoiceOptions {
                field_id: field_id.clone(),
            });
        }

        let mut choice_values = HashSet::new();
        let mut selected_value_exists = false;
        for choice in choices {
            if choice.value().is_empty() {
                return Err(SettingsWindowError::EmptyChoiceOptionValue {
                    field_id: field_id.clone(),
                });
            }
            if !choice_values.insert(choice.value().to_string()) {
                return Err(SettingsWindowError::DuplicateChoiceOptionValue {
                    field_id: field_id.clone(),
                    value: choice.value().to_string(),
                });
            }
            if choice.value() == value {
                selected_value_exists = true;
            }
        }
        if !selected_value_exists {
            return Err(SettingsWindowError::MissingChoiceValue {
                field_id: field_id.clone(),
                value: value.to_string(),
            });
        }
    } else if !choices.is_empty() {
        return Err(SettingsWindowError::MissingChoiceValue {
            field_id: field_id.clone(),
            value: value.to_string(),
        });
    }

    Ok(())
}
