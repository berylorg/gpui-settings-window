use super::*;
use gpui::{
    AnchoredPositionMode, AnyElement, Corner, StatefulInteractiveElement, anchored, deferred,
};
use gpui_scrollbar::{
    Axis, ScrollbarStyle, ScrollbarVisibilityPolicy, render_scroll_handle_scrollbar,
};

impl Render for SettingsPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("settings-panel")
            .size_full()
            .bg(theme_color(self.visual_theme.window_background))
            .p_4()
            .overflow_hidden()
            .child(
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .rounded_md()
                    .border_1()
                    .border_color(theme_color(self.visual_theme.panel.border))
                    .bg(theme_color(self.visual_theme.panel.background))
                    .p_4()
                    .text_color(theme_color(self.visual_theme.panel.foreground))
                    .text_size(px(self.font_size))
                    .track_focus(&self.focus_handle)
                    .key_context(PANEL_KEY_CONTEXT)
                    .on_action(cx.listener(Self::cancel))
                    .on_action(cx.listener(Self::focus_next))
                    .on_action(cx.listener(Self::focus_prev))
                    .child(
                        div()
                            .text_size(px(18.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Settings"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_h(px(0.0))
                            .overflow_hidden()
                            .flex()
                            .gap_4()
                            .child(self.render_section_navigation(cx))
                            .child(self.render_selected_section(cx)),
                    )
                    .child(self.render_buttons(cx)),
            )
    }
}

impl SettingsPanel {
    fn render_section_navigation(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        div()
            .id("settings-sections")
            .w(px(NAVIGATION_WIDTH))
            .h_full()
            .min_h(px(0.0))
            .flex_none()
            .relative()
            .overflow_hidden()
            .on_mouse_move(cx.listener(Self::note_navigation_scrollbar_motion))
            .on_scroll_wheel(cx.listener(Self::note_navigation_scrollbar_scroll))
            .child(
                div()
                    .id("settings-sections-scroll")
                    .overflow_y_scroll()
                    .overflow_x_hidden()
                    .w_full()
                    .h_full()
                    .track_scroll(&self.navigation_scroll_handle)
                    .child(div().w_full().flex().flex_col().gap_1().children(
                        self.model.sections().iter().map(|section| {
                            self.render_section_button(
                                section.section_id().clone(),
                                section.label().to_owned(),
                                cx,
                            )
                        }),
                    )),
            )
            .children(self.render_vertical_scrollbar(
                "settings-sections-scrollbar",
                &self.navigation_scroll_handle,
                self.navigation_scrollbar_visibility_policy(cx.entity()),
            ))
    }

    fn render_section_button(
        &self,
        section_id: SettingsSectionId,
        label: String,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let active = section_id == *self.model.selected_section_id();
        let button_theme = self.visual_theme.navigation_button.clone();
        let state = if active {
            button_theme.active.clone()
        } else {
            button_theme.normal.clone()
        };
        let font_weight = FontWeight(button_theme.font_weight as f32);
        let hover = button_theme.hover;
        let pressed = button_theme.active;

        div()
            .id(SharedString::from(format!(
                "settings-section-{}",
                element_id_suffix(section_id.as_str())
            )))
            .flex_none()
            .w_full()
            .px_3()
            .py_2()
            .rounded_sm()
            .border_1()
            .border_color(theme_color(state.border))
            .bg(theme_color(state.background))
            .hover(move |style| {
                style
                    .border_color(theme_color(hover.border))
                    .bg(theme_color(hover.background))
                    .text_color(theme_color(hover.foreground))
            })
            .active(move |style| {
                style
                    .border_color(theme_color(pressed.border))
                    .bg(theme_color(pressed.background))
                    .text_color(theme_color(pressed.foreground))
            })
            .font_weight(font_weight)
            .text_color(theme_color(state.foreground))
            .cursor_pointer()
            .child(label)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, window, cx| {
                    this.select_section(section_id.clone(), window, cx);
                }),
            )
    }

    fn render_selected_section(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        div()
            .flex_1()
            .min_w_0()
            .min_h(px(0.0))
            .overflow_hidden()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .text_size(px(16.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.model.selected_section().label().to_owned()),
            )
            .child(
                div()
                    .id("settings-scroll")
                    .flex_1()
                    .min_h(px(0.0))
                    .relative()
                    .overflow_hidden()
                    .on_mouse_move(cx.listener(Self::note_content_scrollbar_motion))
                    .on_scroll_wheel(cx.listener(Self::note_content_scrollbar_scroll))
                    .child(
                        div()
                            .id("settings-scroll-surface")
                            .overflow_y_scroll()
                            .overflow_x_hidden()
                            .w_full()
                            .h_full()
                            .track_scroll(&self.scroll_handle)
                            .child(
                                div().w_full().flex().flex_col().gap_2().children(
                                    self.model
                                        .selected_rows()
                                        .iter()
                                        .map(|row| self.render_row(row, cx)),
                                ),
                            ),
                    )
                    .children(self.render_vertical_scrollbar(
                        "settings-selected-section-scrollbar",
                        &self.scroll_handle,
                        self.content_scrollbar_visibility_policy(cx.entity()),
                    )),
            )
    }

    fn render_vertical_scrollbar(
        &self,
        id: &'static str,
        scroll_handle: &ScrollHandle,
        visibility: ScrollbarVisibilityPolicy,
    ) -> Option<AnyElement> {
        let style = ScrollbarStyle {
            thumb_color: self.visual_theme.panel.muted_foreground.packed_rgb(),
            ..ScrollbarStyle::default()
        };

        render_scroll_handle_scrollbar(id, scroll_handle, Axis::Vertical, style, visibility)
    }

    fn render_row(&self, row: &SettingsRow, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let input = self
            .input_for_field(row.field_id())
            .expect("settings field should exist");
        let picker_open = self.color_picker_field.as_ref() == Some(row.field_id());
        let color_preview = (row.kind() == crate::SettingsFieldKind::Color)
            .then(|| self.render_color_preview_swatch(row.field_id().clone(), picker_open, cx));

        div()
            .id(SharedString::from(format!(
                "settings-row-{}",
                element_id_suffix(row.field_id().as_str())
            )))
            .flex()
            .items_start()
            .gap_4()
            .rounded_sm()
            .border_1()
            .border_color(theme_color(self.visual_theme.row.border))
            .bg(theme_color(self.visual_theme.row.background))
            .px_3()
            .py_3()
            .child(
                div()
                    .w(px(LABEL_COLUMN_WIDTH))
                    .pt_2()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .text_color(theme_color(self.visual_theme.row.muted_foreground))
                    .child(row.label().to_owned())
                    .when_some(row.subtext(), |element, subtext| {
                        element.child(
                            div()
                                .text_size(px((self.font_size - 2.0).max(11.0)))
                                .child(subtext.to_owned()),
                        )
                    }),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .when(
                                row.kind() == crate::SettingsFieldKind::MultilineText,
                                |element| element.items_start(),
                            )
                            .when(
                                row.kind() != crate::SettingsFieldKind::MultilineText,
                                |element| element.items_center(),
                            )
                            .gap_2()
                            .min_w_0()
                            .children(color_preview)
                            .child(div().flex_1().min_w_0().child(input))
                            .children(row.actions().iter().cloned().map(|action| {
                                self.render_row_action_button(row.field_id().clone(), action, cx)
                            }))
                            .children(picker_open.then(|| {
                                deferred(
                                    anchored()
                                        .anchor(Corner::TopLeft)
                                        .position_mode(AnchoredPositionMode::Local)
                                        .offset(point(px(12.0), px(0.0)))
                                        .child(self.render_color_picker_popup(row, cx)),
                                )
                                .with_priority(1)
                            })),
                    )
                    .when_some(row.error(), |element, error| {
                        element.child(
                            div()
                                .text_color(theme_color(self.visual_theme.input.error_border))
                                .text_size(px((self.font_size - 1.0).max(12.0)))
                                .child(error.to_owned()),
                        )
                    }),
            )
    }

    fn render_row_action_button(
        &self,
        field_id: SettingsFieldId,
        action: crate::SettingsRowAction,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let action_id = action.action_id().clone();
        let element_id = SharedString::from(format!(
            "settings-row-action-{}-{}",
            element_id_suffix(field_id.as_str()),
            element_id_suffix(action_id.as_str())
        ));

        self.render_button_element(
            element_id,
            action.label().to_owned(),
            SettingsWindowEvent::RowActionRequested {
                field_id,
                action_id,
            },
            false,
            cx,
        )
    }

    fn render_buttons(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div().flex().justify_end().gap_2().children([
            self.render_button_element(
                SharedString::from("settings-button-ok"),
                String::from("OK"),
                SettingsWindowEvent::AcceptRequested,
                true,
                cx,
            ),
            self.render_button_element(
                SharedString::from("settings-button-apply"),
                String::from("Apply"),
                SettingsWindowEvent::ApplyRequested,
                false,
                cx,
            ),
            self.render_button_element(
                SharedString::from("settings-button-cancel"),
                String::from("Cancel"),
                SettingsWindowEvent::CancelRequested,
                false,
                cx,
            ),
        ])
    }

    fn render_button_element(
        &self,
        id: SharedString,
        label: String,
        event: SettingsWindowEvent,
        active: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let button_theme = if active {
            self.visual_theme.primary_button.clone()
        } else {
            self.visual_theme.secondary_button.clone()
        };
        let normal = button_theme.normal;
        let hover = button_theme.hover;
        let pressed = button_theme.active;
        let font_weight = FontWeight(button_theme.font_weight as f32);

        div()
            .id(id)
            .flex_none()
            .px_3()
            .py_1()
            .rounded_sm()
            .border_1()
            .border_color(theme_color(normal.border))
            .bg(theme_color(normal.background))
            .hover(move |style| {
                style
                    .border_color(theme_color(hover.border))
                    .bg(theme_color(hover.background))
                    .text_color(theme_color(hover.foreground))
            })
            .active(move |style| {
                style
                    .border_color(theme_color(pressed.border))
                    .bg(theme_color(pressed.background))
                    .text_color(theme_color(pressed.foreground))
            })
            .font_weight(font_weight)
            .text_color(theme_color(normal.foreground))
            .cursor_pointer()
            .child(label)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, window, cx| {
                    window.focus(&this.focus_handle);
                    cx.emit(event.clone());
                }),
            )
    }
}
