use gpui::IntoElement;
use gpui_settings_window::{
    MAX_PAGE_DETAIL_ROWS, RgbColor, SettingsBreadcrumbSegment, SettingsChoiceOption,
    SettingsFieldId, SettingsFieldKind, SettingsPage, SettingsPageAction,
    SettingsPageActionPriority, SettingsPageId, SettingsPageSplit, SettingsPageSplitItem,
    SettingsPageSplitItemId, SettingsPageSplitItemPreviewStyle, SettingsRow, SettingsRowAction,
    SettingsRowDetailField, SettingsSection, SettingsWindowError, SettingsWindowModel,
    SettingsWindowOptions, SettingsWindowTheme,
};

#[test]
fn selects_first_section_by_default() {
    let appearance = SettingsSection::new("appearance", "Appearance").with_row(SettingsRow::new(
        "accent_color",
        "Accent color",
        "#6699cc",
        SettingsFieldKind::Color,
    ));
    let editor = SettingsSection::new("editor", "Editor");

    let model = SettingsWindowModel::new(vec![appearance, editor]).expect("model should validate");

    assert_eq!(model.selected_section_id().as_str(), "appearance");
    assert_eq!(model.selected_section().label(), "Appearance");
    assert_eq!(
        model
            .row(&SettingsFieldId::from("accent_color"))
            .expect("row should exist")
            .value(),
        "#6699cc",
    );
}

#[test]
fn rejects_duplicate_section_ids() {
    let result = SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance"),
        SettingsSection::new("appearance", "Duplicate"),
    ]);

    assert_eq!(
        result.expect_err("duplicate should fail"),
        SettingsWindowError::DuplicateSectionId("appearance".into()),
    );
}

#[test]
fn rejects_duplicate_field_ids() {
    let result = SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_row(SettingsRow::new(
            "accent_color",
            "Accent color",
            "#6699cc",
            SettingsFieldKind::Color,
        )),
        SettingsSection::new("editor", "Editor").with_row(SettingsRow::new(
            "accent_color",
            "Accent color",
            "#112233",
            SettingsFieldKind::Color,
        )),
    ]);

    assert_eq!(
        result.expect_err("duplicate should fail"),
        SettingsWindowError::DuplicateFieldId("accent_color".into()),
    );
}

#[test]
fn rejects_missing_selected_section() {
    let result = SettingsWindowModel::with_selected_section(
        vec![SettingsSection::new("appearance", "Appearance")],
        "missing",
    );

    assert_eq!(
        result.expect_err("missing selection should fail"),
        SettingsWindowError::MissingSelectedSection("missing".into()),
    );
}

#[test]
fn row_actions_are_part_of_the_presentation_model() {
    let model = SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_row(
            SettingsRow::new(
                "font_family",
                "Font family",
                "Inter",
                SettingsFieldKind::Text,
            )
            .with_action(SettingsRowAction::new("browse", "Browse..."))
            .with_action(SettingsRowAction::new("reset", "Reset")),
        ),
    ])
    .expect("model should validate");

    let row = model
        .row(&SettingsFieldId::from("font_family"))
        .expect("row should exist");

    assert_eq!(row.actions().len(), 2);
    assert_eq!(row.actions()[0].action_id().as_str(), "browse");
    assert_eq!(row.actions()[0].label(), "Browse...");
    assert_eq!(row.actions()[1].action_id().as_str(), "reset");
    assert_eq!(row.actions()[1].label(), "Reset");
}

#[test]
fn row_subtext_is_part_of_the_presentation_model() {
    let model = SettingsWindowModel::new(vec![
        SettingsSection::new("agent", "Agent").with_row(
            SettingsRow::new(
                "developer_instructions",
                "Developer Instructions",
                "",
                SettingsFieldKind::MultilineText,
            )
            .with_subtext("Sent as developer instructions with every user message."),
        ),
    ])
    .expect("model should validate");

    let row = model
        .row(&SettingsFieldId::from("developer_instructions"))
        .expect("row should exist");

    assert_eq!(
        row.subtext(),
        Some("Sent as developer instructions with every user message.")
    );
}

#[test]
fn multiline_text_rows_are_plain_string_fields() {
    let model = SettingsWindowModel::new(vec![SettingsSection::new("agent", "Agent").with_row(
        SettingsRow::new(
            "instructions",
            "Instructions",
            "First line\nSecond line",
            SettingsFieldKind::MultilineText,
        ),
    )])
    .expect("model should validate");

    let row = model
        .row(&SettingsFieldId::from("instructions"))
        .expect("row should exist");

    assert_eq!(row.kind(), SettingsFieldKind::MultilineText);
    assert_eq!(row.value(), "First line\nSecond line");
}

#[test]
fn numeric_rows_are_plain_string_fields_with_compact_presentation() {
    let model = SettingsWindowModel::new(vec![
        SettingsSection::new("operations", "Operations").with_row(SettingsRow::new(
            "context_compaction_timeout_ms",
            "Context compaction timeout",
            "120000",
            SettingsFieldKind::Number,
        )),
    ])
    .expect("model should validate");

    let row = model
        .row(&SettingsFieldId::from("context_compaction_timeout_ms"))
        .expect("row should exist");

    assert_eq!(row.kind(), SettingsFieldKind::Number);
    assert_eq!(row.value(), "120000");
}

#[test]
fn choice_rows_are_plain_string_fields_with_dropdown_options() {
    let model = SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_row(
            SettingsRow::new(
                "source",
                "Source",
                "static_parent",
                SettingsFieldKind::Choice,
            )
            .with_choice(SettingsChoiceOption::new("value", "Value"))
            .with_choice(SettingsChoiceOption::new("static_parent", "Static parent"))
            .with_choice(SettingsChoiceOption::new(
                "ambient_parent",
                "Ambient parent",
            ))
            .with_choice(SettingsChoiceOption::new("fallback", "Fallback")),
        ),
    ])
    .expect("model should validate");

    let row = model
        .row(&SettingsFieldId::from("source"))
        .expect("row should exist");

    assert_eq!(row.kind(), SettingsFieldKind::Choice);
    assert_eq!(row.value(), "static_parent");
    assert_eq!(row.choices().len(), 4);
    assert_eq!(row.choices()[1].label(), "Static parent");
}

#[test]
fn rows_may_carry_one_secondary_detail_field() {
    let model = SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_row(
            SettingsRow::new("source", "Background", "value", SettingsFieldKind::Choice)
                .with_choice(SettingsChoiceOption::new("value", "Value"))
                .with_choice(SettingsChoiceOption::new("fallback", "Fallback"))
                .with_detail_field(
                    SettingsRowDetailField::new("value", "#112233", SettingsFieldKind::Color)
                        .with_modified(true),
                ),
        ),
    ])
    .expect("model should validate");

    let row = model
        .row(&SettingsFieldId::from("source"))
        .expect("row should exist");
    let detail = row.detail_field().expect("detail field should exist");

    assert_eq!(detail.field_id(), &SettingsFieldId::from("value"));
    assert_eq!(detail.kind(), SettingsFieldKind::Color);
    assert_eq!(detail.value(), "#112233");
    assert!(detail.is_modified());
    assert_eq!(
        model.field_kind(&SettingsFieldId::from("value")),
        Some(SettingsFieldKind::Color)
    );
    assert_eq!(
        model.field_value(&SettingsFieldId::from("value")),
        Some("#112233")
    );
}

#[test]
fn rejects_invalid_choice_rows() {
    let missing_selected = SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_row(
            SettingsRow::new("source", "Source", "missing", SettingsFieldKind::Choice)
                .with_choice(SettingsChoiceOption::new("value", "Value")),
        ),
    ]);

    assert_eq!(
        missing_selected.expect_err("missing choice value should fail"),
        SettingsWindowError::MissingChoiceValue {
            field_id: SettingsFieldId::from("source"),
            value: "missing".to_string(),
        },
    );

    let duplicate = SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_row(
            SettingsRow::new("source", "Source", "value", SettingsFieldKind::Choice)
                .with_choice(SettingsChoiceOption::new("value", "Value"))
                .with_choice(SettingsChoiceOption::new("value", "Duplicate")),
        ),
    ]);

    assert_eq!(
        duplicate.expect_err("duplicate choice value should fail"),
        SettingsWindowError::DuplicateChoiceOptionValue {
            field_id: SettingsFieldId::from("source"),
            value: "value".to_string(),
        },
    );

    let duplicate_detail_id = SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_row(
            SettingsRow::new("source", "Source", "value", SettingsFieldKind::Choice)
                .with_choice(SettingsChoiceOption::new("value", "Value"))
                .with_detail_field(SettingsRowDetailField::new(
                    "source",
                    "#112233",
                    SettingsFieldKind::Color,
                )),
        ),
    ]);

    assert_eq!(
        duplicate_detail_id.expect_err("duplicate detail field id should fail"),
        SettingsWindowError::DuplicateFieldId(SettingsFieldId::from("source")),
    );
}

#[test]
fn rejects_duplicate_row_action_ids_per_row() {
    let result = SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_row(
            SettingsRow::new(
                "font_family",
                "Font family",
                "Inter",
                SettingsFieldKind::Text,
            )
            .with_action(SettingsRowAction::new("choose", "Choose"))
            .with_action(SettingsRowAction::new("choose", "Pick")),
        ),
    ]);

    assert_eq!(
        result.expect_err("duplicate action should fail"),
        SettingsWindowError::DuplicateRowActionId {
            field_id: SettingsFieldId::from("font_family"),
            action_id: "choose".into(),
        },
    );
}

#[test]
fn pages_and_subpages_are_part_of_the_presentation_model() {
    let section = SettingsSection::new("appearance", "Appearance")
        .with_row(SettingsRow::navigation(
            "theme_editor_link",
            "Theme editor",
            "theme_editor",
        ))
        .with_page(
            SettingsPage::new("theme_editor", "Theme editor")
                .with_breadcrumb_segment(SettingsBreadcrumbSegment::linked(
                    "Appearance",
                    "appearance",
                ))
                .with_back_target("appearance")
                .with_row(
                    SettingsRow::new(
                        "font_family",
                        "Font family",
                        "Inter",
                        SettingsFieldKind::Text,
                    )
                    .with_modified(true),
                )
                .with_action(
                    SettingsPageAction::new("save", "Save")
                        .with_priority(SettingsPageActionPriority::Primary),
                ),
        );

    let model =
        SettingsWindowModel::with_selected_page(vec![section], "appearance", "theme_editor")
            .expect("model should validate");

    assert_eq!(model.selected_section_id().as_str(), "appearance");
    assert_eq!(model.selected_page_id().as_str(), "theme_editor");
    assert_eq!(model.selected_page().title(), "Theme editor");
    assert_eq!(
        model.selected_page().back_target_page_id(),
        Some(&SettingsPageId::from("appearance")),
    );
    assert!(model.selected_rows()[0].is_modified());
    assert_eq!(
        model.selected_page().actions()[0].action_id().as_str(),
        "save"
    );
}

#[test]
fn pages_may_carry_page_local_split_items_without_changing_detail_rows() {
    let split = SettingsPageSplit::new()
        .with_item(
            SettingsPageSplitItem::new("default", "Default")
                .with_subtext("Built in")
                .with_selected(true)
                .with_preview_style(
                    SettingsPageSplitItemPreviewStyle::default()
                        .with_font_family("Inter")
                        .with_font_size(13)
                        .with_font_weight(600)
                        .with_foreground(RgbColor::new(17, 18, 19))
                        .with_background(RgbColor::new(240, 241, 242))
                        .with_border(RgbColor::new(90, 91, 92)),
                ),
        )
        .with_item(SettingsPageSplitItem::new("large", "Large"));
    let model = SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_root_page(
            SettingsPage::new("appearance", "Appearance")
                .with_local_split(split)
                .with_row(SettingsRow::new(
                    "font_size",
                    "Font size",
                    "14",
                    SettingsFieldKind::Number,
                )),
        ),
    ])
    .expect("model should validate");
    let page = model.selected_page();
    let split = page.local_split().expect("page should carry local split");

    assert_eq!(model.selected_rows()[0].field_id().as_str(), "font_size");
    assert_eq!(split.selected_item().unwrap().item_id().as_str(), "default");
    assert_eq!(split.items()[0].subtext(), Some("Built in"));
    assert_eq!(
        split.items()[0].preview_style().unwrap().font_family(),
        Some("Inter")
    );
    assert_eq!(
        split.items()[0].preview_style().unwrap().font_size(),
        Some(13)
    );
    assert_eq!(
        split.items()[0].preview_style().unwrap().font_weight(),
        Some(600)
    );
    assert_eq!(
        split.items()[0].preview_style().unwrap().foreground(),
        Some(RgbColor::new(17, 18, 19)),
    );
}

#[test]
fn pages_may_request_stacked_custom_body_without_changing_detail_rows() {
    let model = SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_root_page(
            SettingsPage::new("appearance", "Appearance")
                .with_stacked_custom_body(gpui_settings_window::SettingsPageCustomBody::new(
                    "theme_navigator",
                    144,
                ))
                .with_row(SettingsRow::new(
                    "font_size",
                    "Font size",
                    "14",
                    SettingsFieldKind::Number,
                )),
        ),
    ])
    .expect("model should validate");
    let page = model.selected_page();

    assert_eq!(
        page.body_layout(),
        gpui_settings_window::SettingsPageBodyLayout::StackedCustom
    );
    assert_eq!(
        page.stacked_custom_body()
            .expect("page should carry custom body metadata")
            .body_id()
            .as_str(),
        "theme_navigator"
    );
    assert_eq!(
        page.stacked_custom_body()
            .expect("page should carry custom body metadata")
            .height_px(),
        144
    );
    assert!(page.local_split().is_none());
    assert_eq!(model.selected_rows()[0].field_id().as_str(), "font_size");
}

#[test]
fn rejects_duplicate_page_local_split_item_ids_per_page() {
    let result = SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_root_page(
            SettingsPage::new("appearance", "Appearance").with_local_split(
                SettingsPageSplit::new()
                    .with_item(SettingsPageSplitItem::new("default", "Default"))
                    .with_item(SettingsPageSplitItem::new("default", "Duplicate")),
            ),
        ),
    ]);

    assert_eq!(
        result.expect_err("duplicate split item should fail"),
        SettingsWindowError::DuplicatePageSplitItemId {
            page_id: SettingsPageId::from("appearance"),
            item_id: SettingsPageSplitItemId::from("default"),
        },
    );
}

#[test]
fn rejects_multiple_selected_page_local_split_items_per_page() {
    let result = SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_root_page(
            SettingsPage::new("appearance", "Appearance").with_local_split(
                SettingsPageSplit::new()
                    .with_item(SettingsPageSplitItem::new("default", "Default").with_selected(true))
                    .with_item(SettingsPageSplitItem::new("large", "Large").with_selected(true)),
            ),
        ),
    ]);

    assert_eq!(
        result.expect_err("multiple selected split items should fail"),
        SettingsWindowError::MultiplePageSplitItemsSelected {
            page_id: SettingsPageId::from("appearance"),
        },
    );
}

#[test]
fn rejects_duplicate_page_ids() {
    let result = SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance")
            .with_page(SettingsPage::new("theme_editor", "Theme editor")),
        SettingsSection::new("themes", "Themes")
            .with_page(SettingsPage::new("theme_editor", "Duplicate")),
    ]);

    assert_eq!(
        result.expect_err("duplicate page should fail"),
        SettingsWindowError::DuplicatePageId("theme_editor".into()),
    );
}

#[test]
fn rejects_pages_above_static_detail_row_bound() {
    let mut page = SettingsPage::new("appearance", "Appearance");
    for index in 0..=MAX_PAGE_DETAIL_ROWS {
        page = page.with_row(SettingsRow::new(
            format!("field.{index:03}"),
            format!("Field {index:03}"),
            index.to_string(),
            SettingsFieldKind::Text,
        ));
    }

    let result = SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_root_page(page),
    ]);

    assert_eq!(
        result.expect_err("oversized page should fail"),
        SettingsWindowError::TooManyPageRows {
            page_id: SettingsPageId::from("appearance"),
            row_count: MAX_PAGE_DETAIL_ROWS + 1,
            max_row_count: MAX_PAGE_DETAIL_ROWS,
        },
    );
}

#[test]
fn rejects_missing_selected_page() {
    let result = SettingsWindowModel::with_selected_page(
        vec![SettingsSection::new("appearance", "Appearance")],
        "appearance",
        "missing",
    );

    assert_eq!(
        result.expect_err("missing page should fail"),
        SettingsWindowError::MissingSelectedPage("missing".into()),
    );
}

#[test]
fn rejects_navigation_rows_with_missing_targets() {
    let result = SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_row(SettingsRow::navigation(
            "theme_editor_link",
            "Theme editor",
            "missing",
        )),
    ]);

    assert_eq!(
        result.expect_err("missing target should fail"),
        SettingsWindowError::MissingNavigationTargetPage {
            field_id: SettingsFieldId::from("theme_editor_link"),
            target_page_id: "missing".into(),
        },
    );
}

#[test]
fn rejects_duplicate_page_action_ids_per_page() {
    let result = SettingsWindowModel::new(vec![
        SettingsSection::new("appearance", "Appearance").with_root_page(
            SettingsPage::new("appearance", "Appearance")
                .with_action(SettingsPageAction::new("save", "Save"))
                .with_action(SettingsPageAction::new("save", "Write")),
        ),
    ]);

    assert_eq!(
        result.expect_err("duplicate page action should fail"),
        SettingsWindowError::DuplicatePageActionId {
            page_id: SettingsPageId::from("appearance"),
            action_id: "save".into(),
        },
    );
}

#[test]
fn disabled_actions_and_modified_indicators_are_model_state() {
    let model = SettingsWindowModel::new(vec![
        SettingsSection::new("themes", "Themes").with_root_page(
            SettingsPage::new("themes", "Themes")
                .with_modified(true)
                .with_action(
                    SettingsPageAction::new("save", "Save")
                        .with_priority(SettingsPageActionPriority::Primary)
                        .disabled_with_reason("No staged changes"),
                )
                .with_row(
                    SettingsRow::new(
                        "active_theme",
                        "Active theme",
                        "Default",
                        SettingsFieldKind::Text,
                    )
                    .with_modified(true)
                    .with_action(
                        SettingsRowAction::new("delete", "Delete")
                            .disabled_with_reason("The active theme cannot be deleted"),
                    ),
                ),
        ),
    ])
    .expect("model should validate");

    let page = model.selected_page();
    let row = model
        .row(&SettingsFieldId::from("active_theme"))
        .expect("row should exist");

    assert!(page.is_modified());
    assert!(!page.actions()[0].is_enabled());
    assert_eq!(
        page.actions()[0].disabled_reason(),
        Some("No staged changes")
    );
    assert!(row.is_modified());
    assert!(!row.actions()[0].is_enabled());
    assert_eq!(
        row.actions()[0].disabled_reason(),
        Some("The active theme cannot be deleted"),
    );
}

#[test]
fn navigation_chevron_is_crate_owned_presentation() {
    let row = SettingsRow::navigation("theme_editor_link", "Theme editor", "theme_editor");

    assert_eq!(row.label(), "Theme editor");
    assert_eq!(
        row.navigation_target_page_id(),
        Some(&SettingsPageId::from("theme_editor")),
    );
    let render_source = include_str!("../src/panel/render.rs");
    assert!(render_source.contains("NAVIGATION_CHEVRON"));
    assert!(render_source.contains("const NAVIGATION_CHEVRON: &str = \"▸\";"));
    assert!(!render_source.contains("const NAVIGATION_CHEVRON: &str = \">\";"));
    assert!(render_source.contains("render_row_action_button(row.field_id().clone(), action, cx)"));
}

#[test]
fn row_resize_contract_keeps_right_controls_stable() {
    let render_source = include_str!("../src/panel/render.rs");

    assert!(render_source.contains("const TEXT_FIELD_CONTROL_WIDTH: f32 = 208.0;"));
    assert!(render_source.contains("const NUMERIC_FIELD_CONTROL_WIDTH: f32 = 96.0;"));
    assert!(render_source.contains("MULTILINE_FIELD_CONTROL_WIDTH"));
    assert!(render_source.contains("const MULTILINE_FIELD_CONTROL_WIDTH: f32 = 300.0;"));
    assert!(render_source.contains("COLOR_FIELD_CONTROL_WIDTH"));
    assert!(render_source.contains("const CHOICE_FIELD_CONTROL_WIDTH: f32 = 184.0;"));
    assert!(render_source.contains("ROW_CONTROL_GUTTER_WIDTH"));
    assert!(render_source.contains("ROW_LABEL_MIN_WIDTH"));
    assert!(render_source.contains("field_control_width(row.kind())"));
    assert!(render_source.contains("SettingsFieldKind::Number => NUMERIC_FIELD_CONTROL_WIDTH"));
    assert!(render_source.contains("SettingsFieldKind::Choice => CHOICE_FIELD_CONTROL_WIDTH"));
    assert!(render_source.contains("row_control_gutter()"));
    assert!(render_source.contains("render_row_label_stack"));
    assert!(render_source.contains(".whitespace_normal()"));
    assert!(render_source.contains("self.render_field_control("));
    assert!(render_source.contains("(!row.actions().is_empty()).then(||"));
    assert!(render_source.contains("ROW_ACTION_CLUSTER_MIN_WIDTH"));
    assert!(render_source.contains(".justify_end()"));
}

#[test]
fn action_bearing_text_rows_do_not_spend_label_width_on_actions() {
    let model = SettingsWindowModel::new(vec![
        SettingsSection::new("notifications", "Notifications").with_row(
            SettingsRow::new(
                "end_turn_sound",
                "End-turn sound",
                "C:\\Users\\operator\\Music\\Notifications\\very-long-completion-sound.wav",
                SettingsFieldKind::Text,
            )
            .with_action(SettingsRowAction::new("choose", "Choose..."))
            .with_action(SettingsRowAction::new("clear", "Clear")),
        ),
    ])
    .expect("model should validate");
    let row = model
        .row(&SettingsFieldId::from("end_turn_sound"))
        .expect("row should exist");

    assert_eq!(row.kind(), SettingsFieldKind::Text);
    assert_eq!(row.actions().len(), 2);

    let render_source = include_str!("../src/panel/render.rs");
    assert!(render_source.contains("field_row_stacks_actions_below_input(row)"));
    assert!(render_source.contains("let stack_actions_below_field"));
    assert!(render_source.contains(".when(stack_actions_below_field, |element| {"));
    assert!(render_source.contains("element.flex_col().items_end()"));
    assert!(
        render_source
            .contains("row.kind() == crate::SettingsFieldKind::Text && !row.actions().is_empty()")
    );
    assert!(render_source.contains("const TEXT_FIELD_CONTROL_WIDTH: f32 = 208.0;"));
    assert!(render_source.contains("const ROW_LABEL_MIN_WIDTH: f32 = 160.0;"));
    assert!(render_source.contains("const ROW_CONTROL_GUTTER_WIDTH: f32 = 24.0;"));

    let default_row_content_width = 500.0;
    let minimum_label_and_control_width = 160.0 + 24.0 + 208.0;
    assert!(
        minimum_label_and_control_width <= default_row_content_width,
        "the default/minimum window must leave real label width after reserving the field column"
    );
}

#[test]
fn default_window_size_is_the_supported_minimum() {
    let options = SettingsWindowOptions::default();

    assert_eq!(options.window_size(), (800.0, 520.0));
    assert_eq!(options.min_window_size(), options.window_size());
}

#[test]
fn page_header_keeps_actions_out_of_the_flexible_title_region() {
    let render_source = include_str!("../src/panel/render.rs");

    assert!(render_source.contains("self.render_back_button(target_page_id, cx)"));
    assert!(render_source.contains("self.render_page_action_button(page_id.clone(), action, cx)"));
    assert!(render_source.contains(".flex_wrap()"));
    assert!(
        render_source.contains(".flex_none()")
            && render_source.contains(".items_center()")
            && render_source.contains(".gap_2()")
            && render_source.contains(".pr_1()")
    );
}

#[test]
fn color_picker_saved_swatches_are_bounded_inside_popup() {
    let render_source = include_str!("../src/panel/color_render.rs");

    assert!(render_source.contains("COLOR_PICKER_SAVED_SWATCH_MAX_HEIGHT"));
    assert!(render_source.contains("COLOR_PICKER_SAVED_SWATCH_VISIBLE_ROWS"));
    assert!(render_source.contains(".max_h(px(COLOR_PICKER_SAVED_SWATCH_MAX_HEIGHT))"));
    assert!(render_source.contains(".overflow_y_scroll()"));
}

#[test]
fn field_synchronization_uses_keyed_retained_state() {
    let panel_source = include_str!("../src/panel.rs");

    assert!(panel_source.contains("fields: HashMap<SettingsFieldId, Entity<SettingsFieldInput>>"));
    assert!(panel_source.contains("let mut previous = std::mem::take(&mut self.fields);"));
    assert!(panel_source.contains("text_input_field_snapshots()"));
    assert!(panel_source.contains("previous.remove(&field.field_id)"));
    assert!(panel_source.contains("field.uses_text_input()"));
    assert!(!panel_source.contains("swap_remove(index)"));
}

#[test]
fn choice_fields_do_not_create_text_input_retained_state() {
    let render_source = include_str!("../src/panel/render.rs");
    let panel_source = include_str!("../src/panel.rs");

    assert!(render_source.contains("render_choice_control"));
    assert!(render_source.contains("render_choice_popup"));
    assert!(render_source.contains("popup.position(bounds.bottom_left())"));
    assert!(render_source.contains(".snap_to_window_with_margin(px(8.0))"));
    assert!(render_source.contains("SettingsFieldKind::Choice"));
    assert!(render_source.contains("select_choice_value"));
    assert!(panel_source.contains("choice_control_bounds"));
    assert!(panel_source.contains("record_choice_control_bounds"));
    assert!(panel_source.contains("row.uses_text_input()"));
}

#[test]
fn theme_editor_scrolling_renders_only_selected_page_rows() {
    let render_source = include_str!("../src/panel/render.rs");

    assert!(render_source.contains(".selected_rows()"));
    assert!(!render_source.contains(".model.rows()"));
}

#[test]
fn page_local_split_rendering_stays_inside_selected_page_body() {
    let render_source = include_str!("../src/panel/render.rs");

    assert!(render_source.contains("render_page_body"));
    assert!(render_source.contains("page.local_split().cloned()"));
    assert!(render_source.contains("render_page_local_split_list"));
    assert!(render_source.contains("render_detail_rows_scroll(DetailRowsLayout::SplitDetail"));
    assert!(render_source.contains("SettingsWindowEvent::PageSplitItemSelected"));
    assert!(render_source.contains("PAGE_LOCAL_SPLIT_LIST_WIDTH"));
    assert!(render_source.contains("page_local_split_render_window"));
    assert!(render_source.contains("render_page_local_split_window"));
    assert!(!render_source.contains(
        "local_split.items().iter().cloned().map(|item| self.render_page_local_split_item"
    ));
}

#[test]
fn stacked_custom_body_preserves_standard_detail_row_scroll_surface() {
    let render_source = include_str!("../src/panel/render.rs");

    assert!(render_source.contains("settings-page-stacked-custom-body"));
    assert!(render_source.contains("settings-page-stacked-custom-region"));
    assert!(render_source.contains("page.stacked_custom_body().cloned()"));
    assert!(render_source.contains("render_stacked_custom_body_region"));
    assert!(render_source.contains("page_body_renderer"));
    assert!(render_source.contains("renderer.render(&body_id)"));
    assert!(render_source.contains("custom_body.height_px()"));
    assert!(render_source.contains("render_detail_rows_scroll(DetailRowsLayout::Standard"));
}

#[test]
fn window_options_carry_page_body_renderer() {
    let renderer = gpui_settings_window::SettingsPageBodyRenderer::new(|_| {
        Some(gpui::div().into_any_element())
    });
    let options = SettingsWindowOptions::default().with_page_body_renderer(renderer);

    assert!(options.page_body_renderer().is_some());
}

#[test]
fn page_local_split_font_family_preview_hint_is_rendered() {
    let render_source = include_str!("../src/panel/render.rs");

    assert!(render_source.contains("style.font_family()"));
    assert!(render_source.contains(".font_family(font_family)"));
}

#[test]
fn secondary_detail_field_modified_state_is_rendered() {
    let render_source = include_str!("../src/panel/render.rs");
    let detail_source = render_source
        .split("fn render_detail_field_control")
        .nth(1)
        .expect("render_detail_field_control should exist")
        .split("fn render_navigation_row")
        .next()
        .expect("render_navigation_row should follow detail field rendering");

    assert!(render_source.contains("render_detail_field_control"));
    assert!(detail_source.contains(".is_modified()"));
    assert!(detail_source.contains("render_modified_indicator()"));
}

#[test]
fn page_local_split_detail_rows_use_narrow_layout() {
    let render_source = include_str!("../src/panel/render.rs");

    assert!(render_source.contains("enum DetailRowsLayout"));
    assert!(render_source.contains("SplitDetail"));
    assert!(render_source.contains("render_split_detail_field_row"));
    assert!(render_source.contains("render_detail_field_control"));
    assert!(render_source.contains("SPLIT_DETAIL_ROW_LABEL_MIN_WIDTH"));
    assert!(render_source.contains("render_row_label_stack_with_min_width"));
    assert!(render_source.contains("flexible_width: bool"));
    assert!(render_source.contains(".items_start()"));
    assert!(render_source.contains("row_control_gutter()"));

    let default_selected_body_width = 800.0 - 32.0 - 32.0 - 196.0 - 16.0;
    let split_detail_width = default_selected_body_width - 112.0 - 12.0;
    let row_inner_width = split_detail_width - 24.0;
    assert!(
        120.0 + 24.0 + 208.0 <= row_inner_width,
        "split detail rows must still fit a compact label, gutter, and widest single-line control"
    );
}

#[test]
fn scrollbar_activity_does_not_force_unconditional_panel_notify() {
    let source = include_str!("../src/panel/scrollbar.rs");

    assert!(
        source.contains("fn scrollbar_update_callback"),
        "managed scrollbar visibility must still have an owner repaint callback"
    );
    assert!(
        source.contains("cx.notify();"),
        "visibility transition callbacks should still notify the owner"
    );

    for (function, next_function) in [
        (
            "note_content_scrollbar_activity",
            "note_navigation_scrollbar_activity",
        ),
        (
            "note_navigation_scrollbar_activity",
            "note_split_scrollbar_activity",
        ),
        (
            "note_split_scrollbar_activity",
            "note_content_scrollbar_motion",
        ),
    ] {
        let body = source
            .split_once(function)
            .and_then(|(_, rest)| rest.split_once(next_function).map(|(body, _)| body))
            .unwrap_or_else(|| panic!("missing scrollbar activity function {function}"));

        assert!(
            body.contains("record_viewport_activity(window, cx, on_update);"),
            "{function} should continue reporting activity to managed scrollbar visibility"
        );
        assert!(
            !body.contains("cx.notify();"),
            "{function} should not force a panel repaint for every viewport activity event"
        );
    }
}

#[test]
fn window_options_carry_custom_visual_theme() {
    let mut theme = SettingsWindowTheme::default();
    theme.window_background = RgbColor::new(1, 2, 3);
    theme.primary_button.font_weight = 650;
    theme.primary_button.normal.background = RgbColor::new(4, 5, 6);

    let options = SettingsWindowOptions::default().with_visual_theme(theme.clone());

    assert_eq!(options.visual_theme(), &theme);
    assert_eq!(options.visual_theme().primary_button.font_weight, 650);
}
