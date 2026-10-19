use gpui_settings_window::{
    SettingsActionAvailability, SettingsFieldAvailability, SettingsFieldId, SettingsFieldKind,
    SettingsFieldValidity, SettingsFooterCommand, SettingsInteractionGate, SettingsRow,
    SettingsRowDetailField, SettingsSection, SettingsWindowError, SettingsWindowModel,
};

#[test]
fn field_limits_measure_normalized_utf8_for_primary_and_detail_values() {
    let model = SettingsWindowModel::new(vec![
        SettingsSection::new("agent", "Agent").with_row(
            SettingsRow::new("label", "Label", "a\r\n\r\nb", SettingsFieldKind::Text)
                .with_value_byte_limit(Some(3))
                .with_detail_field(
                    SettingsRowDetailField::new(
                        "instructions",
                        "é\r\nx",
                        SettingsFieldKind::MultilineText,
                    )
                    .with_value_byte_limit(Some(4)),
                ),
        ),
    ])
    .unwrap();
    assert_eq!(model.field_value_byte_limit(&"label".into()), Some(3));
    let invalid = SettingsWindowModel::new(vec![
        SettingsSection::new("agent", "Agent").with_row(
            SettingsRow::new("label", "Label", "ok", SettingsFieldKind::Text).with_detail_field(
                SettingsRowDetailField::new(
                    "instructions",
                    "é\r\nx",
                    SettingsFieldKind::MultilineText,
                )
                .with_value_byte_limit(Some(3)),
            ),
        ),
    ]);
    assert_eq!(
        invalid.unwrap_err(),
        SettingsWindowError::FieldValueLimitExceeded {
            field_id: "instructions".into(),
            limit: 3,
            value_bytes: 4
        }
    );
}

#[test]
fn refused_model_value_update_leaves_the_original_coherent_field_value() {
    let mut model = SettingsWindowModel::new(vec![
        SettingsSection::new("agent", "Agent").with_row(
            SettingsRow::new(
                "instructions",
                "Instructions",
                "abc",
                SettingsFieldKind::MultilineText,
            )
            .with_value_byte_limit(Some(3)),
        ),
    ])
    .unwrap();
    assert!(matches!(
        model.set_row_value(&"instructions".into(), "éé"),
        Err(SettingsWindowError::FieldValueLimitExceeded { value_bytes: 4, .. })
    ));
    assert_eq!(model.field_value(&"instructions".into()), Some("abc"));
    model
        .set_row_value(&"instructions".into(), "é\r\n")
        .unwrap();
    assert_eq!(model.field_value(&"instructions".into()), Some("é\r\n"));
}

#[test]
fn field_validity_availability_and_footer_commands_are_independent_of_the_gate() {
    let model = SettingsWindowModel::new(vec![
        SettingsSection::new("agent", "Agent").with_row(
            SettingsRow::new("primary", "Primary", "abc", SettingsFieldKind::Text)
                .with_availability(SettingsFieldAvailability::unavailable("Source unavailable"))
                .with_validity(SettingsFieldValidity::Valid)
                .with_error("A status message")
                .with_detail_field(
                    SettingsRowDetailField::new("detail", "invalid draft", SettingsFieldKind::Text)
                        .with_validity(SettingsFieldValidity::Invalid),
                ),
        ),
    ])
    .unwrap()
    .with_footer_availability(
        SettingsFooterCommand::Apply,
        SettingsActionAvailability::disabled_with_reason("Draft invalid"),
    )
    .with_interaction_gate(SettingsInteractionGate::gated("Reconciling"));
    let detail = SettingsFieldId::from("detail");
    assert!(model.field_availability(&detail).unwrap().is_available());
    assert_eq!(
        model.field_validity(&detail),
        Some(SettingsFieldValidity::Invalid)
    );
    assert!(
        model
            .footer_availability(SettingsFooterCommand::Cancel)
            .is_enabled()
    );
    assert_eq!(
        model
            .footer_availability(SettingsFooterCommand::Apply)
            .disabled_reason(),
        Some("Draft invalid")
    );
    let model = model.with_interaction_gate(SettingsInteractionGate::Open);
    assert!(
        !model
            .field_availability(&"primary".into())
            .unwrap()
            .is_available()
    );
    assert!(
        !model
            .footer_availability(SettingsFooterCommand::Apply)
            .is_enabled()
    );
}
