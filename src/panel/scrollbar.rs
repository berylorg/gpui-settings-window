use super::*;
use gpui_scrollbar::{
    ScrollbarVisibilityKey, ScrollbarVisibilityPolicy, ScrollbarVisibilityUpdateCallback,
};

impl SettingsPanel {
    fn scrollbar_key(owner_id: u64, generation: u64) -> ScrollbarOwnerKey {
        ScrollbarOwnerKey::new(
            ScrollbarOwnerId::new(owner_id),
            ScrollbarMountGeneration::new(generation),
        )
    }

    pub(super) fn new_scrollbar(owner_id: u64, generation: u64) -> ScrollbarState {
        ScrollbarState::new(Self::scrollbar_key(owner_id, generation))
    }

    fn scrollbar_update_callback(
        entity: gpui::WeakEntity<Self>,
    ) -> ScrollbarVisibilityUpdateCallback {
        Rc::new(
            move |_: ScrollbarVisibilityKey, _: &mut Window, cx: &mut App| {
                let _ = entity.update(cx, |_, cx| cx.notify());
            },
        )
    }

    pub(super) fn content_owner(&self) -> ScrollbarOwnerKey {
        self.content_scrollbar
            .current_owner()
            .expect("mounted content scrollbar")
    }
    pub(super) fn navigation_owner(&self) -> ScrollbarOwnerKey {
        self.navigation_scrollbar
            .current_owner()
            .expect("mounted navigation scrollbar")
    }
    pub(super) fn split_owner(&self) -> ScrollbarOwnerKey {
        self.split_scrollbar
            .current_owner()
            .expect("mounted split scrollbar")
    }

    pub(in crate::panel) fn content_scrollbar_visibility_policy(
        &self,
        entity: gpui::WeakEntity<Self>,
    ) -> ScrollbarVisibilityPolicy {
        self.content_scrollbar
            .managed(Self::scrollbar_update_callback(entity))
    }
    pub(in crate::panel) fn navigation_scrollbar_visibility_policy(
        &self,
        entity: gpui::WeakEntity<Self>,
    ) -> ScrollbarVisibilityPolicy {
        self.navigation_scrollbar
            .managed(Self::scrollbar_update_callback(entity))
    }
    pub(in crate::panel) fn split_scrollbar_visibility_policy(
        &self,
        entity: gpui::WeakEntity<Self>,
    ) -> ScrollbarVisibilityPolicy {
        self.split_scrollbar
            .managed(Self::scrollbar_update_callback(entity))
    }

    pub(crate) fn unmount_scrollbars(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for (state, owner) in [
            (
                &self.content_scrollbar,
                self.content_scrollbar.current_owner(),
            ),
            (
                &self.navigation_scrollbar,
                self.navigation_scrollbar.current_owner(),
            ),
            (&self.split_scrollbar, self.split_scrollbar.current_owner()),
        ] {
            if let Some(owner) = owner {
                let _ = state.unmount_viewport(owner, window, cx);
            }
        }
        self.scrollbars_mounted = false;
    }

    pub(crate) fn teardown_scrollbars(&mut self, window: &mut Window, cx: &mut App) {
        for (state, owner) in [
            (
                &self.content_scrollbar,
                self.content_scrollbar.current_owner(),
            ),
            (
                &self.navigation_scrollbar,
                self.navigation_scrollbar.current_owner(),
            ),
            (&self.split_scrollbar, self.split_scrollbar.current_owner()),
        ] {
            if let Some(owner) = owner {
                let _ = state.teardown_window(owner, window, cx);
            }
        }
    }

    pub(crate) fn mount_scrollbars(&mut self, cx: &mut Context<Self>) {
        self.content_scrollbar_generation += 1;
        self.navigation_scrollbar_generation += 1;
        assert!(
            self.content_scrollbar.mount(Self::scrollbar_key(
                CONTENT_SCROLLBAR_OWNER_ID,
                self.content_scrollbar_generation,
            )),
            "content scrollbar must be unmounted before remount"
        );
        assert!(
            self.navigation_scrollbar.mount(Self::scrollbar_key(
                NAVIGATION_SCROLLBAR_OWNER_ID,
                self.navigation_scrollbar_generation,
            )),
            "navigation scrollbar must be unmounted before remount"
        );
        self.scrollbars_mounted = true;
        if self.model.selected_page().paged_split_source().is_some() {
            self.mount_split_scrollbar(cx);
        }
        cx.notify();
    }

    pub(super) fn unmount_split_scrollbar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(owner) = self.split_scrollbar.current_owner() {
            let _ = self.split_scrollbar.unmount_viewport(owner, window, cx);
        }
    }

    pub(super) fn mount_split_scrollbar(&mut self, cx: &mut Context<Self>) {
        if self.split_scrollbar.current_owner().is_some() {
            return;
        }
        self.split_scrollbar_generation += 1;
        assert!(
            self.split_scrollbar.mount(Self::scrollbar_key(
                SPLIT_SCROLLBAR_OWNER_ID,
                self.split_scrollbar_generation,
            )),
            "split scrollbar must remount under a newer generation"
        );
        cx.notify();
    }

    pub(in crate::panel) fn note_content_scrollbar_activity(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.content_scrollbar_visibility_policy(cx.weak_entity())
            .record_viewport_activity(self.content_owner(), window, cx);
    }
    pub(in crate::panel) fn note_navigation_scrollbar_activity(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.navigation_scrollbar_visibility_policy(cx.weak_entity())
            .record_viewport_activity(self.navigation_owner(), window, cx);
    }
    pub(in crate::panel) fn note_split_scrollbar_activity(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.split_scrollbar_visibility_policy(cx.weak_entity())
            .record_viewport_activity(self.split_owner(), window, cx);
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
    pub(in crate::panel) fn note_split_scrollbar_motion(
        &mut self,
        _: &gpui::MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.note_split_scrollbar_activity(window, cx);
    }
    pub(in crate::panel) fn note_split_scrollbar_scroll(
        &mut self,
        _: &gpui::ScrollWheelEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.note_split_scrollbar_activity(window, cx);
    }
}
