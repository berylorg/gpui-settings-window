mod paged_split_support;

use std::cell::RefCell;
use std::rc::Rc;

use gpui_settings_window::{
    SettingsPageId, SettingsPageSplitDelivery, SettingsPageSplitItem, SettingsPageSplitItemId,
    SettingsPageSplitSourceKey, SettingsWindowEvent,
};
use paged_split_support::*;

#[gpui::test]
fn pointer_capture_revalidates_page_source_position_and_identity_after_refresh(
    cx: &mut gpui::TestAppContext,
) {
    let handle = open(source("roles", 1, 1, 100, 16, 4096), cx);
    let first = take_page(&handle, cx);
    assert_eq!(
        deliver_ready(&handle, first.clone(), 100, items_for(&first), cx),
        SettingsPageSplitDelivery::Ready
    );
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
        .update_model(cx, model(source("roles", 1, 2, 100, 16, 4096)))
        .unwrap();
    let refreshed = take_page(&handle, cx);
    let refreshed_items = refreshed
        .range()
        .map(|position| {
            SettingsPageSplitItem::new(
                position,
                format!("revision-2-{position}"),
                format!("Revision 2 item {position}"),
            )
        })
        .collect();
    assert_eq!(
        deliver_ready(&handle, refreshed, 100, refreshed_items, cx),
        SettingsPageSplitDelivery::Ready
    );

    let (stale, current) = handle
        .window_handle()
        .update(cx, |view, window, cx| {
            let stale = view.select_page_split_pointer_capture_for_test(
                SettingsPageId::from("appearance"),
                SettingsPageSplitSourceKey::new("roles", 1, 1),
                2,
                SettingsPageSplitItemId::from("item-2"),
                window,
                cx,
            );
            let current = view.select_page_split_pointer_capture_for_test(
                SettingsPageId::from("appearance"),
                SettingsPageSplitSourceKey::new("roles", 1, 2),
                2,
                SettingsPageSplitItemId::from("revision-2-2"),
                window,
                cx,
            );
            (stale, current)
        })
        .unwrap();
    assert!(!stale);
    assert!(current);
    assert_eq!(
        events.borrow().as_slice(),
        &[SettingsWindowEvent::PageSplitItemSelected {
            page_id: SettingsPageId::from("appearance"),
            item_id: SettingsPageSplitItemId::from("revision-2-2"),
        }]
    );
}
