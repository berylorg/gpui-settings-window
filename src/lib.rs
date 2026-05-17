//! Reusable GPUI settings-window primitives.
//!
//! This crate owns the generic presentation and interaction boundary for a
//! settings window: ordered navigation sections, right-pane pages and subpages,
//! optional page-local split lists, setting rows, navigation rows, row and page
//! actions, text field editing, color value picking, apply/accept/cancel events,
//! and a preheated OS window that a host application can show and hide.
//!
//! Host applications own validation, persistence, and apply/cancel semantics.
//! Hosts may observe and close transient in-window popups through
//! `SettingsWindowHandle::has_transient_popups` and
//! `SettingsWindowHandle::close_transient_popups`. Closing transient popups does
//! not hide the settings window, apply settings, cancel settings, or emit
//! host-domain setting events. Hiding the settings window also clears those
//! transient popups so a preheated window cannot reopen with stale popup state.
//!
//! # Example
//!
//! ```
//! use gpui_settings_window::{
//!     SettingsBreadcrumbSegment, SettingsFieldKind, SettingsPage, SettingsPageAction,
//!     SettingsPageActionPriority, SettingsRow, SettingsSection, SettingsWindowModel,
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
//!     ))
//!     .with_row(SettingsRow::navigation(
//!         "theme_editor_link",
//!         "Theme editor",
//!         "theme_editor",
//!     ))
//!     .with_page(
//!         SettingsPage::new("theme_editor", "Theme editor")
//!             .with_breadcrumb_segment(SettingsBreadcrumbSegment::linked(
//!                 "Appearance",
//!                 "appearance",
//!             ))
//!             .with_back_target("appearance")
//!             .with_action(
//!                 SettingsPageAction::new("save", "Save")
//!                     .with_priority(SettingsPageActionPriority::Primary),
//!             ),
//!     );
//!
//! let model = SettingsWindowModel::new(vec![appearance]).expect("valid settings model");
//!
//! assert_eq!(model.selected_section().label(), "Appearance");
//! ```

mod color;
mod color_picker;
mod diagnostics;
mod input;
mod model;
mod options;
mod panel;
mod theme;
mod window;

pub use color::RgbColor;
pub use diagnostics::{
    SettingsWindowDiagnostics, SettingsWindowPerformanceDiagnostics,
    SettingsWindowRangeDiagnostics, SettingsWindowRowSurfaceDiagnostics,
};
pub use model::{
    MAX_PAGE_DETAIL_ROWS, SettingsActionAvailability, SettingsBreadcrumbSegment,
    SettingsChoiceOption, SettingsFieldId, SettingsFieldKind, SettingsPage, SettingsPageAction,
    SettingsPageActionId, SettingsPageActionPriority, SettingsPageId, SettingsPageSplit,
    SettingsPageSplitItem, SettingsPageSplitItemId, SettingsPageSplitItemPreviewStyle, SettingsRow,
    SettingsRowAction, SettingsRowActionId, SettingsRowDetailField, SettingsRowKind,
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
