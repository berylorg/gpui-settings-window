use std::collections::{BTreeMap, HashMap};
use std::ops::Range;

use crate::{
    MAX_PAGE_SPLIT_ACTIVE_PAGES, SettingsPageId, SettingsPageSplitDelivery,
    SettingsPageSplitDeliveryError, SettingsPageSplitFocusProbe, SettingsPageSplitFocusResolution,
    SettingsPageSplitItem, SettingsPageSplitItemId, SettingsPageSplitPageOutcome,
    SettingsPageSplitPageRequest, SettingsPageSplitPageResult, SettingsPageSplitRequestId,
    SettingsPageSplitSource, SettingsPageSplitWorkReceiver,
};

const MAX_FAILURE_MESSAGE_BYTES: usize = 256;

mod delivery;

#[derive(Clone)]
struct ResidentPage {
    request: SettingsPageSplitPageRequest,
    items: Vec<SettingsPageSplitItem>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SplitRangeState {
    Pending,
    Failed,
    Cancelled,
}

struct MountedSplit {
    page_id: SettingsPageId,
    source: SettingsPageSplitSource,
    visible_range: Range<usize>,
    demand_cursor: usize,
    pages: BTreeMap<usize, ResidentPage>,
    pending: HashMap<SettingsPageSplitRequestId, SettingsPageSplitPageRequest>,
    unavailable: BTreeMap<usize, (Range<usize>, SplitRangeState, Option<String>)>,
    focused_item_id: Option<SettingsPageSplitItemId>,
    focused_position: Option<usize>,
}

struct PendingFocusResolution {
    probe: SettingsPageSplitFocusProbe,
    prior_position: usize,
    request_id: Option<SettingsPageSplitRequestId>,
    superseded: bool,
}

impl MountedSplit {
    fn same_contract(&self, page_id: &SettingsPageId, source: &SettingsPageSplitSource) -> bool {
        &self.page_id == page_id
            && self.source.key() == source.key()
            && self.source.logical_item_count() == source.logical_item_count()
            && self.source.max_page_items() == source.max_page_items()
            && self.source.max_page_decoded_bytes() == source.max_page_decoded_bytes()
    }
}

pub(super) struct SplitPager {
    mounted: Option<MountedSplit>,
    work_receiver: SettingsPageSplitWorkReceiver,
    // Every nonzero id at or below this watermark was uniquely issued by this pager.
    issued_request_watermark: u64,
    stale_result_count: u64,
    pending_focus_resolution: Option<PendingFocusResolution>,
    pending_focus_reveal: Option<usize>,
}

impl SplitPager {
    pub(super) fn new(work_receiver: SettingsPageSplitWorkReceiver) -> Self {
        Self {
            mounted: None,
            work_receiver,
            issued_request_watermark: 0,
            stale_result_count: 0,
            pending_focus_resolution: None,
            pending_focus_reveal: None,
        }
    }

    pub(super) fn bind(&mut self, page_id: SettingsPageId, source: SettingsPageSplitSource) {
        if self
            .mounted
            .as_ref()
            .is_some_and(|mounted| mounted.same_contract(&page_id, &source))
        {
            if let Some(mounted) = self.mounted.as_mut() {
                mounted.source = source;
            }
            return;
        }

        let previous_focus = self.mounted.as_ref().and_then(|mounted| {
            (&mounted.page_id == &page_id
                && mounted.source.key().source_id() == source.key().source_id())
            .then(|| (mounted.focused_item_id.clone(), mounted.focused_position))
        });
        self.cancel_and_release_current();
        let (focused_item_id, focused_position) = previous_focus.unwrap_or((None, None));
        self.pending_focus_resolution =
            focused_item_id
                .clone()
                .zip(focused_position)
                .map(|(item_id, prior_position)| PendingFocusResolution {
                    probe: SettingsPageSplitFocusProbe::new(item_id),
                    prior_position,
                    request_id: None,
                    superseded: false,
                });
        self.mounted = Some(MountedSplit {
            page_id,
            source,
            visible_range: 0..0,
            demand_cursor: 0,
            pages: BTreeMap::new(),
            pending: HashMap::new(),
            unavailable: BTreeMap::new(),
            focused_item_id,
            focused_position,
        });
        if self.pending_focus_resolution.is_none() {
            self.clamp_focus();
        }
    }

    pub(super) fn clear(&mut self) {
        self.cancel_and_release_current();
        self.mounted = None;
    }

    fn cancel_and_release_current(&mut self) {
        let Some(mounted) = self.mounted.as_mut() else {
            return;
        };
        let pending = mounted
            .pending
            .drain()
            .map(|(id, request)| (id, request))
            .collect::<Vec<_>>();
        let pages = std::mem::take(&mut mounted.pages)
            .into_values()
            .map(|page| page.request)
            .collect::<Vec<_>>();
        mounted.unavailable.clear();
        for (_, request) in pending {
            self.work_receiver.cancel(request);
        }
        for request in pages {
            self.work_receiver.release(request);
        }
        self.pending_focus_resolution = None;
        self.pending_focus_reveal = None;
    }

    pub(super) fn ensure_demand(&mut self, visible_range: Range<usize>) {
        let (evicted, cancelled) = {
            let Some(mounted) = self.mounted.as_mut() else {
                return;
            };
            if mounted.visible_range != visible_range {
                mounted.demand_cursor = visible_range.start;
            }
            mounted.visible_range = visible_range.clone();
            let evicted_starts = mounted
                .pages
                .iter()
                .filter(|(_, page)| !ranges_intersect(page.request.range(), visible_range.clone()))
                .map(|(start, _)| *start)
                .collect::<Vec<_>>();
            let evicted = evicted_starts
                .into_iter()
                .filter_map(|start| mounted.pages.remove(&start).map(|page| page.request))
                .collect::<Vec<_>>();
            let cancelled_ids = mounted
                .pending
                .iter()
                .filter(|(_, request)| !ranges_intersect(request.range(), visible_range.clone()))
                .map(|(id, _)| *id)
                .collect::<Vec<_>>();
            let cancelled = cancelled_ids
                .into_iter()
                .filter_map(|id| mounted.pending.remove(&id).map(|request| (id, request)))
                .collect::<Vec<_>>();
            mounted
                .unavailable
                .retain(|_, (range, _, _)| ranges_intersect(range.clone(), visible_range.clone()));
            (evicted, cancelled)
        };
        for request in evicted {
            self.work_receiver.release(request);
        }
        for (id, request) in cancelled {
            if self
                .pending_focus_resolution
                .as_ref()
                .is_some_and(|pending| pending.request_id == Some(id))
            {
                if self
                    .pending_focus_resolution
                    .as_ref()
                    .is_some_and(|pending| pending.superseded)
                {
                    self.pending_focus_resolution = None;
                } else {
                    self.pending_focus_resolution
                        .as_mut()
                        .expect("pending focus checked")
                        .request_id = None;
                }
            }
            self.work_receiver.cancel(request);
        }

        if visible_range.is_empty() {
            return;
        }
        let Some(mounted) = self.mounted.as_ref() else {
            return;
        };
        let page_items = mounted.source.max_page_items();
        let total = mounted.source.logical_item_count();
        let demand_start = visible_range.start.min(total);
        let demand_end = visible_range.end.min(total);
        if demand_start >= demand_end {
            return;
        }
        let mut start = mounted.demand_cursor;
        if start < demand_start || start >= demand_end {
            start = demand_start;
        }
        let mut remaining = demand_end - demand_start;
        while remaining > 0 {
            if start >= demand_end {
                start = demand_start;
            }
            let mounted = self.mounted.as_ref().expect("mounted split checked");
            let covered_end = mounted
                .pages
                .values()
                .map(|page| page.request.range())
                .chain(mounted.pending.values().map(|request| request.range()))
                .chain(
                    mounted
                        .unavailable
                        .values()
                        .map(|(range, _, _)| range.clone()),
                )
                .find(|range| range.contains(&start))
                .map(|range| range.end.min(demand_end));
            if let Some(end) = covered_end {
                let advance = (end - start).min(remaining);
                remaining -= advance;
                start = end;
                self.mounted
                    .as_mut()
                    .expect("mounted split checked")
                    .demand_cursor = if start >= demand_end {
                    demand_start
                } else {
                    start
                };
                continue;
            }
            let end = demand_end.min(start.saturating_add(page_items));
            if mounted.pages.len() + mounted.pending.len() >= MAX_PAGE_SPLIT_ACTIVE_PAGES {
                if !mounted.pending.is_empty() || !self.work_receiver.can_publish_page() {
                    break;
                }
                let oldest_start = mounted
                    .pages
                    .iter()
                    .min_by_key(|(_, page)| page.request.request_id())
                    .map(|(page_start, _)| *page_start);
                let Some(oldest_start) = oldest_start else {
                    break;
                };
                let released = self
                    .mounted
                    .as_mut()
                    .expect("mounted split checked")
                    .pages
                    .remove(&oldest_start)
                    .expect("oldest resident page checked")
                    .request;
                self.work_receiver.release(released);
                continue;
            }
            if !self.work_receiver.can_publish_page() {
                break;
            }
            let Some(next_request_id) = self.issued_request_watermark.checked_add(1) else {
                break;
            };
            let mut request = SettingsPageSplitPageRequest::new(
                mounted.page_id.clone(),
                mounted.source.key().clone(),
                SettingsPageSplitRequestId::new(next_request_id),
                start..end,
            );
            if let Some(pending_focus) = self
                .pending_focus_resolution
                .as_mut()
                .filter(|pending| pending.request_id.is_none())
            {
                request = request.with_focus_probe(pending_focus.probe.clone());
                pending_focus.request_id = Some(request.request_id());
            }
            let request_id = request.request_id();
            self.mounted
                .as_mut()
                .expect("mounted split checked")
                .pending
                .insert(request_id, request.clone());
            if !self.work_receiver.publish_page(request) {
                self.mounted
                    .as_mut()
                    .expect("mounted split checked")
                    .pending
                    .remove(&request_id);
                if self
                    .pending_focus_resolution
                    .as_ref()
                    .is_some_and(|pending| pending.request_id == Some(request_id))
                {
                    if self
                        .pending_focus_resolution
                        .as_ref()
                        .is_some_and(|pending| pending.superseded)
                    {
                        self.pending_focus_resolution = None;
                    } else {
                        self.pending_focus_resolution
                            .as_mut()
                            .expect("pending focus checked")
                            .request_id = None;
                    }
                }
                break;
            }
            self.issued_request_watermark = next_request_id;
            let advance = (end - start).min(remaining);
            remaining -= advance;
            start = end;
            self.mounted
                .as_mut()
                .expect("mounted split checked")
                .demand_cursor = if start >= demand_end {
                demand_start
            } else {
                start
            };
        }
    }

    fn was_issued(&self, request_id: SettingsPageSplitRequestId) -> bool {
        request_id.get() != 0 && request_id.get() <= self.issued_request_watermark
    }

    pub(super) fn logical_item_count(&self) -> usize {
        self.mounted
            .as_ref()
            .map_or(0, |mounted| mounted.source.logical_item_count())
    }

    pub(super) fn item_at(&self, logical_position: usize) -> Option<&SettingsPageSplitItem> {
        self.mounted.as_ref()?.pages.values().find_map(|page| {
            let range = page.request.range();
            range
                .contains(&logical_position)
                .then(|| page.items.get(logical_position - range.start))
                .flatten()
        })
    }

    pub(super) fn contains_item(&self, item_id: &SettingsPageSplitItemId) -> bool {
        self.mounted.as_ref().is_some_and(|mounted| {
            mounted
                .pages
                .values()
                .flat_map(|page| page.items.iter())
                .any(|item| item.item_id() == item_id)
        })
    }

    pub(super) fn matches_current_item(
        &self,
        page_id: &SettingsPageId,
        source_key: &crate::SettingsPageSplitSourceKey,
        logical_position: usize,
        item_id: &SettingsPageSplitItemId,
    ) -> bool {
        self.mounted.as_ref().is_some_and(|mounted| {
            &mounted.page_id == page_id
                && mounted.source.key() == source_key
                && mounted.pages.values().any(|page| {
                    let range = page.request.range();
                    range.contains(&logical_position)
                        && page.items[logical_position - range.start].item_id() == item_id
                })
        })
    }

    pub(super) fn range_state(
        &self,
        logical_position: usize,
    ) -> Option<(SplitRangeState, Option<&str>)> {
        let mounted = self.mounted.as_ref()?;
        if mounted
            .pending
            .values()
            .any(|request| request.range().contains(&logical_position))
        {
            return Some((SplitRangeState::Pending, None));
        }
        mounted
            .unavailable
            .values()
            .find_map(|(range, state, message)| {
                range
                    .contains(&logical_position)
                    .then_some((*state, message.as_deref()))
            })
    }

    pub(super) fn is_selected(&self, item: &SettingsPageSplitItem) -> bool {
        self.mounted
            .as_ref()
            .and_then(|mounted| mounted.source.selected())
            .is_some_and(|selected| {
                selected.item_id() == item.item_id()
                    && selected.logical_position() == item.logical_position()
            })
    }

    pub(super) fn is_focused(&self, item: &SettingsPageSplitItem) -> bool {
        self.mounted.as_ref().is_some_and(|mounted| {
            mounted.focused_position == Some(item.logical_position())
                && mounted
                    .focused_item_id
                    .as_ref()
                    .is_none_or(|id| id == item.item_id())
        })
    }

    pub(super) fn focus_position(&mut self, logical_position: usize) -> bool {
        let Some(mounted) = self.mounted.as_ref() else {
            return false;
        };
        if logical_position >= mounted.source.logical_item_count() {
            return false;
        }
        if self
            .pending_focus_resolution
            .as_ref()
            .is_some_and(|pending| pending.request_id.is_none())
        {
            self.pending_focus_resolution = None;
        } else if let Some(pending) = self.pending_focus_resolution.as_mut() {
            pending.superseded = true;
        }
        let mounted = self.mounted.as_mut().expect("mounted split checked");
        mounted.focused_position = Some(logical_position);
        mounted.focused_item_id = mounted.pages.values().find_map(|page| {
            let range = page.request.range();
            range
                .contains(&logical_position)
                .then(|| page.items.get(logical_position - range.start))
                .flatten()
                .map(|item| item.item_id().clone())
        });
        true
    }

    pub(super) fn move_focus(&mut self, delta: isize) -> Option<usize> {
        let mounted = self.mounted.as_ref()?;
        let count = mounted.source.logical_item_count();
        if count == 0 {
            return None;
        }
        let current = mounted
            .focused_position
            .or_else(|| {
                mounted
                    .source
                    .selected()
                    .map(|selected| selected.logical_position())
            })
            .unwrap_or(0);
        let next = current.saturating_add_signed(delta).min(count - 1);
        self.focus_position(next);
        Some(next)
    }

    pub(super) fn focused_position(&self) -> Option<usize> {
        self.mounted
            .as_ref()
            .and_then(|mounted| mounted.focused_position)
    }

    pub(super) fn resident_page_count(&self) -> usize {
        self.mounted
            .as_ref()
            .map_or(0, |mounted| mounted.pages.len())
    }

    pub(super) fn resident_item_count(&self) -> usize {
        self.mounted.as_ref().map_or(0, |mounted| {
            mounted.pages.values().map(|page| page.items.len()).sum()
        })
    }

    pub(super) fn pending_request_count(&self) -> usize {
        self.mounted
            .as_ref()
            .map_or(0, |mounted| mounted.pending.len())
    }

    pub(super) fn stale_result_count(&self) -> u64 {
        self.stale_result_count
    }

    fn clamp_focus(&mut self) {
        let Some(mounted) = self.mounted.as_mut() else {
            return;
        };
        let count = mounted.source.logical_item_count();
        if count == 0 {
            mounted.focused_item_id = None;
            mounted.focused_position = None;
        } else if let Some(position) = mounted.focused_position {
            mounted.focused_position = Some(position.min(count - 1));
        }
    }
}

fn ranges_intersect(left: Range<usize>, right: Range<usize>) -> bool {
    left.start < right.end && right.start < left.end
}
