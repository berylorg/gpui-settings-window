use std::{cell::RefCell, ops::Range, rc::Rc};

use gpui::{
    AppContext as _, Entity, EntityInputHandler, IntoElement as _, Modifiers, Pixels, Point,
};
use gpui_settings_window::*;
use gpui_text_input::{TextInput, TextInputRetainedCounts};

fn primary(value: &str) -> SettingsRow {
    SettingsRow::new("primary", "Primary", value, SettingsFieldKind::Text)
}

fn detail(value: &str) -> SettingsRowDetailField {
    SettingsRowDetailField::new("detail", value, SettingsFieldKind::MultilineText)
}

fn form(primary: SettingsRow) -> SettingsWindowModel {
    let root = SettingsPage::new("main", "Main")
        .with_action(SettingsPageAction::new("save", "Save"))
        .with_action(
            SettingsPageAction::new("discard", "Discard")
                .disabled_with_reason("No document changes"),
        )
        .with_row(primary.with_action(
            SettingsRowAction::new("reset", "Reset").disabled_with_reason("No staged changes"),
        ))
        .with_row(
            SettingsRow::new("choice", "Choice", "a", SettingsFieldKind::Choice)
                .with_choice(SettingsChoiceOption::new("a", "A"))
                .with_choice(SettingsChoiceOption::new("b", "B")),
        )
        .with_row(SettingsRow::new(
            "color",
            "Color",
            "#112233",
            SettingsFieldKind::Color,
        ))
        .with_row(SettingsRow::navigation("next", "More", "subpage"));
    SettingsWindowModel::new(vec![
        SettingsSection::new("main", "Main")
            .with_root_page(root)
            .with_page(
                SettingsPage::new("subpage", "More")
                    .with_back_target("main")
                    .with_breadcrumb_segment(SettingsBreadcrumbSegment::linked("Main", "main")),
            ),
        SettingsSection::new("other", "Other"),
    ])
    .unwrap()
}

fn observe(
    panel: &Entity<SettingsPanel>,
    cx: &mut gpui::VisualTestContext,
) -> Rc<RefCell<Vec<SettingsWindowEvent>>> {
    let events = Rc::new(RefCell::new(Vec::new()));
    let captured = events.clone();
    cx.cx.update(|app| {
        app.subscribe(panel, move |_, event: &SettingsWindowEvent, _| {
            captured.borrow_mut().push(event.clone())
        })
        .detach()
    });
    events
}

#[derive(Debug, PartialEq)]
struct EditorSnapshot {
    text: String,
    selection: Range<usize>,
    reversed: bool,
    marked: Option<Range<usize>>,
    scroll: Point<Pixels>,
    retained: TextInputRetainedCounts,
}

fn editor_snapshot(input: &TextInput) -> EditorSnapshot {
    EditorSnapshot {
        text: input.text().to_owned(),
        selection: input.selection(),
        reversed: input.state().selection_reversed(),
        marked: input.state().marked_range(),
        scroll: input.scroll_offset(),
        retained: input.retained_counts(),
    }
}

#[gpui::test]
fn independent_field_availability_and_validity_update_the_same_mounted_editors(
    cx: &mut gpui::TestAppContext,
) {
    let initial = form(
        primary("abc")
            .with_availability(SettingsFieldAvailability::unavailable(
                "Primary unavailable",
            ))
            .with_detail_field(detail("def").with_validity(SettingsFieldValidity::Invalid)),
    );
    let (panel, cx) = cx.add_window_view(|window, cx| {
        let mut panel =
            SettingsPanel::new(initial, SettingsPageSplitWorkReceiver::new(), window, cx);
        panel.focus_field(&"primary".into(), window, cx);
        panel
    });
    let events = observe(&panel, cx);
    let first = panel.read_with(cx, |panel, app| {
        panel
            .text_input_for_field_for_test(&"primary".into(), app)
            .unwrap()
    });
    let second = panel.read_with(cx, |panel, app| {
        panel
            .text_input_for_field_for_test(&"detail".into(), app)
            .unwrap()
    });
    cx.simulate_input("x");
    assert_eq!(
        first.read_with(cx, |input, _| input.text().to_owned()),
        "abc"
    );
    assert!(events.borrow().is_empty());
    cx.update(|window, app| {
        panel.update(app, |panel, cx| {
            panel.focus_field(&"detail".into(), window, cx);
        })
    });
    cx.simulate_input("x");
    assert_eq!(
        second.read_with(cx, |input, _| input.text().to_owned()),
        "defx"
    );
    second.update(cx, |input, cx| {
        input.set_selection(1..3, true, cx);
    });
    let before = second.read_with(cx, |input, _| editor_snapshot(input));
    let next =
        form(
            primary("abc")
                .with_validity(SettingsFieldValidity::Invalid)
                .with_detail_field(detail("defx").with_availability(
                    SettingsFieldAvailability::unavailable("Detail unavailable"),
                )),
        );
    cx.update(|window, app| panel.update(app, |panel, cx| panel.sync_model(next, window, cx)));
    assert_eq!(
        panel.read_with(cx, |panel, app| panel
            .text_input_for_field_for_test(&"primary".into(), app)
            .unwrap()
            .entity_id()),
        first.entity_id()
    );
    assert_eq!(
        panel.read_with(cx, |panel, app| panel
            .text_input_for_field_for_test(&"detail".into(), app)
            .unwrap()
            .entity_id()),
        second.entity_id()
    );
    assert!(!second.update(cx, |input, cx| input.replace_selected_text("rejected", cx)));
    assert_eq!(
        second.read_with(cx, |input, _| editor_snapshot(input)),
        before
    );
    assert!(second.read_with(cx, |input, _| input.selection_export().is_some()));
    cx.update(|window, app| {
        panel.update(app, |panel, cx| {
            panel.focus_field(&"primary".into(), window, cx);
        })
    });
    cx.simulate_input("q");
    assert_eq!(
        first.read_with(cx, |input, _| input.text().to_owned()),
        "abcq"
    );
}

#[gpui::test]
fn gated_native_window_rejects_repeated_commands_hide_and_close_until_explicit_clear(
    cx: &mut gpui::TestAppContext,
) {
    let base = form(primary("abc").with_detail_field(detail("def")));
    let handle = cx.update(|app| {
        open_settings_window(
            app,
            base.clone(),
            SettingsWindowOptions::default(),
            SettingsWindowOpenDisposition::Visible {
                focus_requested: false,
            },
        )
        .unwrap()
    });
    let view = handle.entity(cx).unwrap();
    let events = Rc::new(RefCell::new(Vec::new()));
    let captured = events.clone();
    cx.update(|app| {
        app.subscribe(&view, move |_, event: &SettingsWindowEvent, _| {
            captured.borrow_mut().push(event.clone())
        })
        .detach()
    });
    let input = view.read_with(cx, |view, app| {
        view.text_input_for_field_for_test(&"primary".into(), app)
            .unwrap()
    });
    input.update(cx, |input, cx| {
        input.set_selection(0..2, true, cx);
    });
    let before = input.read_with(cx, |input, _| editor_snapshot(input));
    handle
        .update_model(
            cx,
            base.clone()
                .with_interaction_gate(SettingsInteractionGate::gated(
                    "Reconciling original operation",
                )),
        )
        .unwrap();
    for _ in 0..3 {
        handle
            .window_handle()
            .update(cx, |view, window, cx| {
                view.accept_for_test(cx);
                view.apply_for_test(cx);
                view.cancel_for_test(cx);
                assert!(!view.request_row_action_for_test("primary".into(), "reset".into(), cx));
                assert!(!view.request_page_action_for_test("main".into(), "save".into(), cx));
                assert!(!view.request_page_navigation_for_test("subpage".into(), cx));
                assert!(!view.select_choice_for_test("choice".into(), "b".into(), cx));
                view.select_section_for_test("other".into(), window, cx);
                view.open_color_picker_for_test("color".into(), window, cx);
                assert!(!view.request_close_for_test(window, cx));
            })
            .unwrap();
        assert!(!handle.hide(cx).unwrap());
    }
    assert!(events.borrow().is_empty());
    assert!(handle.is_visible(cx).unwrap());
    assert!(!handle.has_transient_popups(cx).unwrap());
    assert!(!input.update(cx, |input, cx| {
        input.replace_selected_text("unavailable", cx)
    }));
    assert_eq!(
        input.read_with(cx, |input, _| editor_snapshot(input)),
        before
    );
    assert!(input.read_with(cx, |input, _| input.selection_export().is_some()));
    let cleared = base
        .with_footer_availability(
            SettingsFooterCommand::Apply,
            SettingsActionAvailability::disabled_with_reason("Draft invalid"),
        )
        .with_footer_availability(
            SettingsFooterCommand::Accept,
            SettingsActionAvailability::disabled_with_reason("Draft invalid"),
        );
    handle.update_model(cx, cleared).unwrap();
    assert_eq!(
        view.read_with(cx, |view, app| view
            .text_input_for_field_for_test(&"primary".into(), app)
            .unwrap()
            .entity_id()),
        input.entity_id()
    );
    assert_eq!(
        input.read_with(cx, |input, _| editor_snapshot(input)),
        before
    );
    handle
        .window_handle()
        .update(cx, |view, window, cx| {
            view.apply_for_test(cx);
            view.accept_for_test(cx);
            view.cancel_for_test(cx);
            assert!(view.request_page_navigation_for_test("subpage".into(), cx));
            assert!(!view.request_close_for_test(window, cx));
        })
        .unwrap();
    assert_eq!(
        events
            .borrow()
            .iter()
            .filter(|event| matches!(event, SettingsWindowEvent::CancelRequested))
            .count(),
        1
    );
    assert!(!events.borrow().iter().any(|event| matches!(
        event,
        SettingsWindowEvent::AcceptRequested | SettingsWindowEvent::ApplyRequested
    )));
    assert!(!handle.is_visible(cx).unwrap());
}

#[gpui::test]
fn retained_footer_and_action_callbacks_check_current_gate_and_disabled_focus_reasons(
    cx: &mut gpui::TestAppContext,
) {
    let base = form(primary("abc"));
    let (panel, cx) = cx.add_window_view(|window, cx| {
        SettingsPanel::new(
            base.clone(),
            SettingsPageSplitWorkReceiver::new(),
            window,
            cx,
        )
    });
    let events = observe(&panel, cx);
    cx.update(|window, app| window.draw(app).clear());
    let apply = cx.debug_bounds("settings-button-apply").unwrap().center();
    cx.update(|window, app| {
        panel.update(app, |panel, cx| {
            panel.sync_model(
                base.clone()
                    .with_interaction_gate(SettingsInteractionGate::gated("Reconciling")),
                window,
                cx,
            )
        })
    });
    cx.simulate_click(apply, Modifiers::none());
    assert!(events.borrow().is_empty());
    cx.update(|window, app| {
        panel.update(app, |panel, cx| panel.sync_model(base.clone(), window, cx))
    });
    for (event, reason) in [
        (
            SettingsWindowEvent::PageActionRequested {
                page_id: "main".into(),
                action_id: "discard".into(),
            },
            "No document changes",
        ),
        (
            SettingsWindowEvent::RowActionRequested {
                field_id: "primary".into(),
                action_id: "reset".into(),
            },
            "No staged changes",
        ),
    ] {
        let focus = panel.read_with(cx, |panel, _| panel.command_focus_for_test(&event).unwrap());
        cx.update(|window, _| window.focus(&focus));
        cx.simulate_keystrokes("enter space");
        cx.update(|window, app| window.draw(app).clear());
        assert_eq!(
            cx.update(|window, app| panel.read(app).focused_command_reason_for_test(window)),
            Some(reason.to_owned())
        );
        assert!(
            cx.debug_bounds("settings-disabled-action-focus-reason")
                .is_some()
        );
        cx.update(|window, app| {
            panel.update(app, |panel, cx| panel.sync_model(base.clone(), window, cx))
        });
        assert_eq!(
            panel.read_with(cx, |panel, _| panel.command_focus_for_test(&event)),
            Some(focus)
        );
        assert!(events.borrow().is_empty());
    }
}

#[gpui::test]
fn field_caps_refuse_utf8_and_ime_edits_without_changes_and_allow_coherent_limit_updates(
    cx: &mut gpui::TestAppContext,
) {
    let initial = form(
        primary("abc")
            .with_value_byte_limit(Some(3))
            .with_detail_field(detail("é").with_value_byte_limit(Some(4))),
    );
    let (panel, cx) = cx.add_window_view(|window, cx| {
        let mut panel =
            SettingsPanel::new(initial, SettingsPageSplitWorkReceiver::new(), window, cx);
        panel.focus_field(&"primary".into(), window, cx);
        panel
    });
    let events = observe(&panel, cx);
    let first = panel.read_with(cx, |panel, app| {
        panel
            .text_input_for_field_for_test(&"primary".into(), app)
            .unwrap()
    });
    let second = panel.read_with(cx, |panel, app| {
        panel
            .text_input_for_field_for_test(&"detail".into(), app)
            .unwrap()
    });
    let before = first.read_with(cx, |input, _| editor_snapshot(input));
    cx.simulate_input("x");
    assert_eq!(
        first.read_with(cx, |input, _| editor_snapshot(input)),
        before
    );
    assert_eq!(
        events.borrow().as_slice(),
        &[SettingsWindowEvent::FieldInputRefused {
            field_id: "primary".into(),
            reason: SettingsFieldInputRefusal::ValueByteLimitExceeded {
                limit: 3,
                attempted_bytes: 4
            }
        }]
    );
    cx.update(|window, app| {
        panel.update(app, |panel, cx| {
            panel.focus_field(&"detail".into(), window, cx);
        })
    });
    cx.simulate_input("é");
    assert_eq!(
        second.read_with(cx, |input, _| input.text().to_owned()),
        "éé"
    );
    cx.update(|window, app| {
        second.update(app, |input, cx| {
            input.replace_and_mark_text_in_range(Some(0..1), "é", Some(0..1), window, cx)
        })
    });
    let before_ime = second.read_with(cx, |input, _| editor_snapshot(input));
    events.borrow_mut().clear();
    cx.update(|window, app| {
        second.update(app, |input, cx| {
            input.replace_and_mark_text_in_range(None, "ééé", Some(0..3), window, cx)
        })
    });
    assert_eq!(
        second.read_with(cx, |input, _| editor_snapshot(input)),
        before_ime
    );
    assert!(
        matches!(events.borrow().as_slice(), [SettingsWindowEvent::FieldInputRefused { field_id, reason: SettingsFieldInputRefusal::ValueByteLimitExceeded { limit: 4, attempted_bytes: 8 } }] if field_id.as_str() == "detail")
    );
    let next = form(
        primary("longer")
            .with_value_byte_limit(Some(6))
            .with_detail_field(detail("éé").with_value_byte_limit(Some(4))),
    );
    cx.update(|window, app| panel.update(app, |panel, cx| panel.sync_model(next, window, cx)));
    assert_eq!(
        first.read_with(cx, |input, _| input.text().to_owned()),
        "longer"
    );
    assert_eq!(
        panel.read_with(cx, |panel, app| panel
            .text_input_for_field_for_test(&"primary".into(), app)
            .unwrap()
            .entity_id()),
        first.entity_id()
    );
    let next = form(
        primary("ab")
            .with_value_byte_limit(Some(2))
            .with_detail_field(detail("éé").with_value_byte_limit(Some(4))),
    );
    cx.update(|window, app| panel.update(app, |panel, cx| panel.sync_model(next, window, cx)));
    assert_eq!(
        first.read_with(cx, |input, _| input.text().to_owned()),
        "ab"
    );
}

#[gpui::test]
fn custom_body_retained_admission_closes_synchronously_and_on_panel_release(
    cx: &mut gpui::TestAppContext,
) {
    let captured = Rc::new(RefCell::new(None::<SettingsInteractionAdmission>));
    let target = captured.clone();
    let options = SettingsWindowOptions::default().with_page_body_renderer(
        SettingsPageBodyRenderer::new(move |body_id, admission| {
            assert_eq!(body_id.as_str(), "body");
            *target.borrow_mut() = Some(admission);
            Some(gpui::div().into_any_element())
        }),
    );
    let model = SettingsWindowModel::new(vec![
        SettingsSection::new("main", "Main").with_root_page(
            SettingsPage::new("main", "Main")
                .with_stacked_custom_body(SettingsPageCustomBody::new("body", 100)),
        ),
    ])
    .unwrap();
    let (panel, cx) = cx.add_window_view(|window, cx| {
        SettingsPanel::new_with_options(
            model.clone(),
            options.clone(),
            SettingsPageSplitWorkReceiver::new(),
            window,
            cx,
        )
    });
    cx.update(|window, app| window.draw(app).clear());
    let admission = captured
        .borrow()
        .clone()
        .expect("mounted custom body received its panel admission");
    assert!(admission.is_open());
    cx.update(|window, app| {
        panel.update(app, |panel, cx| {
            panel.sync_model(
                model
                    .clone()
                    .with_interaction_gate(SettingsInteractionGate::gated("Reconciling")),
                window,
                cx,
            )
        })
    });
    assert!(!admission.is_open());
    cx.update(|window, app| {
        panel.update(app, |panel, cx| panel.sync_model(model.clone(), window, cx))
    });
    assert!(admission.is_open());
    let (released_panel, weak_panel, released_admission) = cx.update(|window, app| {
        let released_panel = app.new(|cx| {
            SettingsPanel::new_with_options(
                model,
                options,
                SettingsPageSplitWorkReceiver::new(),
                window,
                cx,
            )
        });
        released_panel.update(app, |panel, cx| {
            drop(gpui::Render::render(panel, window, cx));
        });
        let weak_panel = released_panel.downgrade();
        let admission = captured.borrow().clone().expect("rendered body admission");
        (released_panel, weak_panel, admission)
    });
    assert!(released_admission.is_open());
    drop(released_panel);
    cx.cx.update(|_| {});
    cx.run_until_parked();
    assert!(weak_panel.upgrade().is_none());
    assert!(!released_admission.is_open());
}

#[gpui::test]
fn color_limits_refuse_before_popup_or_palette_publication_and_gate_preserves_popup_editor(
    cx: &mut gpui::TestAppContext,
) {
    let model = |value: &str, limit| {
        SettingsWindowModel::new(vec![
            SettingsSection::new("main", "Main").with_row(
                SettingsRow::new("color", "Color", value, SettingsFieldKind::Color)
                    .with_value_byte_limit(Some(limit)),
            ),
        ])
        .unwrap()
    };
    let (panel, cx) = cx.add_window_view(|window, cx| {
        SettingsPanel::new(
            model("#112233", 7),
            SettingsPageSplitWorkReceiver::new(),
            window,
            cx,
        )
    });
    let events = observe(&panel, cx);
    cx.update(|window, app| {
        panel.update(app, |panel, cx| {
            panel.open_color_picker_for_test("color".into(), window, cx)
        })
    });
    let input = panel.read_with(cx, |panel, app| {
        panel.text_input_for_color_picker_for_test(app).unwrap()
    });
    let before = input.read_with(cx, |input, _| editor_snapshot(input));
    assert!(!input.update(cx, |input, cx| input.replace_selected_text("oversized", cx)));
    assert_eq!(
        input.read_with(cx, |input, _| editor_snapshot(input)),
        before
    );
    assert_eq!(
        panel.read_with(cx, |panel, _| panel.color_preview_for_test(&"color".into())),
        Some("#112233".into())
    );
    assert!(events.borrow().iter().any(|event| matches!(event, SettingsWindowEvent::FieldInputRefused {field_id, ..} if field_id.as_str() == "color")));
    events.borrow_mut().clear();
    cx.update(|window, app| {
        panel.update(app, |panel, cx| {
            panel.sync_model(
                model("#112233", 7)
                    .with_interaction_gate(SettingsInteractionGate::gated("Reconciling")),
                window,
                cx,
            )
        })
    });
    assert_eq!(
        panel.read_with(cx, |panel, app| panel
            .text_input_for_color_picker_for_test(app)
            .unwrap()
            .entity_id()),
        input.entity_id()
    );
    assert!(!input.update(cx, |input, cx| input.replace_selected_text("#ffffff", cx)));
    assert_eq!(
        input.read_with(cx, |input, _| editor_snapshot(input)),
        before
    );
    assert!(events.borrow().is_empty());
    cx.update(|window, app| {
        panel.update(app, |panel, cx| {
            panel.sync_model(model("", 0), window, cx);
            panel.close_color_picker_for_test(cx);
            panel.open_color_picker_for_test("color".into(), window, cx);
        })
    });
    assert!(panel.read_with(cx, |panel, _| {
        panel.active_color_picker_field_for_test().is_none()
    }));
    assert!(matches!(
        events.borrow().as_slice(),
        [SettingsWindowEvent::FieldInputRefused {
            reason: SettingsFieldInputRefusal::ValueByteLimitExceeded {
                limit: 0,
                attempted_bytes: 7
            },
            ..
        }]
    ));
}
