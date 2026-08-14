use super::*;

impl SettingsWindowView {
    /// Returns the current visual theme.
    pub fn visual_theme_for_test(&self, cx: &App) -> crate::SettingsWindowTheme {
        self.settings_panel.read(cx).visual_theme_for_test()
    }

    pub fn scrollbar_active_states_for_test(&self, cx: &App) -> (bool, bool) {
        self.settings_panel
            .read(cx)
            .scrollbar_active_states_for_test()
    }

    pub fn split_scrollbar_active_for_test(&self, cx: &App) -> bool {
        self.settings_panel
            .read(cx)
            .split_scrollbar_active_for_test()
    }

    pub fn scrollbar_owners_for_test(
        &self,
        cx: &App,
    ) -> (
        Option<gpui_scrollbar::ScrollbarOwnerKey>,
        Option<gpui_scrollbar::ScrollbarOwnerKey>,
        Option<gpui_scrollbar::ScrollbarOwnerKey>,
    ) {
        self.settings_panel.read(cx).scrollbar_owners_for_test()
    }

    pub fn scroll_handles_for_test(
        &self,
        cx: &App,
    ) -> (gpui::ScrollHandle, gpui::ScrollHandle, gpui::ScrollHandle) {
        self.settings_panel.read(cx).scroll_handles_for_test()
    }

    pub fn scrollbar_states_for_test(
        &self,
        cx: &App,
    ) -> (
        gpui_scrollbar::ScrollbarState,
        gpui_scrollbar::ScrollbarState,
        gpui_scrollbar::ScrollbarState,
    ) {
        self.settings_panel.read(cx).scrollbar_states_for_test()
    }

    pub fn record_navigation_scrollbar_activity_for_test(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.settings_panel.update(cx, |panel, cx| {
            panel.record_navigation_scrollbar_activity_for_test(window, cx);
        });
    }

    pub fn record_content_scrollbar_activity_for_test(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.settings_panel.update(cx, |panel, cx| {
            panel.record_content_scrollbar_activity_for_test(window, cx);
        });
    }

    pub fn record_split_scrollbar_activity_for_test(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.settings_panel.update(cx, |panel, cx| {
            panel.record_split_scrollbar_activity_for_test(window, cx);
        });
    }

    pub fn page_split_render_metrics_for_test(
        &self,
        cx: &App,
    ) -> Option<(usize, usize, usize, f32)> {
        self.settings_panel
            .read(cx)
            .page_split_render_metrics_for_test()
    }

    pub fn page_split_scroll_metrics_for_test(&self, cx: &App) -> (f32, f32) {
        self.settings_panel
            .read(cx)
            .page_split_scroll_metrics_for_test()
    }

    pub fn set_page_split_scroll_offset_for_test(
        &mut self,
        scroll_top: f32,
        cx: &mut Context<Self>,
    ) {
        self.settings_panel.update(cx, |panel, cx| {
            panel.set_page_split_scroll_offset_for_test(scroll_top, cx);
        });
    }

    pub fn set_content_scroll_offset_for_test(&mut self, scroll_top: f32, cx: &mut Context<Self>) {
        self.settings_panel.update(cx, |panel, cx| {
            panel.set_content_scroll_offset_for_test(scroll_top, cx);
        });
    }

    /// Replaces a field's text directly and emits the same event as user input.
    pub fn replace_field_text_for_test(
        &mut self,
        field_id: &crate::SettingsFieldId,
        value: &str,
        cx: &mut Context<Self>,
    ) -> bool {
        self.settings_panel.update(cx, |panel, cx| {
            panel.replace_field_text_for_test(field_id, value, cx)
        })
    }

    /// Reads the current synchronized text for a field input.
    pub fn field_text_for_test(
        &self,
        field_id: &crate::SettingsFieldId,
        cx: &App,
    ) -> Option<String> {
        self.settings_panel
            .read(cx)
            .field_text_for_test(field_id, cx)
    }

    /// Returns the active color picker field, when one is open.
    pub fn active_color_picker_field_for_test(&self, cx: &App) -> Option<crate::SettingsFieldId> {
        self.settings_panel
            .read(cx)
            .active_color_picker_field_for_test()
    }

    pub fn focused_field_for_test(
        &self,
        window: &Window,
        cx: &App,
    ) -> Option<crate::SettingsFieldId> {
        self.settings_panel
            .read(cx)
            .focused_field_for_test(window, cx)
    }

    /// Opens the color picker for a color field.
    pub fn open_color_picker_for_test(
        &mut self,
        field_id: crate::SettingsFieldId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.settings_panel.update(cx, |panel, cx| {
            panel.open_color_picker_for_test(field_id, window, cx);
        });
    }

    /// Replaces the active color picker's text.
    pub fn replace_color_picker_text_for_test(&mut self, value: &str, cx: &mut Context<Self>) {
        self.settings_panel.update(cx, |panel, cx| {
            panel.replace_color_picker_text_for_test(value, cx);
        });
    }

    /// Applies one saved color picker swatch by stable identity.
    pub fn apply_color_picker_swatch_for_test(
        &mut self,
        swatch_id: crate::SettingsSavedColorSwatchId,
        cx: &mut Context<Self>,
    ) -> bool {
        self.settings_panel.update(cx, |panel, cx| {
            panel.apply_color_picker_swatch_for_test(swatch_id, cx)
        })
    }

    pub fn focus_saved_color_swatch_for_test(
        &mut self,
        swatch_id: crate::SettingsSavedColorSwatchId,
        cx: &mut Context<Self>,
    ) -> bool {
        self.settings_panel.update(cx, |panel, cx| {
            panel.focus_saved_color_swatch_for_test(swatch_id, cx)
        })
    }

    pub fn move_saved_color_swatch_focus_for_test(
        &mut self,
        delta: isize,
        cx: &mut Context<Self>,
    ) -> bool {
        self.settings_panel.update(cx, |panel, cx| {
            panel.move_saved_color_swatch_focus_for_test(delta, cx)
        })
    }

    pub fn focused_saved_color_swatch_for_test(
        &self,
        cx: &App,
    ) -> Option<crate::SettingsSavedColorSwatchId> {
        self.settings_panel
            .read(cx)
            .focused_saved_color_swatch_for_test()
    }

    pub fn selected_saved_color_swatch_for_test(
        &self,
        cx: &App,
    ) -> Option<crate::SettingsSavedColorSwatchId> {
        self.settings_panel
            .read(cx)
            .selected_saved_color_swatch_for_test()
    }

    pub fn focus_saved_color_grid_for_test(&self, window: &mut Window, cx: &App) {
        self.settings_panel
            .read(cx)
            .focus_saved_color_grid_for_test(window);
    }

    /// Replaces one active color picker channel input.
    pub fn replace_color_picker_channel_text_for_test(
        &mut self,
        field: &str,
        value: &str,
        cx: &mut Context<Self>,
    ) {
        self.settings_panel.update(cx, |panel, cx| {
            panel.replace_color_picker_channel_text_for_test(field, value, cx);
        });
    }

    /// Focuses one active color picker channel input.
    pub fn focus_color_picker_channel_for_test(
        &mut self,
        field: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.settings_panel.update(cx, |panel, cx| {
            panel.focus_color_picker_channel_for_test(field, window, cx);
        });
    }

    /// Materializes an empty focused channel draft as the current channel value.
    pub fn materialize_color_picker_channel_for_test(
        &mut self,
        field: &str,
        cx: &mut Context<Self>,
    ) {
        self.settings_panel.update(cx, |panel, cx| {
            panel.materialize_color_picker_channel_for_test(field, cx);
        });
    }

    /// Applies a chromatic palette selection.
    pub fn apply_color_picker_main_palette_selection_for_test(
        &mut self,
        hue_degrees: u16,
        saturation_percent: u16,
        cx: &mut Context<Self>,
    ) {
        self.settings_panel.update(cx, |panel, cx| {
            panel.apply_color_picker_main_palette_selection_for_test(
                hue_degrees,
                saturation_percent,
                cx,
            );
        });
    }

    /// Applies a neutral strip selection.
    pub fn apply_color_picker_neutral_strip_selection_for_test(
        &mut self,
        lightness_percent: u16,
        cx: &mut Context<Self>,
    ) {
        self.settings_panel.update(cx, |panel, cx| {
            panel.apply_color_picker_neutral_strip_selection_for_test(lightness_percent, cx);
        });
    }

    /// Applies a lightness bar value.
    pub fn apply_color_picker_lightness_for_test(
        &mut self,
        lightness: u16,
        cx: &mut Context<Self>,
    ) {
        self.settings_panel.update(cx, |panel, cx| {
            panel.apply_color_picker_lightness_for_test(lightness, cx);
        });
    }

    /// Closes the active color picker.
    pub fn close_color_picker_for_test(&mut self, cx: &mut Context<Self>) {
        self.settings_panel
            .update(cx, |panel, cx| panel.close_color_picker_for_test(cx));
    }

    /// Returns the preview color for a color field.
    pub fn color_preview_for_test(
        &self,
        field_id: &crate::SettingsFieldId,
        cx: &App,
    ) -> Option<String> {
        self.settings_panel
            .read(cx)
            .color_preview_for_test(field_id)
    }

    /// Returns current active picker channel values by stable test key.
    pub fn color_picker_channel_values_for_test(
        &self,
        cx: &App,
    ) -> std::collections::BTreeMap<String, String> {
        self.settings_panel
            .read(cx)
            .color_picker_channel_values_for_test(cx)
    }

    /// Returns the active color picker's snapped lightness value.
    pub fn color_picker_lightness_value_for_test(&self, cx: &App) -> Option<u16> {
        self.settings_panel
            .read(cx)
            .color_picker_lightness_value_for_test()
    }

    /// Returns the active color picker's chromatic palette selection.
    pub fn color_picker_main_palette_values_for_test(&self, cx: &App) -> Option<(u16, u16)> {
        self.settings_panel
            .read(cx)
            .color_picker_main_palette_values_for_test()
    }

    /// Returns the active color picker's neutral strip selection.
    pub fn color_picker_neutral_strip_value_for_test(&self, cx: &App) -> Option<u16> {
        self.settings_panel
            .read(cx)
            .color_picker_neutral_strip_value_for_test()
    }

    /// Selects a section directly and emits the same event as the navigation row.
    pub fn select_section_for_test(
        &mut self,
        section_id: crate::SettingsSectionId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.settings_panel.update(cx, |panel, cx| {
            panel.select_section_for_test(section_id, window, cx);
        });
        self.model = self.settings_panel.read(cx).model().clone();
    }

    /// Emits an accept request.
    pub fn accept_for_test(&mut self, cx: &mut Context<Self>) {
        self.settings_panel
            .update(cx, |panel, cx| panel.accept_for_test(cx));
    }

    /// Emits an apply request.
    pub fn apply_for_test(&mut self, cx: &mut Context<Self>) {
        self.settings_panel
            .update(cx, |panel, cx| panel.apply_for_test(cx));
    }

    /// Emits a row action request when the row carries that action.
    pub fn request_row_action_for_test(
        &mut self,
        field_id: crate::SettingsFieldId,
        action_id: crate::SettingsRowActionId,
        cx: &mut Context<Self>,
    ) -> bool {
        self.settings_panel.update(cx, |panel, cx| {
            panel.request_row_action_for_test(field_id, action_id, cx)
        })
    }

    /// Emits a page navigation request when the target page exists.
    pub fn request_page_navigation_for_test(
        &mut self,
        page_id: crate::SettingsPageId,
        cx: &mut Context<Self>,
    ) -> bool {
        self.settings_panel.update(cx, |panel, cx| {
            panel.request_page_navigation_for_test(page_id, cx)
        })
    }

    /// Emits a page action request when the page carries an enabled action.
    pub fn request_page_action_for_test(
        &mut self,
        page_id: crate::SettingsPageId,
        action_id: crate::SettingsPageActionId,
        cx: &mut Context<Self>,
    ) -> bool {
        self.settings_panel.update(cx, |panel, cx| {
            panel.request_page_action_for_test(page_id, action_id, cx)
        })
    }

    /// Emits a split item selection request when the page carries that item.
    pub fn request_page_split_item_for_test(
        &mut self,
        page_id: crate::SettingsPageId,
        item_id: crate::SettingsPageSplitItemId,
        cx: &mut Context<Self>,
    ) -> bool {
        self.settings_panel.update(cx, |panel, cx| {
            panel.request_page_split_item_for_test(page_id, item_id, cx)
        })
    }

    /// Emits a field change when a choice field carries that option.
    pub fn select_choice_for_test(
        &mut self,
        field_id: crate::SettingsFieldId,
        value: String,
        cx: &mut Context<Self>,
    ) -> bool {
        self.settings_panel.update(cx, |panel, cx| {
            panel.select_choice_for_test(field_id, value, cx)
        })
    }

    /// Emits a cancel request.
    pub fn cancel_for_test(&mut self, cx: &mut Context<Self>) {
        self.settings_panel
            .update(cx, |panel, cx| panel.cancel_for_test(cx));
    }

    /// Requests the same close handling as the OS window close callback.
    pub fn request_close_for_test(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.handle_window_close_requested(window, cx)
    }
}
