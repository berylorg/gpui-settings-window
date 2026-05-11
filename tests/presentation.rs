use gpui_settings_window::{
    RgbColor, SettingsFieldId, SettingsFieldKind, SettingsRow, SettingsRowAction, SettingsSection,
    SettingsWindowError, SettingsWindowModel, SettingsWindowOptions, SettingsWindowTheme,
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
fn window_options_carry_custom_visual_theme() {
    let mut theme = SettingsWindowTheme::default();
    theme.window_background = RgbColor::new(1, 2, 3);
    theme.primary_button.normal.background = RgbColor::new(4, 5, 6);

    let options = SettingsWindowOptions::default().with_visual_theme(theme.clone());

    assert_eq!(options.visual_theme(), &theme);
}
