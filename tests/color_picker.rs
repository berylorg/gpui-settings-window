use std::cell::RefCell;
use std::rc::Rc;

use gpui_settings_window::{
    RgbColor, SettingsFieldId, SettingsFieldKind, SettingsRow, SettingsSection,
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

fn color_window_options() -> SettingsWindowOptions {
    SettingsWindowOptions::default().with_saved_color_swatches([
        RgbColor::new(0x11, 0x22, 0x33),
        RgbColor::new(0xaa, 0xbb, 0xcc),
    ])
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

            view.apply_color_picker_swatch_for_test("#112233", cx);
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
