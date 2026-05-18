use std::fmt;

macro_rules! stable_id {
    ($(#[$meta:meta])* $name:ident, $label:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(String);

        impl $name {
            /// Creates an identifier.
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Returns the identifier as a string slice.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

stable_id!(
    /// Stable identifier for a settings section.
    SettingsSectionId,
    "settings section"
);

stable_id!(
    /// Stable identifier for a right-pane settings page.
    SettingsPageId,
    "settings page"
);

stable_id!(
    /// Stable identifier for an item in a page-local split list.
    SettingsPageSplitItemId,
    "settings page split item"
);

stable_id!(
    /// Stable identifier for a page-owned custom body region.
    SettingsPageCustomBodyId,
    "settings page custom body"
);

stable_id!(
    /// Stable identifier for a settings row or editable field.
    SettingsFieldId,
    "settings field"
);

stable_id!(
    /// Stable identifier for an action attached to a settings row.
    SettingsRowActionId,
    "settings row action"
);

stable_id!(
    /// Stable identifier for an action attached to a settings page.
    SettingsPageActionId,
    "settings page action"
);
