use std::collections::HashSet;

use super::*;

impl SplitPager {
    pub(in crate::panel) fn deliver(
        &mut self,
        result: SettingsPageSplitPageResult,
    ) -> Result<SettingsPageSplitDelivery, SettingsPageSplitDeliveryError> {
        let Some(mounted) = self.mounted.as_ref() else {
            return if self.was_issued(result.request().request_id()) {
                Ok(self.record_obsolete_delivery())
            } else {
                Err(SettingsPageSplitDeliveryError::MismatchedRequestId)
            };
        };
        let request = result.request();
        let Some(expected) = mounted.pending.get(&request.request_id()) else {
            if self.was_issued(request.request_id()) {
                return Ok(self.record_obsolete_delivery());
            }
            return Err(SettingsPageSplitDeliveryError::MismatchedRequestId);
        };
        if request.page_id() != expected.page_id() {
            return Err(SettingsPageSplitDeliveryError::MismatchedPage);
        }
        if request.source_key().source_id() != expected.source_key().source_id() {
            return Err(SettingsPageSplitDeliveryError::MismatchedSourceIdentity);
        }
        if request.source_key().generation() != expected.source_key().generation() {
            return Err(SettingsPageSplitDeliveryError::MismatchedGeneration);
        }
        if request.source_key().revision() != expected.source_key().revision() {
            return Err(SettingsPageSplitDeliveryError::MismatchedRevision);
        }
        if request.range() != expected.range() {
            return Err(SettingsPageSplitDeliveryError::MismatchedRange);
        }
        if request.focus_probe() != expected.focus_probe() {
            return Err(SettingsPageSplitDeliveryError::MismatchedFocusProbe);
        }
        if result.logical_item_count() != mounted.source.logical_item_count() {
            return Err(SettingsPageSplitDeliveryError::MismatchedLogicalItemCount);
        }
        self.validate_focus_resolution(&result)?;

        match result.outcome() {
            SettingsPageSplitPageOutcome::Ready(items) => self.deliver_ready(
                expected.clone(),
                items.clone(),
                result.focus_resolution().cloned(),
            ),
            SettingsPageSplitPageOutcome::Failed(failure) => {
                if failure.message().len() > MAX_FAILURE_MESSAGE_BYTES {
                    return Err(SettingsPageSplitDeliveryError::FailureMessageTooLarge);
                }
                let mounted = self.mounted.as_mut().expect("mounted split checked");
                mounted.pending.remove(&request.request_id());
                mounted.unavailable.insert(
                    request.range().start,
                    (
                        request.range(),
                        SplitRangeState::Failed,
                        Some(failure.message().to_owned()),
                    ),
                );
                self.finish_request(request.request_id(), result.focus_resolution().cloned());
                Ok(SettingsPageSplitDelivery::Failed)
            }
            SettingsPageSplitPageOutcome::Cancelled => {
                let mounted = self.mounted.as_mut().expect("mounted split checked");
                mounted.pending.remove(&request.request_id());
                mounted.unavailable.insert(
                    request.range().start,
                    (request.range(), SplitRangeState::Cancelled, None),
                );
                self.finish_request(request.request_id(), result.focus_resolution().cloned());
                Ok(SettingsPageSplitDelivery::Cancelled)
            }
        }
    }

    fn deliver_ready(
        &mut self,
        request: SettingsPageSplitPageRequest,
        items: Vec<SettingsPageSplitItem>,
        focus_resolution: Option<SettingsPageSplitFocusResolution>,
    ) -> Result<SettingsPageSplitDelivery, SettingsPageSplitDeliveryError> {
        let mounted = self
            .mounted
            .as_ref()
            .expect("ready request has mounted split");
        let range = request.range();
        if items.len() > mounted.source.max_page_items() {
            return Err(SettingsPageSplitDeliveryError::TooManyItems);
        }
        if items.len() != range.len() {
            return Err(SettingsPageSplitDeliveryError::IncompleteRange);
        }
        if items
            .iter()
            .enumerate()
            .any(|(offset, item)| item.logical_position() != range.start + offset)
        {
            return Err(SettingsPageSplitDeliveryError::MalformedLogicalPositions);
        }
        if items.iter().any(|item| item.item_id().as_str().is_empty()) {
            return Err(SettingsPageSplitDeliveryError::EmptyItemId);
        }
        let mut ids = HashSet::new();
        if items.iter().any(|item| !ids.insert(item.item_id())) {
            return Err(SettingsPageSplitDeliveryError::DuplicateItemId);
        }
        if mounted.pages.values().any(|page| {
            page.items.iter().any(|resident| {
                items
                    .iter()
                    .any(|item| item.item_id() == resident.item_id())
            })
        }) {
            return Err(SettingsPageSplitDeliveryError::DuplicateItemId);
        }
        let decoded_bytes = items.iter().fold(0usize, |total, item| {
            total
                .saturating_add(item.item_id().as_str().len())
                .saturating_add(item.label().len())
                .saturating_add(item.subtext().map_or(0, str::len))
                .saturating_add(
                    item.preview_style()
                        .and_then(|style| style.font_family())
                        .map_or(0, str::len),
                )
        });
        if decoded_bytes > mounted.source.max_page_decoded_bytes() {
            return Err(SettingsPageSplitDeliveryError::TooManyDecodedBytes);
        }

        if let (Some(probe), Some(resolution)) = (request.focus_probe(), focus_resolution.as_ref())
        {
            match resolution {
                SettingsPageSplitFocusResolution::Found(position) => {
                    if range.contains(position)
                        && items[position - range.start].item_id() != probe.item_id()
                    {
                        return Err(SettingsPageSplitDeliveryError::InvalidFocusResolution);
                    }
                    if items.iter().any(|item| {
                        item.item_id() == probe.item_id() && item.logical_position() != *position
                    }) {
                        return Err(SettingsPageSplitDeliveryError::InvalidFocusResolution);
                    }
                }
                SettingsPageSplitFocusResolution::Removed => {
                    if items.iter().any(|item| item.item_id() == probe.item_id()) {
                        return Err(SettingsPageSplitDeliveryError::InvalidFocusResolution);
                    }
                }
            }
        }
        if self.pending_focus_resolution.is_none()
            && let (Some(position), Some(item_id)) =
                (mounted.focused_position, mounted.focused_item_id.as_ref())
            && range.contains(&position)
            && items[position - range.start].item_id() != item_id
        {
            return Err(SettingsPageSplitDeliveryError::InvalidFocusResolution);
        }

        let mounted = self.mounted.as_mut().expect("mounted split checked");
        mounted.pending.remove(&request.request_id());
        mounted.unavailable.remove(&range.start);
        mounted.pages.insert(
            range.start,
            ResidentPage {
                request: request.clone(),
                items,
            },
        );
        self.finish_request(request.request_id(), focus_resolution);
        self.adopt_realized_focus_identity();
        Ok(SettingsPageSplitDelivery::Ready)
    }

    fn validate_focus_resolution(
        &self,
        result: &SettingsPageSplitPageResult,
    ) -> Result<(), SettingsPageSplitDeliveryError> {
        let request = result.request();
        match (request.focus_probe(), result.focus_resolution()) {
            (Some(_), None) => Err(SettingsPageSplitDeliveryError::MissingFocusResolution),
            (None, Some(_)) => Err(SettingsPageSplitDeliveryError::UnexpectedFocusResolution),
            (None, None) => Ok(()),
            (Some(probe), Some(resolution)) => {
                let Some(pending) = self.pending_focus_resolution.as_ref() else {
                    return Err(SettingsPageSplitDeliveryError::InvalidFocusResolution);
                };
                if pending.request_id != Some(request.request_id()) || &pending.probe != probe {
                    return Err(SettingsPageSplitDeliveryError::InvalidFocusResolution);
                }
                if let SettingsPageSplitFocusResolution::Found(position) = resolution {
                    let mounted = self.mounted.as_ref().expect("mounted split checked");
                    if *position >= mounted.source.logical_item_count() {
                        return Err(SettingsPageSplitDeliveryError::InvalidFocusResolution);
                    }
                    if let Some(item) = mounted.pages.values().find_map(|page| {
                        let range = page.request.range();
                        range
                            .contains(position)
                            .then(|| page.items.get(*position - range.start))
                            .flatten()
                    }) && item.item_id() != probe.item_id()
                    {
                        return Err(SettingsPageSplitDeliveryError::InvalidFocusResolution);
                    }
                    if mounted.pages.values().any(|page| {
                        page.items.iter().any(|item| {
                            item.item_id() == probe.item_id()
                                && item.logical_position() != *position
                        })
                    }) {
                        return Err(SettingsPageSplitDeliveryError::InvalidFocusResolution);
                    }
                } else if self
                    .mounted
                    .as_ref()
                    .expect("mounted split checked")
                    .pages
                    .values()
                    .any(|page| {
                        page.items
                            .iter()
                            .any(|item| item.item_id() == probe.item_id())
                    })
                {
                    return Err(SettingsPageSplitDeliveryError::InvalidFocusResolution);
                }
                Ok(())
            }
        }
    }

    fn finish_request(
        &mut self,
        request_id: SettingsPageSplitRequestId,
        focus_resolution: Option<SettingsPageSplitFocusResolution>,
    ) {
        self.work_receiver.settle(request_id);
        let Some(resolution) = focus_resolution else {
            return;
        };
        let pending = self
            .pending_focus_resolution
            .take()
            .expect("validated focus resolution has pending state");
        if pending.superseded {
            return;
        }
        let mounted = self.mounted.as_mut().expect("mounted split checked");
        let reveal = match resolution {
            SettingsPageSplitFocusResolution::Found(position) => {
                mounted.focused_item_id = Some(pending.probe.item_id().clone());
                mounted.focused_position = Some(position);
                Some(position)
            }
            SettingsPageSplitFocusResolution::Removed => {
                mounted.focused_item_id = None;
                if mounted.source.logical_item_count() == 0 {
                    mounted.focused_position = None;
                    None
                } else {
                    let position = pending
                        .prior_position
                        .min(mounted.source.logical_item_count() - 1);
                    mounted.focused_position = Some(position);
                    Some(position)
                }
            }
        };
        self.pending_focus_reveal = reveal;
    }

    fn adopt_realized_focus_identity(&mut self) {
        if self.pending_focus_resolution.is_some() {
            return;
        }
        let Some(mounted) = self.mounted.as_mut() else {
            return;
        };
        if mounted.focused_item_id.is_some() {
            return;
        }
        let Some(position) = mounted.focused_position else {
            return;
        };
        mounted.focused_item_id = mounted.pages.values().find_map(|page| {
            let range = page.request.range();
            range
                .contains(&position)
                .then(|| page.items.get(position - range.start))
                .flatten()
                .map(|item| item.item_id().clone())
        });
    }

    pub(in crate::panel) fn take_focus_reveal(&mut self) -> Option<usize> {
        self.pending_focus_reveal.take()
    }

    fn record_obsolete_delivery(&mut self) -> SettingsPageSplitDelivery {
        self.stale_result_count = self.stale_result_count.saturating_add(1);
        SettingsPageSplitDelivery::Obsolete
    }
}
