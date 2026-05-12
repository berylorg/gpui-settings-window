use gpui::{
    App, AppContext as _, Context, CursorStyle, Entity, EventEmitter, FocusHandle, Focusable,
    Pixels, Render, SharedString, Window, div, prelude::*, px, rgb, rgba,
};
use gpui_text_input::{
    Enter, MoveDown, MoveUp, TextInput, TextInputEnterKey, TextInputEvent, TextInputOptions,
    TextInputSingleLineVerticalKey, TextInputTheme,
};

use super::bindings::*;
use crate::SettingsInputTheme;
use crate::color_picker::ColorPickerChannelField;

#[derive(Debug, Clone)]
pub(crate) enum ColorComponentInputEvent {
    Changed(String),
    Focused,
    FocusLost,
    Accepted,
    Canceled,
}

pub(crate) struct ColorComponentInput {
    field: ColorPickerChannelField,
    focus_handle: FocusHandle,
    input: Entity<TextInput>,
    value: String,
    font_size: f32,
    visual_theme: SettingsInputTheme,
    text_input_undo_byte_limit: usize,
}

impl EventEmitter<ColorComponentInputEvent> for ColorComponentInput {}

impl ColorComponentInput {
    pub(crate) fn new(
        field: ColorPickerChannelField,
        value: &str,
        font_size: f32,
        visual_theme: SettingsInputTheme,
        text_input_undo_byte_limit: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        ensure_color_component_input_bindings(cx);
        let input = Self::build_text_input(value, &visual_theme, text_input_undo_byte_limit, cx);
        let focus_handle = input.read(cx).tab_focus_handle();
        Self::subscribe_to_text_input(&input, cx);

        let mut input = Self {
            field,
            focus_handle: focus_handle.clone(),
            input,
            value: value.to_owned(),
            font_size,
            visual_theme,
            text_input_undo_byte_limit,
        };
        input.install_focus_listener(window, cx, &focus_handle);
        input
    }

    pub(crate) fn sync_visual_theme(
        &mut self,
        visual_theme: &SettingsInputTheme,
        cx: &mut Context<Self>,
    ) {
        if self.visual_theme == *visual_theme {
            return;
        }

        self.visual_theme = visual_theme.clone();
        let theme = text_input_theme(&self.visual_theme);
        let _ = self.input.update(cx, |input, cx| {
            input.set_theme(theme, cx);
        });
        cx.notify();
    }

    pub(crate) fn sync(&mut self, value: &str, font_size: f32, cx: &mut Context<Self>) {
        let mut changed = false;

        if self.value != value {
            self.set_text_without_event(value, cx);
            changed = true;
        }

        if (self.font_size - font_size).abs() > f32::EPSILON {
            self.font_size = font_size;
            changed = true;
        }

        if changed {
            cx.notify();
        }
    }

    pub(crate) fn sync_text_input_undo_byte_limit(
        &mut self,
        text_input_undo_byte_limit: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.text_input_undo_byte_limit == text_input_undo_byte_limit {
            return;
        }

        self.text_input_undo_byte_limit = text_input_undo_byte_limit;
        self.input = Self::build_text_input(
            &self.value,
            &self.visual_theme,
            self.text_input_undo_byte_limit,
            cx,
        );
        self.focus_handle = self.input.read(cx).tab_focus_handle();
        Self::subscribe_to_text_input(&self.input, cx);
        self.install_focus_listener(window, cx, &self.focus_handle.clone());
        cx.notify();
    }

    pub(crate) fn tab_focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }

    pub(crate) fn focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let _ = self.input.update(cx, |input, cx| {
            input.focus(window, cx);
        });
        cx.notify();
    }

    pub(crate) fn set_text_for_test(&mut self, value: &str, cx: &mut Context<Self>) {
        self.set_text_without_event(value, cx);
        cx.notify();
    }

    pub(crate) fn text_for_test(&self) -> String {
        self.value.clone()
    }

    fn set_text_without_event(&mut self, value: &str, cx: &mut Context<Self>) {
        self.value = value.to_owned();
        let _ = self.input.update(cx, |input, cx| {
            input.set_text(value, cx);
        });
    }

    fn install_focus_listener(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        focus_handle: &FocusHandle,
    ) {
        cx.on_focus(focus_handle, window, |_, _, cx| {
            cx.emit(ColorComponentInputEvent::Focused);
        })
        .detach();

        cx.on_blur(focus_handle, window, |_, _, cx| {
            cx.emit(ColorComponentInputEvent::FocusLost);
        })
        .detach();
    }

    fn line_height(&self) -> Pixels {
        px(self.font_size + 8.0)
    }

    fn field_height(&self) -> Pixels {
        self.line_height() + px(12.0)
    }

    fn handle_text_input_event(
        &mut self,
        input: &Entity<TextInput>,
        event: &TextInputEvent,
        cx: &mut Context<Self>,
    ) {
        if !matches!(event, TextInputEvent::Changed(_)) {
            return;
        }

        self.value = input.read(cx).text().to_owned();
        cx.emit(ColorComponentInputEvent::Changed(self.value.clone()));
        cx.notify();
    }

    fn up_from_text_input(&mut self, _: &MoveUp, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_numeric_step(1, window, cx);
    }

    fn down_from_text_input(&mut self, _: &MoveDown, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_numeric_step(-1, window, cx);
    }

    fn up(&mut self, _: &Up, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_numeric_step(1, window, cx);
    }

    fn down(&mut self, _: &Down, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_numeric_step(-1, window, cx);
    }

    fn submit_from_text_enter(&mut self, _: &Enter, window: &mut Window, cx: &mut Context<Self>) {
        self.focus(window, cx);
        cx.emit(ColorComponentInputEvent::Accepted);
    }

    fn submit(&mut self, _: &Submit, window: &mut Window, cx: &mut Context<Self>) {
        self.focus(window, cx);
        cx.emit(ColorComponentInputEvent::Accepted);
    }

    fn cancel(&mut self, _: &Cancel, window: &mut Window, cx: &mut Context<Self>) {
        self.focus(window, cx);
        cx.emit(ColorComponentInputEvent::Canceled);
    }

    fn apply_numeric_step(&mut self, delta: i16, window: &mut Window, cx: &mut Context<Self>) {
        let current = self.value.trim().parse::<i16>().unwrap_or_default();
        let next = (current + delta).clamp(0, self.field.max_value() as i16);
        self.set_text_without_event(next.to_string().as_str(), cx);
        self.focus(window, cx);
        cx.emit(ColorComponentInputEvent::Changed(self.value.clone()));
        cx.notify();
    }

    fn build_text_input(
        value: &str,
        visual_theme: &SettingsInputTheme,
        text_input_undo_byte_limit: usize,
        cx: &mut Context<Self>,
    ) -> Entity<TextInput> {
        cx.new(|cx| {
            let mut input = TextInput::new_with_options(
                value,
                "",
                TextInputOptions::single_line().with_undo_byte_limit(text_input_undo_byte_limit),
                cx,
            );
            input.set_enter_key(TextInputEnterKey::Propagate);
            input.set_single_line_vertical_key(TextInputSingleLineVerticalKey::Propagate);
            input.set_theme(text_input_theme(visual_theme), cx);
            input
        })
    }

    fn subscribe_to_text_input(input: &Entity<TextInput>, cx: &mut Context<Self>) {
        cx.subscribe(input, |this, input, event: &TextInputEvent, cx| {
            this.handle_text_input_event(&input, event, cx);
        })
        .detach();
    }
}

impl Focusable for ColorComponentInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ColorComponentInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let border = if self.focus_handle.is_focused(window) {
            theme_rgb(self.visual_theme.active_border)
        } else {
            theme_rgb(self.visual_theme.border)
        };

        div()
            .id(SharedString::from(format!(
                "settings-color-picker-component-input-{}",
                self.field.id_suffix()
            )))
            .w_full()
            .h(self.field_height())
            .overflow_hidden()
            .rounded_sm()
            .border_1()
            .border_color(border)
            .bg(theme_rgb(self.visual_theme.background))
            .track_focus(&self.focus_handle)
            .key_context(COLOR_COMPONENT_INPUT_KEY_CONTEXT)
            .cursor(CursorStyle::IBeam)
            .on_action(cx.listener(Self::up_from_text_input))
            .on_action(cx.listener(Self::down_from_text_input))
            .on_action(cx.listener(Self::submit_from_text_enter))
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::submit))
            .on_action(cx.listener(Self::cancel))
            .text_size(px(self.font_size))
            .line_height(self.line_height())
            .text_color(theme_rgb(self.visual_theme.foreground))
            .child(
                div()
                    .w_full()
                    .h_full()
                    .px_2()
                    .py_1()
                    .overflow_hidden()
                    .child(self.input.clone()),
            )
    }
}

fn text_input_theme(theme: &SettingsInputTheme) -> TextInputTheme {
    TextInputTheme {
        text: Some(theme_rgb(theme.foreground).into()),
        placeholder: theme_rgb(theme.foreground).into(),
        selection: theme_selection(theme.selection_background).into(),
        caret: theme_rgb(theme.caret).into(),
        marked_underline: theme_rgb(theme.caret).into(),
        ..TextInputTheme::default()
    }
}

fn theme_rgb(color: crate::RgbColor) -> gpui::Rgba {
    rgb(color.packed_rgb())
}

fn theme_selection(color: crate::RgbColor) -> gpui::Rgba {
    rgba((color.packed_rgb() << 8) | 0x44)
}
