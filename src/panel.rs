use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use gpui::{
    App, Bounds, Context, Entity, EventEmitter, FocusHandle, FontWeight, IntoElement, MouseButton,
    ParentElement, Pixels, Render, ScrollHandle, SharedString, Window, actions, div, point,
    prelude::*, px, rgb,
};

use crate::color_picker::{
    ColorComponentInput, ColorPickerChannelField, ensure_color_component_input_bindings,
};
use crate::input::{
    SettingsFieldInput, SettingsFieldInputEvent, SettingsFieldInputRole,
    ensure_settings_input_bindings,
};
use crate::model::{
    SettingsFieldId, SettingsRow, SettingsRowActionId, SettingsSectionId, SettingsWindowEvent,
    SettingsWindowModel, element_id_suffix,
};
use crate::{RgbColor, SettingsWindowOptions, SettingsWindowTheme};

mod color_paint;
mod color_pointer;
mod color_render;
mod picker;
mod render;
mod test_support;

const PANEL_KEY_CONTEXT: &str = "GpuiSettingsWindowPanel";
const NAVIGATION_WIDTH: f32 = 196.0;
const LABEL_COLUMN_WIDTH: f32 = 280.0;
const DEFAULT_FONT_SIZE: f32 = 14.0;

fn theme_color(color: RgbColor) -> gpui::Rgba {
    rgb(color.packed_rgb())
}

actions!(gpui_settings_window_panel, [Cancel, FocusNext, FocusPrev]);

struct FieldSlot {
    field_id: SettingsFieldId,
    input: Entity<SettingsFieldInput>,
}

struct ColorPickerChannelSlot {
    field: ColorPickerChannelField,
    input: Entity<ColorComponentInput>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColorPickerDragTarget {
    MainPalette,
    NeutralStrip,
    LightnessBar,
}

/// GPUI entity that renders the reusable settings panel.
pub struct SettingsPanel {
    model: SettingsWindowModel,
    fields: Vec<FieldSlot>,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    navigation_scroll_handle: ScrollHandle,
    font_size: f32,
    visual_theme: SettingsWindowTheme,
    saved_color_swatches: Vec<RgbColor>,
    text_input_undo_byte_limit: usize,
    latest_known_color_values: Vec<(SettingsFieldId, RgbColor)>,
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
            fields: Vec::new(),
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
            navigation_scroll_handle: ScrollHandle::new(),
            font_size: DEFAULT_FONT_SIZE,
            visual_theme: options.visual_theme().clone(),
            saved_color_swatches: options.saved_color_swatches().to_vec(),
            text_input_undo_byte_limit: options.text_input_undo_byte_limit(),
            latest_known_color_values: Vec::new(),
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
        panel
    }

    /// Synchronizes the panel to a new presentation model.
    pub fn sync_model(
        &mut self,
        model: SettingsWindowModel,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let previous_section = self.model.selected_section_id().clone();
        self.model = model;
        self.sync_latest_known_color_values();
        self.sync_fields(window, cx);
        self.sync_color_picker(cx);

        if previous_section != *self.model.selected_section_id() {
            let current = self.scroll_handle.offset();
            self.scroll_handle.set_offset(point(current.x, px(0.0)));
            self.focus_selected_section_field(window, cx);
        }

        cx.notify();
    }

    /// Returns the current presentation model.
    pub fn model(&self) -> &SettingsWindowModel {
        &self.model
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
        let next_theme = options.visual_theme().clone();
        let next_swatches = options.saved_color_swatches().to_vec();
        let next_text_input_undo_byte_limit = options.text_input_undo_byte_limit();

        let theme_changed = self.visual_theme != next_theme;
        let swatches_changed = self.saved_color_swatches != next_swatches;
        let text_input_undo_byte_limit_changed =
            self.text_input_undo_byte_limit != next_text_input_undo_byte_limit;

        if theme_changed {
            self.visual_theme = next_theme;
            self.sync_input_visual_themes(cx);
        }
        if swatches_changed {
            self.saved_color_swatches = next_swatches;
        }
        if text_input_undo_byte_limit_changed {
            self.text_input_undo_byte_limit = next_text_input_undo_byte_limit;
            self.sync_input_retention_options(window, cx);
        }
        if theme_changed || swatches_changed || text_input_undo_byte_limit_changed {
            cx.notify();
        }
    }

    /// Focuses the first field in the selected section.
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

    /// Returns the current vertical scroll metrics.
    pub fn scroll_metrics(&self) -> (f32, f32) {
        (
            f32::from(self.scroll_handle.offset().y),
            f32::from(self.scroll_handle.max_offset().height),
        )
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
            .rows()
            .cloned()
            .map(|row| self.build_field(&row, window, cx))
            .collect();
    }

    fn sync_fields(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut next = Vec::new();
        for row in self.model.rows().cloned() {
            if let Some(index) = self
                .fields
                .iter()
                .position(|slot| slot.field_id == *row.field_id())
            {
                let slot = self.fields.swap_remove(index);
                let _ = slot.input.update(cx, |input, cx| {
                    input.sync(
                        row.value(),
                        row.kind(),
                        row.error(),
                        self.font_size,
                        self.text_input_undo_byte_limit,
                        cx,
                    );
                    input.sync_visual_theme(&self.visual_theme.input, cx);
                });
                next.push(slot);
            } else {
                next.push(self.build_field(&row, window, cx));
            }
        }
        self.fields = next;
    }

    fn build_field(
        &self,
        row: &SettingsRow,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> FieldSlot {
        let input = cx.new(|cx| {
            SettingsFieldInput::new(
                row.field_id().clone(),
                row.value(),
                row.kind(),
                row.error(),
                self.font_size,
                SettingsFieldInputRole::Row,
                self.visual_theme.input.clone(),
                self.text_input_undo_byte_limit,
                cx,
            )
        });
        self.subscribe_to_field(&input, window, cx);

        FieldSlot {
            field_id: row.field_id().clone(),
            input,
        }
    }

    fn sync_input_visual_themes(&self, cx: &mut Context<Self>) {
        for slot in &self.fields {
            let _ = slot.input.update(cx, |input, cx| {
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
        for slot in &self.fields {
            let _ = slot.input.update(cx, |input, cx| {
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
        self.fields
            .iter()
            .find(|slot| slot.field_id == *field_id)
            .map(|slot| slot.input.clone())
    }

    fn row_has_action(&self, field_id: &SettingsFieldId, action_id: &SettingsRowActionId) -> bool {
        self.model.row(field_id).is_some_and(|row| {
            row.actions()
                .iter()
                .any(|action| action.action_id() == action_id)
        })
    }

    fn focus_selected_section_field(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(field_id) = self
            .model
            .selected_rows()
            .first()
            .map(SettingsRow::field_id)
        else {
            return;
        };
        let field_id = field_id.clone();
        self.focus_field(&field_id, window, cx);
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
            .selected_rows()
            .iter()
            .filter_map(|row| self.input_for_field(row.field_id()))
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
        if self.model.select_section(section_id.clone()).is_err() {
            return;
        }
        let current = self.scroll_handle.offset();
        self.scroll_handle.set_offset(point(current.x, px(0.0)));
        window.focus(&self.focus_handle);
        self.focus_selected_section_field(window, cx);
        cx.emit(SettingsWindowEvent::SectionSelected { section_id });
        cx.notify();
    }
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
