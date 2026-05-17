use super::{SettingsPageActionId, SettingsRowActionId};

/// App-neutral enabled or disabled presentation state for an action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsActionAvailability {
    /// The action is available and may emit a request event.
    Enabled,
    /// The action remains visible but must not emit its normal request event.
    Disabled {
        /// Optional host-supplied reason for the disabled state.
        reason: Option<String>,
    },
}

impl SettingsActionAvailability {
    /// Returns an enabled action state.
    pub fn enabled() -> Self {
        Self::Enabled
    }

    /// Returns a disabled action state without a reason.
    pub fn disabled() -> Self {
        Self::Disabled { reason: None }
    }

    /// Returns a disabled action state with a reason.
    pub fn disabled_with_reason(reason: impl Into<String>) -> Self {
        Self::Disabled {
            reason: Some(reason.into()),
        }
    }

    /// Returns whether this action may emit its normal request event.
    pub fn is_enabled(&self) -> bool {
        matches!(self, Self::Enabled)
    }

    /// Returns the optional disabled reason.
    pub fn disabled_reason(&self) -> Option<&str> {
        match self {
            Self::Enabled => None,
            Self::Disabled { reason } => reason.as_deref(),
        }
    }
}

/// Visual priority for a page-level action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsPageActionPriority {
    /// Primary page action.
    Primary,
    /// Secondary page action.
    Secondary,
}

/// App-neutral action presented for a settings row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsRowAction {
    action_id: SettingsRowActionId,
    label: String,
    availability: SettingsActionAvailability,
}

impl SettingsRowAction {
    /// Creates an enabled row action.
    pub fn new(action_id: impl Into<SettingsRowActionId>, label: impl Into<String>) -> Self {
        Self {
            action_id: action_id.into(),
            label: label.into(),
            availability: SettingsActionAvailability::enabled(),
        }
    }

    /// Returns a copy of this action in a disabled state without a reason.
    pub fn disabled(mut self) -> Self {
        self.availability = SettingsActionAvailability::disabled();
        self
    }

    /// Returns a copy of this action in a disabled state with a reason.
    pub fn disabled_with_reason(mut self, reason: impl Into<String>) -> Self {
        self.availability = SettingsActionAvailability::disabled_with_reason(reason);
        self
    }

    /// Returns the row action's stable identifier.
    pub fn action_id(&self) -> &SettingsRowActionId {
        &self.action_id
    }

    /// Returns the row action display label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the row action availability.
    pub fn availability(&self) -> &SettingsActionAvailability {
        &self.availability
    }

    /// Returns whether the row action may emit a request event.
    pub fn is_enabled(&self) -> bool {
        self.availability.is_enabled()
    }

    /// Returns the optional disabled reason.
    pub fn disabled_reason(&self) -> Option<&str> {
        self.availability.disabled_reason()
    }
}

/// App-neutral action presented for the current settings page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsPageAction {
    action_id: SettingsPageActionId,
    label: String,
    priority: SettingsPageActionPriority,
    availability: SettingsActionAvailability,
}

impl SettingsPageAction {
    /// Creates an enabled secondary page action.
    pub fn new(action_id: impl Into<SettingsPageActionId>, label: impl Into<String>) -> Self {
        Self {
            action_id: action_id.into(),
            label: label.into(),
            priority: SettingsPageActionPriority::Secondary,
            availability: SettingsActionAvailability::enabled(),
        }
    }

    /// Returns a copy of this action with a visual priority.
    pub fn with_priority(mut self, priority: SettingsPageActionPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Returns a copy of this action in a disabled state without a reason.
    pub fn disabled(mut self) -> Self {
        self.availability = SettingsActionAvailability::disabled();
        self
    }

    /// Returns a copy of this action in a disabled state with a reason.
    pub fn disabled_with_reason(mut self, reason: impl Into<String>) -> Self {
        self.availability = SettingsActionAvailability::disabled_with_reason(reason);
        self
    }

    /// Returns the page action's stable identifier.
    pub fn action_id(&self) -> &SettingsPageActionId {
        &self.action_id
    }

    /// Returns the page action display label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the page action visual priority.
    pub fn priority(&self) -> SettingsPageActionPriority {
        self.priority
    }

    /// Returns whether the page action may emit a request event.
    pub fn is_enabled(&self) -> bool {
        self.availability.is_enabled()
    }

    /// Returns the optional disabled reason.
    pub fn disabled_reason(&self) -> Option<&str> {
        self.availability.disabled_reason()
    }
}
