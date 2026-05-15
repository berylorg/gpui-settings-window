use super::*;
use gpui_scrollbar::{ScrollbarVisibilityPolicy, ScrollbarVisibilityUpdateCallback};

impl SettingsPanel {
    fn scrollbar_update_callback(entity: Entity<Self>) -> ScrollbarVisibilityUpdateCallback {
        Rc::new(move |_: &mut Window, cx: &mut App| {
            entity.update(cx, |_, cx| {
                cx.notify();
            });
        })
    }

    pub(in crate::panel) fn content_scrollbar_visibility_policy(
        &self,
        entity: Entity<Self>,
    ) -> ScrollbarVisibilityPolicy {
        self.content_scrollbar_visibility
            .managed(Self::scrollbar_update_callback(entity))
    }

    pub(in crate::panel) fn navigation_scrollbar_visibility_policy(
        &self,
        entity: Entity<Self>,
    ) -> ScrollbarVisibilityPolicy {
        self.navigation_scrollbar_visibility
            .managed(Self::scrollbar_update_callback(entity))
    }

    pub(in crate::panel) fn note_content_scrollbar_activity(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let on_update = Self::scrollbar_update_callback(cx.entity());
        self.content_scrollbar_visibility
            .record_viewport_activity(window, cx, on_update);
        cx.notify();
    }

    pub(in crate::panel) fn note_navigation_scrollbar_activity(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let on_update = Self::scrollbar_update_callback(cx.entity());
        self.navigation_scrollbar_visibility
            .record_viewport_activity(window, cx, on_update);
        cx.notify();
    }

    pub(in crate::panel) fn note_content_scrollbar_motion(
        &mut self,
        _: &gpui::MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.note_content_scrollbar_activity(window, cx);
    }

    pub(in crate::panel) fn note_content_scrollbar_scroll(
        &mut self,
        _: &gpui::ScrollWheelEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.note_content_scrollbar_activity(window, cx);
    }

    pub(in crate::panel) fn note_navigation_scrollbar_motion(
        &mut self,
        _: &gpui::MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.note_navigation_scrollbar_activity(window, cx);
    }

    pub(in crate::panel) fn note_navigation_scrollbar_scroll(
        &mut self,
        _: &gpui::ScrollWheelEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.note_navigation_scrollbar_activity(window, cx);
    }
}
