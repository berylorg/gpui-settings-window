use super::*;
use crate::color_picker::{
    ColorPickerMainPaletteSelection, ColorPickerNeutralStripSelection,
    color_picker_lightness_step_value, color_picker_main_palette_selection,
    color_picker_neutral_strip_selection,
};

impl SettingsPanel {
    pub fn visual_theme_for_test(&self) -> SettingsWindowTheme {
        self.visual_theme.clone()
    }

    pub fn active_color_picker_field_for_test(&self) -> Option<SettingsFieldId> {
        self.color_picker_field.clone()
    }

    pub fn open_color_picker_for_test(
        &mut self,
        field_id: SettingsFieldId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open_color_picker(field_id, window, cx);
    }

    pub fn replace_color_picker_text_for_test(&mut self, value: &str, cx: &mut Context<Self>) {
        let Some(field_id) = self.color_picker_field.clone() else {
            return;
        };
        let Some(input) = self.color_picker_input.clone() else {
            return;
        };
        let _ = input.update(cx, |input, cx| input.set_text_for_test(value, cx));
        self.handle_field_changed(&field_id, value, cx);
    }

    pub fn apply_color_picker_swatch_for_test(&mut self, value: &str, cx: &mut Context<Self>) {
        let Some(color) = RgbColor::parse(value) else {
            return;
        };
        if self.saved_color_swatches.contains(&color) {
            self.apply_color_picker_swatch(color, cx);
        }
    }

    pub fn replace_color_picker_channel_text_for_test(
        &mut self,
        field: &str,
        value: &str,
        cx: &mut Context<Self>,
    ) {
        let Some(field) = ColorPickerChannelField::from_test_key(field) else {
            return;
        };
        self.set_focused_color_picker_channel(field);
        let Some(input) = self.color_picker_channel_input(field) else {
            return;
        };
        let _ = input.update(cx, |input, cx| input.set_text_for_test(value, cx));
        self.apply_color_picker_channel_value(field, value, cx);
    }

    pub fn focus_color_picker_channel_for_test(
        &mut self,
        field: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(field) = ColorPickerChannelField::from_test_key(field) else {
            return;
        };
        self.set_focused_color_picker_channel(field);
        let Some(input) = self.color_picker_channel_input(field) else {
            return;
        };
        let _ = input.update(cx, |input, cx| input.focus(window, cx));
    }

    pub fn materialize_color_picker_channel_for_test(
        &mut self,
        field: &str,
        cx: &mut Context<Self>,
    ) {
        let Some(field) = ColorPickerChannelField::from_test_key(field) else {
            return;
        };
        self.materialize_color_picker_channel_on_blur(field, cx);
    }

    pub fn apply_color_picker_main_palette_selection_for_test(
        &mut self,
        hue_degrees: u16,
        saturation_percent: u16,
        cx: &mut Context<Self>,
    ) {
        self.apply_color_picker_main_palette_selection(
            ColorPickerMainPaletteSelection::nearest(hue_degrees, saturation_percent),
            cx,
        );
    }

    pub fn apply_color_picker_neutral_strip_selection_for_test(
        &mut self,
        lightness_percent: u16,
        cx: &mut Context<Self>,
    ) {
        self.apply_color_picker_neutral_strip_selection(
            ColorPickerNeutralStripSelection::nearest(lightness_percent),
            cx,
        );
    }

    pub fn apply_color_picker_lightness_for_test(
        &mut self,
        lightness: u16,
        cx: &mut Context<Self>,
    ) {
        let lightness = ColorPickerNeutralStripSelection::nearest(lightness).lightness_percent();
        self.apply_color_picker_lightness(lightness, cx);
    }

    pub fn close_color_picker_for_test(&mut self, cx: &mut Context<Self>) {
        self.close_color_picker(cx);
    }

    pub fn color_preview_for_test(&self, field_id: &SettingsFieldId) -> Option<String> {
        self.color_preview_for_field(field_id).map(RgbColor::to_hex)
    }

    pub fn color_picker_channel_values_for_test(&self, cx: &App) -> BTreeMap<String, String> {
        ColorPickerChannelField::ALL
            .into_iter()
            .filter_map(|field| {
                self.color_picker_channel_input(field).map(|input| {
                    (
                        String::from(field.test_key()),
                        input.read(cx).text_for_test(),
                    )
                })
            })
            .collect()
    }

    pub fn color_picker_lightness_value_for_test(&self) -> Option<u16> {
        color_picker_lightness_step_value(self.current_color_picker_color())
    }

    pub fn color_picker_main_palette_values_for_test(&self) -> Option<(u16, u16)> {
        color_picker_main_palette_selection(self.current_color_picker_color())
            .map(|selection| (selection.hue_degrees(), selection.saturation_percent()))
    }

    pub fn color_picker_neutral_strip_value_for_test(&self) -> Option<u16> {
        color_picker_neutral_strip_selection(self.current_color_picker_color())
            .map(ColorPickerNeutralStripSelection::lightness_percent)
    }
}
