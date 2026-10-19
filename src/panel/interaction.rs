use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum ActionFocusKey {
    Row(SettingsFieldId, SettingsRowActionId),
    Page(SettingsPageId, SettingsPageActionId),
}

impl ActionFocusKey {
    pub(super) fn for_event(event: &SettingsWindowEvent) -> Option<Self> {
        match event {
            SettingsWindowEvent::RowActionRequested {
                field_id,
                action_id,
            } => Some(Self::Row(field_id.clone(), action_id.clone())),
            SettingsWindowEvent::PageActionRequested { page_id, action_id } => {
                Some(Self::Page(page_id.clone(), action_id.clone()))
            }
            _ => None,
        }
    }
}

impl SettingsPanel {
    pub(super) fn sync_action_focus(&mut self, cx: &mut Context<Self>) {
        let mut previous = std::mem::take(&mut self.action_focus);
        let mut next = HashMap::new();
        let page = self.model.selected_page();
        for action in page.actions() {
            let key = ActionFocusKey::Page(page.page_id().clone(), action.action_id().clone());
            let focus = previous.remove(&key).unwrap_or_else(|| cx.focus_handle());
            next.insert(key, focus);
        }
        for row in self.model.selected_rows() {
            for action in row.actions() {
                let key = ActionFocusKey::Row(row.field_id().clone(), action.action_id().clone());
                let focus = previous.remove(&key).unwrap_or_else(|| cx.focus_handle());
                next.insert(key, focus);
            }
        }
        self.action_focus = next;
    }
    pub(super) fn command_disabled_reason(&self, event: &SettingsWindowEvent) -> Option<String> {
        if let Some(reason) = self.model.interaction_gate().reason() {
            return Some(reason.to_owned());
        }
        let availability = match event {
            SettingsWindowEvent::AcceptRequested => Some(
                self.model
                    .footer_availability(SettingsFooterCommand::Accept),
            ),
            SettingsWindowEvent::ApplyRequested => {
                Some(self.model.footer_availability(SettingsFooterCommand::Apply))
            }
            SettingsWindowEvent::CancelRequested => Some(
                self.model
                    .footer_availability(SettingsFooterCommand::Cancel),
            ),
            SettingsWindowEvent::RowActionRequested {
                field_id,
                action_id,
            } => self
                .model
                .row(field_id)
                .and_then(|row| {
                    row.actions()
                        .iter()
                        .find(|action| action.action_id() == action_id)
                })
                .map(|action| action.availability()),
            SettingsWindowEvent::PageActionRequested { page_id, action_id } => self
                .model
                .page(page_id)
                .and_then(|page| {
                    page.actions()
                        .iter()
                        .find(|action| action.action_id() == action_id)
                })
                .map(|action| action.availability()),
            SettingsWindowEvent::ColorPickerRequested { field_id } => {
                return self
                    .model
                    .field_availability(field_id)
                    .and_then(|availability| availability.reason())
                    .map(str::to_owned);
            }
            _ => None,
        };
        availability
            .filter(|availability| !availability.is_enabled())
            .map(|availability| {
                availability
                    .disabled_reason()
                    .unwrap_or("Unavailable")
                    .to_owned()
            })
    }
    pub(super) fn interaction_is_open(&self) -> bool {
        self.model.interaction_gate().is_open()
    }

    pub(super) fn field_is_current(&self, field_id: &SettingsFieldId) -> bool {
        self.model.selected_rows().iter().any(|row| {
            (row.is_field() && row.field_id() == field_id)
                || row
                    .detail_field()
                    .is_some_and(|field| field.field_id() == field_id)
        })
    }

    pub(super) fn field_is_mutable(&self, field_id: &SettingsFieldId) -> bool {
        self.interaction_is_open()
            && self.field_is_current(field_id)
            && self
                .model
                .field_availability(field_id)
                .is_some_and(|availability| availability.is_available())
    }

    pub(super) fn input_is_current(&self, input: &Entity<SettingsFieldInput>, cx: &App) -> bool {
        let field_id = input.read(cx).field_id();
        self.field_is_current(field_id)
            && (self
                .fields
                .get(field_id)
                .is_some_and(|current| current == input)
                || (self.color_picker_field.as_ref() == Some(field_id)
                    && self.color_picker_input.as_ref() == Some(input)))
    }

    pub(super) fn event_is_admitted(&self, event: &SettingsWindowEvent) -> bool {
        if !self.interaction_is_open() {
            return false;
        }
        match event {
            SettingsWindowEvent::AcceptRequested => self
                .model
                .footer_availability(SettingsFooterCommand::Accept)
                .is_enabled(),
            SettingsWindowEvent::ApplyRequested => self
                .model
                .footer_availability(SettingsFooterCommand::Apply)
                .is_enabled(),
            SettingsWindowEvent::CancelRequested => self
                .model
                .footer_availability(SettingsFooterCommand::Cancel)
                .is_enabled(),
            SettingsWindowEvent::CloseRequested => true,
            SettingsWindowEvent::FieldChanged { field_id, value } => {
                self.field_is_mutable(field_id)
                    && self.model.validate_field_value(field_id, value).is_ok()
            }
            SettingsWindowEvent::FieldInputRefused { field_id, .. } => {
                self.field_is_mutable(field_id)
            }
            SettingsWindowEvent::ColorPickerRequested { field_id } => {
                self.field_is_mutable(field_id)
                    && self.model.field_kind(field_id) == Some(SettingsFieldKind::Color)
            }
            SettingsWindowEvent::SectionSelected { section_id } => self
                .model
                .sections()
                .iter()
                .any(|section| section.section_id() == section_id),
            SettingsWindowEvent::PageNavigationRequested { page_id } => {
                self.model.page(page_id).is_some()
            }
            SettingsWindowEvent::RowActionRequested {
                field_id,
                action_id,
            } => {
                self.model
                    .selected_rows()
                    .iter()
                    .any(|row| row.field_id() == field_id)
                    && self.row_has_action(field_id, action_id)
            }
            SettingsWindowEvent::PageActionRequested { page_id, action_id } => {
                self.model.selected_page_id() == page_id
                    && self.page_has_enabled_action(page_id, action_id)
            }
            SettingsWindowEvent::PageSplitItemSelected { page_id, item_id } => {
                self.page_has_split_item(page_id, item_id)
            }
        }
    }

    pub(super) fn request_event(
        &mut self,
        event: SettingsWindowEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.event_is_admitted(&event) {
            return false;
        }
        cx.emit(event);
        true
    }

    pub(super) fn value_is_admitted(
        &mut self,
        field_id: &SettingsFieldId,
        value: &str,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.field_is_mutable(field_id) {
            return false;
        }
        match self.model.validate_field_value(field_id, value) {
            Ok(()) => true,
            Err(crate::SettingsWindowError::FieldValueLimitExceeded {
                limit, value_bytes, ..
            }) => {
                self.request_event(
                    SettingsWindowEvent::FieldInputRefused {
                        field_id: field_id.clone(),
                        reason: crate::SettingsFieldInputRefusal::ValueByteLimitExceeded {
                            limit,
                            attempted_bytes: value_bytes,
                        },
                    },
                    cx,
                );
                false
            }
            Err(_) => false,
        }
    }

    pub(super) fn picker_value_is_mutable(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(field_id) = self.color_picker_field.clone() else {
            return false;
        };
        self.value_is_admitted(&field_id, "#000000", cx)
    }

    pub(super) fn sync_picker_admission(&self, cx: &mut Context<Self>) {
        let Some(field_id) = self.color_picker_field.as_ref() else {
            return;
        };
        let read_only = !self.field_is_mutable(field_id);
        if let Some(input) = &self.color_picker_input {
            input.update(cx, |input, cx| {
                input.sync_admission(
                    read_only,
                    self.model.field_validity(field_id).unwrap_or_default(),
                    self.model.field_value_byte_limit(field_id),
                    cx,
                )
            });
        }
        let canonical_fits = self
            .model
            .field_value_byte_limit(field_id)
            .is_none_or(|limit| limit >= 7);
        for slot in &self.color_picker_channel_inputs {
            slot.input.update(cx, |input, cx| {
                input.set_read_only(read_only || !canonical_fits, cx)
            });
        }
    }
}
