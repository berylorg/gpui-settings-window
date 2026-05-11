use gpui::{
    App, CursorStyle, Focusable, ParentElement, Render, SharedString, Styled, div, prelude::*, px,
};

use super::*;
use crate::model::element_id_suffix;

impl Focusable for SettingsFieldInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SettingsFieldInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let border = self.border_color(window);

        div()
            .id(SharedString::from(format!(
                "settings-input-{}",
                element_id_suffix(self.field_id.as_str())
            )))
            .w_full()
            .h(self.field_height())
            .overflow_hidden()
            .rounded_sm()
            .border_1()
            .border_color(border)
            .bg(theme_rgb(self.visual_theme.background))
            .track_focus(&self.focus_handle)
            .key_context(INPUT_KEY_CONTEXT)
            .cursor(CursorStyle::IBeam)
            .on_action(cx.listener(Self::open_color_picker))
            .on_action(cx.listener(Self::submit_from_text_enter))
            .on_action(cx.listener(Self::submit))
            .on_action(cx.listener(Self::cancel))
            .text_size(px(self.font_size))
            .line_height(self.line_height())
            .text_color(theme_rgb(self.visual_theme.foreground))
            .child(
                div()
                    .w_full()
                    .h_full()
                    .px_3()
                    .py_1()
                    .overflow_hidden()
                    .child(self.input.clone()),
            )
    }
}

impl SettingsFieldInput {
    fn border_color(&self, window: &Window) -> gpui::Rgba {
        if self.error.is_some() || self.has_invalid_color_draft() {
            theme_rgb(self.visual_theme.error_border)
        } else if self.focus_handle.is_focused(window) {
            theme_rgb(self.visual_theme.active_border)
        } else {
            theme_rgb(self.visual_theme.border)
        }
    }

    fn has_invalid_color_draft(&self) -> bool {
        self.kind == SettingsFieldKind::Color
            && crate::RgbColor::parse(self.value.as_str()).is_none()
    }
}
