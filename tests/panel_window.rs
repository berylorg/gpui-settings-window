use std::cell::RefCell;
use std::rc::Rc;

use gpui::AppContext as _;
use gpui_settings_window::{
    MAX_PAGE_DETAIL_ROWS, RgbColor, SettingsBreadcrumbSegment, SettingsChoiceOption,
    SettingsFieldId, SettingsFieldKind, SettingsPage, SettingsPageAction, SettingsPageActionId,
    SettingsPageActionPriority, SettingsPageId, SettingsPageSplit, SettingsPageSplitItem,
    SettingsPageSplitItemId, SettingsPanel, SettingsRow, SettingsRowAction, SettingsRowActionId,
    SettingsRowDetailField, SettingsSection, SettingsSectionId, SettingsWindowEvent,
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

fn settings_model_with_choice() -> SettingsWindowModel {
    SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_row(
            SettingsRow::new("source", "Source", "value", SettingsFieldKind::Choice)
                .with_choice(SettingsChoiceOption::new("value", "Value"))
                .with_choice(SettingsChoiceOption::new("static_parent", "Static parent"))
                .with_choice(SettingsChoiceOption::new(
                    "ambient_parent",
                    "Ambient parent",
                ))
                .with_choice(SettingsChoiceOption::new("fallback", "Fallback")),
        ),
    ])
    .expect("valid settings model")
}

fn settings_model_with_detail_field() -> SettingsWindowModel {
    SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_row(
            SettingsRow::new("source", "Background", "value", SettingsFieldKind::Choice)
                .with_choice(SettingsChoiceOption::new("value", "Value"))
                .with_choice(SettingsChoiceOption::new("fallback", "Fallback"))
                .with_detail_field(SettingsRowDetailField::new(
                    "background",
                    "#112233",
                    SettingsFieldKind::Color,
                )),
        ),
    ])
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

fn settings_model_with_pages() -> SettingsWindowModel {
    SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance")
            .with_row(
                SettingsRow::navigation("theme_editor_link", "Theme editor", "theme_editor")
                    .with_action(SettingsRowAction::new("save", "Save"))
                    .with_action(
                        SettingsRowAction::new("discard", "Discard")
                            .disabled_with_reason("No staged changes"),
                    ),
            )
            .with_page(
                SettingsPage::new("theme_editor", "Theme editor")
                    .with_breadcrumb_segment(SettingsBreadcrumbSegment::linked(
                        "Appearance",
                        "appearance",
                    ))
                    .with_back_target("appearance")
                    .with_action(
                        SettingsPageAction::new("save", "Save")
                            .with_priority(SettingsPageActionPriority::Primary),
                    )
                    .with_action(
                        SettingsPageAction::new("discard", "Discard")
                            .disabled_with_reason("No staged changes"),
                    )
                    .with_row(
                        SettingsRow::new(
                            "font_family",
                            "Font family",
                            "Inter",
                            SettingsFieldKind::Text,
                        )
                        .with_action(SettingsRowAction::new("reset", "Reset").disabled()),
                    ),
            ),
    ])
    .expect("valid settings model")
}

fn settings_model_with_same_section_text_pages(selected_page: &str) -> SettingsWindowModel {
    let mut root_page = SettingsPage::new("appearance", "Appearance").with_row(SettingsRow::new(
        "root_font",
        "Root font",
        "Inter",
        SettingsFieldKind::Text,
    ));
    let mut theme_page = SettingsPage::new("theme_editor", "Theme editor")
        .with_breadcrumb_segment(SettingsBreadcrumbSegment::linked(
            "Appearance",
            "appearance",
        ))
        .with_back_target("appearance")
        .with_row(SettingsRow::new(
            "theme_font",
            "Theme font",
            "Fira Code",
            SettingsFieldKind::Text,
        ));
    for index in 0..24 {
        root_page = root_page.with_row(SettingsRow::new(
            format!("root.extra.{index:02}"),
            format!("Root extra {index:02}"),
            index.to_string(),
            SettingsFieldKind::Text,
        ));
        theme_page = theme_page.with_row(SettingsRow::new(
            format!("theme.extra.{index:02}"),
            format!("Theme extra {index:02}"),
            index.to_string(),
            SettingsFieldKind::Text,
        ));
    }

    let mut model = SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance")
            .with_root_page(root_page)
            .with_page(theme_page),
    ])
    .expect("valid settings model");
    model
        .select_page(selected_page)
        .expect("selected page should exist");
    model
}

fn settings_model_with_local_split() -> SettingsWindowModel {
    SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_root_page(
            SettingsPage::new("appearance", "Appearance")
                .with_local_split(
                    SettingsPageSplit::new()
                        .with_item(
                            SettingsPageSplitItem::new("default", "Default").with_selected(true),
                        )
                        .with_item(SettingsPageSplitItem::new("large", "Large")),
                )
                .with_row(SettingsRow::new(
                    "font_size",
                    "Font size",
                    "private-font-size-value",
                    SettingsFieldKind::Number,
                )),
        ),
    ])
    .expect("valid settings model")
}

fn settings_model_with_long_local_split(
    item_count: usize,
    selected_index: usize,
) -> SettingsWindowModel {
    let mut split = SettingsPageSplit::new();
    for index in 0..item_count {
        split = split.with_item(
            SettingsPageSplitItem::new(format!("role.{index:03}"), format!("Role {index:03}"))
                .with_subtext("static parent: app.window")
                .with_selected(index == selected_index),
        );
    }

    SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_root_page(
            SettingsPage::new("appearance", "Appearance")
                .with_local_split(split)
                .with_row(SettingsRow::new(
                    "font_size",
                    "Font size",
                    "14",
                    SettingsFieldKind::Number,
                )),
        ),
    ])
    .expect("valid settings model")
}

fn settings_model_with_local_split_order(
    item_indices: &[usize],
    selected_item_index: usize,
) -> SettingsWindowModel {
    let mut split = SettingsPageSplit::new();
    for index in item_indices {
        split = split.with_item(
            SettingsPageSplitItem::new(format!("role.{index:03}"), format!("Role {index:03}"))
                .with_subtext("static parent: app.window")
                .with_selected(*index == selected_item_index),
        );
    }

    SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_root_page(
            SettingsPage::new("appearance", "Appearance")
                .with_local_split(split)
                .with_row(SettingsRow::new(
                    "font_size",
                    "Font size",
                    "14",
                    SettingsFieldKind::Number,
                )),
        ),
    ])
    .expect("valid settings model")
}

fn settings_model_with_detail_rows(row_count: usize) -> SettingsWindowModel {
    let mut page = SettingsPage::new("appearance", "Appearance");
    for index in 0..row_count {
        page = page.with_row(SettingsRow::new(
            format!("field.{index:03}"),
            format!("Field {index:03}"),
            index.to_string(),
            SettingsFieldKind::Text,
        ));
    }

    SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_root_page(page),
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
fn scrollbar_activity_is_scoped_to_settings_scroll_region(cx: &mut gpui::TestAppContext) {
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
            assert_eq!(view.scrollbar_active_states_for_test(cx), (false, false));

            view.record_navigation_scrollbar_activity_for_test(window, cx);
            let (navigation_active, content_active) = view.scrollbar_active_states_for_test(cx);
            assert!(navigation_active);
            assert!(!content_active);

            view.reset_scrollbar_visibility_for_test(cx);
            assert_eq!(view.scrollbar_active_states_for_test(cx), (false, false));

            view.record_content_scrollbar_activity_for_test(window, cx);
            let (navigation_active, content_active) = view.scrollbar_active_states_for_test(cx);
            assert!(!navigation_active);
            assert!(content_active);
        })
        .expect("settings window should update");
}

#[gpui::test]
fn preheated_window_show_and_hide_reset_scrollbar_visibility(cx: &mut gpui::TestAppContext) {
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
            view.record_navigation_scrollbar_activity_for_test(window, cx);
            view.record_content_scrollbar_activity_for_test(window, cx);
            assert_eq!(view.scrollbar_active_states_for_test(cx), (true, true));
        })
        .expect("settings window should update");

    handle.hide(cx).expect("hide should succeed");
    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            assert_eq!(view.scrollbar_active_states_for_test(cx), (false, false));
        })
        .expect("settings window should be readable");

    handle
        .show(cx, settings_model("appearance", "Inter"), false)
        .expect("show should succeed");
    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            assert_eq!(view.scrollbar_active_states_for_test(cx), (false, false));
        })
        .expect("settings window should be readable");
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
fn same_section_page_change_resets_detail_scroll_and_focus(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            settings_model_with_same_section_text_pages("appearance"),
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
            view.set_content_scroll_offset_for_test(96.0, cx);
            assert!(view.focus_field(&SettingsFieldId::from("root_font"), window, cx));
            assert_eq!(
                view.focused_field_for_test(window, cx),
                Some(SettingsFieldId::from("root_font")),
            );
            assert_eq!(view.settings_scroll_metrics(cx).0, -96.0);
        })
        .expect("settings window should update");

    handle
        .update_model(
            cx,
            settings_model_with_same_section_text_pages("theme_editor"),
        )
        .expect("sync should succeed");

    handle
        .window_handle()
        .update(cx, |view, window, cx| {
            assert_eq!(view.model().selected_section_id().as_str(), "appearance");
            assert_eq!(view.model().selected_page_id().as_str(), "theme_editor");
            assert_eq!(view.settings_scroll_metrics(cx).0, 0.0);
            assert_eq!(
                view.focused_field_for_test(window, cx),
                Some(SettingsFieldId::from("theme_font")),
            );
        })
        .expect("settings window should update");
}

#[gpui::test]
fn same_page_model_refresh_preserves_detail_scroll_and_focus(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            settings_model_with_same_section_text_pages("appearance"),
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
            view.set_content_scroll_offset_for_test(64.0, cx);
            assert!(view.focus_field(&SettingsFieldId::from("root_font"), window, cx));
        })
        .expect("settings window should update");

    handle
        .update_model(
            cx,
            settings_model_with_same_section_text_pages("appearance"),
        )
        .expect("sync should succeed");

    handle
        .window_handle()
        .update(cx, |view, window, cx| {
            assert_eq!(view.settings_scroll_metrics(cx).0, -64.0);
            assert_eq!(
                view.focused_field_for_test(window, cx),
                Some(SettingsFieldId::from("root_font")),
            );
        })
        .expect("settings window should update");
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
    assert!(
        !events.contains(&SettingsWindowEvent::PageNavigationRequested {
            page_id: SettingsPageId::from("editor"),
        }),
        "section selection should not also emit a redundant root-page navigation event"
    );
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
fn emits_choice_field_change_events(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            settings_model_with_choice(),
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
            assert!(view.select_choice_for_test(
                SettingsFieldId::from("source"),
                "ambient_parent".to_string(),
                cx,
            ));
            assert!(!view.select_choice_for_test(
                SettingsFieldId::from("source"),
                "missing".to_string(),
                cx,
            ));
        })
        .expect("settings window should update");

    let events = events.borrow();
    assert!(events.contains(&SettingsWindowEvent::FieldChanged {
        field_id: SettingsFieldId::from("source"),
        value: "ambient_parent".to_string(),
    }));
    assert!(!events.contains(&SettingsWindowEvent::FieldChanged {
        field_id: SettingsFieldId::from("source"),
        value: "missing".to_string(),
    }));
}

#[gpui::test]
fn secondary_detail_fields_sync_and_emit_field_changes(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            settings_model_with_detail_field(),
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
                view.field_text_for_test(&SettingsFieldId::from("background"), cx),
                Some("#112233".to_string())
            );
            assert!(view.replace_field_text_for_test(
                &SettingsFieldId::from("background"),
                "#334455",
                cx,
            ));
        })
        .expect("settings window should update");

    assert!(
        events
            .borrow()
            .contains(&SettingsWindowEvent::FieldChanged {
                field_id: SettingsFieldId::from("background"),
                value: "#334455".to_string(),
            })
    );
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
fn emits_page_navigation_and_page_action_events(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            settings_model_with_pages(),
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
            view.select_section_for_test(SettingsSectionId::from("appearance"), window, cx);
            assert!(
                view.request_page_navigation_for_test(SettingsPageId::from("theme_editor"), cx,)
            );
            assert!(view.request_page_navigation_for_test(SettingsPageId::from("appearance"), cx,));
            assert!(view.request_row_action_for_test(
                SettingsFieldId::from("theme_editor_link"),
                SettingsRowActionId::from("save"),
                cx,
            ));
            assert!(!view.request_row_action_for_test(
                SettingsFieldId::from("theme_editor_link"),
                SettingsRowActionId::from("discard"),
                cx,
            ));
            assert!(view.request_page_action_for_test(
                SettingsPageId::from("theme_editor"),
                SettingsPageActionId::from("save"),
                cx,
            ));
            assert!(!view.request_page_action_for_test(
                SettingsPageId::from("theme_editor"),
                SettingsPageActionId::from("discard"),
                cx,
            ));
        })
        .expect("settings window should update");

    let events = events.borrow();
    assert!(
        events.contains(&SettingsWindowEvent::PageNavigationRequested {
            page_id: SettingsPageId::from("appearance"),
        })
    );
    assert!(
        events.contains(&SettingsWindowEvent::PageNavigationRequested {
            page_id: SettingsPageId::from("theme_editor"),
        })
    );
    assert!(events.contains(&SettingsWindowEvent::PageActionRequested {
        page_id: SettingsPageId::from("theme_editor"),
        action_id: SettingsPageActionId::from("save"),
    }));
    assert!(events.contains(&SettingsWindowEvent::RowActionRequested {
        field_id: SettingsFieldId::from("theme_editor_link"),
        action_id: SettingsRowActionId::from("save"),
    }));
    assert!(!events.contains(&SettingsWindowEvent::RowActionRequested {
        field_id: SettingsFieldId::from("theme_editor_link"),
        action_id: SettingsRowActionId::from("discard"),
    }));
    assert!(!events.contains(&SettingsWindowEvent::PageActionRequested {
        page_id: SettingsPageId::from("theme_editor"),
        action_id: SettingsPageActionId::from("discard"),
    }));
}

#[gpui::test]
fn emits_page_local_split_item_selection_events(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            settings_model_with_local_split(),
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
            assert!(view.request_page_split_item_for_test(
                SettingsPageId::from("appearance"),
                SettingsPageSplitItemId::from("large"),
                cx,
            ));
            assert!(!view.request_page_split_item_for_test(
                SettingsPageId::from("appearance"),
                SettingsPageSplitItemId::from("missing"),
                cx,
            ));
        })
        .expect("settings window should update");

    let events = events.borrow();
    assert!(
        events.contains(&SettingsWindowEvent::PageSplitItemSelected {
            page_id: SettingsPageId::from("appearance"),
            item_id: SettingsPageSplitItemId::from("large"),
        })
    );
    assert!(
        !events.contains(&SettingsWindowEvent::PageSplitItemSelected {
            page_id: SettingsPageId::from("appearance"),
            item_id: SettingsPageSplitItemId::from("missing"),
        })
    );
}

#[gpui::test]
fn long_page_local_split_lists_report_bounded_render_window(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            settings_model_with_long_local_split(176, 0),
            SettingsWindowOptions::default(),
            SettingsWindowOpenDisposition::Visible {
                focus_requested: false,
            },
        )
        .expect("settings window should open")
    });

    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            let (total, start, end, total_height) = view
                .page_split_render_metrics_for_test(cx)
                .expect("selected page should carry a split list");
            assert_eq!(total, 176);
            assert_eq!(start, 0);
            assert!(
                end - start <= 12,
                "split list should render a bounded visible window, not all items"
            );
            assert!(end < total);
            assert_eq!(total_height, 176.0 * 88.0 + 175.0 * 4.0);
        })
        .expect("settings window should be readable");
}

#[gpui::test]
fn settings_window_diagnostics_report_bounded_content_free_surfaces(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            settings_model_with_long_local_split(176, 0),
            SettingsWindowOptions::default(),
            SettingsWindowOpenDisposition::Visible {
                focus_requested: false,
            },
        )
        .expect("settings window should open")
    });

    let diagnostics = handle
        .diagnostics_snapshot(cx)
        .expect("settings diagnostics should be readable");
    let split = diagnostics
        .split_list
        .as_ref()
        .expect("selected page should carry a split list");

    assert!(diagnostics.visible);
    assert_eq!(diagnostics.selected_page_id, "appearance");
    assert_eq!(diagnostics.detail_rows.total_row_count, 1);
    assert_eq!(diagnostics.detail_rows.rendered_row_count, 1);
    assert_eq!(diagnostics.detail_rows.visible_range, None);
    assert_eq!(split.total_row_count, 176);
    assert_eq!(split.visible_range.unwrap().start, 0);
    assert!(
        split.rendered_row_count <= 12,
        "diagnostics should report bounded split-list rendering"
    );
    assert!(split.rendered_row_count < split.total_row_count);
    assert_eq!(split.row_height_strategy, "fixed_height_windowed");

    let debug = format!("{diagnostics:?}");
    assert!(!debug.contains("Role 000"));
    assert!(!debug.contains("static parent: app.window"));
    assert!(!debug.contains("private-font-size-value"));
}

#[gpui::test]
fn settings_window_diagnostics_do_not_expose_labels_values_or_paths(cx: &mut gpui::TestAppContext) {
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

    handle
        .update_model(
            cx,
            SettingsWindowModel::new(vec![
                SettingsSection::new("notifications", "Notifications").with_row(
                    SettingsRow::new(
                        "end_turn_sound",
                        "End-turn sound",
                        "C:\\Users\\operator\\Music\\Notifications\\very-long-completion-sound.wav",
                        SettingsFieldKind::Text,
                    )
                    .with_subtext("Local notification file")
                    .with_action(SettingsRowAction::new("choose", "Choose..."))
                    .with_action(SettingsRowAction::new("clear", "Clear")),
                ),
            ])
            .expect("valid model"),
        )
        .expect("sync should succeed");

    let diagnostics = handle
        .diagnostics_snapshot(cx)
        .expect("settings diagnostics should be readable");
    let debug = format!("{diagnostics:?}");

    assert_eq!(diagnostics.selected_section_id, "notifications");
    assert_eq!(diagnostics.detail_rows.total_row_count, 1);
    assert!(!debug.contains("End-turn sound"));
    assert!(!debug.contains("very-long-completion-sound.wav"));
    assert!(!debug.contains("Local notification file"));
    assert!(!debug.contains("Choose"));
    assert!(!debug.contains("Clear"));
}

#[gpui::test]
fn selected_detail_row_diagnostics_document_bounded_full_render_strategy(
    cx: &mut gpui::TestAppContext,
) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            settings_model_with_detail_rows(MAX_PAGE_DETAIL_ROWS),
            SettingsWindowOptions::default(),
            SettingsWindowOpenDisposition::Visible {
                focus_requested: false,
            },
        )
        .expect("settings window should open")
    });

    let diagnostics = handle
        .diagnostics_snapshot(cx)
        .expect("settings diagnostics should be readable");

    assert_eq!(
        diagnostics.detail_rows.total_row_count,
        MAX_PAGE_DETAIL_ROWS
    );
    assert_eq!(
        diagnostics.detail_rows.rendered_row_count,
        MAX_PAGE_DETAIL_ROWS
    );
    assert_eq!(diagnostics.detail_rows.visible_range, None);
    assert_eq!(diagnostics.detail_rows.overscan_count, 0);
    assert_eq!(
        diagnostics.detail_rows.row_height_strategy,
        "full_selected_page"
    );
}

#[gpui::test]
fn offscreen_selected_page_local_split_item_is_revealed(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            settings_model_with_long_local_split(176, 150),
            SettingsWindowOptions::default(),
            SettingsWindowOpenDisposition::Visible {
                focus_requested: false,
            },
        )
        .expect("settings window should open")
    });

    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            let (scroll_top, _) = view.page_split_scroll_metrics_for_test(cx);
            assert!(
                scroll_top > 0.0,
                "initially offscreen selected split item should request a revealed offset"
            );
            let (total, start, end, _) = view
                .page_split_render_metrics_for_test(cx)
                .expect("selected page should carry a split list");
            assert_eq!(total, 176);
            assert!(
                start <= 150 && 150 < end,
                "rendered split range {start}..{end} should include selected item"
            );
            assert!(end - start <= 12);
        })
        .expect("settings window should be readable");
}

#[gpui::test]
fn moved_selected_page_local_split_item_is_revealed_after_refresh(cx: &mut gpui::TestAppContext) {
    let mut initial_order = vec![150];
    initial_order.extend((0..176).filter(|index| *index != 150));
    let refreshed_order: Vec<_> = (0..176).collect();
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            settings_model_with_local_split_order(&initial_order, 150),
            SettingsWindowOptions::default(),
            SettingsWindowOpenDisposition::Visible {
                focus_requested: false,
            },
        )
        .expect("settings window should open")
    });

    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            let (total, start, end, _) = view
                .page_split_render_metrics_for_test(cx)
                .expect("selected page should carry a split list");
            assert_eq!(total, 176);
            assert!(
                start == 0 && 0 < end,
                "initial rendered split range {start}..{end} should include selected item"
            );
        })
        .expect("settings window should be readable");

    handle
        .update_model(
            cx,
            settings_model_with_local_split_order(&refreshed_order, 150),
        )
        .expect("sync should succeed");

    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            let (scroll_top, _) = view.page_split_scroll_metrics_for_test(cx);
            assert!(
                scroll_top > 0.0,
                "moved offscreen selected split item should be revealed"
            );
            let (total, start, end, _) = view
                .page_split_render_metrics_for_test(cx)
                .expect("selected page should carry a split list");
            assert_eq!(total, 176);
            assert!(
                start <= 150 && 150 < end,
                "rendered split range {start}..{end} should include selected item after refresh"
            );
            assert!(end - start <= 12);
        })
        .expect("settings window should be readable");
}

#[gpui::test]
fn page_local_split_refresh_clamps_scroll_after_list_shrink(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            settings_model_with_long_local_split(176, 0),
            SettingsWindowOptions::default(),
            SettingsWindowOpenDisposition::Visible {
                focus_requested: false,
            },
        )
        .expect("settings window should open")
    });

    handle
        .window_handle()
        .update(cx, |view, _, cx| {
            view.set_page_split_scroll_offset_for_test(12_000.0, cx);
        })
        .expect("settings window should update");

    handle
        .update_model(cx, settings_model_with_long_local_split(4, 0))
        .expect("sync should succeed");

    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            let (scroll_top, _) = view.page_split_scroll_metrics_for_test(cx);
            assert!(
                scroll_top < 100.0,
                "split scroll should be clamped after the list shrinks, got {scroll_top}"
            );

            let (total, start, end, _) = view
                .page_split_render_metrics_for_test(cx)
                .expect("selected page should carry a split list");
            assert_eq!(total, 4);
            assert!(
                start <= end && end <= total,
                "rendered split range {start}..{end} must stay inside total {total}"
            );
            assert!(end > start, "non-empty split list should not render blank");
        })
        .expect("settings window should be readable");

    let diagnostics = handle
        .diagnostics_snapshot(cx)
        .expect("settings diagnostics should be readable");
    let split = diagnostics
        .split_list
        .expect("selected page should carry a split list");
    let range = split
        .visible_range
        .expect("windowed split list should report a visible range");

    assert_eq!(split.total_row_count, 4);
    assert!(
        range.start <= range.end && range.end <= split.total_row_count,
        "diagnostic range {}..{} must stay inside total {}",
        range.start,
        range.end,
        split.total_row_count
    );
    assert_eq!(split.rendered_row_count, range.end - range.start);
    assert!(split.rendered_row_count > 0);
}

#[gpui::test]
fn page_local_split_scroll_is_independent_from_detail_scroll(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            settings_model_with_long_local_split(176, 0),
            SettingsWindowOptions::default(),
            SettingsWindowOpenDisposition::Visible {
                focus_requested: false,
            },
        )
        .expect("settings window should open")
    });

    handle
        .window_handle()
        .update(cx, |view, _, cx| {
            view.set_page_split_scroll_offset_for_test(500.0, cx);
            view.set_content_scroll_offset_for_test(80.0, cx);
            assert_eq!(view.page_split_scroll_metrics_for_test(cx).0, 500.0);
            assert_eq!(view.settings_scroll_metrics(cx).0, -80.0);
        })
        .expect("settings window should update");

    handle
        .update_model(cx, settings_model_with_long_local_split(176, 0))
        .expect("sync should succeed");

    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            assert_eq!(view.page_split_scroll_metrics_for_test(cx).0, 500.0);
        })
        .expect("settings window should be readable");
}

#[gpui::test]
fn row_actions_on_navigation_rows_do_not_navigate(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            settings_model_with_pages(),
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
                SettingsFieldId::from("theme_editor_link"),
                SettingsRowActionId::from("save"),
                cx,
            ));
        })
        .expect("settings window should update");

    let events = events.borrow();
    assert!(events.contains(&SettingsWindowEvent::RowActionRequested {
        field_id: SettingsFieldId::from("theme_editor_link"),
        action_id: SettingsRowActionId::from("save"),
    }));
    assert!(
        !events.contains(&SettingsWindowEvent::PageNavigationRequested {
            page_id: SettingsPageId::from("theme_editor"),
        })
    );
}

#[gpui::test]
fn disabled_row_actions_do_not_emit_events(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            settings_model_with_pages(),
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
            assert!(!view.request_row_action_for_test(
                SettingsFieldId::from("font_family"),
                SettingsRowActionId::from("reset"),
                cx,
            ));
        })
        .expect("settings window should update");

    assert!(
        !events
            .borrow()
            .contains(&SettingsWindowEvent::RowActionRequested {
                field_id: SettingsFieldId::from("font_family"),
                action_id: SettingsRowActionId::from("reset"),
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

#[gpui::test]
fn settings_window_diagnostics_distinguish_sync_and_lookup_counters(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            settings_model_with_detail_field(),
            SettingsWindowOptions::default(),
            SettingsWindowOpenDisposition::Visible {
                focus_requested: false,
            },
        )
        .expect("settings window should open")
    });

    handle
        .update_model(cx, settings_model_with_detail_field())
        .expect("model sync should succeed");
    handle
        .update_options(
            cx,
            SettingsWindowOptions::default().with_saved_color_swatches([RgbColor::new(1, 2, 3)]),
        )
        .expect("option sync should succeed");
    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            assert_eq!(
                view.color_preview_for_test(&SettingsFieldId::from("background"), cx),
                Some("#112233".to_string()),
            );
        })
        .expect("settings window should be readable");
    cx.update_window(handle.window_handle().into(), |_, window, cx| {
        window.draw(cx).clear();
    })
    .expect("settings window should draw");

    let diagnostics = handle
        .diagnostics_snapshot(cx)
        .expect("settings diagnostics should be readable");
    let performance = diagnostics.performance;

    assert_eq!(performance.model_sync_count, 1);
    assert_eq!(performance.option_sync_count, 1);
    assert!(performance.input_sync_count >= 1);
    assert!(performance.last_input_sync_entity_count >= 1);
    assert!(performance.color_preview_lookup_count >= 1);
    assert!(performance.color_model_lookup_count >= 1);
    assert_eq!(performance.last_render_color_preview_lookup_count, 1);
    assert_eq!(performance.last_render_color_model_lookup_count, 0);
}
