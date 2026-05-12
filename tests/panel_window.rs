use std::cell::RefCell;
use std::rc::Rc;

use gpui_settings_window::{
    RgbColor, SettingsFieldId, SettingsFieldKind, SettingsPanel, SettingsRow, SettingsRowAction,
    SettingsRowActionId, SettingsSection, SettingsSectionId, SettingsWindowEvent,
    SettingsWindowModel, SettingsWindowOpenDisposition, SettingsWindowOptions, SettingsWindowTheme,
    open_settings_window,
};

fn settings_model(selected_section: &str, font_value: &str) -> SettingsWindowModel {
    SettingsWindowModel::with_selected_section(
        vec![
            SettingsSection::new("appearance", "Appearance").with_row(SettingsRow::new(
                "font_family",
                "Font family",
                font_value,
                SettingsFieldKind::Text,
            )),
            SettingsSection::new("editor", "Editor").with_row(SettingsRow::new(
                "tab_size",
                "Tab size",
                "4",
                SettingsFieldKind::Text,
            )),
        ],
        selected_section,
    )
    .expect("valid settings model")
}

fn settings_model_with_row_action() -> SettingsWindowModel {
    SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_row(
            SettingsRow::new(
                "font_family",
                "Font family",
                "Inter",
                SettingsFieldKind::Text,
            )
            .with_action(SettingsRowAction::new("choose", "Choose...")),
        ),
    ])
    .expect("valid settings model")
}

fn settings_model_with_multiline(value: &str) -> SettingsWindowModel {
    SettingsWindowModel::new(vec![SettingsSection::new("agent", "Agent").with_row(
        SettingsRow::new(
            "instructions",
            "Instructions",
            value,
            SettingsFieldKind::MultilineText,
        ),
    )])
    .expect("valid settings model")
}

#[gpui::test]
fn hidden_window_can_be_shown_hidden_and_reused(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            settings_model("appearance", "Inter"),
            SettingsWindowOptions::default(),
            SettingsWindowOpenDisposition::Hidden,
        )
        .expect("settings window should open")
    });

    let window_id = handle.window_handle().window_id().as_u64();

    assert!(!handle.is_visible(cx).expect("window should exist"));
    handle
        .show(cx, settings_model("appearance", "Inter"), true)
        .expect("show should succeed");
    assert!(handle.is_visible(cx).expect("window should exist"));
    handle.hide(cx).expect("hide should succeed");
    assert!(!handle.is_visible(cx).expect("window should exist"));
    assert_eq!(handle.window_handle().window_id().as_u64(), window_id);
}

#[gpui::test]
fn sync_model_updates_selected_section_and_field_text(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            settings_model("appearance", "Inter"),
            SettingsWindowOptions::default(),
            SettingsWindowOpenDisposition::Visible {
                focus_requested: false,
            },
        )
        .expect("settings window should open")
    });

    handle
        .update_model(cx, settings_model("editor", "JetBrains Mono"))
        .expect("sync should succeed");

    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            assert_eq!(view.model().selected_section_id().as_str(), "editor");
            assert_eq!(
                view.field_text_for_test(&SettingsFieldId::from("font_family"), cx),
                Some(String::from("JetBrains Mono")),
            );
        })
        .expect("settings window should be readable");
}

#[gpui::test]
fn emits_section_text_button_and_close_events(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            settings_model("appearance", "Inter"),
            SettingsWindowOptions::default(),
            SettingsWindowOpenDisposition::Visible {
                focus_requested: false,
            },
        )
        .expect("settings window should open")
    });
    let view = handle.entity(cx).expect("root entity should exist");
    let events = Rc::new(RefCell::new(Vec::new()));
    let captured_events = events.clone();

    cx.update(|cx| {
        cx.subscribe(&view, move |_, event: &SettingsWindowEvent, _| {
            captured_events.borrow_mut().push(event.clone());
        })
        .detach();
    });

    handle
        .window_handle()
        .update(cx, |view, window, cx| {
            view.select_section_for_test(SettingsSectionId::from("editor"), window, cx);
            view.replace_field_text_for_test(&SettingsFieldId::from("font_family"), "Fira", cx);
            view.accept_for_test(cx);
            view.apply_for_test(cx);
            view.cancel_for_test(cx);
            assert!(!view.request_close_for_test(window, cx));
        })
        .expect("settings window should update");

    let events = events.borrow();
    assert!(events.contains(&SettingsWindowEvent::SectionSelected {
        section_id: SettingsSectionId::from("editor"),
    }));
    assert!(events.contains(&SettingsWindowEvent::FieldChanged {
        field_id: SettingsFieldId::from("font_family"),
        value: String::from("Fira"),
    }));
    assert!(events.contains(&SettingsWindowEvent::AcceptRequested));
    assert!(events.contains(&SettingsWindowEvent::ApplyRequested));
    assert!(events.contains(&SettingsWindowEvent::CancelRequested));
    assert!(events.contains(&SettingsWindowEvent::CloseRequested));
    assert!(!handle.is_visible(cx).expect("window should still exist"));
}

#[gpui::test]
fn emits_row_action_events(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            settings_model_with_row_action(),
            SettingsWindowOptions::default(),
            SettingsWindowOpenDisposition::Visible {
                focus_requested: false,
            },
        )
        .expect("settings window should open")
    });
    let view = handle.entity(cx).expect("root entity should exist");
    let events = Rc::new(RefCell::new(Vec::new()));
    let captured_events = events.clone();

    cx.update(|cx| {
        cx.subscribe(&view, move |_, event: &SettingsWindowEvent, _| {
            captured_events.borrow_mut().push(event.clone());
        })
        .detach();
    });

    handle
        .window_handle()
        .update(cx, |view, _, cx| {
            assert!(view.request_row_action_for_test(
                SettingsFieldId::from("font_family"),
                SettingsRowActionId::from("choose"),
                cx,
            ));
            assert!(!view.request_row_action_for_test(
                SettingsFieldId::from("font_family"),
                SettingsRowActionId::from("missing"),
                cx,
            ));
        })
        .expect("settings window should update");

    assert!(
        events
            .borrow()
            .contains(&SettingsWindowEvent::RowActionRequested {
                field_id: SettingsFieldId::from("font_family"),
                action_id: SettingsRowActionId::from("choose"),
            })
    );
}

#[gpui::test]
fn multiline_fields_sync_and_emit_plain_text(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            settings_model_with_multiline("Line one"),
            SettingsWindowOptions::default(),
            SettingsWindowOpenDisposition::Visible {
                focus_requested: false,
            },
        )
        .expect("settings window should open")
    });
    let view = handle.entity(cx).expect("root entity should exist");
    let events = Rc::new(RefCell::new(Vec::new()));
    let captured_events = events.clone();

    cx.update(|cx| {
        cx.subscribe(&view, move |_, event: &SettingsWindowEvent, _| {
            captured_events.borrow_mut().push(event.clone());
        })
        .detach();
    });

    handle
        .window_handle()
        .update(cx, |view, _, cx| {
            assert_eq!(
                view.field_text_for_test(&SettingsFieldId::from("instructions"), cx),
                Some(String::from("Line one")),
            );
            view.replace_field_text_for_test(
                &SettingsFieldId::from("instructions"),
                "Line one\nLine two",
                cx,
            );
        })
        .expect("settings window should update");

    assert!(
        events
            .borrow()
            .contains(&SettingsWindowEvent::FieldChanged {
                field_id: SettingsFieldId::from("instructions"),
                value: String::from("Line one\nLine two"),
            })
    );
}

#[gpui::test]
fn multiline_field_enter_edits_text_instead_of_accepting(cx: &mut gpui::TestAppContext) {
    let (panel, cx) = cx.add_window_view(|window, cx| {
        let mut panel = SettingsPanel::new(settings_model_with_multiline("Line one"), window, cx);
        panel.focus_field(&SettingsFieldId::from("instructions"), window, cx);
        panel
    });
    let events = Rc::new(RefCell::new(Vec::new()));
    let captured_events = events.clone();

    cx.cx.update(|cx| {
        cx.subscribe(&panel, move |_, event: &SettingsWindowEvent, _| {
            captured_events.borrow_mut().push(event.clone());
        })
        .detach();
    });

    cx.simulate_keystrokes("enter");
    cx.simulate_input("Line two");

    panel.read_with(cx, |panel, cx| {
        assert_eq!(
            panel.field_text_for_test(&SettingsFieldId::from("instructions"), cx),
            Some(String::from("Line one\nLine two")),
        );
    });

    assert!(
        events
            .borrow()
            .contains(&SettingsWindowEvent::FieldChanged {
                field_id: SettingsFieldId::from("instructions"),
                value: String::from("Line one\nLine two"),
            })
    );
    assert!(
        !events
            .borrow()
            .contains(&SettingsWindowEvent::AcceptRequested)
    );
}

#[gpui::test]
fn settings_text_input_uses_configured_undo_byte_limit(cx: &mut gpui::TestAppContext) {
    let (panel, cx) = cx.add_window_view(|window, cx| {
        let mut panel = SettingsPanel::new_with_options(
            settings_model_with_multiline(""),
            SettingsWindowOptions::default().with_text_input_undo_byte_limit(3),
            window,
            cx,
        );
        panel.focus_field(&SettingsFieldId::from("instructions"), window, cx);
        panel
    });

    cx.simulate_input("aa");
    cx.simulate_input("bb");
    cx.simulate_input("cc");

    panel.read_with(cx, |panel, cx| {
        let counts = panel
            .field_retained_counts_for_test(&SettingsFieldId::from("instructions"), cx)
            .expect("field should exist");
        assert!(counts.undo_text_bytes <= 3);
    });
}

#[gpui::test]
fn sync_options_preserves_unsynchronized_field_text(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            settings_model("appearance", "Inter"),
            SettingsWindowOptions::default(),
            SettingsWindowOpenDisposition::Visible {
                focus_requested: false,
            },
        )
        .expect("settings window should open")
    });

    handle
        .window_handle()
        .update(cx, |view, window, cx| {
            view.focus_field(&SettingsFieldId::from("font_family"), window, cx);
            view.replace_field_text_for_test(
                &SettingsFieldId::from("font_family"),
                "Draft Mono",
                cx,
            );
        })
        .expect("settings window should be writable");

    let mut theme = SettingsWindowTheme::default();
    theme.input.border = RgbColor::new(1, 2, 3);
    handle
        .update_options(
            cx,
            SettingsWindowOptions::default()
                .with_visual_theme(theme)
                .with_saved_color_swatches([RgbColor::new(9, 8, 7)]),
        )
        .expect("options update should succeed");

    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            assert_eq!(
                view.field_text_for_test(&SettingsFieldId::from("font_family"), cx),
                Some(String::from("Draft Mono")),
            );
        })
        .expect("settings window should be readable");

    handle
        .update_options(
            cx,
            SettingsWindowOptions::default().with_text_input_undo_byte_limit(3),
        )
        .expect("options update should succeed");

    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            assert_eq!(
                view.field_text_for_test(&SettingsFieldId::from("font_family"), cx),
                Some(String::from("Draft Mono")),
            );
        })
        .expect("settings window should be readable");
}
