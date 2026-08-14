use std::cell::RefCell;
use std::rc::Rc;

use gpui::AppContext as _;
use gpui_settings_window::{
    RgbColor, SettingsBreadcrumbSegment, SettingsFieldId, SettingsFieldKind, SettingsPage,
    SettingsRow, SettingsSavedColorSwatch, SettingsSavedColorSwatchId, SettingsSection,
    SettingsWindowEvent, SettingsWindowModel, SettingsWindowOpenDisposition, SettingsWindowOptions,
    open_settings_window,
};

fn color_settings_model(value: &str) -> SettingsWindowModel {
    SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_row(SettingsRow::new(
            "accent_color",
            "Accent color",
            value,
            SettingsFieldKind::Color,
        )),
    ])
    .expect("valid settings model")
}

fn color_settings_model_with_pages(selected_page: &str) -> SettingsWindowModel {
    let mut model = SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance")
            .with_root_page(SettingsPage::new("appearance", "Appearance").with_row(
                SettingsRow::new(
                    "accent_color",
                    "Accent color",
                    "#6699cc",
                    SettingsFieldKind::Color,
                ),
            ))
            .with_page(
                SettingsPage::new("theme_editor", "Theme editor")
                    .with_breadcrumb_segment(SettingsBreadcrumbSegment::linked(
                        "Appearance",
                        "appearance",
                    ))
                    .with_back_target("appearance")
                    .with_row(SettingsRow::new(
                        "theme_color",
                        "Theme color",
                        "#112233",
                        SettingsFieldKind::Color,
                    )),
            ),
    ])
    .expect("valid settings model");
    model
        .select_page(selected_page)
        .expect("selected page should exist");
    model
}

fn color_window_options() -> SettingsWindowOptions {
    SettingsWindowOptions::default()
        .with_saved_color_swatches([
            SettingsSavedColorSwatch::new("blue", RgbColor::new(0x11, 0x22, 0x33)),
            SettingsSavedColorSwatch::new("accent", RgbColor::new(0xaa, 0xbb, 0xcc)),
        ])
        .expect("bounded colors")
}

#[test]
fn parses_and_formats_canonical_rgb_hex() {
    let color = RgbColor::parse("#Aa09fF").expect("valid color");

    assert_eq!(color.red(), 0xaa);
    assert_eq!(color.green(), 0x09);
    assert_eq!(color.blue(), 0xff);
    assert_eq!(color.to_hex(), "#aa09ff");
    assert_eq!(RgbColor::parse("aa09ff"), None);
    assert_eq!(RgbColor::parse("#aa09f"), None);
    assert_eq!(RgbColor::parse("#aa09fg"), None);
}

#[gpui::test]
fn opens_picker_and_tracks_valid_and_invalid_text_drafts(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            color_settings_model("#6699cc"),
            color_window_options(),
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
            let field_id = SettingsFieldId::from("accent_color");
            view.open_color_picker_for_test(field_id.clone(), window, cx);
            assert_eq!(
                view.active_color_picker_field_for_test(cx),
                Some(field_id.clone()),
            );
            assert_eq!(
                view.color_preview_for_test(&field_id, cx).as_deref(),
                Some("#6699cc")
            );

            view.replace_color_picker_text_for_test("#AABBCC", cx);
            assert_eq!(
                view.color_preview_for_test(&field_id, cx).as_deref(),
                Some("#aabbcc")
            );

            view.replace_color_picker_text_for_test("not a color", cx);
            assert_eq!(
                view.color_preview_for_test(&field_id, cx).as_deref(),
                Some("#aabbcc")
            );
        })
        .expect("settings window should update");

    let events = events.borrow();
    assert!(events.contains(&SettingsWindowEvent::FieldChanged {
        field_id: SettingsFieldId::from("accent_color"),
        value: String::from("#aabbcc"),
    }));
    assert!(events.contains(&SettingsWindowEvent::FieldChanged {
        field_id: SettingsFieldId::from("accent_color"),
        value: String::from("not a color"),
    }));
}

#[gpui::test]
fn invalid_row_drafts_keep_latest_known_valid_color(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            color_settings_model("#6699cc"),
            color_window_options(),
            SettingsWindowOpenDisposition::Visible {
                focus_requested: false,
            },
        )
        .expect("settings window should open")
    });
    let field_id = SettingsFieldId::from("accent_color");

    handle
        .update_model(cx, color_settings_model("not a color"))
        .expect("model sync should succeed");
    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            assert_eq!(
                view.color_preview_for_test(&field_id, cx).as_deref(),
                Some("#6699cc"),
            );
        })
        .expect("settings window should be readable");

    handle
        .update_model(cx, color_settings_model("#112233"))
        .expect("model sync should succeed");
    handle
        .update_model(cx, color_settings_model("still not a color"))
        .expect("model sync should succeed");
    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            assert_eq!(
                view.color_preview_for_test(&field_id, cx).as_deref(),
                Some("#112233"),
            );
        })
        .expect("settings window should be readable");
}

#[gpui::test]
fn active_picker_render_uses_current_preview_without_model_lookup(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            color_settings_model("#6699cc"),
            color_window_options(),
            SettingsWindowOpenDisposition::Visible {
                focus_requested: false,
            },
        )
        .expect("settings window should open")
    });
    let field_id = SettingsFieldId::from("accent_color");

    handle
        .window_handle()
        .update(cx, |view, window, cx| {
            view.open_color_picker_for_test(field_id.clone(), window, cx);
            view.replace_color_picker_text_for_test("#AABBCC", cx);
        })
        .expect("settings window should update");
    cx.update_window(handle.window_handle().into(), |_, window, cx| {
        window.draw(cx).clear();
    })
    .expect("settings window should draw");

    let performance = handle
        .diagnostics_snapshot(cx)
        .expect("settings diagnostics should be readable")
        .performance;
    assert_eq!(performance.last_render_color_preview_lookup_count, 1);
    assert_eq!(performance.last_render_color_model_lookup_count, 0);

    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            assert_eq!(
                view.color_preview_for_test(&field_id, cx).as_deref(),
                Some("#aabbcc"),
            );
        })
        .expect("settings window should be readable");
}

#[gpui::test]
fn applies_saved_swatches_and_channel_values(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            color_settings_model("#6699cc"),
            color_window_options(),
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
            let field_id = SettingsFieldId::from("accent_color");
            view.open_color_picker_for_test(field_id.clone(), window, cx);

            assert!(
                view.apply_color_picker_swatch_for_test(
                    SettingsSavedColorSwatchId::from("blue"),
                    cx
                )
            );
            assert_eq!(
                view.color_preview_for_test(&field_id, cx).as_deref(),
                Some("#112233"),
            );

            view.replace_color_picker_channel_text_for_test("rgb.red", "255", cx);
            assert_eq!(
                view.color_preview_for_test(&field_id, cx).as_deref(),
                Some("#ff2233"),
            );
            assert_eq!(
                view.color_picker_channel_values_for_test(cx)
                    .get("rgb.red")
                    .map(String::as_str),
                Some("255"),
            );
        })
        .expect("settings window should update");

    let events = events.borrow();
    assert!(events.contains(&SettingsWindowEvent::FieldChanged {
        field_id: SettingsFieldId::from("accent_color"),
        value: String::from("#112233"),
    }));
    assert!(events.contains(&SettingsWindowEvent::FieldChanged {
        field_id: SettingsFieldId::from("accent_color"),
        value: String::from("#ff2233"),
    }));
}

#[gpui::test]
fn saved_swatch_grid_keys_use_stable_identities_and_ten_column_traversal(
    cx: &mut gpui::TestAppContext,
) {
    let swatches: Vec<_> = (0..20)
        .map(|index| {
            SettingsSavedColorSwatch::new(
                format!("swatch-{index}"),
                RgbColor::new(index as u8, 2, 3),
            )
        })
        .collect();
    let options = SettingsWindowOptions::default()
        .with_saved_color_swatches(swatches)
        .expect("distinct identities are valid");
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            color_settings_model("#6699cc"),
            options,
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
            view.open_color_picker_for_test(SettingsFieldId::from("accent_color"), window, cx);
            view.focus_saved_color_grid_for_test(window, cx);
        })
        .expect("settings window should update");

    cx.simulate_keystrokes(handle.window_handle().into(), "right right down enter");
    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            assert_eq!(
                view.focused_saved_color_swatch_for_test(cx),
                Some(SettingsSavedColorSwatchId::from("swatch-12")),
            );
        })
        .expect("settings window should be readable");
    assert!(
        events
            .borrow()
            .contains(&SettingsWindowEvent::FieldChanged {
                field_id: SettingsFieldId::from("accent_color"),
                value: RgbColor::new(12, 2, 3).to_hex(),
            })
    );
}

#[gpui::test]
fn saved_swatch_focus_reconciles_by_identity_then_nearest_position(cx: &mut gpui::TestAppContext) {
    let options = SettingsWindowOptions::default()
        .with_saved_color_swatches([
            SettingsSavedColorSwatch::new("a", RgbColor::new(1, 1, 1)),
            SettingsSavedColorSwatch::new("b", RgbColor::new(1, 1, 1)),
            SettingsSavedColorSwatch::new("c", RgbColor::new(2, 2, 2)),
        ])
        .expect("duplicate colors with distinct identities are valid");
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            color_settings_model("#6699cc"),
            options,
            SettingsWindowOpenDisposition::Visible {
                focus_requested: false,
            },
        )
        .expect("settings window should open")
    });
    handle
        .window_handle()
        .update(cx, |view, window, cx| {
            view.open_color_picker_for_test(SettingsFieldId::from("accent_color"), window, cx);
            assert!(
                view.focus_saved_color_swatch_for_test(SettingsSavedColorSwatchId::from("b"), cx)
            );
            assert!(
                view.apply_color_picker_swatch_for_test(SettingsSavedColorSwatchId::from("b"), cx,)
            );
        })
        .expect("settings window should update");
    handle
        .update_options(
            cx,
            SettingsWindowOptions::default()
                .with_saved_color_swatches([
                    SettingsSavedColorSwatch::new("c", RgbColor::new(2, 2, 2)),
                    SettingsSavedColorSwatch::new("b", RgbColor::new(1, 1, 1)),
                ])
                .expect("refresh options are valid"),
        )
        .expect("options should update");
    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            assert_eq!(
                view.focused_saved_color_swatch_for_test(cx),
                Some(SettingsSavedColorSwatchId::from("b"))
            );
            assert_eq!(
                view.selected_saved_color_swatch_for_test(cx),
                Some(SettingsSavedColorSwatchId::from("b"))
            );
        })
        .expect("settings window should be readable");
    handle
        .update_options(
            cx,
            SettingsWindowOptions::default()
                .with_saved_color_swatches([SettingsSavedColorSwatch::new(
                    "c",
                    RgbColor::new(2, 2, 2),
                )])
                .expect("refresh options are valid"),
        )
        .expect("options should update");
    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            assert_eq!(
                view.focused_saved_color_swatch_for_test(cx),
                Some(SettingsSavedColorSwatchId::from("c"))
            );
            assert_eq!(view.selected_saved_color_swatch_for_test(cx), None);
        })
        .expect("settings window should be readable");
    handle
        .update_options(cx, SettingsWindowOptions::default())
        .expect("empty options should update");
    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            assert_eq!(view.focused_saved_color_swatch_for_test(cx), None);
            assert_eq!(view.selected_saved_color_swatch_for_test(cx), None);
        })
        .expect("settings window should be readable");
}

#[gpui::test]
fn color_channel_up_and_down_keys_step_values(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            color_settings_model("#6699cc"),
            color_window_options(),
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
            let field_id = SettingsFieldId::from("accent_color");
            view.open_color_picker_for_test(field_id, window, cx);
            view.focus_color_picker_channel_for_test("rgb.red", window, cx);
        })
        .expect("settings window should update");

    cx.simulate_keystrokes(handle.window_handle().into(), "up down");

    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            assert_eq!(
                view.color_picker_channel_values_for_test(cx)
                    .get("rgb.red")
                    .map(String::as_str),
                Some("102"),
            );
            assert_eq!(
                view.color_preview_for_test(&SettingsFieldId::from("accent_color"), cx)
                    .as_deref(),
                Some("#6699cc"),
            );
        })
        .expect("settings window should be readable");

    let events = events.borrow();
    assert!(events.contains(&SettingsWindowEvent::FieldChanged {
        field_id: SettingsFieldId::from("accent_color"),
        value: String::from("#6799cc"),
    }));
    assert!(events.contains(&SettingsWindowEvent::FieldChanged {
        field_id: SettingsFieldId::from("accent_color"),
        value: String::from("#6699cc"),
    }));
}

#[gpui::test]
fn applies_palette_neutral_and_lightness_values(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            color_settings_model("#6699cc"),
            color_window_options(),
            SettingsWindowOpenDisposition::Visible {
                focus_requested: false,
            },
        )
        .expect("settings window should open")
    });

    handle
        .window_handle()
        .update(cx, |view, window, cx| {
            let field_id = SettingsFieldId::from("accent_color");
            view.open_color_picker_for_test(field_id.clone(), window, cx);
            view.apply_color_picker_main_palette_selection_for_test(0, 100, cx);
            assert_eq!(
                view.color_picker_main_palette_values_for_test(cx),
                Some((0, 100)),
            );
            view.apply_color_picker_neutral_strip_selection_for_test(48, cx);
            assert_eq!(view.color_picker_neutral_strip_value_for_test(cx), Some(48));
            view.apply_color_picker_lightness_for_test(74, cx);
            assert_eq!(view.color_picker_lightness_value_for_test(cx), Some(74));
            assert!(view.color_preview_for_test(&field_id, cx).is_some());
        })
        .expect("settings window should update");
}

#[gpui::test]
fn host_can_close_color_picker_as_transient_popup(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            color_settings_model("#6699cc"),
            color_window_options(),
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
            let field_id = SettingsFieldId::from("accent_color");
            view.open_color_picker_for_test(field_id.clone(), window, cx);
            assert_eq!(
                view.active_color_picker_field_for_test(cx),
                Some(field_id.clone()),
            );
        })
        .expect("settings window should update");
    assert!(
        handle
            .has_transient_popups(cx)
            .expect("transient popup state should be readable")
    );

    assert!(
        handle
            .close_transient_popups(cx)
            .expect("transient popup close should succeed")
    );
    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            assert_eq!(view.active_color_picker_field_for_test(cx), None);
        })
        .expect("settings window should be readable");
    assert!(
        !handle
            .has_transient_popups(cx)
            .expect("transient popup state should be readable")
    );
    assert!(
        !handle
            .close_transient_popups(cx)
            .expect("transient popup close should be idempotent")
    );
    assert!(events.borrow().is_empty());
}

#[gpui::test]
fn hiding_window_closes_color_picker_transient_popup(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            color_settings_model("#6699cc"),
            color_window_options(),
            SettingsWindowOpenDisposition::Visible {
                focus_requested: false,
            },
        )
        .expect("settings window should open")
    });

    handle
        .window_handle()
        .update(cx, |view, window, cx| {
            let field_id = SettingsFieldId::from("accent_color");
            view.open_color_picker_for_test(field_id.clone(), window, cx);
            assert_eq!(
                view.active_color_picker_field_for_test(cx),
                Some(field_id.clone()),
            );
        })
        .expect("settings window should update");
    assert!(
        handle
            .has_transient_popups(cx)
            .expect("transient popup state should be readable")
    );

    handle.hide(cx).expect("settings window should hide");

    assert!(
        !handle
            .is_visible(cx)
            .expect("settings window visibility should be readable")
    );
    assert!(
        !handle
            .has_transient_popups(cx)
            .expect("transient popup state should be readable")
    );
    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            assert_eq!(view.active_color_picker_field_for_test(cx), None);
        })
        .expect("settings window should be readable");

    handle
        .show(cx, color_settings_model("#6699cc"), false)
        .expect("settings window should show again");
    assert!(
        !handle
            .has_transient_popups(cx)
            .expect("transient popup state should be readable")
    );
    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            assert_eq!(view.active_color_picker_field_for_test(cx), None);
        })
        .expect("settings window should be readable");
}

#[gpui::test]
fn same_section_page_change_closes_color_picker_popup(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            color_settings_model_with_pages("appearance"),
            color_window_options(),
            SettingsWindowOpenDisposition::Visible {
                focus_requested: false,
            },
        )
        .expect("settings window should open")
    });

    handle
        .window_handle()
        .update(cx, |view, window, cx| {
            let field_id = SettingsFieldId::from("accent_color");
            view.open_color_picker_for_test(field_id.clone(), window, cx);
            assert_eq!(
                view.active_color_picker_field_for_test(cx),
                Some(field_id.clone()),
            );
        })
        .expect("settings window should update");
    assert!(
        handle
            .has_transient_popups(cx)
            .expect("transient popup state should be readable")
    );

    handle
        .update_model(cx, color_settings_model_with_pages("theme_editor"))
        .expect("sync should succeed");

    assert!(
        !handle
            .has_transient_popups(cx)
            .expect("transient popup state should be readable")
    );
    handle
        .window_handle()
        .read_with(cx, |view, cx| {
            assert_eq!(view.active_color_picker_field_for_test(cx), None);
        })
        .expect("settings window should be readable");
}
