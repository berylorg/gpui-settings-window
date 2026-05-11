use std::collections::HashSet;
use std::error::Error;
use std::fmt;

mod element_id;

pub(crate) use element_id::element_id_suffix;

/// Stable identifier for a settings section.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SettingsSectionId(String);

impl SettingsSectionId {
    /// Creates a section identifier.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SettingsSectionId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for SettingsSectionId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for SettingsSectionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Stable identifier for a settings field.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SettingsFieldId(String);

impl SettingsFieldId {
    /// Creates a field identifier.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SettingsFieldId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for SettingsFieldId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for SettingsFieldId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Stable identifier for an action attached to a settings row.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SettingsRowActionId(String);

impl SettingsRowActionId {
    /// Creates a row action identifier.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SettingsRowActionId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for SettingsRowActionId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for SettingsRowActionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Presentation kind for a settings row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SettingsFieldKind {
    /// A plain single-line text value.
    Text,
    /// A plain multiline text value.
    MultilineText,
    /// A canonical RGB hex color value such as `#6699cc`.
    Color,
}

/// App-neutral action presented beside a settings row input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsRowAction {
    action_id: SettingsRowActionId,
    label: String,
}

impl SettingsRowAction {
    /// Creates a row action.
    pub fn new(action_id: impl Into<SettingsRowActionId>, label: impl Into<String>) -> Self {
        Self {
            action_id: action_id.into(),
            label: label.into(),
        }
    }

    /// Returns the row action's stable identifier.
    pub fn action_id(&self) -> &SettingsRowActionId {
        &self.action_id
    }

    /// Returns the row action display label.
    pub fn label(&self) -> &str {
        &self.label
    }
}

/// One key-value row in the settings content area.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsRow {
    field_id: SettingsFieldId,
    label: String,
    subtext: Option<String>,
    value: String,
    kind: SettingsFieldKind,
    error: Option<String>,
    actions: Vec<SettingsRowAction>,
}

impl SettingsRow {
    /// Creates a settings row.
    pub fn new(
        field_id: impl Into<SettingsFieldId>,
        label: impl Into<String>,
        value: impl Into<String>,
        kind: SettingsFieldKind,
    ) -> Self {
        Self {
            field_id: field_id.into(),
            label: label.into(),
            subtext: None,
            value: value.into(),
            kind,
            error: None,
            actions: Vec::new(),
        }
    }

    /// Returns a copy of this row with secondary label-side subtext.
    pub fn with_subtext(mut self, subtext: impl Into<String>) -> Self {
        let subtext = subtext.into();
        self.subtext = (!subtext.is_empty()).then_some(subtext);
        self
    }

    /// Returns a copy of this row with an attached validation message.
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Returns a copy of this row without a validation message.
    pub fn without_error(mut self) -> Self {
        self.error = None;
        self
    }

    /// Returns a copy of this row with another presentation value.
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    /// Returns a copy of this row with an appended app-neutral action.
    pub fn with_action(mut self, action: SettingsRowAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Returns the row's stable field identifier.
    pub fn field_id(&self) -> &SettingsFieldId {
        &self.field_id
    }

    /// Returns the row label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the optional row subtext.
    pub fn subtext(&self) -> Option<&str> {
        self.subtext.as_deref()
    }

    /// Returns the row value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Returns the row field kind.
    pub fn kind(&self) -> SettingsFieldKind {
        self.kind
    }

    /// Returns the optional validation message.
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Returns the ordered row actions.
    pub fn actions(&self) -> &[SettingsRowAction] {
        &self.actions
    }

    pub(crate) fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
    }
}

/// One left-navigation section and its ordered setting rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsSection {
    section_id: SettingsSectionId,
    label: String,
    rows: Vec<SettingsRow>,
}

impl SettingsSection {
    /// Creates a settings section.
    pub fn new(section_id: impl Into<SettingsSectionId>, label: impl Into<String>) -> Self {
        Self {
            section_id: section_id.into(),
            label: label.into(),
            rows: Vec::new(),
        }
    }

    /// Returns a copy of this section with an appended row.
    pub fn with_row(mut self, row: SettingsRow) -> Self {
        self.rows.push(row);
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

    /// Returns the ordered rows in this section.
    pub fn rows(&self) -> &[SettingsRow] {
        &self.rows
    }
}

/// Top-level presentation model for a settings window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsWindowModel {
    sections: Vec<SettingsSection>,
    selected_section_id: SettingsSectionId,
}

impl SettingsWindowModel {
    /// Creates a model and selects the first section.
    pub fn new(sections: Vec<SettingsSection>) -> Result<Self, SettingsWindowError> {
        let selected_section_id = sections
            .first()
            .map(|section| section.section_id.clone())
            .ok_or(SettingsWindowError::EmptySections)?;

        Self::with_selected_section(sections, selected_section_id)
    }

    /// Creates a model and selects a specific section.
    pub fn with_selected_section(
        sections: Vec<SettingsSection>,
        selected_section_id: impl Into<SettingsSectionId>,
    ) -> Result<Self, SettingsWindowError> {
        validate_sections(&sections)?;

        let selected_section_id = selected_section_id.into();
        if !sections
            .iter()
            .any(|section| section.section_id == selected_section_id)
        {
            return Err(SettingsWindowError::MissingSelectedSection(
                selected_section_id,
            ));
        }

        Ok(Self {
            sections,
            selected_section_id,
        })
    }

    /// Selects another section.
    pub fn select_section(
        &mut self,
        section_id: impl Into<SettingsSectionId>,
    ) -> Result<(), SettingsWindowError> {
        let section_id = section_id.into();
        if !self
            .sections
            .iter()
            .any(|section| section.section_id == section_id)
        {
            return Err(SettingsWindowError::MissingSelectedSection(section_id));
        }

        self.selected_section_id = section_id;
        Ok(())
    }

    /// Returns all ordered sections.
    pub fn sections(&self) -> &[SettingsSection] {
        &self.sections
    }

    /// Returns all rows across all sections in presentation order.
    pub fn rows(&self) -> impl Iterator<Item = &SettingsRow> {
        self.sections.iter().flat_map(SettingsSection::rows)
    }

    /// Returns the currently selected section identifier.
    pub fn selected_section_id(&self) -> &SettingsSectionId {
        &self.selected_section_id
    }

    /// Returns the currently selected section.
    pub fn selected_section(&self) -> &SettingsSection {
        self.sections
            .iter()
            .find(|section| section.section_id == self.selected_section_id)
            .expect("selected section is validated when the model is created or updated")
    }

    /// Returns the rows for the currently selected section.
    pub fn selected_rows(&self) -> &[SettingsRow] {
        self.selected_section().rows()
    }

    /// Finds a row by its field identifier.
    pub fn row(&self, field_id: &SettingsFieldId) -> Option<&SettingsRow> {
        self.sections
            .iter()
            .flat_map(SettingsSection::rows)
            .find(|row| row.field_id() == field_id)
    }

    /// Updates a row value by field identifier.
    pub fn set_row_value(
        &mut self,
        field_id: &SettingsFieldId,
        value: impl Into<String>,
    ) -> Result<(), SettingsWindowError> {
        let Some(row) = self
            .sections
            .iter_mut()
            .flat_map(|section| section.rows.iter_mut())
            .find(|row| row.field_id() == field_id)
        else {
            return Err(SettingsWindowError::MissingField(field_id.clone()));
        };

        row.set_value(value);
        Ok(())
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
        /// Field identifier for the row that owns the action.
        field_id: SettingsFieldId,
        /// Requested action identifier.
        action_id: SettingsRowActionId,
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
    /// A field has an empty identifier.
    EmptyFieldId,
    /// More than one row uses the same field identifier.
    DuplicateFieldId(SettingsFieldId),
    /// A row action has an empty identifier.
    EmptyRowActionId {
        /// Field identifier for the row that owns the invalid action.
        field_id: SettingsFieldId,
    },
    /// More than one action on a row uses the same identifier.
    DuplicateRowActionId {
        /// Field identifier for the row that owns the duplicate action.
        field_id: SettingsFieldId,
        /// Duplicate action identifier.
        action_id: SettingsRowActionId,
    },
    /// A field identifier does not exist in the model.
    MissingField(SettingsFieldId),
    /// The selected section is not present in the model.
    MissingSelectedSection(SettingsSectionId),
}

impl fmt::Display for SettingsWindowError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySections => write!(formatter, "settings model has no sections"),
            Self::EmptySectionId => write!(formatter, "settings section id is empty"),
            Self::DuplicateSectionId(section_id) => {
                write!(formatter, "duplicate settings section id `{section_id}`")
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
            Self::DuplicateRowActionId {
                field_id,
                action_id,
            } => {
                write!(
                    formatter,
                    "duplicate settings row action id `{action_id}` for `{field_id}`"
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
        }
    }
}

impl Error for SettingsWindowError {}

fn validate_sections(sections: &[SettingsSection]) -> Result<(), SettingsWindowError> {
    if sections.is_empty() {
        return Err(SettingsWindowError::EmptySections);
    }

    let mut section_ids = HashSet::new();
    let mut field_ids = HashSet::new();

    for section in sections {
        if section.section_id.as_str().is_empty() {
            return Err(SettingsWindowError::EmptySectionId);
        }

        if !section_ids.insert(section.section_id.clone()) {
            return Err(SettingsWindowError::DuplicateSectionId(
                section.section_id.clone(),
            ));
        }

        for row in section.rows() {
            if row.field_id().as_str().is_empty() {
                return Err(SettingsWindowError::EmptyFieldId);
            }

            if !field_ids.insert(row.field_id().clone()) {
                return Err(SettingsWindowError::DuplicateFieldId(
                    row.field_id().clone(),
                ));
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
        }
    }

    Ok(())
}
