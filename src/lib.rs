//! Reusable GPUI settings-window primitives.
//!
//! This crate owns the generic presentation and interaction boundary for a
//! settings window: ordered navigation sections, ordered setting rows, text
//! field editing, color value picking, apply/accept/cancel events, and a
//! preheated OS window that a host application can show and hide.
//!
//! Host applications own validation, persistence, and apply/cancel semantics.
//!
//! # Example
//!
//! ```
//! use gpui_settings_window::{
//!     SettingsFieldKind, SettingsRow, SettingsSection, SettingsWindowModel,
//! };
//!
//! let appearance = SettingsSection::new("appearance", "Appearance")
//!     .with_row(SettingsRow::new(
//!         "accent_color",
//!         "Accent color",
//!         "#6699cc",
//!         SettingsFieldKind::Color,
//!     ))
//!     .with_row(SettingsRow::new(
//!         "notes",
//!         "Notes",
//!         "One note\nAnother note",
//!         SettingsFieldKind::MultilineText,
//!     ));
//!
//! let model = SettingsWindowModel::new(vec![appearance]).expect("valid settings model");
//!
//! assert_eq!(model.selected_section().label(), "Appearance");
//! ```

mod color;
mod color_picker;
mod input;
mod model;
mod options;
mod panel;
mod theme;
mod window;

pub use color::RgbColor;
pub use model::{
    SettingsFieldId, SettingsFieldKind, SettingsRow, SettingsRowAction, SettingsRowActionId,
    SettingsSection, SettingsSectionId, SettingsWindowError, SettingsWindowEvent,
    SettingsWindowModel,
};
pub use options::SettingsWindowOptions;
pub use panel::SettingsPanel;
pub use theme::{
    SettingsButtonStateTheme, SettingsButtonTheme, SettingsInputTheme, SettingsSurfaceTheme,
    SettingsWindowTheme,
};
pub use window::{
    SettingsWindowHandle, SettingsWindowOpenDisposition, SettingsWindowView, open_settings_window,
};
