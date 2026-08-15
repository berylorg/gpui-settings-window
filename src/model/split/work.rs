use std::{
    cell::RefCell,
    collections::{HashSet, VecDeque},
    rc::Rc,
};

use super::{SettingsPageSplitPageRequest, SettingsPageSplitRequestId};

/// Typed host work emitted by the split pager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsPageSplitWork {
    /// Fetch the exact bounded range and publish one matching terminal result.
    Page(SettingsPageSplitPageRequest),
    /// Stop the exact exposed host fetch; no later result can become visible.
    Cancel(SettingsPageSplitPageRequest),
    /// Discard host data retained for this previously published exact page.
    Release(SettingsPageSplitPageRequest),
}

/// Maximum number of split pages or requests retained by one mounted panel.
pub const MAX_PAGE_SPLIT_ACTIVE_PAGES: usize = 16;
/// Maximum undrained host work retained by one split work receiver.
pub const MAX_PAGE_SPLIT_WORK_ITEMS: usize = 64;

#[derive(Default)]
struct SplitWorkQueue {
    work: VecDeque<SettingsPageSplitWork>,
    dispatched_pages: HashSet<SettingsPageSplitRequestId>,
}

/// Clone-stable bounded receiver for page, cancellation, and release work.
///
/// The receiver outlives the GPUI panel when retained by the host. Hosts must
/// drain it promptly, fetch only exact `Page` ranges, publish one exact result,
/// and honor every `Cancel` and `Release`. Admission pauses before terminal
/// lifecycle work could exceed the fixed receiver capacity.
#[derive(Clone, Default)]
pub struct SettingsPageSplitWorkReceiver {
    inner: Rc<RefCell<SplitWorkQueue>>,
}

impl std::fmt::Debug for SettingsPageSplitWorkReceiver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SettingsPageSplitWorkReceiver")
            .field("pending_work_count", &self.pending_work_count())
            .finish_non_exhaustive()
    }
}

impl PartialEq for SettingsPageSplitWorkReceiver {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for SettingsPageSplitWorkReceiver {}

impl SettingsPageSplitWorkReceiver {
    /// Creates an empty bounded receiver for one settings panel.
    pub fn new() -> Self {
        Self::default()
    }

    /// Removes the next exact host work item, marking `Page` work exposed.
    ///
    /// An exposed `Page` is followed by exactly one accepted result or by `Cancel`. A ready page
    /// that later leaves residency is followed by `Release`. Retain a clone of this receiver so
    /// terminal teardown work can be drained after the GPUI entity disappears.
    pub fn take_work(&self) -> Option<SettingsPageSplitWork> {
        let mut inner = self.inner.borrow_mut();
        let work = inner.work.pop_front()?;
        if let SettingsPageSplitWork::Page(request) = &work {
            inner.dispatched_pages.insert(request.request_id());
        }
        Some(work)
    }

    /// Returns the bounded count of undrained work items.
    pub fn pending_work_count(&self) -> usize {
        self.inner.borrow().work.len()
    }

    /// Returns the fixed undrained-work capacity.
    pub const fn capacity(&self) -> usize {
        MAX_PAGE_SPLIT_WORK_ITEMS
    }

    pub(crate) fn can_publish_page(&self) -> bool {
        self.pending_work_count()
            < MAX_PAGE_SPLIT_WORK_ITEMS.saturating_sub(MAX_PAGE_SPLIT_ACTIVE_PAGES * 2)
    }

    pub(crate) fn publish_page(&self, request: SettingsPageSplitPageRequest) -> bool {
        if !self.can_publish_page() {
            return false;
        }
        let mut inner = self.inner.borrow_mut();
        if inner.work.iter().any(|work| {
            matches!(work, SettingsPageSplitWork::Page(queued) if queued.request_id() == request.request_id())
        }) {
            return true;
        }
        inner.work.push_back(SettingsPageSplitWork::Page(request));
        true
    }

    pub(crate) fn cancel(&self, request: SettingsPageSplitPageRequest) {
        let mut inner = self.inner.borrow_mut();
        if inner.dispatched_pages.remove(&request.request_id()) {
            push_terminal(&mut inner.work, SettingsPageSplitWork::Cancel(request));
        } else {
            inner.work.retain(|work| {
                !matches!(work, SettingsPageSplitWork::Page(queued) if queued.request_id() == request.request_id())
            });
        }
    }

    pub(crate) fn release(&self, request: SettingsPageSplitPageRequest) {
        let mut inner = self.inner.borrow_mut();
        inner.dispatched_pages.remove(&request.request_id());
        push_terminal(&mut inner.work, SettingsPageSplitWork::Release(request));
    }

    pub(crate) fn settle(&self, id: SettingsPageSplitRequestId) {
        self.inner.borrow_mut().dispatched_pages.remove(&id);
    }
}

fn push_terminal(queue: &mut VecDeque<SettingsPageSplitWork>, work: SettingsPageSplitWork) {
    let duplicate = queue.iter().any(|queued| match (queued, &work) {
        (SettingsPageSplitWork::Cancel(left), SettingsPageSplitWork::Cancel(right))
        | (SettingsPageSplitWork::Release(left), SettingsPageSplitWork::Release(right)) => {
            left.request_id() == right.request_id()
        }
        _ => false,
    });
    if duplicate {
        return;
    }
    assert!(
        queue.len() < MAX_PAGE_SPLIT_WORK_ITEMS,
        "split work receiver capacity invariant"
    );
    queue.push_back(work);
}
