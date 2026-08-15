mod paged_split_support;

use gpui_settings_window::{
    MAX_PAGE_SPLIT_ACTIVE_PAGES, MAX_PAGE_SPLIT_WORK_ITEMS, SettingsPage,
    SettingsPageSplitDelivery, SettingsPageSplitPageFailure, SettingsPageSplitPageResult,
    SettingsPageSplitWork, SettingsSection, SettingsWindowModel, SettingsWindowOpenDisposition,
    SettingsWindowOptions, open_settings_window,
};
use paged_split_support::*;

#[gpui::test]
fn source_key_replacement_cancels_once_and_late_completion_is_obsolete(
    cx: &mut gpui::TestAppContext,
) {
    let handle = open(source("roles", 1, 1, 100, 16, 4096), cx);
    let old = take_page(&handle, cx);
    handle
        .update_model(cx, model(source("roles", 1, 2, 100, 16, 4096)))
        .unwrap();
    let work = drain_work(&handle, cx);
    assert_eq!(
        work.iter()
            .filter(
                |work| matches!(work, SettingsPageSplitWork::Cancel(request) if request == &old)
            )
            .count(),
        1
    );
    assert_eq!(
        handle
            .deliver_page_split_result(
                cx,
                SettingsPageSplitPageResult::ready(old.clone(), 100, items_for(&old)),
            )
            .unwrap()
            .unwrap(),
        SettingsPageSplitDelivery::Obsolete
    );
}

#[gpui::test]
fn identity_generation_revision_page_hide_and_release_all_cancel(cx: &mut gpui::TestAppContext) {
    for replacement in [
        source("other", 1, 1, 100, 16, 4096),
        source("roles", 2, 1, 100, 16, 4096),
        source("roles", 1, 2, 100, 16, 4096),
    ] {
        let handle = open(source("roles", 1, 1, 100, 16, 4096), cx);
        let pending = take_page(&handle, cx);
        handle.update_model(cx, model(replacement)).unwrap();
        assert_eq!(
            drain_work(&handle, cx)
                .iter()
                .filter(|work| matches!(work, SettingsPageSplitWork::Cancel(request) if request == &pending))
                .count(),
            1
        );
    }

    let hidden = open(source("roles", 1, 1, 100, 16, 4096), cx);
    let pending = take_page(&hidden, cx);
    hidden.hide(cx).unwrap();
    assert!(
        drain_work(&hidden, cx).iter().any(
            |work| matches!(work, SettingsPageSplitWork::Cancel(request) if request == &pending)
        )
    );

    let resident = open(source("roles", 1, 1, 100, 16, 4096), cx);
    let page = take_page(&resident, cx);
    deliver_ready(&resident, page.clone(), 100, items_for(&page), cx);
    resident.hide(cx).unwrap();
    assert_eq!(
        drain_work(&resident, cx)
            .iter()
            .filter(
                |work| matches!(work, SettingsPageSplitWork::Release(request) if request == &page)
            )
            .count(),
        1
    );
    resident.hide(cx).unwrap();
    assert!(drain_work(&resident, cx).is_empty());

    let released = open(source("roles", 1, 1, 100, 16, 4096), cx);
    let pending = take_page(&released, cx);
    released
        .window_handle()
        .update(cx, |view, _, cx| view.release_page_split_for_test(cx))
        .unwrap();
    assert!(
        drain_work(&released, cx).iter().any(
            |work| matches!(work, SettingsPageSplitWork::Cancel(request) if request == &pending)
        )
    );
}

#[gpui::test]
fn page_source_generation_and_revision_replacement_release_resident_pages(
    cx: &mut gpui::TestAppContext,
) {
    let replacements = [
        model(source("other", 1, 1, 100, 16, 4096)),
        model(source("roles", 2, 1, 100, 16, 4096)),
        model(source("roles", 1, 2, 100, 16, 4096)),
        SettingsWindowModel::new(vec![
            SettingsSection::new("other", "Other").with_root_page(
                SettingsPage::new("other-page", "Other page")
                    .with_paged_split_source(source("roles", 1, 1, 100, 16, 4096)),
            ),
        ])
        .unwrap(),
    ];

    for replacement in replacements {
        let handle = open(source("roles", 1, 1, 100, 16, 4096), cx);
        let resident = take_page(&handle, cx);
        assert_eq!(
            deliver_ready(&handle, resident.clone(), 100, items_for(&resident), cx,),
            SettingsPageSplitDelivery::Ready
        );
        handle.update_model(cx, replacement).unwrap();
        assert_eq!(
            drain_work(&handle, cx)
                .iter()
                .filter(
                    |work| matches!(work, SettingsPageSplitWork::Release(request) if request == &resident)
                )
                .count(),
            1
        );
    }
}

#[gpui::test]
fn selected_page_change_cancels_the_previous_page_request(cx: &mut gpui::TestAppContext) {
    let first = source("first", 1, 1, 100, 16, 4096);
    let second = source("second", 1, 1, 100, 16, 4096);
    let sections = vec![
        SettingsSection::new("appearance", "Appearance")
            .with_root_page(
                SettingsPage::new("appearance", "Appearance").with_paged_split_source(first),
            )
            .with_page(SettingsPage::new("alternate", "Alternate").with_paged_split_source(second)),
    ];
    let handle = open(
        model(source("first", 1, 1, 100, 16, 4096))
            .selected_page()
            .paged_split_source()
            .unwrap()
            .clone(),
        cx,
    );
    let pending = take_page(&handle, cx);
    handle
        .update_model(
            cx,
            SettingsWindowModel::with_selected_page(sections, "appearance", "alternate").unwrap(),
        )
        .unwrap();
    assert!(
        drain_work(&handle, cx).iter().any(
            |work| matches!(work, SettingsPageSplitWork::Cancel(request) if request == &pending)
        )
    );
}

#[gpui::test]
fn failed_ranges_never_mix_pages_across_generations(cx: &mut gpui::TestAppContext) {
    let handle = open(source("roles", 1, 1, 100, 16, 4096), cx);
    let ready = take_page(&handle, cx);
    assert_eq!(
        deliver_ready(&handle, ready.clone(), 100, items_for(&ready), cx),
        SettingsPageSplitDelivery::Ready
    );
    handle
        .window_handle()
        .update(cx, |view, _, cx| {
            view.demand_page_split_range_for_test(32..40, cx)
        })
        .unwrap();
    let failed = take_page(&handle, cx);
    handle
        .deliver_page_split_result(
            cx,
            SettingsPageSplitPageResult::failed(
                failed,
                100,
                SettingsPageSplitPageFailure::Unavailable("Unavailable".to_owned()),
            ),
        )
        .unwrap()
        .unwrap();

    handle
        .update_model(cx, model(source("roles", 2, 1, 100, 16, 4096)))
        .unwrap();
    cx.run_until_parked();
    let diagnostics = handle
        .diagnostics_snapshot(cx)
        .unwrap()
        .split_pager
        .unwrap();
    assert_eq!(diagnostics.resident_page_count, 0);
    assert_eq!(diagnostics.resident_item_count, 0);
}

#[gpui::test]
fn residency_and_pending_work_stay_bounded_as_logical_count_grows(cx: &mut gpui::TestAppContext) {
    for count in [1_000usize, 1_000_000usize] {
        let handle = open(source("roles", 1, count as u64, count, 8, 4096), cx);
        let initial = drain_work(&handle, cx);
        for work in initial {
            if let SettingsPageSplitWork::Page(request) = work {
                assert_eq!(
                    deliver_ready(&handle, request.clone(), count, items_for(&request), cx),
                    SettingsPageSplitDelivery::Ready
                );
            }
        }
        handle
            .window_handle()
            .update(cx, |view, _, cx| {
                view.set_page_split_scroll_offset_for_test((count - 12) as f32 * 92.0, cx)
            })
            .unwrap();
        cx.run_until_parked();
        let far = drain_work(&handle, cx);
        for work in far {
            if let SettingsPageSplitWork::Page(request) = work {
                deliver_ready(&handle, request.clone(), count, items_for(&request), cx);
            }
        }
        let pager = handle
            .diagnostics_snapshot(cx)
            .unwrap()
            .split_pager
            .unwrap();
        assert!(pager.resident_page_count <= 2);
        assert!(pager.resident_item_count <= 16);
        assert_eq!(pager.pending_request_count, 0);
    }
}

#[gpui::test]
fn shared_receiver_stays_bounded_during_undrained_revision_churn(cx: &mut gpui::TestAppContext) {
    let handle = open(source("roles", 1, 1, 50_000, 1, 4096), cx);
    let receiver = handle.page_split_work_receiver();
    handle
        .window_handle()
        .update(cx, |view, _, cx| {
            view.demand_page_split_range_for_test(0..16, cx)
        })
        .unwrap();
    let initially_exposed = drain_work(&handle, cx)
        .into_iter()
        .filter(|work| matches!(work, SettingsPageSplitWork::Page(_)))
        .count();
    assert!(initially_exposed > 0);

    for revision in 2..=500 {
        handle
            .update_model(cx, model(source("roles", 1, revision, 50_000, 1, 4096)))
            .unwrap();
        assert!(receiver.pending_work_count() <= MAX_PAGE_SPLIT_WORK_ITEMS);
    }
    assert!(receiver.pending_work_count() <= receiver.capacity());
}

#[gpui::test]
fn preheated_hidden_window_requests_nothing_until_shown(cx: &mut gpui::TestAppContext) {
    let split = source("roles", 1, 1, 100, 16, 4096);
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            model(split.clone()),
            SettingsWindowOptions::default(),
            SettingsWindowOpenDisposition::Hidden,
        )
        .unwrap()
    });
    cx.run_until_parked();
    assert!(drain_work(&handle, cx).is_empty());
    handle.show(cx, model(split), false).unwrap();
    cx.run_until_parked();
    assert!(
        drain_work(&handle, cx)
            .iter()
            .any(|work| matches!(work, SettingsPageSplitWork::Page(_)))
    );
}

#[gpui::test]
fn over_cap_one_item_demand_turns_pages_over_and_reaches_every_position(
    cx: &mut gpui::TestAppContext,
) {
    let demanded_end = MAX_PAGE_SPLIT_ACTIVE_PAGES + 4;
    let handle = open(source("roles", 1, 1, 128, 1, 4096), cx);
    let receiver = handle.page_split_work_receiver();
    handle
        .window_handle()
        .update(cx, |view, _, cx| {
            view.demand_page_split_range_for_test(0..demanded_end, cx)
        })
        .unwrap();

    let mut reached = vec![false; demanded_end];
    let mut release_count = 0;
    for _ in 0..(demanded_end * 3) {
        let work = drain_work(&handle, cx);
        assert!(
            !work.is_empty(),
            "bounded turnover must keep making progress"
        );
        for work in work {
            match work {
                SettingsPageSplitWork::Page(request) => {
                    for position in request.range() {
                        if position < demanded_end {
                            reached[position] = true;
                        }
                    }
                    let result = SettingsPageSplitPageResult::ready(
                        request.clone(),
                        128,
                        items_for(&request),
                    );
                    assert_eq!(
                        handle
                            .window_handle()
                            .update(cx, |view, _, cx| {
                                view.deliver_page_split_result_without_notify_for_test(result, cx)
                            })
                            .unwrap()
                            .unwrap(),
                        SettingsPageSplitDelivery::Ready
                    );
                }
                SettingsPageSplitWork::Release(_) => release_count += 1,
                SettingsPageSplitWork::Cancel(_) => {}
            }
        }
        handle
            .window_handle()
            .update(cx, |view, _, cx| {
                view.demand_page_split_range_for_test(0..demanded_end, cx)
            })
            .unwrap();
        let pager = handle
            .diagnostics_snapshot(cx)
            .unwrap()
            .split_pager
            .unwrap();
        assert!(pager.resident_page_count <= MAX_PAGE_SPLIT_ACTIVE_PAGES);
        assert!(pager.pending_request_count <= MAX_PAGE_SPLIT_ACTIVE_PAGES);
        assert!(receiver.pending_work_count() <= MAX_PAGE_SPLIT_WORK_ITEMS);
        if reached.iter().all(|reached| *reached) {
            break;
        }
    }

    assert!(reached.iter().all(|reached| *reached));
    assert!(
        release_count > 0,
        "over-cap progress must turn resident pages over"
    );
}
