mod paged_split_support;

use gpui_settings_window::{
    MAX_PAGE_SPLIT_WORK_ITEMS, SettingsPageSplitDelivery, SettingsPageSplitDeliveryError,
    SettingsPageSplitFocusResolution, SettingsPageSplitItem, SettingsPageSplitPageFailure,
    SettingsPageSplitPageRequest, SettingsPageSplitPageResult, SettingsPageSplitRequestId,
    SettingsPageSplitSourceKey, SettingsPageSplitWork,
};

use paged_split_support::*;

#[gpui::test]
fn request_is_registered_before_exposure_and_deduplicated(cx: &mut gpui::TestAppContext) {
    let handle = open(source("roles", 1, 1, 100, 16, 4096), cx);
    let request = take_page(&handle, cx);
    assert_eq!(
        deliver_ready(&handle, request.clone(), 100, items_for(&request), cx),
        SettingsPageSplitDelivery::Ready
    );
    cx.run_until_parked();

    assert!(drain_work(&handle, cx).iter().all(
        |work| !matches!(work, SettingsPageSplitWork::Page(next) if next.range() == request.range())
    ));
    let diagnostics = handle.diagnostics_snapshot(cx).unwrap();
    let pager = diagnostics.split_pager.unwrap();
    assert_eq!(pager.pending_request_count, 0);
    assert_eq!(pager.resident_page_count, 1);
}

#[gpui::test]
fn ready_failure_cancellation_and_late_obsolete_remain_distinct(cx: &mut gpui::TestAppContext) {
    let handle = open(source("roles", 1, 1, 100, 16, 4096), cx);
    let request = take_page(&handle, cx);
    let failure = SettingsPageSplitPageResult::failed(
        request.clone(),
        100,
        SettingsPageSplitPageFailure::Unavailable("Range unavailable".to_owned()),
    );
    assert_eq!(
        handle
            .deliver_page_split_result(cx, failure)
            .unwrap()
            .unwrap(),
        SettingsPageSplitDelivery::Failed
    );

    handle
        .window_handle()
        .update(cx, |view, _, cx| {
            view.demand_page_split_range_for_test(32..40, cx)
        })
        .unwrap();
    let cancelled = take_page(&handle, cx);
    assert_eq!(
        handle
            .deliver_page_split_result(
                cx,
                SettingsPageSplitPageResult::cancelled(cancelled.clone(), 100),
            )
            .unwrap()
            .unwrap(),
        SettingsPageSplitDelivery::Cancelled
    );
    assert_eq!(
        handle
            .deliver_page_split_result(cx, SettingsPageSplitPageResult::cancelled(cancelled, 100),)
            .unwrap()
            .unwrap(),
        SettingsPageSplitDelivery::Obsolete
    );
}

#[gpui::test]
fn obsolete_delivery_changes_only_the_stale_counter(cx: &mut gpui::TestAppContext) {
    let handle = open(source("roles", 1, 1, 100, 16, 4096), cx);
    let request = take_page(&handle, cx);
    let result = SettingsPageSplitPageResult::ready(request.clone(), 100, items_for(&request));
    assert_eq!(
        handle
            .deliver_page_split_result(cx, result.clone())
            .unwrap()
            .unwrap(),
        SettingsPageSplitDelivery::Ready
    );
    cx.run_until_parked();
    let before = handle.diagnostics_snapshot(cx).unwrap();

    assert_eq!(
        handle
            .deliver_page_split_result(cx, result)
            .unwrap()
            .unwrap(),
        SettingsPageSplitDelivery::Obsolete
    );
    cx.run_until_parked();
    let after = handle.diagnostics_snapshot(cx).unwrap();
    let mut expected = before;
    expected
        .split_pager
        .as_mut()
        .expect("split pager diagnostics")
        .stale_result_count += 1;
    assert_eq!(after, expected);
}

#[gpui::test]
fn duplicate_completion_stays_obsolete_beyond_work_queue_capacity(cx: &mut gpui::TestAppContext) {
    let logical_item_count = MAX_PAGE_SPLIT_WORK_ITEMS * 4;
    let handle = open(source("roles", 1, 1, logical_item_count, 1, 4096), cx);
    let mut first_result = None;
    let mut last_request = None;

    for position in 0..=(MAX_PAGE_SPLIT_WORK_ITEMS + 1) {
        handle
            .window_handle()
            .update(cx, |view, _, cx| {
                view.demand_page_split_range_for_test(position..position + 1, cx)
            })
            .unwrap();
        let request = take_page(&handle, cx);
        let result = SettingsPageSplitPageResult::ready(
            request.clone(),
            logical_item_count,
            items_for(&request),
        );
        first_result.get_or_insert_with(|| result.clone());
        last_request = Some(request);
        assert_eq!(
            handle
                .deliver_page_split_result(cx, result)
                .unwrap()
                .unwrap(),
            SettingsPageSplitDelivery::Ready
        );
    }

    cx.run_until_parked();
    let before = handle.diagnostics_snapshot(cx).unwrap();
    assert_eq!(
        handle
            .deliver_page_split_result(cx, first_result.expect("first settled result"))
            .unwrap()
            .unwrap(),
        SettingsPageSplitDelivery::Obsolete
    );
    cx.run_until_parked();
    let after = handle.diagnostics_snapshot(cx).unwrap();
    let mut expected = before;
    expected
        .split_pager
        .as_mut()
        .expect("split pager diagnostics")
        .stale_result_count += 1;
    assert_eq!(after, expected);

    let last_request = last_request.expect("last settled request");
    let never_issued = SettingsPageSplitPageRequest::new(
        last_request.page_id().clone(),
        last_request.source_key().clone(),
        SettingsPageSplitRequestId::new(u64::MAX),
        last_request.range(),
    );
    assert_eq!(
        deliver_cancelled_error(&handle, never_issued, logical_item_count, cx),
        SettingsPageSplitDeliveryError::MismatchedRequestId
    );
    assert_eq!(handle.diagnostics_snapshot(cx).unwrap(), after);
}

#[gpui::test]
fn malformed_oversized_and_short_results_leave_the_request_pending(cx: &mut gpui::TestAppContext) {
    let handle = open(source("roles", 1, 1, 100, 16, 64), cx);
    let request = take_page(&handle, cx);
    let mut malformed = items_for(&request);
    malformed[0] = SettingsPageSplitItem::new(
        request.range().start + 1,
        "wrong-position",
        "Wrong position",
    );
    assert_eq!(
        handle
            .deliver_page_split_result(
                cx,
                SettingsPageSplitPageResult::ready(request.clone(), 100, malformed),
            )
            .unwrap()
            .unwrap_err(),
        SettingsPageSplitDeliveryError::MalformedLogicalPositions
    );

    let short = items_for(&request)
        .into_iter()
        .take(request.range().len() - 1)
        .collect();
    assert_eq!(
        handle
            .deliver_page_split_result(
                cx,
                SettingsPageSplitPageResult::ready(request.clone(), 100, short),
            )
            .unwrap()
            .unwrap_err(),
        SettingsPageSplitDeliveryError::IncompleteRange
    );

    let oversized = request
        .range()
        .map(|position| {
            SettingsPageSplitItem::new(position, format!("item-{position}"), "x".repeat(80))
        })
        .collect();
    assert_eq!(
        handle
            .deliver_page_split_result(
                cx,
                SettingsPageSplitPageResult::ready(request.clone(), 100, oversized),
            )
            .unwrap()
            .unwrap_err(),
        SettingsPageSplitDeliveryError::TooManyDecodedBytes
    );
    assert_eq!(
        handle
            .deliver_page_split_result(
                cx,
                SettingsPageSplitPageResult::failed(
                    request.clone(),
                    100,
                    SettingsPageSplitPageFailure::Unavailable("x".repeat(257)),
                ),
            )
            .unwrap()
            .unwrap_err(),
        SettingsPageSplitDeliveryError::FailureMessageTooLarge
    );

    let too_many = (0..17)
        .map(|offset| {
            let position = request.range().start + offset;
            SettingsPageSplitItem::new(position, format!("many-{position}"), "Many")
        })
        .collect();
    assert_eq!(
        handle
            .deliver_page_split_result(
                cx,
                SettingsPageSplitPageResult::ready(request.clone(), 100, too_many),
            )
            .unwrap()
            .unwrap_err(),
        SettingsPageSplitDeliveryError::TooManyItems
    );

    let mut empty_id = items_for(&request);
    empty_id[0] = SettingsPageSplitItem::new(request.range().start, "", "Empty identity");
    assert_eq!(
        handle
            .deliver_page_split_result(
                cx,
                SettingsPageSplitPageResult::ready(request.clone(), 100, empty_id),
            )
            .unwrap()
            .unwrap_err(),
        SettingsPageSplitDeliveryError::EmptyItemId
    );

    let mut duplicate = items_for(&request);
    duplicate[1] = SettingsPageSplitItem::new(
        request.range().start + 1,
        duplicate[0].item_id().clone(),
        "Duplicate identity",
    );
    assert_eq!(
        handle
            .deliver_page_split_result(
                cx,
                SettingsPageSplitPageResult::ready(request.clone(), 100, duplicate),
            )
            .unwrap()
            .unwrap_err(),
        SettingsPageSplitDeliveryError::DuplicateItemId
    );
    let pager = handle
        .diagnostics_snapshot(cx)
        .unwrap()
        .split_pager
        .unwrap();
    assert_eq!(pager.pending_request_count, 1);
    assert_eq!(pager.stale_result_count, 0);
}

#[gpui::test]
fn mismatched_total_is_rejected_without_settling_current_work(cx: &mut gpui::TestAppContext) {
    let handle = open(source("roles", 1, 1, 100, 16, 4096), cx);
    let request = take_page(&handle, cx);
    assert_eq!(
        handle
            .deliver_page_split_result(
                cx,
                SettingsPageSplitPageResult::ready(request.clone(), 99, items_for(&request)),
            )
            .unwrap()
            .unwrap_err(),
        SettingsPageSplitDeliveryError::MismatchedLogicalItemCount
    );
    assert_eq!(
        handle
            .deliver_page_split_result(
                cx,
                SettingsPageSplitPageResult::cancelled(
                    SettingsPageSplitPageRequest::new(
                        "wrong-page".into(),
                        request.source_key().clone(),
                        request.request_id(),
                        request.range().start + 1..request.range().end,
                    ),
                    100,
                ),
            )
            .unwrap()
            .unwrap_err(),
        SettingsPageSplitDeliveryError::MismatchedPage
    );
    assert_eq!(
        deliver_ready(&handle, request.clone(), 100, items_for(&request), cx),
        SettingsPageSplitDelivery::Ready
    );
}

#[gpui::test]
fn each_current_request_identity_mismatch_is_typed_and_not_stale(cx: &mut gpui::TestAppContext) {
    let handle = open(source("roles", 7, 11, 100, 16, 4096), cx);
    let request = take_page(&handle, cx);
    let page_id = request.page_id().clone();
    let key = request.source_key().clone();
    let request_id = request.request_id();
    let range = request.range();

    assert_eq!(
        deliver_cancelled_error(
            &handle,
            SettingsPageSplitPageRequest::new(
                "wrong-page".into(),
                key.clone(),
                request_id,
                range.clone(),
            ),
            100,
            cx,
        ),
        SettingsPageSplitDeliveryError::MismatchedPage
    );
    assert_eq!(
        deliver_cancelled_error(
            &handle,
            SettingsPageSplitPageRequest::new(
                page_id.clone(),
                SettingsPageSplitSourceKey::new("other-source", 7, 11),
                request_id,
                range.clone(),
            ),
            100,
            cx,
        ),
        SettingsPageSplitDeliveryError::MismatchedSourceIdentity
    );
    assert_eq!(
        deliver_cancelled_error(
            &handle,
            SettingsPageSplitPageRequest::new(
                page_id.clone(),
                SettingsPageSplitSourceKey::new("roles", 8, 11),
                request_id,
                range.clone(),
            ),
            100,
            cx,
        ),
        SettingsPageSplitDeliveryError::MismatchedGeneration
    );
    assert_eq!(
        deliver_cancelled_error(
            &handle,
            SettingsPageSplitPageRequest::new(
                page_id.clone(),
                SettingsPageSplitSourceKey::new("roles", 7, 12),
                request_id,
                range.clone(),
            ),
            100,
            cx,
        ),
        SettingsPageSplitDeliveryError::MismatchedRevision
    );
    assert_eq!(
        deliver_cancelled_error(
            &handle,
            SettingsPageSplitPageRequest::new(
                page_id.clone(),
                key.clone(),
                SettingsPageSplitRequestId::new(request_id.get() + 1000),
                range.clone(),
            ),
            100,
            cx,
        ),
        SettingsPageSplitDeliveryError::MismatchedRequestId
    );
    assert_eq!(
        deliver_cancelled_error(
            &handle,
            SettingsPageSplitPageRequest::new(page_id, key, request_id, range.start + 1..range.end,),
            100,
            cx,
        ),
        SettingsPageSplitDeliveryError::MismatchedRange
    );

    let diagnostics = handle
        .diagnostics_snapshot(cx)
        .unwrap()
        .split_pager
        .unwrap();
    assert_eq!(diagnostics.pending_request_count, 1);
    assert_eq!(diagnostics.stale_result_count, 0);
    assert_eq!(
        deliver_ready(&handle, request.clone(), 100, items_for(&request), cx),
        SettingsPageSplitDelivery::Ready
    );
}

#[gpui::test]
fn never_issued_foreign_and_unmounted_completions_are_request_id_mismatches(
    cx: &mut gpui::TestAppContext,
) {
    let handle = open(source("roles", 1, 1, 100, 16, 4096), cx);
    let issued = take_page(&handle, cx);
    let foreign_future = SettingsPageSplitPageRequest::new(
        "foreign-page".into(),
        SettingsPageSplitSourceKey::new("foreign-source", 9, 9),
        SettingsPageSplitRequestId::new(u64::MAX),
        issued.range(),
    );
    assert_eq!(
        deliver_cancelled_error(&handle, foreign_future, 100, cx),
        SettingsPageSplitDeliveryError::MismatchedRequestId
    );
    let before_hide = handle.diagnostics_snapshot(cx).unwrap();

    handle.hide(cx).unwrap();
    assert_eq!(
        handle
            .deliver_page_split_result(
                cx,
                SettingsPageSplitPageResult::ready(issued.clone(), 100, items_for(&issued)),
            )
            .unwrap()
            .unwrap(),
        SettingsPageSplitDelivery::Obsolete
    );
    let never_issued = SettingsPageSplitPageRequest::new(
        issued.page_id().clone(),
        issued.source_key().clone(),
        SettingsPageSplitRequestId::new(u64::MAX),
        issued.range(),
    );
    assert_eq!(
        deliver_cancelled_error(&handle, never_issued, 100, cx),
        SettingsPageSplitDeliveryError::MismatchedRequestId
    );
    let after_hide = handle.diagnostics_snapshot(cx).unwrap();
    assert_eq!(
        after_hide
            .split_pager
            .as_ref()
            .map(|pager| pager.stale_result_count),
        before_hide
            .split_pager
            .as_ref()
            .map(|pager| pager.stale_result_count + 1)
    );
}

#[gpui::test]
fn focus_resolution_rejects_contradictory_out_of_order_resident_page_proof(
    cx: &mut gpui::TestAppContext,
) {
    for resolution in [
        SettingsPageSplitFocusResolution::Found(100),
        SettingsPageSplitFocusResolution::Removed,
    ] {
        let handle = open(source("roles", 1, 1, 200, 8, 4096), cx);
        let first = take_page(&handle, cx);
        deliver_ready(&handle, first.clone(), 200, items_for(&first), cx);
        handle
            .window_handle()
            .update(cx, |view, _, cx| {
                assert!(view.focus_page_split_position_for_test(2, cx));
            })
            .unwrap();
        handle
            .update_model(cx, model(source("roles", 1, 2, 200, 8, 4096)))
            .unwrap();
        handle
            .window_handle()
            .update(cx, |view, _, cx| {
                view.demand_page_split_range_for_test(0..16, cx)
            })
            .unwrap();
        let requests = drain_work(&handle, cx)
            .into_iter()
            .filter_map(|work| match work {
                SettingsPageSplitWork::Page(request) if request.source_key().revision() == 2 => {
                    Some(request)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let probed = requests
            .iter()
            .find(|request| request.focus_probe().is_some())
            .cloned()
            .expect("replacement demand should carry the exact focus probe");
        let resident = requests
            .into_iter()
            .find(|request| request.focus_probe().is_none() && request.range().contains(&10))
            .expect("concurrent out-of-order page should cover proof position");
        let resident_items = resident
            .range()
            .map(|position| {
                let item_id = if position == 10 {
                    "item-2".to_owned()
                } else {
                    format!("resident-{position}")
                };
                SettingsPageSplitItem::new(position, item_id, format!("Resident {position}"))
            })
            .collect();
        let resident_result = SettingsPageSplitPageResult::ready(resident, 200, resident_items);
        assert_eq!(
            handle
                .window_handle()
                .update(cx, |view, _, cx| {
                    view.deliver_page_split_result_without_notify_for_test(resident_result, cx)
                })
                .unwrap()
                .unwrap(),
            SettingsPageSplitDelivery::Ready
        );
        let probed_items = probed
            .range()
            .map(|position| {
                SettingsPageSplitItem::new(
                    position,
                    format!("probed-{position}"),
                    format!("Probed {position}"),
                )
            })
            .collect();
        assert_eq!(
            handle
                .deliver_page_split_result(
                    cx,
                    SettingsPageSplitPageResult::ready(probed, 200, probed_items)
                        .with_focus_resolution(resolution),
                )
                .unwrap()
                .unwrap_err(),
            SettingsPageSplitDeliveryError::InvalidFocusResolution
        );
    }
}
