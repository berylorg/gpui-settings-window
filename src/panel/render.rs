use super::*;
use gpui::{
    AnchoredPositionMode, AnyElement, Corner, MouseDownEvent, StatefulInteractiveElement, anchored,
    deferred,
};
use gpui_scrollbar::{
    Axis, ScrollbarStyle, ScrollbarVisibilityPolicy, render_scroll_handle_scrollbar,
};
use std::ops::Range;

const NAVIGATION_CHEVRON: &str = "▸";
const TEXT_FIELD_CONTROL_WIDTH: f32 = 208.0;
const NUMERIC_FIELD_CONTROL_WIDTH: f32 = 96.0;
const MULTILINE_FIELD_CONTROL_WIDTH: f32 = 300.0;
const COLOR_FIELD_CONTROL_WIDTH: f32 = 132.0;
const CHOICE_FIELD_CONTROL_WIDTH: f32 = 184.0;
const ROW_LABEL_MIN_WIDTH: f32 = 160.0;
const SPLIT_DETAIL_ROW_LABEL_MIN_WIDTH: f32 = 120.0;
const ROW_CONTROL_GUTTER_WIDTH: f32 = 24.0;
const ROW_ACTION_CLUSTER_MIN_WIDTH: f32 = 72.0;
const PAGE_LOCAL_SPLIT_LIST_WIDTH: f32 = 112.0;
const CHOICE_DROPDOWN_TRIANGLE: &str = "\u{25BE}";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DetailRowsLayout {
    Standard,
    SplitDetail,
}

impl Render for SettingsPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let lookup_counts_before = self.diagnostic_color_lookup_counts();
        let render_started = std::time::Instant::now();
        let element = div()
            .id("settings-panel")
            .size_full()
            .bg(theme_color(self.visual_theme.window_background))
            .p_4()
            .overflow_hidden()
            .child(
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .rounded_md()
                    .border_1()
                    .border_color(theme_color(self.visual_theme.panel.border))
                    .bg(theme_color(self.visual_theme.panel.background))
                    .p_4()
                    .text_color(theme_color(self.visual_theme.panel.foreground))
                    .text_size(px(self.font_size))
                    .track_focus(&self.focus_handle)
                    .key_context(PANEL_KEY_CONTEXT)
                    .on_action(cx.listener(Self::cancel))
                    .on_action(cx.listener(Self::focus_next))
                    .on_action(cx.listener(Self::focus_prev))
                    .child(
                        div()
                            .text_size(px(18.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Settings"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_h(px(0.0))
                            .overflow_hidden()
                            .flex()
                            .gap_4()
                            .child(self.render_section_navigation(cx))
                            .child(self.render_selected_section(cx)),
                    )
                    .child(self.render_buttons(cx)),
            );
        self.record_render_diagnostics(render_started.elapsed(), lookup_counts_before);
        element
    }
}

impl SettingsPanel {
    fn render_section_navigation(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        div()
            .id("settings-sections")
            .w(px(NAVIGATION_WIDTH))
            .h_full()
            .min_h(px(0.0))
            .flex_none()
            .relative()
            .overflow_hidden()
            .on_mouse_move(cx.listener(Self::note_navigation_scrollbar_motion))
            .on_scroll_wheel(cx.listener(Self::note_navigation_scrollbar_scroll))
            .child(
                div()
                    .id("settings-sections-scroll")
                    .overflow_y_scroll()
                    .overflow_x_hidden()
                    .w_full()
                    .h_full()
                    .track_scroll(&self.navigation_scroll_handle)
                    .child(div().w_full().flex().flex_col().gap_1().children(
                        self.model.sections().iter().map(|section| {
                            self.render_section_button(
                                section.section_id().clone(),
                                section.label().to_owned(),
                                cx,
                            )
                        }),
                    )),
            )
            .children(self.render_vertical_scrollbar(
                "settings-sections-scrollbar",
                &self.navigation_scroll_handle,
                self.navigation_scrollbar_visibility_policy(cx.entity()),
            ))
    }

    fn render_section_button(
        &self,
        section_id: SettingsSectionId,
        label: String,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let active = section_id == *self.model.selected_section_id();
        let button_theme = self.visual_theme.navigation_button.clone();
        let state = if active {
            button_theme.active.clone()
        } else {
            button_theme.normal.clone()
        };
        let font_weight = FontWeight(button_theme.font_weight as f32);
        let hover = button_theme.hover;
        let pressed = button_theme.active;

        div()
            .id(SharedString::from(format!(
                "settings-section-{}",
                element_id_suffix(section_id.as_str())
            )))
            .flex_none()
            .w_full()
            .px_3()
            .py_2()
            .rounded_sm()
            .border_1()
            .border_color(theme_color(state.border))
            .bg(theme_color(state.background))
            .hover(move |style| {
                style
                    .border_color(theme_color(hover.border))
                    .bg(theme_color(hover.background))
                    .text_color(theme_color(hover.foreground))
            })
            .active(move |style| {
                style
                    .border_color(theme_color(pressed.border))
                    .bg(theme_color(pressed.background))
                    .text_color(theme_color(pressed.foreground))
            })
            .font_weight(font_weight)
            .text_color(theme_color(state.foreground))
            .whitespace_normal()
            .cursor_pointer()
            .child(label)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, window, cx| {
                    this.select_section(section_id.clone(), window, cx);
                }),
            )
    }

    fn render_selected_section(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        div()
            .flex_1()
            .min_w_0()
            .min_h(px(0.0))
            .overflow_hidden()
            .flex()
            .flex_col()
            .gap_3()
            .child(self.render_page_header(cx))
            .child(self.render_page_body(cx))
    }

    fn render_page_body(&self, cx: &mut Context<Self>) -> AnyElement {
        let page = self.model.selected_page();
        let page_id = page.page_id().clone();
        let local_split = page.local_split().cloned();

        if let Some(local_split) = local_split {
            return div()
                .id("settings-page-local-split")
                .flex_1()
                .min_w_0()
                .min_h(px(0.0))
                .overflow_hidden()
                .flex()
                .gap_3()
                .child(self.render_page_local_split_list(page_id, local_split, cx))
                .child(self.render_detail_rows_scroll(DetailRowsLayout::SplitDetail, cx))
                .into_any_element();
        }

        self.render_detail_rows_scroll(DetailRowsLayout::Standard, cx)
    }

    fn render_detail_rows_scroll(
        &self,
        layout: DetailRowsLayout,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .id("settings-scroll")
            .flex_1()
            .min_w_0()
            .min_h(px(0.0))
            .relative()
            .overflow_hidden()
            .on_mouse_move(cx.listener(Self::note_content_scrollbar_motion))
            .on_scroll_wheel(cx.listener(Self::note_content_scrollbar_scroll))
            .child(
                div()
                    .id("settings-scroll-surface")
                    .overflow_y_scroll()
                    .overflow_x_hidden()
                    .w_full()
                    .h_full()
                    .track_scroll(&self.scroll_handle)
                    .child(
                        div().w_full().flex().flex_col().gap_2().children(
                            self.model
                                .selected_rows()
                                .iter()
                                .map(|row| self.render_row(row, layout, cx)),
                        ),
                    ),
            )
            .children(self.render_vertical_scrollbar(
                "settings-selected-section-scrollbar",
                &self.scroll_handle,
                self.content_scrollbar_visibility_policy(cx.entity()),
            ))
            .into_any_element()
    }

    fn render_page_local_split_list(
        &self,
        page_id: crate::SettingsPageId,
        local_split: crate::SettingsPageSplit,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let item_count = local_split.items().len();
        let rendered_range = page_local_split_render_window(
            item_count,
            self.split_scroll_handle.offset().y,
            self.split_scroll_handle.bounds().size.height,
        );
        let children =
            self.render_page_local_split_window(page_id.clone(), &local_split, rendered_range, cx);

        div()
            .id(SharedString::from(format!(
                "settings-page-local-split-list-{}",
                element_id_suffix(page_id.as_str())
            )))
            .flex_none()
            .w(px(PAGE_LOCAL_SPLIT_LIST_WIDTH))
            .h_full()
            .min_h(px(0.0))
            .relative()
            .overflow_hidden()
            .on_mouse_move(cx.listener(Self::note_split_scrollbar_motion))
            .on_scroll_wheel(cx.listener(Self::note_split_scrollbar_scroll))
            .child(
                div()
                    .id(SharedString::from(format!(
                        "settings-page-local-split-list-surface-{}",
                        element_id_suffix(page_id.as_str())
                    )))
                    .overflow_y_scroll()
                    .overflow_x_hidden()
                    .w_full()
                    .h_full()
                    .track_scroll(&self.split_scroll_handle)
                    .child(div().w_full().children(children)),
            )
            .children(self.render_vertical_scrollbar(
                "settings-page-local-split-list-scrollbar",
                &self.split_scroll_handle,
                self.split_scrollbar_visibility_policy(cx.entity()),
            ))
            .into_any_element()
    }

    fn render_page_local_split_window(
        &self,
        page_id: crate::SettingsPageId,
        local_split: &crate::SettingsPageSplit,
        rendered_range: Range<usize>,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let items = local_split.items();
        let mut children = Vec::with_capacity(rendered_range.len() * 2 + 2);
        let top_spacer_height = page_local_split_offset_for_index(rendered_range.start);
        if top_spacer_height > 0.0 {
            children.push(
                div()
                    .flex_none()
                    .h(px(top_spacer_height))
                    .into_any_element(),
            );
        }

        for index in rendered_range.clone() {
            if let Some(item) = items.get(index).cloned() {
                children.push(self.render_page_local_split_item(page_id.clone(), item, cx));
                if index + 1 < rendered_range.end {
                    children.push(
                        div()
                            .flex_none()
                            .h(px(PAGE_LOCAL_SPLIT_ITEM_GAP))
                            .into_any_element(),
                    );
                }
            }
        }

        let rendered_height = page_local_split_segment_height(rendered_range.len());
        let bottom_spacer_height =
            (page_local_split_total_height(items.len()) - top_spacer_height - rendered_height)
                .max(0.0);
        if bottom_spacer_height > 0.0 {
            children.push(
                div()
                    .flex_none()
                    .h(px(bottom_spacer_height))
                    .into_any_element(),
            );
        }
        children
    }

    fn render_page_local_split_item(
        &self,
        page_id: crate::SettingsPageId,
        item: crate::SettingsPageSplitItem,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let item_id = item.item_id().clone();
        let selected = item.is_selected();
        let button_theme = self.visual_theme.navigation_button.clone();
        let state = if selected {
            button_theme.active.clone()
        } else {
            button_theme.normal.clone()
        };
        let hover = button_theme.hover;
        let pressed = button_theme.active;
        let preview_style = item.preview_style().cloned();
        let foreground = preview_style
            .as_ref()
            .and_then(|style| style.foreground())
            .unwrap_or(state.foreground);
        let background = preview_style
            .as_ref()
            .and_then(|style| style.background())
            .unwrap_or(state.background);
        let border = preview_style
            .as_ref()
            .and_then(|style| style.border())
            .unwrap_or(state.border);
        let font_size = preview_style
            .as_ref()
            .and_then(|style| style.font_size())
            .map(f32::from)
            .unwrap_or(self.font_size);
        let font_weight = preview_style
            .as_ref()
            .and_then(|style| style.font_weight())
            .map(|weight| FontWeight(weight as f32))
            .unwrap_or(FontWeight(button_theme.font_weight as f32));
        let font_family = preview_style
            .as_ref()
            .and_then(|style| style.font_family())
            .map(|font_family| SharedString::from(font_family.to_owned()));

        div()
            .id(SharedString::from(format!(
                "settings-page-local-split-item-{}-{}",
                element_id_suffix(page_id.as_str()),
                element_id_suffix(item_id.as_str())
            )))
            .flex_none()
            .w_full()
            .h(px(PAGE_LOCAL_SPLIT_ITEM_HEIGHT))
            .overflow_hidden()
            .px_2()
            .py_2()
            .rounded_sm()
            .border_1()
            .border_color(theme_color(border))
            .bg(theme_color(background))
            .text_color(theme_color(foreground))
            .text_size(px(font_size))
            .font_weight(font_weight)
            .hover(move |style| {
                style
                    .border_color(theme_color(hover.border))
                    .bg(theme_color(hover.background))
                    .text_color(theme_color(hover.foreground))
            })
            .active(move |style| {
                style
                    .border_color(theme_color(pressed.border))
                    .bg(theme_color(pressed.background))
                    .text_color(theme_color(pressed.foreground))
            })
            .cursor_pointer()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .overflow_hidden()
                    .child(
                        div()
                            .whitespace_normal()
                            .line_clamp(2)
                            .when_some(font_family, |element, font_family| {
                                element.font_family(font_family)
                            })
                            .child(item.label().to_owned()),
                    )
                    .when_some(item.subtext(), |element, subtext| {
                        element.child(
                            div()
                                .whitespace_normal()
                                .line_clamp(2)
                                .text_size(px((self.font_size - 2.0).max(11.0)))
                                .text_color(theme_color(self.visual_theme.row.muted_foreground))
                                .child(subtext.to_owned()),
                        )
                    }),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, window, cx| {
                    window.focus(&this.focus_handle);
                    cx.emit(SettingsWindowEvent::PageSplitItemSelected {
                        page_id: page_id.clone(),
                        item_id: item_id.clone(),
                    });
                }),
            )
            .into_any_element()
    }

    fn render_page_header(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let page = self.model.selected_page();
        let page_id = page.page_id().clone();
        let title = page.title().to_owned();
        let modified = page.is_modified();
        let back_target = page.back_target_page_id().cloned();
        let breadcrumbs: Vec<_> = page.breadcrumb_path().to_vec();
        let actions: Vec<_> = page.actions().to_vec();

        div()
            .flex_none()
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .items_center()
                            .gap_2()
                            .children(
                                back_target.map(|target_page_id| {
                                    self.render_back_button(target_page_id, cx)
                                }),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .whitespace_normal()
                                    .text_size(px(16.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(title),
                            )
                            .children(modified.then(|| self.render_modified_indicator())),
                    )
                    .child(
                        div()
                            .flex_none()
                            .flex()
                            .items_center()
                            .gap_2()
                            .pr_1()
                            .children(actions.into_iter().map(|action| {
                                self.render_page_action_button(page_id.clone(), action, cx)
                            })),
                    ),
            )
            .when(!breadcrumbs.is_empty(), |element| {
                element.child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_1()
                        .min_w_0()
                        .text_size(px((self.font_size - 2.0).max(11.0)))
                        .text_color(theme_color(self.visual_theme.panel.muted_foreground))
                        .children(breadcrumbs.into_iter().enumerate().flat_map(
                            |(index, segment)| {
                                let mut elements = Vec::new();
                                if index > 0 {
                                    elements.push(div().child("/").into_any_element());
                                }
                                elements.push(self.render_breadcrumb_segment(segment, cx));
                                elements
                            },
                        )),
                )
            })
    }

    fn render_back_button(
        &self,
        page_id: crate::SettingsPageId,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.render_button_element(
            SharedString::from(format!(
                "settings-page-back-{}",
                element_id_suffix(page_id.as_str())
            )),
            String::from("Back"),
            SettingsWindowEvent::PageNavigationRequested { page_id },
            false,
            true,
            cx,
        )
        .into_any_element()
    }

    fn render_breadcrumb_segment(
        &self,
        segment: crate::SettingsBreadcrumbSegment,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let label = segment.label().to_owned();
        if let Some(page_id) = segment.target_page_id().cloned() {
            return div()
                .id(SharedString::from(format!(
                    "settings-breadcrumb-{}",
                    element_id_suffix(page_id.as_str())
                )))
                .px_1()
                .rounded_sm()
                .hover({
                    let hover = self.visual_theme.navigation_button.hover.clone();
                    move |style| {
                        style
                            .bg(theme_color(hover.background))
                            .text_color(theme_color(hover.foreground))
                    }
                })
                .cursor_pointer()
                .child(label)
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |_, _, _, cx| {
                        cx.emit(SettingsWindowEvent::PageNavigationRequested {
                            page_id: page_id.clone(),
                        });
                    }),
                )
                .into_any_element();
        }

        div().px_1().child(label).into_any_element()
    }

    fn render_page_action_button(
        &self,
        page_id: crate::SettingsPageId,
        action: crate::SettingsPageAction,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let action_id = action.action_id().clone();
        let primary = action.priority() == crate::SettingsPageActionPriority::Primary;
        let element_id = SharedString::from(format!(
            "settings-page-action-{}-{}",
            element_id_suffix(page_id.as_str()),
            element_id_suffix(action_id.as_str())
        ));

        self.render_button_element(
            element_id,
            action.label().to_owned(),
            SettingsWindowEvent::PageActionRequested { page_id, action_id },
            primary,
            action.is_enabled(),
            cx,
        )
        .into_any_element()
    }

    fn render_modified_indicator(&self) -> AnyElement {
        div()
            .flex_none()
            .text_color(theme_color(self.visual_theme.row.muted_foreground))
            .child("*")
            .into_any_element()
    }

    fn render_vertical_scrollbar(
        &self,
        id: &'static str,
        scroll_handle: &ScrollHandle,
        visibility: ScrollbarVisibilityPolicy,
    ) -> Option<AnyElement> {
        let style = ScrollbarStyle {
            thumb_color: self.visual_theme.panel.muted_foreground.packed_rgb(),
            ..ScrollbarStyle::default()
        };

        render_scroll_handle_scrollbar(id, scroll_handle, Axis::Vertical, style, visibility)
    }

    fn render_row(
        &self,
        row: &SettingsRow,
        layout: DetailRowsLayout,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match row.row_kind() {
            crate::SettingsRowKind::Field => self.render_field_row(row, layout, cx),
            crate::SettingsRowKind::Navigation { target_page_id } => {
                self.render_navigation_row(row, target_page_id.clone(), cx)
            }
            crate::SettingsRowKind::ActionOnly => self.render_action_only_row(row, cx),
        }
    }

    fn render_field_row(
        &self,
        row: &SettingsRow,
        layout: DetailRowsLayout,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if layout == DetailRowsLayout::SplitDetail {
            return self.render_split_detail_field_row(row, cx);
        }

        let control_width = field_control_width(row.kind());
        let stack_actions_below_field = field_row_stacks_actions_below_input(row);
        let picker_open = self.color_picker_field.as_ref() == Some(row.field_id());
        let color_preview_color = (row.kind() == crate::SettingsFieldKind::Color)
            .then(|| self.color_preview_for_rendered_field(row.field_id(), row.value()))
            .flatten();
        let color_preview = (row.kind() == crate::SettingsFieldKind::Color).then(|| {
            self.render_color_preview_swatch(
                row.field_id().clone(),
                color_preview_color,
                picker_open,
                cx,
            )
        });
        let control = self.render_field_control(
            row.field_id().clone(),
            row.kind(),
            row.value().to_owned(),
            row.choices().to_vec(),
            control_width,
            cx,
        );

        div()
            .id(SharedString::from(format!(
                "settings-row-{}",
                element_id_suffix(row.field_id().as_str())
            )))
            .flex()
            .items_start()
            .rounded_sm()
            .border_1()
            .border_color(theme_color(self.visual_theme.row.border))
            .bg(theme_color(self.visual_theme.row.background))
            .px_3()
            .py_3()
            .child(self.render_row_label_stack(
                row,
                self.visual_theme.row.muted_foreground,
                true,
                true,
            ))
            .child(row_control_gutter())
            .child(
                div()
                    .flex_none()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .when(stack_actions_below_field, |element| {
                                element.flex_col().items_end()
                            })
                            .when(!stack_actions_below_field, |element| {
                                element
                                    .when(
                                        row.kind() == crate::SettingsFieldKind::MultilineText,
                                        |element| element.items_start(),
                                    )
                                    .when(
                                        row.kind() != crate::SettingsFieldKind::MultilineText,
                                        |element| element.items_center(),
                                    )
                            })
                            .gap_2()
                            .children(color_preview)
                            .child(control)
                            .children((!row.actions().is_empty()).then(|| {
                                div()
                                    .flex_none()
                                    .min_w(px(ROW_ACTION_CLUSTER_MIN_WIDTH))
                                    .flex()
                                    .items_center()
                                    .justify_end()
                                    .gap_2()
                                    .children(row.actions().iter().cloned().map(|action| {
                                        self.render_row_action_button(
                                            row.field_id().clone(),
                                            action,
                                            cx,
                                        )
                                    }))
                            }))
                            .children(picker_open.then(|| {
                                deferred(
                                    anchored()
                                        .anchor(Corner::TopLeft)
                                        .position_mode(AnchoredPositionMode::Local)
                                        .offset(point(px(12.0), px(0.0)))
                                        .child(
                                            self.render_color_picker_popup(color_preview_color, cx),
                                        ),
                                )
                                .with_priority(1)
                            })),
                    )
                    .children(
                        row.detail_field()
                            .map(|field| self.render_detail_field_control(field, cx)),
                    )
                    .when_some(row.error(), |element, error| {
                        element.child(
                            div()
                                .text_color(theme_color(self.visual_theme.input.error_border))
                                .text_size(px((self.font_size - 1.0).max(12.0)))
                                .child(error.to_owned()),
                        )
                    }),
            )
            .into_any_element()
    }

    fn render_split_detail_field_row(
        &self,
        row: &SettingsRow,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let control_width = field_control_width(row.kind());
        let stack_actions_below_field = field_row_stacks_actions_below_input(row);
        let picker_open = self.color_picker_field.as_ref() == Some(row.field_id());
        let color_preview_color = (row.kind() == crate::SettingsFieldKind::Color)
            .then(|| self.color_preview_for_rendered_field(row.field_id(), row.value()))
            .flatten();
        let color_preview = (row.kind() == crate::SettingsFieldKind::Color).then(|| {
            self.render_color_preview_swatch(
                row.field_id().clone(),
                color_preview_color,
                picker_open,
                cx,
            )
        });
        let control = self.render_field_control(
            row.field_id().clone(),
            row.kind(),
            row.value().to_owned(),
            row.choices().to_vec(),
            control_width,
            cx,
        );

        div()
            .id(SharedString::from(format!(
                "settings-row-{}",
                element_id_suffix(row.field_id().as_str())
            )))
            .flex()
            .items_start()
            .rounded_sm()
            .border_1()
            .border_color(theme_color(self.visual_theme.row.border))
            .bg(theme_color(self.visual_theme.row.background))
            .px_3()
            .py_3()
            .child(self.render_row_label_stack_with_min_width(
                row,
                self.visual_theme.row.muted_foreground,
                false,
                true,
                SPLIT_DETAIL_ROW_LABEL_MIN_WIDTH,
            ))
            .child(row_control_gutter())
            .child(
                div()
                    .flex_none()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .when(stack_actions_below_field, |element| {
                                element.flex_col().items_end()
                            })
                            .when(!stack_actions_below_field, |element| {
                                element
                                    .when(
                                        row.kind() == crate::SettingsFieldKind::MultilineText,
                                        |element| element.items_start(),
                                    )
                                    .when(
                                        row.kind() != crate::SettingsFieldKind::MultilineText,
                                        |element| element.items_center(),
                                    )
                            })
                            .gap_2()
                            .children(color_preview)
                            .child(control)
                            .children((!row.actions().is_empty()).then(|| {
                                div()
                                    .flex_none()
                                    .min_w(px(ROW_ACTION_CLUSTER_MIN_WIDTH))
                                    .flex()
                                    .items_center()
                                    .justify_end()
                                    .gap_2()
                                    .children(row.actions().iter().cloned().map(|action| {
                                        self.render_row_action_button(
                                            row.field_id().clone(),
                                            action,
                                            cx,
                                        )
                                    }))
                            }))
                            .children(picker_open.then(|| {
                                deferred(
                                    anchored()
                                        .anchor(Corner::TopLeft)
                                        .position_mode(AnchoredPositionMode::Local)
                                        .offset(point(px(12.0), px(0.0)))
                                        .child(
                                            self.render_color_picker_popup(color_preview_color, cx),
                                        ),
                                )
                                .with_priority(1)
                            })),
                    )
                    .children(
                        row.detail_field()
                            .map(|field| self.render_detail_field_control(field, cx)),
                    )
                    .when_some(row.error(), |element, error| {
                        element.child(
                            div()
                                .text_color(theme_color(self.visual_theme.input.error_border))
                                .text_size(px((self.font_size - 1.0).max(12.0)))
                                .child(error.to_owned()),
                        )
                    }),
            )
            .into_any_element()
    }

    fn render_field_control(
        &self,
        field_id: SettingsFieldId,
        kind: crate::SettingsFieldKind,
        value: String,
        choices: Vec<crate::SettingsChoiceOption>,
        control_width: f32,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if kind == crate::SettingsFieldKind::Choice {
            return self.render_choice_control(field_id, value, choices, control_width, cx);
        }

        let input = self
            .input_for_field(&field_id)
            .expect("settings field should exist");
        div()
            .flex_none()
            .w(px(control_width))
            .child(input)
            .into_any_element()
    }

    fn render_choice_control(
        &self,
        field_id: SettingsFieldId,
        selected_value: String,
        choices: Vec<crate::SettingsChoiceOption>,
        control_width: f32,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selected_label = choices
            .iter()
            .find(|choice| choice.value() == selected_value)
            .map(|choice| choice.label().to_owned())
            .unwrap_or_else(|| selected_value.clone());
        let popup_open = self.choice_popup_field.as_ref() == Some(&field_id);
        let popup_anchor_bounds = self.choice_control_bounds.get(&field_id).copied();
        let choice_control_height = self.font_size + 20.0;
        let button_theme = if popup_open {
            self.visual_theme.primary_button.clone()
        } else {
            self.visual_theme.secondary_button.clone()
        };
        let normal = button_theme.normal;
        let hover = button_theme.hover;
        let pressed = button_theme.active;
        let entity = cx.entity();
        let bounds_field_id = field_id.clone();

        div()
            .flex_none()
            .on_children_prepainted(move |children, _, cx| {
                let bounds = children.first().copied();
                entity.update(cx, |panel, cx| {
                    panel.record_choice_control_bounds(bounds_field_id.clone(), bounds, cx)
                });
            })
            .child(
                div()
                    .id(SharedString::from(format!(
                        "settings-choice-{}",
                        element_id_suffix(field_id.as_str())
                    )))
                    .flex_none()
                    .w(px(control_width))
                    .min_h(px(choice_control_height))
                    .relative()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .border_1()
                    .border_color(theme_color(normal.border))
                    .bg(theme_color(normal.background))
                    .text_color(theme_color(normal.foreground))
                    .font_weight(FontWeight(button_theme.font_weight as f32))
                    .text_size(px((self.font_size - 1.0).max(11.0)))
                    .cursor_pointer()
                    .hover(move |style| {
                        style
                            .border_color(theme_color(hover.border))
                            .bg(theme_color(hover.background))
                            .text_color(theme_color(hover.foreground))
                    })
                    .active(move |style| {
                        style
                            .border_color(theme_color(pressed.border))
                            .bg(theme_color(pressed.background))
                            .text_color(theme_color(pressed.foreground))
                    })
                    .child(div().flex_1().min_w_0().line_clamp(1).child(selected_label))
                    .child(div().flex_none().child(CHOICE_DROPDOWN_TRIANGLE))
                    .children(popup_open.then(|| {
                        let popup = anchored()
                            .anchor(Corner::TopLeft)
                            .offset(point(px(0.0), px(4.0)))
                            .snap_to_window_with_margin(px(8.0));
                        let popup = if let Some(bounds) = popup_anchor_bounds {
                            popup.position(bounds.bottom_left())
                        } else {
                            popup
                                .position_mode(AnchoredPositionMode::Local)
                                .position(point(px(0.0), px(choice_control_height)))
                        };
                        deferred(popup.child(self.render_choice_popup(
                            field_id.clone(),
                            choices,
                            selected_value,
                            cx,
                        )))
                        .with_priority(1)
                    }))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, window, cx| {
                            cx.stop_propagation();
                            window.focus(&this.focus_handle);
                            this.toggle_choice_popup(field_id.clone(), cx);
                        }),
                    ),
            )
            .into_any_element()
    }

    fn render_choice_popup(
        &self,
        field_id: SettingsFieldId,
        choices: Vec<crate::SettingsChoiceOption>,
        selected_value: String,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .id(SharedString::from(format!(
                "settings-choice-popup-{}",
                element_id_suffix(field_id.as_str())
            )))
            .w(px(CHOICE_FIELD_CONTROL_WIDTH))
            .flex()
            .flex_col()
            .gap_1()
            .occlude()
            .rounded_sm()
            .border_1()
            .border_color(theme_color(self.visual_theme.popup.border))
            .bg(theme_color(self.visual_theme.popup.background))
            .p_1()
            .on_mouse_down_out(cx.listener(|this, event: &MouseDownEvent, _, cx| {
                if event.button == MouseButton::Left {
                    this.close_choice_popup(cx);
                }
            }))
            .children(choices.into_iter().map(|choice| {
                let selected = choice.value() == selected_value;
                let button_theme = if selected {
                    self.visual_theme.primary_button.clone()
                } else {
                    self.visual_theme.secondary_button.clone()
                };
                let normal = button_theme.normal;
                let hover = button_theme.hover;
                let pressed = button_theme.active;
                let value = choice.value().to_owned();
                div()
                    .id(SharedString::from(format!(
                        "settings-choice-popup-option-{}-{}",
                        element_id_suffix(field_id.as_str()),
                        element_id_suffix(value.as_str())
                    )))
                    .w_full()
                    .min_h(px(self.font_size + 18.0))
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .border_1()
                    .border_color(theme_color(normal.border))
                    .bg(theme_color(normal.background))
                    .text_color(theme_color(normal.foreground))
                    .font_weight(FontWeight(button_theme.font_weight as f32))
                    .text_size(px((self.font_size - 1.0).max(11.0)))
                    .flex()
                    .items_center()
                    .cursor_pointer()
                    .hover(move |style| {
                        style
                            .border_color(theme_color(hover.border))
                            .bg(theme_color(hover.background))
                            .text_color(theme_color(hover.foreground))
                    })
                    .active(move |style| {
                        style
                            .border_color(theme_color(pressed.border))
                            .bg(theme_color(pressed.background))
                            .text_color(theme_color(pressed.foreground))
                    })
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .line_clamp(1)
                            .child(choice.label().to_owned()),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener({
                            let field_id = field_id.clone();
                            move |this, _, window, cx| {
                                cx.stop_propagation();
                                window.focus(&this.focus_handle);
                                this.select_choice_value(field_id.clone(), value.clone(), cx);
                            }
                        }),
                    )
            }))
            .into_any_element()
    }

    fn render_detail_field_control(
        &self,
        field: &crate::SettingsRowDetailField,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let control_width = field_control_width(field.kind());
        let picker_open = self.color_picker_field.as_ref() == Some(field.field_id());
        let color_preview_color = (field.kind() == crate::SettingsFieldKind::Color)
            .then(|| self.color_preview_for_rendered_field(field.field_id(), field.value()))
            .flatten();
        let color_preview = (field.kind() == crate::SettingsFieldKind::Color).then(|| {
            self.render_color_preview_swatch(
                field.field_id().clone(),
                color_preview_color,
                picker_open,
                cx,
            )
        });
        let control = self.render_field_control(
            field.field_id().clone(),
            field.kind(),
            field.value().to_owned(),
            field.choices().to_vec(),
            control_width,
            cx,
        );

        div()
            .w_full()
            .flex()
            .justify_end()
            .child(
                div()
                    .flex_none()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .children(
                                field
                                    .is_modified()
                                    .then(|| self.render_modified_indicator()),
                            )
                            .children(color_preview)
                            .child(control)
                            .children(picker_open.then(|| {
                                deferred(
                                    anchored()
                                        .anchor(Corner::TopLeft)
                                        .position_mode(AnchoredPositionMode::Local)
                                        .offset(point(px(12.0), px(0.0)))
                                        .child(
                                            self.render_color_picker_popup(color_preview_color, cx),
                                        ),
                                )
                                .with_priority(1)
                            })),
                    )
                    .when_some(field.error(), |element, error| {
                        element.child(
                            div()
                                .text_color(theme_color(self.visual_theme.input.error_border))
                                .text_size(px((self.font_size - 1.0).max(12.0)))
                                .child(error.to_owned()),
                        )
                    }),
            )
            .into_any_element()
    }

    fn render_navigation_row(
        &self,
        row: &SettingsRow,
        target_page_id: crate::SettingsPageId,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .id(SharedString::from(format!(
                "settings-row-{}",
                element_id_suffix(row.field_id().as_str())
            )))
            .flex()
            .items_center()
            .rounded_sm()
            .border_1()
            .border_color(theme_color(self.visual_theme.row.border))
            .bg(theme_color(self.visual_theme.row.background))
            .px_3()
            .py_3()
            .hover({
                let hover = self.visual_theme.navigation_button.hover.clone();
                move |style| {
                    style
                        .border_color(theme_color(hover.border))
                        .bg(theme_color(hover.background))
                        .text_color(theme_color(hover.foreground))
                }
            })
            .cursor_pointer()
            .child(self.render_row_label_stack(row, self.visual_theme.row.foreground, false, true))
            .child(row_control_gutter())
            .child(div().flex_none().flex().items_center().gap_2().children(
                row.actions().iter().cloned().map(|action| {
                    self.render_row_action_button(row.field_id().clone(), action, cx)
                }),
            ))
            .child(
                div()
                    .flex_none()
                    .text_color(theme_color(self.visual_theme.row.muted_foreground))
                    .child(NAVIGATION_CHEVRON),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, window, cx| {
                    window.focus(&this.focus_handle);
                    cx.emit(SettingsWindowEvent::PageNavigationRequested {
                        page_id: target_page_id.clone(),
                    });
                }),
            )
            .into_any_element()
    }

    fn render_action_only_row(&self, row: &SettingsRow, cx: &mut Context<Self>) -> AnyElement {
        div()
            .id(SharedString::from(format!(
                "settings-row-{}",
                element_id_suffix(row.field_id().as_str())
            )))
            .flex()
            .items_center()
            .rounded_sm()
            .border_1()
            .border_color(theme_color(self.visual_theme.row.border))
            .bg(theme_color(self.visual_theme.row.background))
            .px_3()
            .py_3()
            .child(self.render_row_label_stack(row, self.visual_theme.row.foreground, false, true))
            .child(row_control_gutter())
            .child(div().flex_none().flex().items_center().gap_2().children(
                row.actions().iter().cloned().map(|action| {
                    self.render_row_action_button(row.field_id().clone(), action, cx)
                }),
            ))
            .into_any_element()
    }

    fn render_row_label_stack(
        &self,
        row: &SettingsRow,
        foreground: RgbColor,
        align_with_single_line_control: bool,
        flexible_width: bool,
    ) -> AnyElement {
        self.render_row_label_stack_with_min_width(
            row,
            foreground,
            align_with_single_line_control,
            flexible_width,
            ROW_LABEL_MIN_WIDTH,
        )
    }

    fn render_row_label_stack_with_min_width(
        &self,
        row: &SettingsRow,
        foreground: RgbColor,
        align_with_single_line_control: bool,
        flexible_width: bool,
        min_width: f32,
    ) -> AnyElement {
        div()
            .when(flexible_width, |element| element.flex_1())
            .when(!flexible_width, |element| element.w_full())
            .min_w(px(min_width))
            .when(align_with_single_line_control, |element| element.pt_2())
            .flex()
            .flex_col()
            .gap_1()
            .text_color(theme_color(foreground))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .min_w_0()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .whitespace_normal()
                            .child(row.label().to_owned()),
                    )
                    .children(row.is_modified().then(|| self.render_modified_indicator())),
            )
            .when_some(row.subtext(), |element, subtext| {
                element.child(
                    div()
                        .min_w_0()
                        .whitespace_normal()
                        .text_size(px((self.font_size - 2.0).max(11.0)))
                        .text_color(theme_color(self.visual_theme.row.muted_foreground))
                        .child(subtext.to_owned()),
                )
            })
            .into_any_element()
    }

    fn render_row_action_button(
        &self,
        field_id: SettingsFieldId,
        action: crate::SettingsRowAction,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let action_id = action.action_id().clone();
        let element_id = SharedString::from(format!(
            "settings-row-action-{}-{}",
            element_id_suffix(field_id.as_str()),
            element_id_suffix(action_id.as_str())
        ));

        self.render_button_element(
            element_id,
            action.label().to_owned(),
            SettingsWindowEvent::RowActionRequested {
                field_id,
                action_id,
            },
            false,
            action.is_enabled(),
            cx,
        )
    }

    fn render_buttons(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div().flex().justify_end().gap_2().children([
            self.render_button_element(
                SharedString::from("settings-button-ok"),
                String::from("OK"),
                SettingsWindowEvent::AcceptRequested,
                true,
                true,
                cx,
            ),
            self.render_button_element(
                SharedString::from("settings-button-apply"),
                String::from("Apply"),
                SettingsWindowEvent::ApplyRequested,
                false,
                true,
                cx,
            ),
            self.render_button_element(
                SharedString::from("settings-button-cancel"),
                String::from("Cancel"),
                SettingsWindowEvent::CancelRequested,
                false,
                true,
                cx,
            ),
        ])
    }

    fn render_button_element(
        &self,
        id: SharedString,
        label: String,
        event: SettingsWindowEvent,
        active: bool,
        enabled: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let button_theme = if active {
            self.visual_theme.primary_button.clone()
        } else {
            self.visual_theme.secondary_button.clone()
        };
        let normal = if enabled {
            button_theme.normal
        } else {
            button_theme.disabled
        };
        let hover = button_theme.hover;
        let pressed = button_theme.active;
        let font_weight = FontWeight(button_theme.font_weight as f32);

        div()
            .id(id)
            .flex_none()
            .px_3()
            .py_1()
            .rounded_sm()
            .border_1()
            .border_color(theme_color(normal.border))
            .bg(theme_color(normal.background))
            .font_weight(font_weight)
            .text_color(theme_color(normal.foreground))
            .whitespace_nowrap()
            .child(label)
            .when(enabled, |element| {
                element
                    .hover(move |style| {
                        style
                            .border_color(theme_color(hover.border))
                            .bg(theme_color(hover.background))
                            .text_color(theme_color(hover.foreground))
                    })
                    .active(move |style| {
                        style
                            .border_color(theme_color(pressed.border))
                            .bg(theme_color(pressed.background))
                            .text_color(theme_color(pressed.foreground))
                    })
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, window, cx| {
                            cx.stop_propagation();
                            window.focus(&this.focus_handle);
                            cx.emit(event.clone());
                        }),
                    )
            })
    }
}

fn field_control_width(kind: crate::SettingsFieldKind) -> f32 {
    match kind {
        crate::SettingsFieldKind::Text => TEXT_FIELD_CONTROL_WIDTH,
        crate::SettingsFieldKind::Number => NUMERIC_FIELD_CONTROL_WIDTH,
        crate::SettingsFieldKind::MultilineText => MULTILINE_FIELD_CONTROL_WIDTH,
        crate::SettingsFieldKind::Color => COLOR_FIELD_CONTROL_WIDTH,
        crate::SettingsFieldKind::Choice => CHOICE_FIELD_CONTROL_WIDTH,
    }
}

fn field_row_stacks_actions_below_input(row: &SettingsRow) -> bool {
    row.kind() == crate::SettingsFieldKind::Text && !row.actions().is_empty()
}

fn row_control_gutter() -> AnyElement {
    div()
        .flex_none()
        .w(px(ROW_CONTROL_GUTTER_WIDTH))
        .into_any_element()
}
