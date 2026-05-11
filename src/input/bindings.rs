use gpui::{App, Global, actions};

pub(super) const INPUT_KEY_CONTEXT: &str = "GpuiSettingsWindowInput";

actions!(
    gpui_settings_window_input,
    [OpenColorPicker, Submit, Cancel]
);

struct SettingsInputBindingsInstalled;

impl Global for SettingsInputBindingsInstalled {}

pub(crate) fn ensure_settings_input_bindings(cx: &mut App) {
    gpui_text_input::ensure_text_input_bindings(cx);

    if cx.has_global::<SettingsInputBindingsInstalled>() {
        return;
    }

    cx.bind_keys([
        gpui::KeyBinding::new("ctrl-space", OpenColorPicker, Some(INPUT_KEY_CONTEXT)),
        gpui::KeyBinding::new("enter", Submit, Some(INPUT_KEY_CONTEXT)),
        gpui::KeyBinding::new("escape", Cancel, Some(INPUT_KEY_CONTEXT)),
    ]);
    cx.set_global(SettingsInputBindingsInstalled);
}
