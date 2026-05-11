use gpui::{App, Global, actions};

pub(super) const COLOR_COMPONENT_INPUT_KEY_CONTEXT: &str = "GpuiSettingsWindowColorComponentInput";

actions!(
    gpui_settings_window_color_component_input,
    [Up, Down, Submit, Cancel]
);

struct ColorComponentInputBindingsInstalled;

impl Global for ColorComponentInputBindingsInstalled {}

pub(crate) fn ensure_color_component_input_bindings(cx: &mut App) {
    gpui_text_input::ensure_text_input_bindings(cx);

    if cx.has_global::<ColorComponentInputBindingsInstalled>() {
        return;
    }

    cx.bind_keys([
        gpui::KeyBinding::new("up", Up, Some(COLOR_COMPONENT_INPUT_KEY_CONTEXT)),
        gpui::KeyBinding::new("down", Down, Some(COLOR_COMPONENT_INPUT_KEY_CONTEXT)),
        gpui::KeyBinding::new("enter", Submit, Some(COLOR_COMPONENT_INPUT_KEY_CONTEXT)),
        gpui::KeyBinding::new("escape", Cancel, Some(COLOR_COMPONENT_INPUT_KEY_CONTEXT)),
    ]);
    cx.set_global(ColorComponentInputBindingsInstalled);
}
