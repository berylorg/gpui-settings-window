use gpui::{MouseDownEvent, MouseMoveEvent, MouseUpEvent, Point};

use super::*;
use crate::color_picker::{
    ColorPickerMainPaletteSelection, ColorPickerNeutralStripSelection,
    color_picker_palette_axis_index_at,
};

impl SettingsPanel {
    pub(super) fn on_color_picker_popup_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match self.color_picker_drag_target {
            Some(ColorPickerDragTarget::MainPalette) => {
                self.drag_color_picker_main_palette_to(event.position, cx);
            }
            Some(ColorPickerDragTarget::NeutralStrip) => {
                self.drag_color_picker_neutral_strip_to(event.position, cx);
            }
            Some(ColorPickerDragTarget::LightnessBar) => {
                self.drag_color_picker_lightness_to(event.position, cx);
            }
            None => {}
        }
    }

    pub(super) fn on_color_picker_popup_mouse_up_inside(
        &mut self,
        _: &MouseUpEvent,
        _: &mut Window,
        _: &mut Context<Self>,
    ) {
        self.color_picker_drag_target = None;
        self.color_picker_pending_outside_mouse_up_field = None;
    }

    pub(super) fn on_color_picker_popup_mouse_up_outside(
        &mut self,
        _: &MouseUpEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.color_picker_drag_target = None;
        let pending_field = self.color_picker_pending_outside_mouse_up_field.take();
        if pending_field.is_some() && pending_field == self.color_picker_field {
            self.close_color_picker(cx);
        }
    }

    pub(super) fn on_color_picker_popup_mouse_down_out(
        &mut self,
        event: &MouseDownEvent,
        _: &mut Window,
        _: &mut Context<Self>,
    ) {
        if event.button == MouseButton::Left {
            self.color_picker_pending_outside_mouse_up_field = self.color_picker_field.clone();
        }
    }

    pub(super) fn on_color_picker_main_palette_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.color_picker_drag_target = Some(ColorPickerDragTarget::MainPalette);
        self.drag_color_picker_main_palette_to(event.position, cx);
    }

    pub(super) fn on_color_picker_lightness_bar_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.color_picker_drag_target = Some(ColorPickerDragTarget::LightnessBar);
        self.drag_color_picker_lightness_to(event.position, cx);
    }

    pub(super) fn on_color_picker_neutral_strip_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.color_picker_drag_target = Some(ColorPickerDragTarget::NeutralStrip);
        self.drag_color_picker_neutral_strip_to(event.position, cx);
    }

    fn color_picker_main_palette_selection_for_pointer(
        &self,
        position: Point<Pixels>,
    ) -> Option<ColorPickerMainPaletteSelection> {
        let bounds = *self.color_picker_main_palette_bounds.borrow();
        let bounds = bounds?;
        let width = f32::from(bounds.size.width);
        let height = f32::from(bounds.size.height);
        if width <= f32::EPSILON || height <= f32::EPSILON {
            return None;
        }

        let local_x = (f32::from(position.x) - f32::from(bounds.left())).clamp(0.0, width);
        let local_y = (f32::from(position.y) - f32::from(bounds.top())).clamp(0.0, height);
        let hue_index = color_picker_palette_axis_index_at(
            local_x,
            ColorPickerMainPaletteSelection::HUES.len(),
            width,
        )?;
        let saturation_index = color_picker_palette_axis_index_at(
            local_y,
            ColorPickerMainPaletteSelection::SATURATIONS.len(),
            height,
        )?;

        Some(ColorPickerMainPaletteSelection::new(
            ColorPickerMainPaletteSelection::HUES[hue_index],
            ColorPickerMainPaletteSelection::SATURATIONS[saturation_index],
        ))
    }

    fn color_picker_neutral_strip_selection_for_pointer(
        &self,
        position: Point<Pixels>,
    ) -> Option<ColorPickerNeutralStripSelection> {
        let bounds = *self.color_picker_neutral_strip_bounds.borrow();
        let bounds = bounds?;
        let width = f32::from(bounds.size.width);
        if width <= f32::EPSILON {
            return None;
        }

        let local_x = (f32::from(position.x) - f32::from(bounds.left())).clamp(0.0, width);
        let lightness_index = color_picker_palette_axis_index_at(
            local_x,
            ColorPickerNeutralStripSelection::LIGHTNESSES.len(),
            width,
        )?;

        Some(ColorPickerNeutralStripSelection::new(
            ColorPickerNeutralStripSelection::LIGHTNESSES[lightness_index],
        ))
    }

    fn color_picker_lightness_for_pointer(&self, position: Point<Pixels>) -> Option<u16> {
        let bounds = *self.color_picker_lightness_bar_bounds.borrow();
        let bounds = bounds?;
        let width = f32::from(bounds.size.width);
        if width <= f32::EPSILON {
            return None;
        }

        let local_x = (f32::from(position.x) - f32::from(bounds.left())).clamp(0.0, width);
        let lightness_index = color_picker_palette_axis_index_at(
            local_x,
            ColorPickerNeutralStripSelection::LIGHTNESSES.len(),
            width,
        )?;
        Some(ColorPickerNeutralStripSelection::LIGHTNESSES[lightness_index])
    }

    fn drag_color_picker_main_palette_to(
        &mut self,
        position: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        let Some(selection) = self.color_picker_main_palette_selection_for_pointer(position) else {
            return;
        };
        self.apply_color_picker_main_palette_selection(selection, cx);
    }

    fn drag_color_picker_neutral_strip_to(
        &mut self,
        position: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        let Some(selection) = self.color_picker_neutral_strip_selection_for_pointer(position)
        else {
            return;
        };
        self.apply_color_picker_neutral_strip_selection(selection, cx);
    }

    fn drag_color_picker_lightness_to(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
        let Some(lightness) = self.color_picker_lightness_for_pointer(position) else {
            return;
        };
        self.apply_color_picker_lightness(lightness, cx);
    }
}
