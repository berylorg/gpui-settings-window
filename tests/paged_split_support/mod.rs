#![allow(dead_code)]

use gpui_settings_window::{
    SettingsPage, SettingsPageSplitDelivery, SettingsPageSplitDeliveryError, SettingsPageSplitItem,
    SettingsPageSplitPageRequest, SettingsPageSplitPageResult, SettingsPageSplitSource,
    SettingsPageSplitSourceKey, SettingsPageSplitWork, SettingsSection, SettingsWindowHandle,
    SettingsWindowModel, SettingsWindowOpenDisposition, SettingsWindowOptions,
    open_settings_window,
};

pub(crate) fn source(
    identity: &str,
    generation: u64,
    revision: u64,
    count: usize,
    page_items: usize,
    page_bytes: usize,
) -> SettingsPageSplitSource {
    SettingsPageSplitSource::new(
        SettingsPageSplitSourceKey::new(identity, generation, revision),
        count,
        page_items,
        page_bytes,
    )
}

pub(crate) fn model(source: SettingsPageSplitSource) -> SettingsWindowModel {
    SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_root_page(
            SettingsPage::new("appearance", "Appearance").with_paged_split_source(source),
        ),
    ])
    .expect("paged split model should validate")
}

pub(crate) fn open(
    source: SettingsPageSplitSource,
    cx: &mut gpui::TestAppContext,
) -> SettingsWindowHandle {
    let handle = cx.update(|cx| {
        open_settings_window(
            cx,
            model(source),
            SettingsWindowOptions::default(),
            SettingsWindowOpenDisposition::Visible {
                focus_requested: false,
            },
        )
        .expect("settings window should open")
    });
    cx.run_until_parked();
    handle
}

pub(crate) fn take_page(
    handle: &SettingsWindowHandle,
    cx: &mut gpui::TestAppContext,
) -> SettingsPageSplitPageRequest {
    loop {
        match handle
            .take_page_split_work(cx)
            .expect("settings window should remain available")
            .expect("split pager should expose work")
        {
            SettingsPageSplitWork::Page(request) => return request,
            SettingsPageSplitWork::Cancel(_) | SettingsPageSplitWork::Release(_) => {}
        }
    }
}

pub(crate) fn items_for(request: &SettingsPageSplitPageRequest) -> Vec<SettingsPageSplitItem> {
    request
        .range()
        .map(|position| {
            SettingsPageSplitItem::new(
                position,
                format!("item-{position}"),
                format!("Item {position}"),
            )
        })
        .collect()
}

pub(crate) fn deliver_ready(
    handle: &SettingsWindowHandle,
    request: SettingsPageSplitPageRequest,
    count: usize,
    items: Vec<SettingsPageSplitItem>,
    cx: &mut gpui::TestAppContext,
) -> SettingsPageSplitDelivery {
    handle
        .deliver_page_split_result(
            cx,
            SettingsPageSplitPageResult::ready(request, count, items),
        )
        .expect("settings window should remain available")
        .expect("page should satisfy the exact contract")
}

pub(crate) fn drain_work(
    handle: &SettingsWindowHandle,
    cx: &mut gpui::TestAppContext,
) -> Vec<SettingsPageSplitWork> {
    let mut work = Vec::new();
    while let Some(next) = handle
        .take_page_split_work(cx)
        .expect("settings window should remain available")
    {
        work.push(next);
    }
    work
}

pub(crate) fn deliver_cancelled_error(
    handle: &SettingsWindowHandle,
    request: SettingsPageSplitPageRequest,
    logical_item_count: usize,
    cx: &mut gpui::TestAppContext,
) -> SettingsPageSplitDeliveryError {
    handle
        .deliver_page_split_result(
            cx,
            SettingsPageSplitPageResult::cancelled(request, logical_item_count),
        )
        .expect("settings window should remain available")
        .expect_err("mismatched current result should be rejected")
}
