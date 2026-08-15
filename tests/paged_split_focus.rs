mod paged_split_support;

use gpui_settings_window::{
    SettingsPageSplitDelivery, SettingsPageSplitDeliveryError, SettingsPageSplitFocusResolution,
    SettingsPageSplitItem, SettingsPageSplitPageRequest, SettingsPageSplitPageResult,
    SettingsPageSplitWork,
};
use paged_split_support::*;

#[gpui::test]
fn focus_resolution_metadata_is_required_and_validated_atomically(cx: &mut gpui::TestAppContext) {
    let handle = open(source("roles", 1, 1, 100, 16, 4096), cx);
    let first = take_page(&handle, cx);
    assert_eq!(
        handle
            .deliver_page_split_result(
                cx,
                SettingsPageSplitPageResult::ready(first.clone(), 100, items_for(&first))
                    .with_focus_resolution(SettingsPageSplitFocusResolution::Found(0)),
            )
            .unwrap()
            .unwrap_err(),
        SettingsPageSplitDeliveryError::UnexpectedFocusResolution
    );
    deliver_ready(&handle, first.clone(), 100, items_for(&first), cx);
    handle
        .window_handle()
        .update(cx, |view, _, cx| {
            assert!(view.focus_page_split_position_for_test(2, cx));
        })
        .unwrap();
    handle
        .update_model(cx, model(source("roles", 1, 2, 100, 16, 4096)))
        .unwrap();
    let probed = take_page(&handle, cx);
    assert!(probed.focus_probe().is_some());

    let without_probe = SettingsPageSplitPageRequest::new(
        probed.page_id().clone(),
        probed.source_key().clone(),
        probed.request_id(),
        probed.range(),
    );
    assert_eq!(
        deliver_cancelled_error(&handle, without_probe, 100, cx),
        SettingsPageSplitDeliveryError::MismatchedFocusProbe
    );
    assert_eq!(
        handle
            .deliver_page_split_result(
                cx,
                SettingsPageSplitPageResult::ready(probed.clone(), 100, items_for(&probed)),
            )
            .unwrap()
            .unwrap_err(),
        SettingsPageSplitDeliveryError::MissingFocusResolution
    );
    assert_eq!(
        handle
            .deliver_page_split_result(
                cx,
                SettingsPageSplitPageResult::ready(probed.clone(), 100, items_for(&probed))
                    .with_focus_resolution(SettingsPageSplitFocusResolution::Found(100)),
            )
            .unwrap()
            .unwrap_err(),
        SettingsPageSplitDeliveryError::InvalidFocusResolution
    );
    let replacement = probed
        .range()
        .map(|position| {
            SettingsPageSplitItem::new(
                position,
                format!("replacement-{position}"),
                format!("Replacement {position}"),
            )
        })
        .collect();
    assert_eq!(
        handle
            .deliver_page_split_result(
                cx,
                SettingsPageSplitPageResult::ready(probed, 100, replacement)
                    .with_focus_resolution(SettingsPageSplitFocusResolution::Removed),
            )
            .unwrap()
            .unwrap(),
        SettingsPageSplitDelivery::Ready
    );
    let diagnostics = handle
        .diagnostics_snapshot(cx)
        .unwrap()
        .split_pager
        .unwrap();
    assert_eq!(diagnostics.pending_request_count, 0);
    assert_eq!(diagnostics.stale_result_count, 0);
}

#[gpui::test]
fn focus_resolution_tracks_large_reorder_then_falls_back_after_removal(
    cx: &mut gpui::TestAppContext,
) {
    let handle = open(source("roles", 1, 1, 10_000, 8, 4096), cx);
    let first = take_page(&handle, cx);
    deliver_ready(&handle, first.clone(), 10_000, items_for(&first), cx);
    handle
        .window_handle()
        .update(cx, |view, _, cx| {
            assert!(view.focus_page_split_position_for_test(6, cx));
        })
        .unwrap();

    handle
        .update_model(cx, model(source("roles", 1, 2, 10_000, 8, 4096)))
        .unwrap();
    let reordered = take_page(&handle, cx);
    assert_eq!(
        reordered
            .focus_probe()
            .map(|probe| probe.item_id().as_str()),
        Some("item-6")
    );
    let replacement = reordered
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
        handle
            .deliver_page_split_result(
                cx,
                SettingsPageSplitPageResult::ready(reordered, 10_000, replacement)
                    .with_focus_resolution(SettingsPageSplitFocusResolution::Found(9_000)),
            )
            .unwrap()
            .unwrap(),
        SettingsPageSplitDelivery::Ready
    );
    cx.run_until_parked();

    let far = drain_work(&handle, cx)
        .into_iter()
        .find_map(|work| match work {
            SettingsPageSplitWork::Page(request) if request.range().contains(&9_000) => {
                Some(request)
            }
            _ => None,
        })
        .expect("resolved focus should reveal and request its distant page");
    let far_items = far
        .range()
        .map(|position| {
            let item_id = if position == 9_000 {
                "item-6".to_owned()
            } else {
                format!("revision-2-{position}")
            };
            SettingsPageSplitItem::new(position, item_id, format!("Revision 2 item {position}"))
        })
        .collect();
    assert_eq!(
        deliver_ready(&handle, far.clone(), 10_000, far_items, cx),
        SettingsPageSplitDelivery::Ready
    );
    assert_eq!(
        handle
            .window_handle()
            .update(cx, |view, _, cx| view
                .focused_page_split_position_for_test(cx))
            .unwrap(),
        Some(9_000)
    );
    handle
        .window_handle()
        .update(cx, |view, _, cx| {
            view.demand_page_split_range_for_test(0..8, cx)
        })
        .unwrap();
    assert!(
        drain_work(&handle, cx)
            .into_iter()
            .any(|work| matches!(work, SettingsPageSplitWork::Release(request) if request == far))
    );

    handle
        .update_model(cx, model(source("roles", 1, 3, 5_000, 8, 4096)))
        .unwrap();
    let removed = take_page(&handle, cx);
    assert_eq!(
        removed.focus_probe().map(|probe| probe.item_id().as_str()),
        Some("item-6")
    );
    assert!(
        removed.range().contains(&4_999),
        "removed-focus fallback must be realized by this result"
    );
    let replacement = removed
        .range()
        .map(|position| {
            SettingsPageSplitItem::new(
                position,
                format!("revision-3-{position}"),
                format!("Revision 3 item {position}"),
            )
        })
        .collect();
    assert_eq!(
        handle
            .deliver_page_split_result(
                cx,
                SettingsPageSplitPageResult::ready(removed.clone(), 5_000, replacement)
                    .with_focus_resolution(SettingsPageSplitFocusResolution::Removed),
            )
            .unwrap()
            .unwrap(),
        SettingsPageSplitDelivery::Ready
    );
    assert_eq!(
        handle
            .window_handle()
            .update(cx, |view, _, cx| view
                .focused_page_split_position_for_test(cx))
            .unwrap(),
        Some(4_999)
    );
    handle
        .window_handle()
        .update(cx, |view, _, cx| {
            view.demand_page_split_range_for_test(0..8, cx)
        })
        .unwrap();
    assert!(
        drain_work(&handle, cx).into_iter().any(
            |work| matches!(work, SettingsPageSplitWork::Release(request) if request == removed)
        )
    );

    handle
        .update_model(cx, model(source("roles", 1, 4, 5_000, 8, 4096)))
        .unwrap();
    let refreshed = take_page(&handle, cx);
    assert_eq!(
        refreshed
            .focus_probe()
            .map(|probe| probe.item_id().as_str()),
        Some("revision-3-4999")
    );
    let replacement = refreshed
        .range()
        .map(|position| {
            SettingsPageSplitItem::new(
                position,
                format!("revision-4-{position}"),
                format!("Revision 4 item {position}"),
            )
        })
        .collect();
    assert_eq!(
        handle
            .deliver_page_split_result(
                cx,
                SettingsPageSplitPageResult::ready(refreshed, 5_000, replacement)
                    .with_focus_resolution(SettingsPageSplitFocusResolution::Removed),
            )
            .unwrap()
            .unwrap(),
        SettingsPageSplitDelivery::Ready
    );
    assert_eq!(
        handle
            .window_handle()
            .update(cx, |view, _, cx| view
                .focused_page_split_position_for_test(cx))
            .unwrap(),
        Some(4_999)
    );
}

#[gpui::test]
fn keyboard_focus_moves_logically_and_reveals_offscreen_items(cx: &mut gpui::TestAppContext) {
    let handle = open(source("roles", 1, 1, 100, 16, 4096), cx);
    let page = take_page(&handle, cx);
    deliver_ready(&handle, page.clone(), 100, items_for(&page), cx);
    handle
        .window_handle()
        .update(cx, |view, window, cx| {
            assert!(view.focus_page_split_position_for_test(0, cx));
            view.focus_page_split_container_for_test(window, cx);
        })
        .unwrap();
    cx.simulate_keystrokes(handle.window_handle().into(), "down down");
    assert_eq!(
        handle
            .window_handle()
            .update(cx, |view, _, cx| view
                .focused_page_split_position_for_test(cx))
            .unwrap(),
        Some(2)
    );

    handle
        .window_handle()
        .update(cx, |view, _, cx| {
            assert!(view.focus_page_split_position_for_test(90, cx));
        })
        .unwrap();
    cx.run_until_parked();
    let diagnostics = handle.diagnostics_snapshot(cx).unwrap().split_list.unwrap();
    let range = diagnostics.visible_range.unwrap();
    assert!(range.start <= 90 && 90 < range.end);
    let realized = drain_work(&handle, cx)
        .into_iter()
        .find_map(|work| match work {
            SettingsPageSplitWork::Page(request) if request.range().contains(&90) => Some(request),
            _ => None,
        })
        .expect("revealed keyboard focus should request its page");
    assert_eq!(
        deliver_ready(&handle, realized.clone(), 100, items_for(&realized), cx),
        SettingsPageSplitDelivery::Ready
    );
    handle
        .window_handle()
        .update(cx, |view, _, cx| {
            view.demand_page_split_range_for_test(0..16, cx)
        })
        .unwrap();
    assert!(drain_work(&handle, cx).into_iter().any(
        |work| matches!(work, SettingsPageSplitWork::Release(request) if request == realized)
    ));

    handle
        .update_model(cx, model(source("roles", 1, 2, 100, 16, 4096)))
        .unwrap();
    let refreshed = take_page(&handle, cx);
    assert_eq!(
        refreshed
            .focus_probe()
            .map(|probe| probe.item_id().as_str()),
        Some("item-90")
    );
    let replacement = refreshed
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
        handle
            .deliver_page_split_result(
                cx,
                SettingsPageSplitPageResult::ready(refreshed, 100, replacement)
                    .with_focus_resolution(SettingsPageSplitFocusResolution::Found(10)),
            )
            .unwrap()
            .unwrap(),
        SettingsPageSplitDelivery::Ready
    );
    cx.run_until_parked();

    let reordered = drain_work(&handle, cx)
        .into_iter()
        .find_map(|work| match work {
            SettingsPageSplitWork::Page(request) if request.range().contains(&10) => Some(request),
            _ => None,
        })
        .expect("exact focus resolution should request the distant reordered row");
    let reordered_items = reordered
        .range()
        .map(|position| {
            let item_id = if position == 10 {
                "item-90".to_owned()
            } else {
                format!("revision-2-{position}")
            };
            SettingsPageSplitItem::new(position, item_id, format!("Revision 2 item {position}"))
        })
        .collect();
    assert_eq!(
        deliver_ready(&handle, reordered, 100, reordered_items, cx),
        SettingsPageSplitDelivery::Ready
    );
    assert_eq!(
        handle
            .window_handle()
            .update(cx, |view, _, cx| view
                .focused_page_split_position_for_test(cx))
            .unwrap(),
        Some(10)
    );
}

#[gpui::test]
fn newer_user_focus_supersedes_older_found_and_removed_probes(cx: &mut gpui::TestAppContext) {
    for old_resolution in [
        SettingsPageSplitFocusResolution::Found(90),
        SettingsPageSplitFocusResolution::Removed,
    ] {
        let handle = open(source("roles", 1, 1, 100, 16, 4096), cx);
        let first = take_page(&handle, cx);
        deliver_ready(&handle, first.clone(), 100, items_for(&first), cx);
        handle
            .window_handle()
            .update(cx, |view, _, cx| {
                assert!(view.focus_page_split_position_for_test(2, cx));
            })
            .unwrap();
        handle
            .update_model(cx, model(source("roles", 1, 2, 100, 16, 4096)))
            .unwrap();
        let probed = take_page(&handle, cx);
        assert_eq!(
            probed.focus_probe().map(|probe| probe.item_id().as_str()),
            Some("item-2")
        );
        handle
            .window_handle()
            .update(cx, |view, _, cx| {
                assert!(view.focus_page_split_position_for_test(3, cx));
            })
            .unwrap();
        let replacement = probed
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
            handle
                .deliver_page_split_result(
                    cx,
                    SettingsPageSplitPageResult::ready(probed, 100, replacement)
                        .with_focus_resolution(old_resolution),
                )
                .unwrap()
                .unwrap(),
            SettingsPageSplitDelivery::Ready
        );
        assert_eq!(
            handle
                .window_handle()
                .update(cx, |view, _, cx| view
                    .focused_page_split_position_for_test(cx))
                .unwrap(),
            Some(3)
        );

        handle
            .update_model(cx, model(source("roles", 1, 3, 100, 16, 4096)))
            .unwrap();
        let refreshed = take_page(&handle, cx);
        assert_eq!(
            refreshed
                .focus_probe()
                .map(|probe| probe.item_id().as_str()),
            Some("revision-2-3")
        );
    }
}

#[gpui::test]
fn split_container_participates_in_ordinary_keyboard_focus_traversal(
    cx: &mut gpui::TestAppContext,
) {
    let handle = open(source("roles", 1, 1, 100, 16, 4096), cx);
    handle
        .window_handle()
        .update(cx, |view, window, cx| view.focus_panel_for_test(window, cx))
        .unwrap();
    cx.simulate_keystrokes(handle.window_handle().into(), "tab");
    assert!(
        handle
            .window_handle()
            .update(cx, |view, window, cx| view
                .page_split_container_focused_for_test(window, cx))
            .unwrap()
    );
}
