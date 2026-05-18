use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::ops::Range;
use std::rc::Rc;
use std::time::{Duration, Instant};

use gpui::{
    App, Bounds, Context, Entity, EventEmitter, FocusHandle, FontWeight, IntoElement, MouseButton,
    ParentElement, Pixels, Render, ScrollHandle, SharedString, Window, actions, div, point,
    prelude::*, px, rgb,
};
use gpui_scrollbar::ScrollbarVisibilityState;

use crate::color_picker::{
    ColorComponentInput, ColorPickerChannelField, ensure_color_component_input_bindings,
};
use crate::input::{
    SettingsFieldInput, SettingsFieldInputEvent, SettingsFieldInputRole,
    ensure_settings_input_bindings,
};
use crate::model::{
    SettingsFieldId, SettingsFieldKind, SettingsPageActionId, SettingsPageId,
    SettingsPageSplitItemId, SettingsRow, SettingsRowActionId, SettingsSectionId,
    SettingsWindowEvent, SettingsWindowModel, element_id_suffix,
};
use crate::{
    RgbColor, SettingsPageBodyRenderer, SettingsWindowDiagnostics, SettingsWindowOptions,
    SettingsWindowPerformanceDiagnostics, SettingsWindowRangeDiagnostics,
    SettingsWindowRowSurfaceDiagnostics, SettingsWindowTheme,
};

mod color_paint;
mod color_pointer;
mod color_render;
mod picker;
mod render;
mod scrollbar;
mod test_support;

const PANEL_KEY_CONTEXT: &str = "GpuiSettingsWindowPanel";
const NAVIGATION_WIDTH: f32 = 196.0;
const DEFAULT_FONT_SIZE: f32 = 14.0;
const PAGE_LOCAL_SPLIT_ITEM_HEIGHT: f32 = 88.0;
const PAGE_LOCAL_SPLIT_ITEM_GAP: f32 = 4.0;
const PAGE_LOCAL_SPLIT_OVERSCAN_ROWS: usize = 3;
const PAGE_LOCAL_SPLIT_FALLBACK_VIEWPORT_HEIGHT: f32 = 360.0;

fn theme_color(color: RgbColor) -> gpui::Rgba {
    rgb(color.packed_rgb())
}

actions!(gpui_settings_window_panel, [Cancel, FocusNext, FocusPrev]);

struct ColorPickerChannelSlot {
    field: ColorPickerChannelField,
    input: Entity<ColorComponentInput>,
}

#[derive(Clone)]
struct SettingsFieldSnapshot {
    field_id: SettingsFieldId,
    value: String,
    kind: SettingsFieldKind,
    error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColorPickerDragTarget {
    MainPalette,
    NeutralStrip,
    LightnessBar,
}

#[derive(Clone, Copy, Debug, Default)]
struct SettingsPanelColorLookupCounts {
    color_preview_lookup_count: u64,
    color_model_lookup_count: u64,
}

#[derive(Clone, Copy, Debug, Default)]
struct SettingsPanelDiagnosticsState {
    render_count: u64,
    last_render_tree_micros: u64,
    model_sync_count: u64,
    last_model_sync_micros: u64,
    option_sync_count: u64,
    last_option_sync_micros: u64,
    input_sync_count: u64,
    last_input_sync_entity_count: usize,
    color_preview_lookup_count: u64,
    last_render_color_preview_lookup_count: u64,
    color_model_lookup_count: u64,
    last_render_color_model_lookup_count: u64,
}

/// GPUI entity that renders the reusable settings panel.
pub struct SettingsPanel {
    model: SettingsWindowModel,
    fields: HashMap<SettingsFieldId, Entity<SettingsFieldInput>>,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    navigation_scroll_handle: ScrollHandle,
    split_scroll_handle: ScrollHandle,
    content_scrollbar_visibility: ScrollbarVisibilityState,
    navigation_scrollbar_visibility: ScrollbarVisibilityState,
    split_scrollbar_visibility: ScrollbarVisibilityState,
    font_size: f32,
    visual_theme: SettingsWindowTheme,
    page_body_renderer: Option<SettingsPageBodyRenderer>,
    saved_color_swatches: Vec<RgbColor>,
    text_input_undo_byte_limit: usize,
    latest_known_color_values: HashMap<SettingsFieldId, RgbColor>,
    diagnostics: RefCell<SettingsPanelDiagnosticsState>,
    choice_popup_field: Option<SettingsFieldId>,
    choice_control_bounds: HashMap<SettingsFieldId, Bounds<Pixels>>,
    color_picker_field: Option<SettingsFieldId>,
    color_picker_input: Option<Entity<SettingsFieldInput>>,
    color_picker_channel_inputs: Vec<ColorPickerChannelSlot>,
    color_picker_channel_empty_drafts: BTreeMap<ColorPickerChannelField, String>,
    color_picker_focused_channel: Option<ColorPickerChannelField>,
    color_picker_preview_color: Option<RgbColor>,
    color_picker_drag_target: Option<ColorPickerDragTarget>,
    color_picker_pending_outside_mouse_up_field: Option<SettingsFieldId>,
    color_picker_main_palette_bounds: Rc<RefCell<Option<Bounds<Pixels>>>>,
    color_picker_neutral_strip_bounds: Rc<RefCell<Option<Bounds<Pixels>>>>,
    color_picker_lightness_bar_bounds: Rc<RefCell<Option<Bounds<Pixels>>>>,
}

impl EventEmitter<SettingsWindowEvent> for SettingsPanel {}

impl SettingsWindowModel {
    fn text_input_field_snapshots(&self) -> Vec<SettingsFieldSnapshot> {
        self.rows()
            .flat_map(|row| {
                let mut fields = Vec::with_capacity(2);
                if row.uses_text_input() {
                    fields.push(SettingsFieldSnapshot {
                        field_id: row.field_id().clone(),
                        value: row.value().to_owned(),
                        kind: row.kind(),
                        error: row.error().map(str::to_owned),
                    });
                }
                if let Some(field) = row.detail_field().filter(|field| field.uses_text_input()) {
                    fields.push(SettingsFieldSnapshot {
                        field_id: field.field_id().clone(),
                        value: field.value().to_owned(),
                        kind: field.kind(),
                        error: field.error().map(str::to_owned),
                    });
                }
                fields
            })
            .collect()
    }

    fn selected_text_input_field_ids(&self) -> Vec<SettingsFieldId> {
        self.selected_rows()
            .iter()
            .flat_map(|row| {
                let mut fields = Vec::with_capacity(2);
                if row.uses_text_input() {
                    fields.push(row.field_id().clone());
                }
                if let Some(field) = row.detail_field().filter(|field| field.uses_text_input()) {
                    fields.push(field.field_id().clone());
                }
                fields
            })
            .collect()
    }
}

impl SettingsPanel {
    /// Creates a settings panel from a presentation model.
    pub fn new(model: SettingsWindowModel, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::new_with_options(model, SettingsWindowOptions::default(), window, cx)
    }

    /// Creates a settings panel from a presentation model and saved color swatches.
    pub fn new_with_saved_color_swatches(
        model: SettingsWindowModel,
        saved_color_swatches: Vec<RgbColor>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::new_with_options(
            model,
            SettingsWindowOptions::default().with_saved_color_swatches(saved_color_swatches),
            window,
            cx,
        )
    }

    /// Creates a settings panel from a presentation model and visual options.
    pub fn new_with_options(
        model: SettingsWindowModel,
        options: SettingsWindowOptions,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        ensure_settings_panel_bindings(cx);
        ensure_color_component_input_bindings(cx);
        let mut panel = Self {
            model,
            fields: HashMap::new(),
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
            navigation_scroll_handle: ScrollHandle::new(),
            split_scroll_handle: ScrollHandle::new(),
            content_scrollbar_visibility: ScrollbarVisibilityState::default(),
            navigation_scrollbar_visibility: ScrollbarVisibilityState::default(),
            split_scrollbar_visibility: ScrollbarVisibilityState::default(),
            font_size: DEFAULT_FONT_SIZE,
            visual_theme: options.visual_theme().clone(),
            page_body_renderer: options.page_body_renderer().cloned(),
            saved_color_swatches: options.saved_color_swatches().to_vec(),
            text_input_undo_byte_limit: options.text_input_undo_byte_limit(),
            latest_known_color_values: HashMap::new(),
            diagnostics: RefCell::new(SettingsPanelDiagnosticsState::default()),
            choice_popup_field: None,
            choice_control_bounds: HashMap::new(),
            color_picker_field: None,
            color_picker_input: None,
            color_picker_channel_inputs: Vec::new(),
            color_picker_channel_empty_drafts: BTreeMap::new(),
            color_picker_focused_channel: None,
            color_picker_preview_color: None,
            color_picker_drag_target: None,
            color_picker_pending_outside_mouse_up_field: None,
            color_picker_main_palette_bounds: Rc::new(RefCell::new(None)),
            color_picker_neutral_strip_bounds: Rc::new(RefCell::new(None)),
            color_picker_lightness_bar_bounds: Rc::new(RefCell::new(None)),
        };
        panel.sync_latest_known_color_values();
        panel.rebuild_fields(window, cx);
        panel.sync_split_scroll_for_model(None, None);
        panel
    }

    /// Synchronizes the panel to a new presentation model.
    pub fn sync_model(
        &mut self,
        model: SettingsWindowModel,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let started = Instant::now();
        let previous_section = self.model.selected_section_id().clone();
        let previous_page = self.model.selected_page_id().clone();
        let previous_content_offset = self.scroll_handle.offset();
        let previous_split_selection = selected_page_split_selection(&self.model);
        self.model = model;
        let selected_page_changed = previous_page != *self.model.selected_page_id();
        self.sync_latest_known_color_values();
        let input_sync_entity_count = self.sync_fields(window, cx);
        if !selected_page_changed {
            self.sync_choice_popup();
            self.sync_color_picker(cx);
        }
        self.sync_split_scroll_for_model(Some(previous_page), previous_split_selection);

        if selected_page_changed {
            self.reset_selected_page_view_state(window, cx);
        } else if previous_section != *self.model.selected_section_id() {
            self.reset_selected_page_view_state(window, cx);
        } else {
            self.scroll_handle.set_offset(previous_content_offset);
        }

        self.record_model_sync_diagnostics(started.elapsed(), input_sync_entity_count);
        cx.notify();
    }

    /// Returns the current presentation model.
    pub fn model(&self) -> &SettingsWindowModel {
        &self.model
    }

    /// Returns a content-free diagnostics snapshot for host-owned profiling.
    pub fn diagnostics_snapshot(&self, visible: bool) -> SettingsWindowDiagnostics {
        let selected_page = self.model.selected_page();
        let selected_row_count = self.model.selected_rows().len();
        let detail_rows = SettingsWindowRowSurfaceDiagnostics {
            surface_id: "selected_page_detail_rows".to_string(),
            total_row_count: selected_row_count,
            rendered_row_count: selected_row_count,
            visible_range: None,
            overscan_count: 0,
            row_height_strategy: "full_selected_page".to_string(),
        };
        let split_list = selected_page.local_split().map(|split| {
            let item_count = split.items().len();
            let range = page_local_split_render_window(
                item_count,
                self.split_scroll_handle.offset().y,
                self.split_scroll_handle.bounds().size.height,
            );
            SettingsWindowRowSurfaceDiagnostics {
                surface_id: "page_local_split_list".to_string(),
                total_row_count: item_count,
                rendered_row_count: range.len(),
                visible_range: Some(SettingsWindowRangeDiagnostics::from_range(range)),
                overscan_count: PAGE_LOCAL_SPLIT_OVERSCAN_ROWS,
                row_height_strategy: "fixed_height_windowed".to_string(),
            }
        });

        SettingsWindowDiagnostics {
            visible,
            selected_section_id: self.model.selected_section_id().as_str().to_string(),
            selected_page_id: selected_page.page_id().as_str().to_string(),
            detail_rows,
            split_list,
            performance: self.performance_diagnostics(),
        }
    }

    /// Returns the current visual theme.
    pub fn visual_theme(&self) -> &SettingsWindowTheme {
        &self.visual_theme
    }

    /// Synchronizes panel-level visual options.
    pub fn sync_options(
        &mut self,
        options: &SettingsWindowOptions,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let started = Instant::now();
        let next_theme = options.visual_theme().clone();
        let next_page_body_renderer = options.page_body_renderer().cloned();
        let next_swatches = options.saved_color_swatches().to_vec();
        let next_text_input_undo_byte_limit = options.text_input_undo_byte_limit();

        let theme_changed = self.visual_theme != next_theme;
        let page_body_renderer_changed = self.page_body_renderer != next_page_body_renderer;
        let swatches_changed = self.saved_color_swatches != next_swatches;
        let text_input_undo_byte_limit_changed =
            self.text_input_undo_byte_limit != next_text_input_undo_byte_limit;

        if theme_changed {
            self.visual_theme = next_theme;
            self.sync_input_visual_themes(cx);
        }
        if page_body_renderer_changed {
            self.page_body_renderer = next_page_body_renderer;
        }
        if swatches_changed {
            self.saved_color_swatches = next_swatches;
        }
        if text_input_undo_byte_limit_changed {
            self.text_input_undo_byte_limit = next_text_input_undo_byte_limit;
            self.sync_input_retention_options(window, cx);
        }
        if theme_changed
            || page_body_renderer_changed
            || swatches_changed
            || text_input_undo_byte_limit_changed
        {
            cx.notify();
        }
        self.record_option_sync_diagnostics(started.elapsed());
    }

    /// Focuses the first field in the selected page.
    pub fn focus_first_field(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_selected_section_field(window, cx);
    }

    /// Focuses the panel itself.
    pub fn focus_panel(&self, window: &mut Window) {
        window.focus(&self.focus_handle);
    }

    /// Focuses a field by identifier.
    pub fn focus_field(
        &mut self,
        field_id: &SettingsFieldId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(input) = self.input_for_field(field_id) else {
            return false;
        };
        input.update(cx, |input, cx| input.focus(window, cx));
        true
    }

    pub(crate) fn reset_scrollbar_visibility(&mut self, cx: &mut Context<Self>) {
        self.content_scrollbar_visibility = ScrollbarVisibilityState::default();
        self.navigation_scrollbar_visibility = ScrollbarVisibilityState::default();
        self.split_scrollbar_visibility = ScrollbarVisibilityState::default();
        cx.notify();
    }

    /// Returns the current vertical scroll metrics.
    pub fn scroll_metrics(&self) -> (f32, f32) {
        (
            f32::from(self.scroll_handle.offset().y),
            f32::from(self.scroll_handle.max_offset().height),
        )
    }

    /// Returns whether any transient settings popup is open.
    pub fn has_transient_popups(&self) -> bool {
        self.color_picker_field.is_some() || self.choice_popup_field.is_some()
    }

    /// Closes transient settings popups without applying or canceling settings.
    pub fn close_transient_popups(&mut self, cx: &mut Context<Self>) -> bool {
        let had_popup = self.has_transient_popups();
        self.choice_popup_field = None;
        self.close_color_picker(cx);
        if had_popup {
            cx.notify();
        }
        had_popup
    }

    fn performance_diagnostics(&self) -> SettingsWindowPerformanceDiagnostics {
        self.diagnostics.borrow().performance_diagnostics()
    }

    fn diagnostic_color_lookup_counts(&self) -> SettingsPanelColorLookupCounts {
        self.diagnostics.borrow().color_lookup_counts()
    }

    fn record_render_diagnostics(
        &self,
        duration: Duration,
        before: SettingsPanelColorLookupCounts,
    ) {
        self.diagnostics
            .borrow_mut()
            .record_render(duration, before);
    }

    fn record_model_sync_diagnostics(&self, duration: Duration, input_sync_entity_count: usize) {
        self.diagnostics
            .borrow_mut()
            .record_model_sync(duration, input_sync_entity_count);
    }

    fn record_option_sync_diagnostics(&self, duration: Duration) {
        self.diagnostics.borrow_mut().record_option_sync(duration);
    }

    fn record_input_sync_diagnostics(&self, input_sync_entity_count: usize) {
        self.diagnostics
            .borrow_mut()
            .record_input_sync(input_sync_entity_count);
    }

    fn record_color_preview_lookup(&self) {
        let mut diagnostics = self.diagnostics.borrow_mut();
        diagnostics.color_preview_lookup_count =
            diagnostics.color_preview_lookup_count.saturating_add(1);
    }

    fn record_color_model_lookup(&self) {
        let mut diagnostics = self.diagnostics.borrow_mut();
        diagnostics.color_model_lookup_count =
            diagnostics.color_model_lookup_count.saturating_add(1);
    }

    /// Replaces a field's text directly and emits the same event as user input.
    pub fn replace_field_text_for_test(
        &mut self,
        field_id: &SettingsFieldId,
        value: &str,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(input) = self.input_for_field(field_id) else {
            return false;
        };
        input.update(cx, |input, cx| {
            input.replace_all_for_test(value, cx);
        });
        true
    }

    /// Reads the synchronized text for a field input.
    pub fn field_text_for_test(&self, field_id: &SettingsFieldId, cx: &App) -> Option<String> {
        self.input_for_field(field_id)
            .map(|input| input.read(cx).text().to_owned())
    }

    pub fn field_retained_counts_for_test(
        &self,
        field_id: &SettingsFieldId,
        cx: &App,
    ) -> Option<gpui_text_input::TextInputRetainedCounts> {
        self.input_for_field(field_id)
            .map(|input| input.read(cx).retained_counts_for_test(cx))
    }

    /// Selects a section directly and emits the same event as the navigation row.
    pub fn select_section_for_test(
        &mut self,
        section_id: SettingsSectionId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_section(section_id, window, cx);
    }

    /// Emits an apply request.
    pub fn apply_for_test(&mut self, cx: &mut Context<Self>) {
        cx.emit(SettingsWindowEvent::ApplyRequested);
    }

    /// Emits a row action request when the row carries that action.
    pub fn request_row_action_for_test(
        &mut self,
        field_id: SettingsFieldId,
        action_id: SettingsRowActionId,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.row_has_action(&field_id, &action_id) {
            return false;
        }

        cx.emit(SettingsWindowEvent::RowActionRequested {
            field_id,
            action_id,
        });
        true
    }

    /// Emits a page navigation request when the target page exists.
    pub fn request_page_navigation_for_test(
        &mut self,
        page_id: SettingsPageId,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.model.page(&page_id).is_none() {
            return false;
        }

        cx.emit(SettingsWindowEvent::PageNavigationRequested { page_id });
        true
    }

    /// Emits a page action request when the page carries an enabled action.
    pub fn request_page_action_for_test(
        &mut self,
        page_id: SettingsPageId,
        action_id: SettingsPageActionId,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.page_has_enabled_action(&page_id, &action_id) {
            return false;
        }

        cx.emit(SettingsWindowEvent::PageActionRequested { page_id, action_id });
        true
    }

    /// Emits a split item selection request when the page carries that item.
    pub fn request_page_split_item_for_test(
        &mut self,
        page_id: SettingsPageId,
        item_id: SettingsPageSplitItemId,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.page_has_split_item(&page_id, &item_id) {
            return false;
        }

        cx.emit(SettingsWindowEvent::PageSplitItemSelected { page_id, item_id });
        true
    }

    /// Emits a field change when a choice field carries that option.
    pub fn select_choice_for_test(
        &mut self,
        field_id: SettingsFieldId,
        value: String,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.row_has_choice(&field_id, &value) {
            return false;
        }

        cx.emit(SettingsWindowEvent::FieldChanged { field_id, value });
        true
    }

    /// Emits an accept request.
    pub fn accept_for_test(&mut self, cx: &mut Context<Self>) {
        cx.emit(SettingsWindowEvent::AcceptRequested);
    }

    /// Emits a cancel request.
    pub fn cancel_for_test(&mut self, cx: &mut Context<Self>) {
        cx.emit(SettingsWindowEvent::CancelRequested);
    }

    fn rebuild_fields(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.fields = self
            .model
            .text_input_field_snapshots()
            .into_iter()
            .map(|field| (field.field_id.clone(), self.build_field(&field, window, cx)))
            .collect();
    }

    fn sync_fields(&mut self, window: &mut Window, cx: &mut Context<Self>) -> usize {
        let mut previous = std::mem::take(&mut self.fields);
        let mut next = HashMap::with_capacity(previous.len());
        let mut input_sync_entity_count = 0usize;
        for field in self.model.text_input_field_snapshots() {
            if let Some(input) = previous.remove(&field.field_id) {
                let _ = input.update(cx, |input, cx| {
                    input.sync(
                        field.value.as_str(),
                        field.kind,
                        field.error.as_deref(),
                        self.font_size,
                        self.text_input_undo_byte_limit,
                        cx,
                    );
                    input.sync_visual_theme(&self.visual_theme.input, cx);
                });
                input_sync_entity_count = input_sync_entity_count.saturating_add(1);
                next.insert(field.field_id, input);
            } else {
                next.insert(field.field_id.clone(), self.build_field(&field, window, cx));
                input_sync_entity_count = input_sync_entity_count.saturating_add(1);
            }
        }
        self.fields = next;
        self.record_input_sync_diagnostics(input_sync_entity_count);
        input_sync_entity_count
    }

    fn build_field(
        &self,
        field: &SettingsFieldSnapshot,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<SettingsFieldInput> {
        let input = cx.new(|cx| {
            SettingsFieldInput::new(
                field.field_id.clone(),
                field.value.as_str(),
                field.kind,
                field.error.as_deref(),
                self.font_size,
                SettingsFieldInputRole::Row,
                self.visual_theme.input.clone(),
                self.text_input_undo_byte_limit,
                cx,
            )
        });
        self.subscribe_to_field(&input, window, cx);

        input
    }

    fn sync_input_visual_themes(&self, cx: &mut Context<Self>) {
        for input in self.fields.values() {
            let _ = input.update(cx, |input, cx| {
                input.sync_visual_theme(&self.visual_theme.input, cx);
            });
        }
        if let Some(input) = self.color_picker_input.clone() {
            let _ = input.update(cx, |input, cx| {
                input.sync_visual_theme(&self.visual_theme.input, cx);
            });
        }
        for slot in &self.color_picker_channel_inputs {
            let _ = slot.input.update(cx, |input, cx| {
                input.sync_visual_theme(&self.visual_theme.input, cx);
            });
        }
    }

    fn sync_input_retention_options(&self, window: &mut Window, cx: &mut Context<Self>) {
        for input in self.fields.values() {
            let _ = input.update(cx, |input, cx| {
                input.sync_text_input_undo_byte_limit(self.text_input_undo_byte_limit, cx);
            });
        }
        if let Some(input) = self.color_picker_input.clone() {
            let _ = input.update(cx, |input, cx| {
                input.sync_text_input_undo_byte_limit(self.text_input_undo_byte_limit, cx);
            });
        }
        for slot in &self.color_picker_channel_inputs {
            let _ = slot.input.update(cx, |input, cx| {
                input.sync_text_input_undo_byte_limit(self.text_input_undo_byte_limit, window, cx);
            });
        }
    }

    fn subscribe_to_field(
        &self,
        input: &Entity<SettingsFieldInput>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.subscribe_in(input, window, |this, _, event, window, cx| match event {
            SettingsFieldInputEvent::Window(SettingsWindowEvent::FieldChanged {
                field_id,
                value,
            }) => this.handle_field_changed(field_id, value, cx),
            SettingsFieldInputEvent::Window(event) => cx.emit(event.clone()),
            SettingsFieldInputEvent::OpenColorPickerRequested(field_id) => {
                this.open_color_picker(field_id.clone(), window, cx);
                cx.emit(SettingsWindowEvent::ColorPickerRequested {
                    field_id: field_id.clone(),
                });
            }
        })
        .detach();
    }

    fn input_for_field(&self, field_id: &SettingsFieldId) -> Option<Entity<SettingsFieldInput>> {
        self.fields.get(field_id).cloned()
    }

    fn sync_choice_popup(&mut self) {
        self.choice_control_bounds.retain(|field_id, _| {
            self.model.field_kind(field_id) == Some(SettingsFieldKind::Choice)
        });
        if self.choice_popup_field.as_ref().is_some_and(|field_id| {
            self.model.field_kind(field_id) != Some(SettingsFieldKind::Choice)
        }) {
            self.choice_popup_field = None;
        }
    }

    fn record_choice_control_bounds(
        &mut self,
        field_id: SettingsFieldId,
        bounds: Option<Bounds<Pixels>>,
        _: &mut Context<Self>,
    ) {
        if self.model.field_kind(&field_id) != Some(SettingsFieldKind::Choice) {
            self.choice_control_bounds.remove(&field_id);
            return;
        }
        if let Some(bounds) = bounds {
            self.choice_control_bounds.insert(field_id, bounds);
        } else {
            self.choice_control_bounds.remove(&field_id);
        }
    }

    fn toggle_choice_popup(&mut self, field_id: SettingsFieldId, cx: &mut Context<Self>) {
        if self.model.field_kind(&field_id) != Some(SettingsFieldKind::Choice) {
            return;
        }
        if self.choice_popup_field.as_ref() == Some(&field_id) {
            self.choice_popup_field = None;
        } else {
            self.choice_popup_field = Some(field_id);
        }
        cx.notify();
    }

    fn close_choice_popup(&mut self, cx: &mut Context<Self>) {
        if self.choice_popup_field.take().is_some() {
            cx.notify();
        }
    }

    fn select_choice_value(
        &mut self,
        field_id: SettingsFieldId,
        value: String,
        cx: &mut Context<Self>,
    ) {
        if !self.row_has_choice(&field_id, value.as_str()) {
            return;
        }
        self.choice_popup_field = None;
        cx.emit(SettingsWindowEvent::FieldChanged { field_id, value });
        cx.notify();
    }

    fn row_has_action(&self, field_id: &SettingsFieldId, action_id: &SettingsRowActionId) -> bool {
        self.model.row(field_id).is_some_and(|row| {
            row.actions()
                .iter()
                .any(|action| action.action_id() == action_id && action.is_enabled())
        })
    }

    fn row_has_choice(&self, field_id: &SettingsFieldId, value: &str) -> bool {
        self.model.field_kind(field_id) == Some(SettingsFieldKind::Choice)
            && self
                .model
                .field_choices(field_id)
                .is_some_and(|choices| choices.iter().any(|choice| choice.value() == value))
    }

    fn page_has_enabled_action(
        &self,
        page_id: &SettingsPageId,
        action_id: &SettingsPageActionId,
    ) -> bool {
        self.model.page(page_id).is_some_and(|page| {
            page.actions()
                .iter()
                .any(|action| action.action_id() == action_id && action.is_enabled())
        })
    }

    fn page_has_split_item(
        &self,
        page_id: &SettingsPageId,
        item_id: &SettingsPageSplitItemId,
    ) -> bool {
        self.model
            .page(page_id)
            .and_then(|page| page.local_split())
            .is_some_and(|split| split.items().iter().any(|item| item.item_id() == item_id))
    }

    fn focus_selected_section_field(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(field_id) = self
            .model
            .selected_text_input_field_ids()
            .into_iter()
            .next()
        else {
            return;
        };
        self.focus_field(&field_id, window, cx);
    }

    fn reset_selected_page_view_state(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let current = self.scroll_handle.offset();
        self.scroll_handle.set_offset(point(current.x, px(0.0)));
        self.content_scrollbar_visibility = ScrollbarVisibilityState::default();
        self.close_transient_popups(cx);
        window.focus(&self.focus_handle);
        self.focus_selected_section_field(window, cx);
    }

    fn focus_next(&mut self, _: &FocusNext, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_relative_input(window, cx, 1);
        cx.stop_propagation();
    }

    fn focus_prev(&mut self, _: &FocusPrev, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_relative_input(window, cx, -1);
        cx.stop_propagation();
    }

    fn focus_relative_input(&self, window: &mut Window, cx: &mut Context<Self>, delta: isize) {
        let targets = self.ordered_input_focus_targets(cx);
        if targets.is_empty() {
            return;
        }

        let current_index = targets
            .iter()
            .position(|focus_handle| focus_handle.is_focused(window));
        let next_index = match (current_index, delta.is_negative()) {
            (Some(index), false) => (index + 1) % targets.len(),
            (Some(0), true) => targets.len() - 1,
            (Some(index), true) => index - 1,
            (None, false) => 0,
            (None, true) => targets.len() - 1,
        };

        window.focus(&targets[next_index]);
    }

    fn ordered_input_focus_targets(&self, cx: &App) -> Vec<FocusHandle> {
        let mut targets: Vec<_> = self
            .model
            .selected_text_input_field_ids()
            .iter()
            .filter_map(|field_id| self.input_for_field(field_id))
            .map(|input| input.read(cx).tab_focus_handle())
            .collect();

        if self.color_picker_field.is_some() {
            if let Some(input) = self.color_picker_input.clone() {
                targets.push(input.read(cx).tab_focus_handle());
            }
            targets.extend(
                self.color_picker_channel_inputs
                    .iter()
                    .map(|slot| slot.input.read(cx).tab_focus_handle()),
            );
        }

        targets
    }

    fn cancel(&mut self, _: &Cancel, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle);
        cx.emit(SettingsWindowEvent::CancelRequested);
    }

    fn select_section(
        &mut self,
        section_id: SettingsSectionId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let previous_page = self.model.selected_page_id().clone();
        let previous_split_selection = selected_page_split_selection(&self.model);
        if self.model.select_section(section_id.clone()).is_err() {
            return;
        }
        self.sync_split_scroll_for_model(Some(previous_page), previous_split_selection);
        self.reset_selected_page_view_state(window, cx);
        cx.emit(SettingsWindowEvent::SectionSelected { section_id });
        cx.notify();
    }

    fn sync_split_scroll_for_model(
        &mut self,
        previous_page: Option<SettingsPageId>,
        previous_selection: Option<(SettingsPageId, SettingsPageSplitItemId, usize, usize)>,
    ) {
        let Some((page_id, item_id, selected_index, item_count)) =
            selected_page_split_selection(&self.model)
        else {
            let current = self.split_scroll_handle.offset();
            self.split_scroll_handle
                .set_offset(point(current.x, px(0.0)));
            self.split_scrollbar_visibility = ScrollbarVisibilityState::default();
            return;
        };

        if previous_page.as_ref() != Some(&page_id) {
            let current = self.split_scroll_handle.offset();
            self.split_scroll_handle
                .set_offset(point(current.x, px(0.0)));
            self.split_scrollbar_visibility = ScrollbarVisibilityState::default();
        }

        let selected_item_moved = !matches!(
            previous_selection.as_ref(),
            Some((previous_page_id, previous_item_id, previous_index, _))
                if previous_page_id == &page_id
                    && previous_item_id == &item_id
                    && *previous_index == selected_index
        );

        if selected_item_moved {
            self.reveal_split_item_index(selected_index, item_count);
        } else {
            self.clamp_split_scroll_to_item_count(item_count);
        }
    }

    fn clamp_split_scroll_to_item_count(&self, item_count: usize) {
        let viewport_height =
            split_list_viewport_height(self.split_scroll_handle.bounds().size.height);
        let total_height = page_local_split_total_height(item_count);
        let max_scroll_top = (total_height - viewport_height).max(0.0);
        let current_offset = self.split_scroll_handle.offset();
        let current_scroll_top = f32::from(-current_offset.y).max(0.0);
        let clamped_scroll_top = current_scroll_top.min(max_scroll_top);

        self.split_scroll_handle
            .set_offset(point(current_offset.x, px(-clamped_scroll_top)));
    }

    fn reveal_split_item_index(&self, item_index: usize, item_count: usize) {
        let viewport_height =
            split_list_viewport_height(self.split_scroll_handle.bounds().size.height);
        let total_height = page_local_split_total_height(item_count);
        let item_top = page_local_split_offset_for_index(item_index);
        let item_bottom = item_top + PAGE_LOCAL_SPLIT_ITEM_HEIGHT;
        let current_offset = self.split_scroll_handle.offset();
        let current_scroll_top = f32::from(-current_offset.y).max(0.0);

        let target_scroll_top = if item_top < current_scroll_top {
            item_top
        } else if item_bottom > current_scroll_top + viewport_height {
            item_bottom - viewport_height
        } else {
            current_scroll_top
        }
        .max(0.0)
        .min((total_height - viewport_height).max(0.0));

        self.split_scroll_handle
            .set_offset(point(current_offset.x, px(-target_scroll_top)));
    }
}

impl SettingsPanelDiagnosticsState {
    fn color_lookup_counts(&self) -> SettingsPanelColorLookupCounts {
        SettingsPanelColorLookupCounts {
            color_preview_lookup_count: self.color_preview_lookup_count,
            color_model_lookup_count: self.color_model_lookup_count,
        }
    }

    fn record_render(&mut self, duration: Duration, before: SettingsPanelColorLookupCounts) {
        let after = self.color_lookup_counts();
        self.render_count = self.render_count.saturating_add(1);
        self.last_render_tree_micros = duration_micros(duration);
        self.last_render_color_preview_lookup_count = after
            .color_preview_lookup_count
            .saturating_sub(before.color_preview_lookup_count);
        self.last_render_color_model_lookup_count = after
            .color_model_lookup_count
            .saturating_sub(before.color_model_lookup_count);
    }

    fn record_model_sync(&mut self, duration: Duration, input_sync_entity_count: usize) {
        self.model_sync_count = self.model_sync_count.saturating_add(1);
        self.last_model_sync_micros = duration_micros(duration);
        self.last_input_sync_entity_count = input_sync_entity_count;
    }

    fn record_option_sync(&mut self, duration: Duration) {
        self.option_sync_count = self.option_sync_count.saturating_add(1);
        self.last_option_sync_micros = duration_micros(duration);
    }

    fn record_input_sync(&mut self, input_sync_entity_count: usize) {
        self.input_sync_count = self
            .input_sync_count
            .saturating_add(input_sync_entity_count as u64);
        self.last_input_sync_entity_count = input_sync_entity_count;
    }

    fn performance_diagnostics(&self) -> SettingsWindowPerformanceDiagnostics {
        SettingsWindowPerformanceDiagnostics {
            render_count: self.render_count,
            last_render_tree_micros: self.last_render_tree_micros,
            model_sync_count: self.model_sync_count,
            last_model_sync_micros: self.last_model_sync_micros,
            option_sync_count: self.option_sync_count,
            last_option_sync_micros: self.last_option_sync_micros,
            input_sync_count: self.input_sync_count,
            last_input_sync_entity_count: self.last_input_sync_entity_count,
            color_preview_lookup_count: self.color_preview_lookup_count,
            last_render_color_preview_lookup_count: self.last_render_color_preview_lookup_count,
            color_model_lookup_count: self.color_model_lookup_count,
            last_render_color_model_lookup_count: self.last_render_color_model_lookup_count,
            dominant_cost_category: self.dominant_cost_category().to_string(),
        }
    }

    fn dominant_cost_category(&self) -> &'static str {
        let candidates = [
            ("render_tree", self.last_render_tree_micros),
            ("model_sync", self.last_model_sync_micros),
            ("option_sync", self.last_option_sync_micros),
        ];
        candidates
            .into_iter()
            .max_by_key(|(_, micros)| *micros)
            .filter(|(_, micros)| *micros > 0)
            .map(|(category, _)| category)
            .unwrap_or("none")
    }
}

fn selected_page_split_selection(
    model: &SettingsWindowModel,
) -> Option<(SettingsPageId, SettingsPageSplitItemId, usize, usize)> {
    let page = model.selected_page();
    let split = page.local_split()?;
    let (index, item) = split
        .items()
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_selected())?;
    Some((
        page.page_id().clone(),
        item.item_id().clone(),
        index,
        split.items().len(),
    ))
}

fn page_local_split_item_pitch() -> f32 {
    PAGE_LOCAL_SPLIT_ITEM_HEIGHT + PAGE_LOCAL_SPLIT_ITEM_GAP
}

fn page_local_split_offset_for_index(index: usize) -> f32 {
    index as f32 * page_local_split_item_pitch()
}

fn page_local_split_total_height(item_count: usize) -> f32 {
    if item_count == 0 {
        0.0
    } else {
        item_count as f32 * PAGE_LOCAL_SPLIT_ITEM_HEIGHT
            + item_count.saturating_sub(1) as f32 * PAGE_LOCAL_SPLIT_ITEM_GAP
    }
}

fn page_local_split_segment_height(item_count: usize) -> f32 {
    page_local_split_total_height(item_count)
}

fn split_list_viewport_height(viewport_height: Pixels) -> f32 {
    let viewport_height = f32::from(viewport_height);
    if viewport_height > 0.0 {
        viewport_height
    } else {
        PAGE_LOCAL_SPLIT_FALLBACK_VIEWPORT_HEIGHT
    }
}

fn page_local_split_render_window(
    item_count: usize,
    scroll_offset: Pixels,
    viewport_height: Pixels,
) -> Range<usize> {
    if item_count == 0 {
        return 0..0;
    }

    let viewport_height = split_list_viewport_height(viewport_height);
    let max_scroll_top = (page_local_split_total_height(item_count) - viewport_height).max(0.0);
    let scroll_top = f32::from(-scroll_offset).max(0.0).min(max_scroll_top);
    let pitch = page_local_split_item_pitch();
    let first_visible = (scroll_top / pitch).floor().max(0.0) as usize;
    let visible_count = (viewport_height / pitch).ceil().max(1.0) as usize + 1;
    let start = first_visible.saturating_sub(PAGE_LOCAL_SPLIT_OVERSCAN_ROWS);
    let end = item_count.min(first_visible + visible_count + PAGE_LOCAL_SPLIT_OVERSCAN_ROWS);
    start..end
}

fn duration_micros(duration: Duration) -> u64 {
    duration.as_micros().min(u128::from(u64::MAX)) as u64
}

pub(crate) fn ensure_settings_panel_bindings(cx: &mut App) {
    ensure_settings_input_bindings(cx);
    if cx.has_global::<SettingsPanelBindingsInstalled>() {
        return;
    }
    cx.bind_keys([
        gpui::KeyBinding::new("escape", Cancel, Some(PANEL_KEY_CONTEXT)),
        gpui::KeyBinding::new("tab", FocusNext, Some(PANEL_KEY_CONTEXT)),
        gpui::KeyBinding::new("shift-tab", FocusPrev, Some(PANEL_KEY_CONTEXT)),
    ]);
    cx.set_global(SettingsPanelBindingsInstalled);
}

struct SettingsPanelBindingsInstalled;

impl gpui::Global for SettingsPanelBindingsInstalled {}
