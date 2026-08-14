use super::*;
use std::collections::HashSet;

use crate::color_picker::{
    ColorComponentInputEvent, ColorPickerMainPaletteSelection, ColorPickerNeutralStripSelection,
    apply_color_picker_channel_text, apply_color_picker_lightness, color_picker_channel_text,
    color_picker_chromatic_selection_lightness, color_picker_main_palette_color,
    color_picker_neutral_strip_color,
};

impl SettingsPanel {
    pub(super) fn handle_field_changed(
        &mut self,
        field_id: &SettingsFieldId,
        value: &str,
        cx: &mut Context<Self>,
    ) {
        let is_color = self
            .model
            .field_kind(field_id)
            .is_some_and(|kind| kind == crate::SettingsFieldKind::Color);

        if is_color {
            if let Some(color) = RgbColor::parse(value) {
                self.remember_latest_color(field_id.clone(), color);
                if self.color_picker_field.as_ref() == Some(field_id) {
                    self.clear_color_picker_channel_drafts();
                    self.sync_color_picker_channel_inputs(Some(color), cx);
                }
                cx.emit(SettingsWindowEvent::FieldChanged {
                    field_id: field_id.clone(),
                    value: color.to_hex(),
                });
                return;
            }
        }

        cx.emit(SettingsWindowEvent::FieldChanged {
            field_id: field_id.clone(),
            value: value.to_owned(),
        });
    }

    pub(super) fn activate_color_preview_swatch(
        &mut self,
        field_id: SettingsFieldId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let _ = self.focus_field(&field_id, window, cx);
        self.open_color_picker(field_id, window, cx);
    }

    pub(super) fn open_color_picker(
        &mut self,
        field_id: SettingsFieldId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self
            .model
            .field_kind(&field_id)
            .is_none_or(|kind| kind != crate::SettingsFieldKind::Color)
        {
            return;
        }

        self.color_picker_pending_outside_mouse_up_field = None;
        if self.color_picker_field.as_ref() != Some(&field_id) {
            self.clear_color_picker_channel_drafts();
            self.color_picker_drag_target = None;
            *self.color_picker_main_palette_bounds.borrow_mut() = None;
            *self.color_picker_neutral_strip_bounds.borrow_mut() = None;
            *self.color_picker_lightness_bar_bounds.borrow_mut() = None;
        }

        let Some(value) = self.model.field_value(&field_id).map(str::to_owned) else {
            return;
        };
        let error = self.model.field_error(&field_id).map(str::to_owned);
        self.color_picker_field = Some(field_id.clone());
        self.color_picker_focused_swatch = self
            .color_picker_focused_swatch
            .as_ref()
            .filter(|focused| {
                self.saved_color_swatches
                    .iter()
                    .any(|swatch| swatch.swatch_id() == *focused)
            })
            .cloned()
            .or_else(|| {
                self.saved_color_swatches
                    .first()
                    .map(|swatch| swatch.swatch_id().clone())
            });

        if self.color_picker_channel_inputs.is_empty() {
            self.color_picker_channel_inputs = self.build_color_picker_channel_inputs(window, cx);
        }

        if let Some(input) = self.color_picker_input.clone() {
            let _ = input.update(cx, |input, cx| {
                input.retarget(
                    field_id.clone(),
                    value.as_str(),
                    crate::SettingsFieldKind::Color,
                    error.as_deref(),
                    self.font_size,
                    self.text_input_undo_byte_limit,
                    cx,
                );
            });
        } else {
            let input = cx.new(|cx| {
                SettingsFieldInput::new(
                    field_id.clone(),
                    value.as_str(),
                    crate::SettingsFieldKind::Color,
                    error.as_deref(),
                    self.font_size,
                    SettingsFieldInputRole::Picker,
                    self.visual_theme.input.clone(),
                    self.text_input_undo_byte_limit,
                    cx,
                )
            });
            self.subscribe_to_field(&input, window, cx);
            self.color_picker_input = Some(input);
        }

        self.sync_color_picker_channel_inputs(self.synced_color_for_field(&field_id), cx);
        cx.notify();
    }

    pub(super) fn close_color_picker(&mut self, cx: &mut Context<Self>) {
        if self.color_picker_field.take().is_some() {
            self.clear_color_picker_channel_drafts();
            self.color_picker_preview_color = None;
            self.color_picker_focused_swatch = None;
            self.color_picker_selected_swatch = None;
            self.color_picker_drag_target = None;
            self.color_picker_pending_outside_mouse_up_field = None;
            *self.color_picker_main_palette_bounds.borrow_mut() = None;
            *self.color_picker_neutral_strip_bounds.borrow_mut() = None;
            *self.color_picker_lightness_bar_bounds.borrow_mut() = None;
            cx.notify();
        }
    }

    pub(super) fn build_color_picker_channel_inputs(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<ColorPickerChannelSlot> {
        ColorPickerChannelField::ALL
            .into_iter()
            .map(|field| {
                let value = color_picker_channel_text(None, field);
                let input = cx.new(|cx| {
                    ColorComponentInput::new(
                        field,
                        value.as_str(),
                        self.font_size,
                        self.visual_theme.input.clone(),
                        self.text_input_undo_byte_limit,
                        window,
                        cx,
                    )
                });
                self.subscribe_to_color_picker_channel(field, &input, window, cx);
                ColorPickerChannelSlot { field, input }
            })
            .collect()
    }

    fn subscribe_to_color_picker_channel(
        &self,
        field: ColorPickerChannelField,
        input: &Entity<ColorComponentInput>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.subscribe_in(
            input,
            window,
            move |this, _, event, window, cx| match event {
                ColorComponentInputEvent::Changed(value) => {
                    this.apply_color_picker_channel_value(field, value.as_str(), cx);
                }
                ColorComponentInputEvent::Focused => this.set_focused_color_picker_channel(field),
                ColorComponentInputEvent::FocusLost => {
                    this.materialize_color_picker_channel_on_blur(field, cx);
                }
                ColorComponentInputEvent::Accepted => {
                    window.focus(&this.focus_handle);
                    cx.emit(SettingsWindowEvent::AcceptRequested);
                }
                ColorComponentInputEvent::Canceled => {
                    window.focus(&this.focus_handle);
                    cx.emit(SettingsWindowEvent::CancelRequested);
                }
            },
        )
        .detach();
    }

    pub(super) fn color_picker_channel_input(
        &self,
        field: ColorPickerChannelField,
    ) -> Option<Entity<ColorComponentInput>> {
        self.color_picker_channel_inputs
            .iter()
            .find(|slot| slot.field == field)
            .map(|slot| slot.input.clone())
    }

    pub(super) fn set_focused_color_picker_channel(&mut self, field: ColorPickerChannelField) {
        self.color_picker_focused_channel = Some(field);
    }

    pub(super) fn clear_color_picker_channel_drafts(&mut self) {
        self.color_picker_channel_empty_drafts.clear();
        self.color_picker_focused_channel = None;
    }

    pub(super) fn sync_latest_known_color_values(&mut self) {
        let color_field_ids: HashSet<_> = self
            .model
            .text_input_field_snapshots()
            .into_iter()
            .filter(|field| field.kind == crate::SettingsFieldKind::Color)
            .map(|field| field.field_id)
            .collect();
        self.latest_known_color_values
            .retain(|field_id, _| color_field_ids.contains(field_id));

        let color_rows: Vec<(SettingsFieldId, RgbColor)> = self
            .model
            .text_input_field_snapshots()
            .into_iter()
            .filter(|field| field.kind == crate::SettingsFieldKind::Color)
            .filter_map(|field| RgbColor::parse(&field.value).map(|color| (field.field_id, color)))
            .collect();

        for (field_id, color) in color_rows {
            self.remember_latest_color(field_id, color);
        }
    }

    pub(super) fn remember_latest_color(&mut self, field_id: SettingsFieldId, color: RgbColor) {
        self.latest_known_color_values.insert(field_id, color);
    }

    pub(super) fn latest_known_color_for_field(
        &self,
        field_id: &SettingsFieldId,
    ) -> Option<RgbColor> {
        self.latest_known_color_values.get(field_id).copied()
    }

    pub(super) fn synced_color_for_field(&self, field_id: &SettingsFieldId) -> Option<RgbColor> {
        self.record_color_model_lookup();
        self.model
            .field_value(field_id)
            .and_then(RgbColor::parse)
            .or_else(|| self.latest_known_color_for_field(field_id))
    }

    pub(super) fn color_preview_for_rendered_field(
        &self,
        field_id: &SettingsFieldId,
        value: &str,
    ) -> Option<RgbColor> {
        self.record_color_preview_lookup();
        if self.color_picker_field.as_ref() == Some(field_id) {
            self.color_picker_preview_color
                .or_else(|| RgbColor::parse(value))
                .or_else(|| self.latest_known_color_for_field(field_id))
        } else {
            RgbColor::parse(value).or_else(|| self.latest_known_color_for_field(field_id))
        }
    }

    pub(super) fn color_preview_for_field(&self, field_id: &SettingsFieldId) -> Option<RgbColor> {
        self.record_color_preview_lookup();
        if self.color_picker_field.as_ref() == Some(field_id) {
            self.current_color_picker_color()
        } else {
            self.synced_color_for_field(field_id)
        }
    }

    pub(super) fn current_color_picker_color(&self) -> Option<RgbColor> {
        self.color_picker_preview_color.or_else(|| {
            self.color_picker_field
                .as_ref()
                .and_then(|field_id| self.synced_color_for_field(field_id))
        })
    }

    pub(super) fn sync_color_picker(&mut self, cx: &mut Context<Self>) {
        let Some(field_id) = self.color_picker_field.clone() else {
            return;
        };
        let Some(kind) = self.model.field_kind(&field_id) else {
            self.color_picker_field = None;
            self.color_picker_preview_color = None;
            return;
        };
        if kind != crate::SettingsFieldKind::Color {
            self.close_color_picker(cx);
            return;
        }

        let Some(value) = self.model.field_value(&field_id).map(str::to_owned) else {
            self.close_color_picker(cx);
            return;
        };
        let error = self.model.field_error(&field_id).map(str::to_owned);
        if let Some(input) = self.color_picker_input.clone() {
            let _ = input.update(cx, |input, cx| {
                input.retarget(
                    field_id.clone(),
                    value.as_str(),
                    crate::SettingsFieldKind::Color,
                    error.as_deref(),
                    self.font_size,
                    self.text_input_undo_byte_limit,
                    cx,
                );
            });
        }
        self.sync_color_picker_channel_inputs(self.synced_color_for_field(&field_id), cx);
    }

    pub(super) fn sync_color_picker_channel_inputs(
        &mut self,
        color: Option<RgbColor>,
        cx: &mut Context<Self>,
    ) {
        self.color_picker_preview_color = color;
        let mut input_sync_entity_count = 0usize;
        for field in ColorPickerChannelField::ALL {
            let value = if self.color_picker_focused_channel == Some(field) {
                self.color_picker_channel_empty_drafts
                    .get(&field)
                    .cloned()
                    .unwrap_or_else(|| color_picker_channel_text(color, field))
            } else {
                color_picker_channel_text(color, field)
            };
            let Some(input) = self.color_picker_channel_input(field) else {
                continue;
            };
            let _ = input.update(cx, |input, cx| {
                input.sync(value.as_str(), self.font_size, cx);
                input.sync_visual_theme(&self.visual_theme.input, cx);
            });
            input_sync_entity_count = input_sync_entity_count.saturating_add(1);
        }
        self.record_input_sync_diagnostics(input_sync_entity_count);
    }

    pub(super) fn apply_color_picker_swatch(&mut self, color: RgbColor, cx: &mut Context<Self>) {
        self.apply_color_picker_color(color, cx);
    }

    pub(super) fn apply_color_picker_main_palette_selection(
        &mut self,
        selection: ColorPickerMainPaletteSelection,
        cx: &mut Context<Self>,
    ) {
        let lightness =
            color_picker_chromatic_selection_lightness(self.current_color_picker_color());
        self.apply_color_picker_color(color_picker_main_palette_color(selection, lightness), cx);
    }

    pub(super) fn apply_color_picker_neutral_strip_selection(
        &mut self,
        selection: ColorPickerNeutralStripSelection,
        cx: &mut Context<Self>,
    ) {
        self.apply_color_picker_color(color_picker_neutral_strip_color(selection), cx);
    }

    pub(super) fn apply_color_picker_lightness(&mut self, lightness: u16, cx: &mut Context<Self>) {
        let color = apply_color_picker_lightness(self.current_color_picker_color(), lightness);
        self.apply_color_picker_color(color, cx);
    }

    pub(super) fn apply_color_picker_channel_value(
        &mut self,
        field: ColorPickerChannelField,
        value: &str,
        cx: &mut Context<Self>,
    ) {
        if value.trim().is_empty() {
            self.color_picker_channel_empty_drafts
                .insert(field, value.to_owned());
        } else {
            self.color_picker_channel_empty_drafts.remove(&field);
        }

        let current = self.current_color_picker_color();
        let Some(next) = apply_color_picker_channel_text(current, field, value) else {
            self.sync_color_picker_channel_inputs(current, cx);
            return;
        };
        self.apply_color_picker_color(next, cx);
    }

    pub(super) fn materialize_color_picker_channel_on_blur(
        &mut self,
        field: ColorPickerChannelField,
        cx: &mut Context<Self>,
    ) {
        if self.color_picker_focused_channel == Some(field) {
            self.color_picker_focused_channel = None;
        }
        if self
            .color_picker_channel_empty_drafts
            .remove(&field)
            .is_some()
        {
            self.sync_color_picker_channel_inputs(self.current_color_picker_color(), cx);
        }
    }

    fn apply_color_picker_color(&mut self, color: RgbColor, cx: &mut Context<Self>) {
        let Some(field_id) = self.color_picker_field.clone() else {
            return;
        };
        self.color_picker_drag_target = None;
        self.clear_color_picker_channel_drafts();
        self.remember_latest_color(field_id.clone(), color);
        self.sync_color_picker_channel_inputs(Some(color), cx);
        cx.emit(SettingsWindowEvent::FieldChanged {
            field_id,
            value: color.to_hex(),
        });
    }
}
