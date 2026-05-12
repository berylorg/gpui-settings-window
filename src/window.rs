use gpui::{
    App, AppContext as _, Bounds, Context, Entity, EventEmitter, IntoElement, ParentElement,
    Render, Result, Window, WindowBounds, WindowHandle, WindowOptions, div, prelude::*, px, size,
};

#[cfg(target_os = "windows")]
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{SW_HIDE, SW_SHOWNA, ShowWindowAsync};

use crate::{SettingsPanel, SettingsWindowEvent, SettingsWindowModel, SettingsWindowOptions};

mod test_support;

/// Initial visibility for a newly opened settings window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsWindowOpenDisposition {
    /// Create the OS window without showing it yet.
    Hidden,
    /// Create and show the OS window.
    Visible {
        /// Whether the window should take focus immediately.
        focus_requested: bool,
    },
}

/// Handle to a preheated settings window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettingsWindowHandle {
    handle: WindowHandle<SettingsWindowView>,
}

impl SettingsWindowHandle {
    /// Returns the underlying GPUI window handle.
    pub fn window_handle(&self) -> WindowHandle<SettingsWindowView> {
        self.handle
    }

    /// Returns the root settings-window entity.
    pub fn entity<C>(&self, cx: &C) -> Result<Entity<SettingsWindowView>>
    where
        C: gpui::AppContext,
    {
        self.handle.entity(cx)
    }

    /// Shows this preheated settings window with the supplied presentation model.
    pub fn show<C>(
        &self,
        cx: &mut C,
        model: SettingsWindowModel,
        focus_requested: bool,
    ) -> Result<()>
    where
        C: gpui::AppContext,
    {
        self.handle
            .update(cx, |view, window, cx| {
                view.show(model, focus_requested, window, cx);
            })
            .map(|_| ())
    }

    /// Hides the OS window without removing it.
    pub fn hide<C>(&self, cx: &mut C) -> Result<()>
    where
        C: gpui::AppContext,
    {
        self.handle
            .update(cx, |view, window, cx| {
                view.hide(window, cx);
            })
            .map(|_| ())
    }

    /// Updates the presentation model without changing visibility.
    pub fn update_model<C>(&self, cx: &mut C, model: SettingsWindowModel) -> Result<()>
    where
        C: gpui::AppContext,
    {
        self.handle
            .update(cx, |view, window, cx| {
                view.sync_model(model, window, cx);
            })
            .map(|_| ())
    }

    /// Updates visual options without changing visibility.
    pub fn update_options<C>(&self, cx: &mut C, options: SettingsWindowOptions) -> Result<()>
    where
        C: gpui::AppContext,
    {
        self.handle
            .update(cx, |view, window, cx| {
                view.sync_options(options, window, cx);
            })
            .map(|_| ())
    }

    /// Focuses the first setting field in the selected section.
    pub fn focus_primary_control<C>(&self, cx: &mut C) -> Result<()>
    where
        C: gpui::AppContext,
    {
        self.handle
            .update(cx, |view, window, cx| {
                view.focus_primary_control(window, cx);
            })
            .map(|_| ())
    }

    /// Returns whether this wrapper currently considers the window visible.
    pub fn is_visible<C>(&self, cx: &C) -> Result<bool>
    where
        C: gpui::AppContext,
    {
        self.handle.read_with(cx, |view, _| view.is_visible())
    }
}

/// Opens a dedicated settings window, optionally hidden for preheating.
pub fn open_settings_window(
    cx: &mut App,
    model: SettingsWindowModel,
    options: SettingsWindowOptions,
    disposition: SettingsWindowOpenDisposition,
) -> Result<SettingsWindowHandle> {
    let (show, focus_requested, visible) = match disposition {
        SettingsWindowOpenDisposition::Hidden => (false, false, false),
        SettingsWindowOpenDisposition::Visible { focus_requested } => (true, focus_requested, true),
    };

    let (width, height) = options.window_size();
    let (min_width, min_height) = options.min_window_size();
    let title = options.title().to_owned();
    let bounds = Bounds::centered(None, size(px(width), px(height)), cx);
    let panel_options = options.clone();
    let window = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            focus: false,
            show,
            window_min_size: Some(size(px(min_width), px(min_height))),
            ..Default::default()
        },
        move |window, cx| {
            window.set_window_title(&title);
            let model = model.clone();
            let options = panel_options.clone();
            cx.new(|cx| SettingsWindowView::new(model, visible, options, window, cx))
        },
    )?;

    if show && focus_requested {
        let _ = window.update(cx, |view, window, cx| {
            view.defer_activate_and_focus_primary_control_if_visible(window, cx);
        });
    }

    Ok(SettingsWindowHandle { handle: window })
}

/// Root GPUI view for a settings window.
pub struct SettingsWindowView {
    model: SettingsWindowModel,
    options: SettingsWindowOptions,
    settings_panel: Entity<SettingsPanel>,
    visible: bool,
}

impl EventEmitter<SettingsWindowEvent> for SettingsWindowView {}

impl SettingsWindowView {
    pub(crate) fn new(
        model: SettingsWindowModel,
        visible: bool,
        options: SettingsWindowOptions,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let settings_panel = cx
            .new(|cx| SettingsPanel::new_with_options(model.clone(), options.clone(), window, cx));
        cx.subscribe(&settings_panel, |_, _, event: &SettingsWindowEvent, cx| {
            cx.emit(event.clone())
        })
        .detach();

        let view = cx.entity().downgrade();
        window.on_window_should_close(cx, move |window, cx| {
            view.update(cx, |view, cx| {
                view.handle_window_close_requested(window, cx)
            })
            .unwrap_or(true)
        });

        Self {
            model,
            options,
            settings_panel,
            visible,
        }
    }

    /// Returns whether the reusable settings window is currently shown.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Returns the current presentation model.
    pub fn model(&self) -> &SettingsWindowModel {
        &self.model
    }

    /// Returns the current visual/window options.
    pub fn options(&self) -> &SettingsWindowOptions {
        &self.options
    }

    /// Synchronizes the window content to a new presentation model.
    pub fn sync_model(
        &mut self,
        model: SettingsWindowModel,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.model = model.clone();
        let _ = self
            .settings_panel
            .update(cx, |panel, cx| panel.sync_model(model, window, cx));
        cx.notify();
    }

    /// Synchronizes visual options without replacing the presentation model.
    pub fn sync_options(
        &mut self,
        options: SettingsWindowOptions,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.set_window_title(options.title());
        self.options = options;
        let panel_options = self.options.clone();
        let _ = self.settings_panel.update(cx, |panel, cx| {
            panel.sync_options(&panel_options, window, cx)
        });
        cx.notify();
    }

    /// Shows the window and synchronizes its model.
    pub fn show(
        &mut self,
        model: SettingsWindowModel,
        focus_requested: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.sync_model(model, window, cx);
        self.visible = true;
        show_native_settings_window(window);
        if focus_requested {
            self.defer_activate_and_focus_primary_control_if_visible(window, cx);
        }
    }

    /// Hides the OS window but keeps the GPUI window and panel entities alive.
    pub fn hide(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.visible = false;
        hide_native_settings_window(window);
        cx.notify();
    }

    /// Focuses the first setting field in the selected section.
    pub fn focus_primary_control(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let _ = self
            .settings_panel
            .update(cx, |panel, cx| panel.focus_first_field(window, cx));
    }

    fn defer_activate_and_focus_primary_control_if_visible(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        cx.defer_in(window, |view, window, cx| {
            if view.visible {
                window.activate_window();
                view.focus_primary_control(window, cx);
            }
        });
    }

    /// Focuses a field by id.
    pub fn focus_field(
        &mut self,
        field_id: &crate::SettingsFieldId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        self.settings_panel
            .update(cx, |panel, cx| panel.focus_field(field_id, window, cx))
    }

    /// Returns scroll metrics for the settings content.
    pub fn settings_scroll_metrics(&self, cx: &App) -> (f32, f32) {
        self.settings_panel.read(cx).scroll_metrics()
    }

    fn handle_window_close_requested(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        self.hide(window, cx);
        cx.emit(SettingsWindowEvent::CloseRequested);
        false
    }
}

impl Render for SettingsWindowView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("settings-window")
            .size_full()
            .child(self.settings_panel.clone())
    }
}

#[cfg(target_os = "windows")]
fn native_window_handle(window: &Window) -> Option<HWND> {
    let handle = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        HasWindowHandle::window_handle(window)
    }))
    .ok()?
    .ok()?;
    match handle.as_raw() {
        RawWindowHandle::Win32(handle) => Some(HWND(handle.hwnd.get() as _)),
        _ => None,
    }
}

fn show_native_settings_window(window: &mut Window) {
    #[cfg(target_os = "windows")]
    {
        if let Some(handle) = native_window_handle(window) {
            unsafe {
                let _ = ShowWindowAsync(handle, SW_SHOWNA);
            }
        }
    }
    window.refresh();
}

fn hide_native_settings_window(window: &mut Window) {
    #[cfg(target_os = "windows")]
    {
        if let Some(handle) = native_window_handle(window) {
            unsafe {
                let _ = ShowWindowAsync(handle, SW_HIDE);
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        window.minimize_window();
    }
}
