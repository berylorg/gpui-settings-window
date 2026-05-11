use gpui::{
    AppContext as _, Context, Entity, EventEmitter, FocusHandle, Pixels, Window, px, rgb, rgba,
};
use gpui_text_input::{
    TextInput, TextInputEnterKey, TextInputEvent, TextInputOptions, TextInputTheme,
};

use crate::SettingsInputTheme;
use crate::model::{SettingsFieldId, SettingsFieldKind, SettingsWindowEvent};

mod bindings;
mod render;

pub(crate) use bindings::ensure_settings_input_bindings;
use bindings::*;

#[derive(Debug, Clone)]
pub(crate) enum SettingsFieldInputEvent {
    Window(SettingsWindowEvent),
    OpenColorPickerRequested(SettingsFieldId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SettingsFieldInputRole {
    Row,
    Picker,
}

pub(crate) struct SettingsFieldInput {
    field_id: SettingsFieldId,
    kind: SettingsFieldKind,
    role: SettingsFieldInputRole,
    focus_handle: FocusHandle,
    input: Entity<TextInput>,
    value: String,
    error: Option<String>,
    font_size: f32,
    visual_theme: SettingsInputTheme,
}

impl EventEmitter<SettingsFieldInputEvent> for SettingsFieldInput {}

impl SettingsFieldInput {
    pub(crate) fn new(
        field_id: SettingsFieldId,
        value: &str,
        kind: SettingsFieldKind,
        error: Option<&str>,
        font_size: f32,
        role: SettingsFieldInputRole,
        visual_theme: SettingsInputTheme,
        cx: &mut Context<Self>,
    ) -> Self {
        let input = Self::build_text_input(value, kind, &visual_theme, cx);
        let focus_handle = input.read(cx).tab_focus_handle();
        Self::subscribe_to_text_input(&input, cx);

        Self {
            field_id,
            kind,
            role,
            focus_handle,
            input,
            value: value.to_owned(),
            error: error.map(String::from),
            font_size,
            visual_theme,
        }
    }

    pub(crate) fn sync(
        &mut self,
        value: &str,
        kind: SettingsFieldKind,
        error: Option<&str>,
        font_size: f32,
        cx: &mut Context<Self>,
    ) {
        let mut changed = false;

        if self.kind != kind {
            self.kind = kind;
            self.value = value.to_owned();
            self.input = Self::build_text_input(value, kind, &self.visual_theme, cx);
            self.focus_handle = self.input.read(cx).tab_focus_handle();
            Self::subscribe_to_text_input(&self.input, cx);
            changed = true;
        } else if self.value != value {
            self.value = value.to_owned();
            let _ = self.input.update(cx, |input, cx| {
                input.set_text(value, cx);
            });
            changed = true;
        }

        let next_error = error.map(String::from);
        if self.error != next_error {
            self.error = next_error;
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

    pub(crate) fn retarget(
        &mut self,
        field_id: SettingsFieldId,
        value: &str,
        kind: SettingsFieldKind,
        error: Option<&str>,
        font_size: f32,
        cx: &mut Context<Self>,
    ) {
        let field_changed = self.field_id != field_id;
        self.field_id = field_id;
        self.sync(value, kind, error, font_size, cx);
        if field_changed {
            self.set_text_without_event(value, cx);
            cx.notify();
        }
    }

    pub(crate) fn focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let _ = self.input.update(cx, |input, cx| {
            input.focus(window, cx);
        });
        cx.notify();
    }

    pub(crate) fn tab_focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }

    pub(crate) fn text(&self) -> &str {
        &self.value
    }

    pub(crate) fn replace_all_for_test(&mut self, value: &str, cx: &mut Context<Self>) {
        self.set_text_without_event(value, cx);
        cx.emit(SettingsFieldInputEvent::Window(
            SettingsWindowEvent::FieldChanged {
                field_id: self.field_id.clone(),
                value: value.to_owned(),
            },
        ));
        cx.notify();
    }

    pub(crate) fn set_text_for_test(&mut self, value: &str, cx: &mut Context<Self>) {
        self.set_text_without_event(value, cx);
        cx.notify();
    }

    fn set_text_without_event(&mut self, value: &str, cx: &mut Context<Self>) {
        self.value = value.to_owned();
        let _ = self.input.update(cx, |input, cx| {
            input.set_text(value, cx);
        });
    }

    fn line_height(&self) -> Pixels {
        px(self.font_size + 8.0)
    }

    fn field_height(&self) -> Pixels {
        if self.kind == SettingsFieldKind::MultilineText {
            px(f32::from(self.line_height()) * 8.0 + 12.0)
        } else {
            self.line_height() + px(12.0)
        }
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
        cx.emit(SettingsFieldInputEvent::Window(
            SettingsWindowEvent::FieldChanged {
                field_id: self.field_id.clone(),
                value: self.value.clone(),
            },
        ));
        cx.notify();
    }

    fn open_color_picker(
        &mut self,
        _: &OpenColorPicker,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.kind != SettingsFieldKind::Color {
            return;
        }
        if self.role != SettingsFieldInputRole::Row {
            return;
        }

        self.focus(window, cx);
        cx.emit(SettingsFieldInputEvent::OpenColorPickerRequested(
            self.field_id.clone(),
        ));
    }

    fn submit_from_text_enter(
        &mut self,
        _: &gpui_text_input::Enter,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus(window, cx);
        cx.emit(SettingsFieldInputEvent::Window(
            SettingsWindowEvent::AcceptRequested,
        ));
    }

    fn submit(&mut self, _: &Submit, window: &mut Window, cx: &mut Context<Self>) {
        self.focus(window, cx);
        cx.emit(SettingsFieldInputEvent::Window(
            SettingsWindowEvent::AcceptRequested,
        ));
    }

    fn cancel(&mut self, _: &Cancel, window: &mut Window, cx: &mut Context<Self>) {
        self.focus(window, cx);
        cx.emit(SettingsFieldInputEvent::Window(
            SettingsWindowEvent::CancelRequested,
        ));
    }

    fn build_text_input(
        value: &str,
        kind: SettingsFieldKind,
        visual_theme: &SettingsInputTheme,
        cx: &mut Context<Self>,
    ) -> Entity<TextInput> {
        cx.new(|cx| {
            let mut input = TextInput::new_with_options(value, "", text_input_options(kind), cx);
            input.set_enter_key(text_input_enter_key(kind));
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

fn text_input_options(kind: SettingsFieldKind) -> TextInputOptions {
    match kind {
        SettingsFieldKind::MultilineText => TextInputOptions::multiline(),
        SettingsFieldKind::Text | SettingsFieldKind::Color => TextInputOptions::single_line(),
    }
}

fn text_input_enter_key(kind: SettingsFieldKind) -> TextInputEnterKey {
    match kind {
        SettingsFieldKind::MultilineText => TextInputEnterKey::InsertNewline,
        SettingsFieldKind::Text | SettingsFieldKind::Color => TextInputEnterKey::Propagate,
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
