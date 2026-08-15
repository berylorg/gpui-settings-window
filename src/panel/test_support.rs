use super::*;
use crate::color_picker::{
    ColorPickerMainPaletteSelection, ColorPickerNeutralStripSelection,
    color_picker_lightness_step_value, color_picker_main_palette_selection,
    color_picker_neutral_strip_selection,
};

impl SettingsPanel {
    pub fn demand_page_split_range_for_test(&mut self, range: std::ops::Range<usize>) {
        self.split_pager.ensure_demand(range);
    }

    pub fn deliver_page_split_result_without_notify_for_test(
        &mut self,
        result: SettingsPageSplitPageResult,
    ) -> Result<SettingsPageSplitDelivery, SettingsPageSplitDeliveryError> {
        self.split_pager.deliver(result)
    }

    pub fn focus_page_split_position_for_test(
        &mut self,
        position: usize,
        cx: &mut Context<Self>,
    ) -> bool {
        let focused = self.split_pager.focus_position(position);
        if focused {
            self.reveal_split_item_index(position, self.split_pager.logical_item_count());
            cx.notify();
        }
        focused
    }

    pub fn focused_page_split_position_for_test(&self) -> Option<usize> {
        self.split_pager.focused_position()
    }

    pub fn focus_page_split_container_for_test(&self, window: &mut Window) {
        window.focus(&self.split_focus_handle);
    }

    pub fn focus_panel_for_test(&self, window: &mut Window) {
        window.focus(&self.focus_handle);
    }

    pub fn page_split_container_focused_for_test(&self, window: &Window) -> bool {
        self.split_focus_handle.is_focused(window)
    }

    pub fn select_page_split_pointer_capture_for_test(
        &mut self,
        page_id: SettingsPageId,
        source_key: crate::SettingsPageSplitSourceKey,
        logical_position: usize,
        item_id: SettingsPageSplitItemId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        self.select_split_item_from_pointer(
            page_id,
            source_key,
            logical_position,
            item_id,
            window,
            cx,
        )
    }

    pub fn release_page_split_for_test(&mut self) {
        self.suspend_page_split();
    }

    pub fn visual_theme_for_test(&self) -> SettingsWindowTheme {
        self.visual_theme.clone()
    }

    pub fn scrollbar_active_states_for_test(&self) -> (bool, bool) {
        (
            self.navigation_scrollbar
                .current_owner()
                .is_some_and(|owner| {
                    self.navigation_scrollbar
                        .opacity_at(owner, std::time::Instant::now())
                        .unwrap_or(0.0)
                        > 0.0
                }),
            self.content_scrollbar.current_owner().is_some_and(|owner| {
                self.content_scrollbar
                    .opacity_at(owner, std::time::Instant::now())
                    .unwrap_or(0.0)
                    > 0.0
            }),
        )
    }

    pub fn split_scrollbar_active_for_test(&self) -> bool {
        self.split_scrollbar.current_owner().is_some_and(|owner| {
            self.split_scrollbar
                .opacity_at(owner, std::time::Instant::now())
                .unwrap_or(0.0)
                > 0.0
        })
    }

    pub fn scrollbar_owners_for_test(
        &self,
    ) -> (
        Option<gpui_scrollbar::ScrollbarOwnerKey>,
        Option<gpui_scrollbar::ScrollbarOwnerKey>,
        Option<gpui_scrollbar::ScrollbarOwnerKey>,
    ) {
        (
            self.navigation_scrollbar.current_owner(),
            self.content_scrollbar.current_owner(),
            self.split_scrollbar.current_owner(),
        )
    }

    pub fn scroll_handles_for_test(&self) -> (ScrollHandle, ScrollHandle, ScrollHandle) {
        (
            self.navigation_scroll_handle.clone(),
            self.scroll_handle.clone(),
            self.split_scroll_handle.clone(),
        )
    }

    pub fn scrollbar_states_for_test(&self) -> (ScrollbarState, ScrollbarState, ScrollbarState) {
        (
            self.navigation_scrollbar.clone(),
            self.content_scrollbar.clone(),
            self.split_scrollbar.clone(),
        )
    }

    pub fn record_navigation_scrollbar_activity_for_test(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.note_navigation_scrollbar_activity(window, cx);
    }

    pub fn record_content_scrollbar_activity_for_test(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.note_content_scrollbar_activity(window, cx);
    }

    pub fn record_split_scrollbar_activity_for_test(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.note_split_scrollbar_activity(window, cx);
    }

    pub fn page_split_render_metrics_for_test(&self) -> Option<(usize, usize, usize, f32)> {
        let split = self.model.selected_page().paged_split_source()?;
        let item_count = split.logical_item_count();
        let range = page_local_split_render_window(
            item_count,
            self.split_scroll_handle.offset().y,
            self.split_scroll_handle.bounds().size.height,
        );
        Some((
            item_count,
            range.start,
            range.end,
            page_local_split_total_height(item_count),
        ))
    }

    pub fn page_split_scroll_metrics_for_test(&self) -> (f32, f32) {
        (
            f32::from(-self.split_scroll_handle.offset().y).max(0.0),
            f32::from(self.split_scroll_handle.max_offset().height),
        )
    }

    pub fn set_page_split_scroll_offset_for_test(
        &mut self,
        scroll_top: f32,
        cx: &mut Context<Self>,
    ) {
        let current = self.split_scroll_handle.offset();
        self.split_scroll_handle
            .set_offset(point(current.x, px(-scroll_top.max(0.0))));
        cx.notify();
    }

    pub fn set_content_scroll_offset_for_test(&mut self, scroll_top: f32, cx: &mut Context<Self>) {
        let current = self.scroll_handle.offset();
        self.scroll_handle
            .set_offset(point(current.x, px(-scroll_top.max(0.0))));
        cx.notify();
    }

    pub fn active_color_picker_field_for_test(&self) -> Option<SettingsFieldId> {
        self.color_picker_field.clone()
    }

    pub fn focused_field_for_test(&self, window: &Window, cx: &App) -> Option<SettingsFieldId> {
        self.fields.iter().find_map(|(field_id, input)| {
            input
                .read(cx)
                .tab_focus_handle()
                .is_focused(window)
                .then(|| field_id.clone())
        })
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

    pub fn apply_color_picker_swatch_for_test(
        &mut self,
        swatch_id: SettingsSavedColorSwatchId,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.focus_saved_color_swatch(swatch_id, cx) {
            return false;
        }
        self.apply_focused_saved_color_swatch(cx)
    }

    pub fn focus_saved_color_swatch_for_test(
        &mut self,
        swatch_id: SettingsSavedColorSwatchId,
        cx: &mut Context<Self>,
    ) -> bool {
        self.focus_saved_color_swatch(swatch_id, cx)
    }

    pub fn move_saved_color_swatch_focus_for_test(
        &mut self,
        delta: isize,
        cx: &mut Context<Self>,
    ) -> bool {
        self.focus_relative_saved_color_swatch(delta, cx)
    }

    pub fn focused_saved_color_swatch_for_test(&self) -> Option<SettingsSavedColorSwatchId> {
        self.color_picker_focused_swatch.clone()
    }

    pub fn selected_saved_color_swatch_for_test(&self) -> Option<SettingsSavedColorSwatchId> {
        self.color_picker_selected_swatch.clone()
    }

    pub fn focus_saved_color_grid_for_test(&self, window: &mut Window) {
        window.focus(&self.saved_color_grid_focus_handle);
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
