use gpui::canvas;

use super::color_paint::{
    paint_color_picker_lightness_bar, paint_color_picker_main_palette,
    paint_color_picker_neutral_strip,
};
use super::*;
use crate::color_picker::{
    ColorPickerChannelField, ColorPickerMainPaletteSelection, ColorPickerPaletteSelection,
    color_picker_lightness_step_value, color_picker_main_palette_selection,
    color_picker_neutral_strip_selection, color_picker_palette_selection,
};

const COLOR_PICKER_SWATCH_FALLBACK: u32 = 0x2B3137;
const COLOR_PICKER_PALETTE_CELL_SIZE: f32 = 14.0;
const COLOR_PICKER_INTERIOR_WIDTH: f32 =
    COLOR_PICKER_PALETTE_CELL_SIZE * ColorPickerMainPaletteSelection::HUES.len() as f32;
const COLOR_PICKER_WIDTH: f32 = COLOR_PICKER_INTERIOR_WIDTH + 24.0;
const COLOR_PICKER_SQUARE_SIZE: f32 = 34.0;
const COLOR_PICKER_MAIN_PALETTE_HEIGHT: f32 =
    COLOR_PICKER_PALETTE_CELL_SIZE * ColorPickerMainPaletteSelection::SATURATIONS.len() as f32;
const COLOR_PICKER_NEUTRAL_STRIP_HEIGHT: f32 = COLOR_PICKER_PALETTE_CELL_SIZE;
const COLOR_PICKER_LIGHTNESS_BAR_HEIGHT: f32 = COLOR_PICKER_PALETTE_CELL_SIZE;
const COLOR_PICKER_SAVED_SWATCH_SIZE: f32 = 24.0;
const COLOR_ROW_PREVIEW_SIZE: f32 = 24.0;

impl SettingsPanel {
    pub(super) fn render_color_preview_swatch(
        &self,
        field_id: SettingsFieldId,
        picker_open: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let preview = self
            .color_preview_for_field(&field_id)
            .unwrap_or(fallback_color());
        let border = if picker_open {
            self.visual_theme.input.active_border
        } else {
            self.visual_theme.popup.border
        };

        div()
            .id(SharedString::from(format!(
                "settings-color-preview-{}",
                element_id_suffix(field_id.as_str())
            )))
            .flex_none()
            .size(px(COLOR_ROW_PREVIEW_SIZE))
            .rounded_sm()
            .border_1()
            .border_color(theme_color(border))
            .bg(rgb(preview.packed_rgb()))
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, window, cx| {
                    this.activate_color_preview_swatch(field_id.clone(), window, cx);
                }),
            )
    }

    pub(super) fn render_color_picker_popup(
        &self,
        row: &SettingsRow,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let input = self
            .color_picker_input
            .clone()
            .expect("active color picker should have an input");
        let selected = self.color_preview_for_field(row.field_id());
        let swatch = selected.unwrap_or(fallback_color()).packed_rgb();

        div()
            .id(SharedString::from("settings-color-picker"))
            .w(px(COLOR_PICKER_WIDTH))
            .flex_none()
            .occlude()
            .rounded_md()
            .border_1()
            .border_color(theme_color(self.visual_theme.popup.border))
            .bg(theme_color(self.visual_theme.popup.background))
            .px_3()
            .py_2()
            .on_mouse_down_out(cx.listener(Self::on_color_picker_popup_mouse_down_out))
            .on_mouse_move(cx.listener(Self::on_color_picker_popup_mouse_move))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(Self::on_color_picker_popup_mouse_up_inside),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(Self::on_color_picker_popup_mouse_up_outside),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex_none()
                            .size(px(COLOR_PICKER_SQUARE_SIZE))
                            .rounded_sm()
                            .border_1()
                            .border_color(theme_color(self.visual_theme.popup.border))
                            .bg(rgb(swatch)),
                    )
                    .child(div().flex_1().min_w_0().child(input)),
            )
            .child(
                div()
                    .mt_3()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(self.render_color_picker_main_palette(selected, cx))
                            .child(self.render_color_picker_neutral_strip(selected, cx))
                            .child(self.render_color_picker_lightness_bar(selected, cx)),
                    )
                    .child(self.render_color_picker_channel_group(ColorPickerChannelField::RGB))
                    .child(self.render_color_picker_channel_group(ColorPickerChannelField::HSL))
                    .child(self.render_color_picker_channel_group(ColorPickerChannelField::HSV))
                    .child(
                        div()
                            .text_size(px((self.font_size - 2.0).max(11.0)))
                            .text_color(theme_color(self.visual_theme.popup.muted_foreground))
                            .child("Saved colors"),
                    )
                    .child(div().flex().flex_wrap().gap_2().children(
                        self.saved_color_swatches.iter().copied().map(|color| {
                            self.render_saved_color_picker_swatch(color, selected, cx)
                        }),
                    )),
            )
    }

    fn render_color_picker_main_palette(
        &self,
        selected: Option<RgbColor>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let palette_bounds = self.color_picker_main_palette_bounds.clone();
        let selection = color_picker_main_palette_selection(selected);
        let selection_border = theme_color(self.visual_theme.popup.foreground);

        div()
            .id("settings-color-picker-main-palette")
            .relative()
            .flex_none()
            .w_full()
            .h(px(COLOR_PICKER_MAIN_PALETTE_HEIGHT))
            .rounded_sm()
            .border_1()
            .border_color(theme_color(self.visual_theme.popup.border))
            .bg(theme_color(self.visual_theme.row.background))
            .overflow_hidden()
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(Self::on_color_picker_main_palette_mouse_down),
            )
            .child(
                canvas(|_, _, _| (), {
                    move |bounds, _, window, _| {
                        *palette_bounds.borrow_mut() = Some(bounds);
                        paint_color_picker_main_palette(
                            bounds,
                            selection,
                            selection_border,
                            window,
                        );
                    }
                })
                .size_full(),
            )
    }

    fn render_color_picker_neutral_strip(
        &self,
        selected: Option<RgbColor>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let neutral_bounds = self.color_picker_neutral_strip_bounds.clone();
        let selection = color_picker_neutral_strip_selection(selected);
        let selection_border = theme_color(self.visual_theme.popup.foreground);

        div()
            .id("settings-color-picker-neutral-strip")
            .relative()
            .flex_none()
            .w_full()
            .h(px(COLOR_PICKER_NEUTRAL_STRIP_HEIGHT))
            .rounded_sm()
            .border_1()
            .border_color(theme_color(self.visual_theme.popup.border))
            .bg(theme_color(self.visual_theme.row.background))
            .overflow_hidden()
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(Self::on_color_picker_neutral_strip_mouse_down),
            )
            .child(
                canvas(|_, _, _| (), {
                    move |bounds, _, window, _| {
                        *neutral_bounds.borrow_mut() = Some(bounds);
                        paint_color_picker_neutral_strip(
                            bounds,
                            selection,
                            selection_border,
                            window,
                        );
                    }
                })
                .size_full(),
            )
    }

    fn render_color_picker_lightness_bar(
        &self,
        selected: Option<RgbColor>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let lightness_bounds = self.color_picker_lightness_bar_bounds.clone();
        let selection = color_picker_palette_selection(selected).unwrap_or(
            ColorPickerPaletteSelection::Chromatic(ColorPickerMainPaletteSelection::new(0, 100)),
        );
        let lightness = color_picker_lightness_step_value(selected).unwrap_or(48);
        let selection_border = theme_color(self.visual_theme.popup.foreground);

        div()
            .id("settings-color-picker-lightness-bar")
            .relative()
            .flex_none()
            .w_full()
            .h(px(COLOR_PICKER_LIGHTNESS_BAR_HEIGHT))
            .rounded_sm()
            .border_1()
            .border_color(theme_color(self.visual_theme.popup.border))
            .bg(theme_color(self.visual_theme.row.background))
            .overflow_hidden()
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(Self::on_color_picker_lightness_bar_mouse_down),
            )
            .child(
                canvas(|_, _, _| (), {
                    move |bounds, _, window, _| {
                        *lightness_bounds.borrow_mut() = Some(bounds);
                        paint_color_picker_lightness_bar(
                            bounds,
                            selection,
                            lightness,
                            selection_border,
                            window,
                        );
                    }
                })
                .size_full(),
            )
    }

    fn render_color_picker_channel_group(
        &self,
        fields: [ColorPickerChannelField; 3],
    ) -> impl IntoElement + use<> {
        div().flex().items_center().gap_2().children(
            fields
                .into_iter()
                .map(|field| self.render_color_picker_channel_input(field)),
        )
    }

    fn render_color_picker_channel_input(
        &self,
        field: ColorPickerChannelField,
    ) -> impl IntoElement + use<> {
        let input = self
            .color_picker_channel_input(field)
            .expect("active color picker should have synchronized numeric inputs");

        div()
            .flex_1()
            .min_w_0()
            .flex()
            .gap_1()
            .items_center()
            .child(
                div()
                    .flex_none()
                    .text_size(px((self.font_size - 2.0).max(11.0)))
                    .text_color(theme_color(self.visual_theme.popup.muted_foreground))
                    .child(format!("{}:", field.label())),
            )
            .child(div().flex_1().min_w_0().child(input))
    }

    fn render_saved_color_picker_swatch(
        &self,
        color: RgbColor,
        selected: Option<RgbColor>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let hex = color.to_hex();
        let active = selected == Some(color);

        div()
            .id(SharedString::from(format!(
                "settings-color-picker-swatch-{hex}"
            )))
            .flex_none()
            .size(px(COLOR_PICKER_SAVED_SWATCH_SIZE))
            .rounded_sm()
            .border_1()
            .border_color(theme_color(if active {
                self.visual_theme.popup.foreground
            } else {
                self.visual_theme.popup.border
            }))
            .bg(rgb(color.packed_rgb()))
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.apply_color_picker_swatch(color, cx);
                }),
            )
    }
}

fn fallback_color() -> RgbColor {
    RgbColor::new(
        ((COLOR_PICKER_SWATCH_FALLBACK >> 16) & 0xFF) as u8,
        ((COLOR_PICKER_SWATCH_FALLBACK >> 8) & 0xFF) as u8,
        (COLOR_PICKER_SWATCH_FALLBACK & 0xFF) as u8,
    )
}
