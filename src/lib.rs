//! Reusable GPUI settings-window primitives.
//!
//! This crate owns the generic presentation and interaction boundary for a
//! settings window: ordered navigation sections, right-pane pages and subpages,
//! optional revision-bound paged page-local split sources, setting rows, navigation rows, row and page
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
//!     SettingsPageActionPriority, SettingsPageSplitSelection, SettingsPageSplitSource,
//!     SettingsPageSplitSourceKey, SettingsRow, SettingsSection, SettingsWindowModel,
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
//!             .with_paged_split_source(
//!                 SettingsPageSplitSource::new(
//!                     SettingsPageSplitSourceKey::new("themes", 1, 7),
//!                     50_000,
//!                     32,
//!                     16 * 1024,
//!                 )
//!                 .with_selected(SettingsPageSplitSelection::new("default", 0)),
//!             )
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
//! assert_eq!(
//!     model.page(&"theme_editor".into())
//!         .unwrap()
//!         .paged_split_source()
//!         .unwrap()
//!         .logical_item_count(),
//!     50_000,
//! );
//! ```

//! # Paged split work lifecycle
//!
//! Retain `SettingsWindowHandle::page_split_work_receiver` independently of the GPUI entity.
//! Drain `Page` work in order, fetch only its exact source key and range, and publish exactly one
//! matching ready, failed, or cancelled result through
//! `SettingsWindowHandle::deliver_page_split_result`. A request carrying a focus probe also
//! requires an exact `Found` or `Removed` resolution. Stop the corresponding host fetch for
//! `Cancel`, and discard host data retained for the exact request on `Release`. Both terminal work
//! items remain drainable from the retained receiver after panel teardown.
//!
//! The following transport loop illustrates the complete data-only part of that contract:
//!
//! ```
//! use gpui_settings_window::{
//!     SettingsPageSplitFocusResolution, SettingsPageSplitItem, SettingsPageSplitPageRequest,
//!     SettingsPageSplitPageResult, SettingsPageSplitRequestId, SettingsPageSplitWork,
//!     SettingsPageSplitWorkReceiver,
//! };
//!
//! fn service_split_work(
//!     receiver: &SettingsPageSplitWorkReceiver,
//!     mut fetch: impl FnMut(
//!         &SettingsPageSplitPageRequest,
//!     ) -> (
//!         usize,
//!         Vec<SettingsPageSplitItem>,
//!         Option<SettingsPageSplitFocusResolution>,
//!     ),
//!     mut publish: impl FnMut(SettingsPageSplitPageResult),
//!     mut cancel: impl FnMut(SettingsPageSplitRequestId),
//!     mut release: impl FnMut(SettingsPageSplitRequestId),
//! ) {
//!     while let Some(work) = receiver.take_work() {
//!         match work {
//!             SettingsPageSplitWork::Page(request) => {
//!                 let (logical_count, items, focus_resolution) = fetch(&request);
//!                 let mut result =
//!                     SettingsPageSplitPageResult::ready(request, logical_count, items);
//!                 if let Some(resolution) = focus_resolution {
//!                     result = result.with_focus_resolution(resolution);
//!                 }
//!                 publish(result);
//!             }
//!             SettingsPageSplitWork::Cancel(request) => cancel(request.request_id()),
//!             SettingsPageSplitWork::Release(request) => release(request.request_id()),
//!         }
//!     }
//! }
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
    SettingsWindowSplitPagerDiagnostics,
};
pub use model::{
    MAX_PAGE_DETAIL_ROWS, MAX_PAGE_SPLIT_ACTIVE_PAGES, MAX_PAGE_SPLIT_WORK_ITEMS,
    SettingsActionAvailability, SettingsBreadcrumbSegment, SettingsChoiceOption, SettingsFieldId,
    SettingsFieldKind, SettingsPage, SettingsPageAction, SettingsPageActionId,
    SettingsPageActionPriority, SettingsPageBodyLayout, SettingsPageCustomBody,
    SettingsPageCustomBodyId, SettingsPageId, SettingsPageSplitDelivery,
    SettingsPageSplitDeliveryError, SettingsPageSplitFocusProbe, SettingsPageSplitFocusResolution,
    SettingsPageSplitItem, SettingsPageSplitItemId, SettingsPageSplitItemPreviewStyle,
    SettingsPageSplitPageFailure, SettingsPageSplitPageOutcome, SettingsPageSplitPageRequest,
    SettingsPageSplitPageResult, SettingsPageSplitRequestId, SettingsPageSplitSelection,
    SettingsPageSplitSource, SettingsPageSplitSourceId, SettingsPageSplitSourceKey,
    SettingsPageSplitWork, SettingsPageSplitWorkReceiver, SettingsRow, SettingsRowAction,
    SettingsRowActionId, SettingsRowDetailField, SettingsRowKind, SettingsSavedColorSwatchId,
    SettingsSection, SettingsSectionId, SettingsWindowError, SettingsWindowEvent,
    SettingsWindowModel,
};
pub use options::{
    MAX_SAVED_COLOR_SWATCHES, SettingsPageBodyRenderer, SettingsSavedColorSwatch,
    SettingsWindowOptions, SettingsWindowOptionsError,
};
pub use panel::SettingsPanel;
pub use theme::{
    SettingsButtonStateTheme, SettingsButtonTheme, SettingsInputTheme, SettingsSurfaceTheme,
    SettingsWindowTheme,
};
pub use window::{
    SettingsWindowHandle, SettingsWindowOpenDisposition, SettingsWindowView, open_settings_window,
};
