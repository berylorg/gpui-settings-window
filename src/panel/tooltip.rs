use super::*;

pub(super) struct SettingsCommandTooltip {
    pub(super) source: gpui::WeakEntity<SettingsPanel>,
    pub(super) event: SettingsWindowEvent,
    pub(super) _observer: Option<gpui::Subscription>,
}

impl Render for SettingsCommandTooltip {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(source) = self.source.upgrade() else {
            return div().into_any_element();
        };
        let panel = source.read(cx);
        let Some(reason) = panel.command_disabled_reason(&self.event) else {
            return div().into_any_element();
        };
        div()
            .id("settings-disabled-command-tooltip")
            .when_some(panel.font_family.clone(), |element, family| {
                element.font_family(family)
            })
            .max_w(px(360.0))
            .px_2()
            .py_1()
            .rounded_sm()
            .border_1()
            .border_color(theme_color(panel.visual_theme.popup.border))
            .bg(theme_color(panel.visual_theme.popup.background))
            .text_color(theme_color(panel.visual_theme.popup.foreground))
            .text_size(px(panel.font_size))
            .child(reason)
            .into_any_element()
    }
}
