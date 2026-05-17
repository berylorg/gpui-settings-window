use super::{SettingsFieldId, SettingsPageId, SettingsRowAction};

/// Presentation kind for an editable settings field row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SettingsFieldKind {
    /// A plain single-line text value.
    Text,
    /// A plain numeric-looking single-line value with compact presentation width.
    Number,
    /// A plain multiline text value.
    MultilineText,
    /// A canonical RGB hex color value such as `#6699cc`.
    Color,
    /// A choice value selected from host-supplied string options.
    Choice,
}

/// App-neutral presentation role for a right-pane row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsRowKind {
    /// An editable field row with a string presentation value.
    Field,
    /// A row that requests navigation to another page.
    Navigation {
        /// Target page identifier supplied by the host model.
        target_page_id: SettingsPageId,
    },
    /// A row that presents one or more actions without an editable field.
    ActionOnly,
}

/// One selectable option for a choice settings field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsChoiceOption {
    value: String,
    label: String,
}

/// Optional secondary editable field rendered inside the same settings row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsRowDetailField {
    field_id: SettingsFieldId,
    value: String,
    kind: SettingsFieldKind,
    error: Option<String>,
    modified: bool,
    choices: Vec<SettingsChoiceOption>,
}

/// One app-neutral row in a right-pane settings page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsRow {
    field_id: SettingsFieldId,
    label: String,
    subtext: Option<String>,
    value: String,
    kind: SettingsFieldKind,
    row_kind: SettingsRowKind,
    error: Option<String>,
    modified: bool,
    actions: Vec<SettingsRowAction>,
    choices: Vec<SettingsChoiceOption>,
    detail_field: Option<SettingsRowDetailField>,
}

impl SettingsChoiceOption {
    /// Creates one choice option with a stable value and display label.
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }

    /// Returns the option value emitted in field-change events.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Returns the option display label.
    pub fn label(&self) -> &str {
        &self.label
    }
}

impl SettingsRow {
    /// Creates an editable settings field row.
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
            row_kind: SettingsRowKind::Field,
            error: None,
            modified: false,
            actions: Vec::new(),
            choices: Vec::new(),
            detail_field: None,
        }
    }

    /// Creates a navigation row that targets another page.
    pub fn navigation(
        field_id: impl Into<SettingsFieldId>,
        label: impl Into<String>,
        target_page_id: impl Into<SettingsPageId>,
    ) -> Self {
        Self {
            field_id: field_id.into(),
            label: label.into(),
            subtext: None,
            value: String::new(),
            kind: SettingsFieldKind::Text,
            row_kind: SettingsRowKind::Navigation {
                target_page_id: target_page_id.into(),
            },
            error: None,
            modified: false,
            actions: Vec::new(),
            choices: Vec::new(),
            detail_field: None,
        }
    }

    /// Creates an action-only row with its first action.
    pub fn action_only(
        field_id: impl Into<SettingsFieldId>,
        label: impl Into<String>,
        action: SettingsRowAction,
    ) -> Self {
        Self {
            field_id: field_id.into(),
            label: label.into(),
            subtext: None,
            value: String::new(),
            kind: SettingsFieldKind::Text,
            row_kind: SettingsRowKind::ActionOnly,
            error: None,
            modified: false,
            actions: vec![action],
            choices: Vec::new(),
            detail_field: None,
        }
    }

    /// Returns a copy of this row with secondary label-side subtext.
    pub fn with_subtext(mut self, subtext: impl Into<String>) -> Self {
        let subtext = subtext.into();
        self.subtext = (!subtext.is_empty()).then_some(subtext);
        self
    }

    /// Returns a copy of this row with an attached validation or status message.
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

    /// Returns a copy of this row with host-supplied modified presentation state.
    pub fn with_modified(mut self, modified: bool) -> Self {
        self.modified = modified;
        self
    }

    /// Returns a copy of this row with an appended app-neutral action.
    pub fn with_action(mut self, action: SettingsRowAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Returns a copy of this row with an appended choice option.
    pub fn with_choice(mut self, option: SettingsChoiceOption) -> Self {
        self.choices.push(option);
        self
    }

    /// Returns a copy of this row with an optional secondary field in the same visual surface.
    pub fn with_detail_field(mut self, field: SettingsRowDetailField) -> Self {
        self.detail_field = Some(field);
        self
    }

    /// Returns the row's stable field or row identifier.
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

    /// Returns the editable field kind used by field rows.
    pub fn kind(&self) -> SettingsFieldKind {
        self.kind
    }

    /// Returns the row presentation role.
    pub fn row_kind(&self) -> &SettingsRowKind {
        &self.row_kind
    }

    /// Returns whether this row owns an editable field.
    pub fn is_field(&self) -> bool {
        matches!(self.row_kind, SettingsRowKind::Field)
    }

    /// Returns the target page for a navigation row.
    pub fn navigation_target_page_id(&self) -> Option<&SettingsPageId> {
        match &self.row_kind {
            SettingsRowKind::Navigation { target_page_id } => Some(target_page_id),
            SettingsRowKind::Field | SettingsRowKind::ActionOnly => None,
        }
    }

    /// Returns the optional validation or status message.
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Returns the host-supplied modified presentation state.
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Returns the ordered row actions.
    pub fn actions(&self) -> &[SettingsRowAction] {
        &self.actions
    }

    /// Returns ordered choices for a choice field row.
    pub fn choices(&self) -> &[SettingsChoiceOption] {
        &self.choices
    }

    /// Returns the optional secondary field rendered in this row.
    pub fn detail_field(&self) -> Option<&SettingsRowDetailField> {
        self.detail_field.as_ref()
    }

    pub(crate) fn detail_field_mut(&mut self) -> Option<&mut SettingsRowDetailField> {
        self.detail_field.as_mut()
    }

    pub(crate) fn uses_text_input(&self) -> bool {
        self.is_field() && self.kind != SettingsFieldKind::Choice
    }

    pub(crate) fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
    }
}

impl SettingsRowDetailField {
    /// Creates a secondary editable field rendered inside a settings row.
    pub fn new(
        field_id: impl Into<SettingsFieldId>,
        value: impl Into<String>,
        kind: SettingsFieldKind,
    ) -> Self {
        Self {
            field_id: field_id.into(),
            value: value.into(),
            kind,
            error: None,
            modified: false,
            choices: Vec::new(),
        }
    }

    /// Returns a copy of this detail field with validation or status text.
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Returns a copy of this detail field with host-supplied modified presentation state.
    pub fn with_modified(mut self, modified: bool) -> Self {
        self.modified = modified;
        self
    }

    /// Returns a copy of this detail field with an appended choice option.
    pub fn with_choice(mut self, option: SettingsChoiceOption) -> Self {
        self.choices.push(option);
        self
    }

    /// Returns this detail field's stable field identifier.
    pub fn field_id(&self) -> &SettingsFieldId {
        &self.field_id
    }

    /// Returns this detail field's presentation value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Returns this detail field's presentation kind.
    pub fn kind(&self) -> SettingsFieldKind {
        self.kind
    }

    /// Returns optional validation or status text.
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Returns the host-supplied modified presentation state.
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Returns ordered choices for this detail field.
    pub fn choices(&self) -> &[SettingsChoiceOption] {
        &self.choices
    }

    pub(crate) fn uses_text_input(&self) -> bool {
        self.kind != SettingsFieldKind::Choice
    }

    pub(crate) fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
    }
}
