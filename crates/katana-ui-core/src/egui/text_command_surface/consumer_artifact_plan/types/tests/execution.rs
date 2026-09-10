use super::super::execution::interaction_error;
use super::super::support::{map_root_error, sha256};
use super::super::text_interactions::{ensure_record_changed, ensure_scroll_changed};
use super::super::unicode_evidence::{
    artifact_unicode_evidence_options, bind_unicode_evidence, capture_unicode_evidence,
};
use super::super::*;
use super::{binding, binding_from_encoder_with_interaction, complete_bindings, temp_dir, token};
#[cfg(target_os = "linux")]
use super::{binding_from_encoder, complete_semantic_bindings};
use crate::egui::text_command_surface::EguiTextCommandSurfaceRootFactoryError;
use crate::egui::text_command_surface::KucUnicodeColorGlyphEvidenceOptions;

#[cfg(target_os = "linux")]
const SHA256_HEX_LENGTH: usize = 64;

#[cfg(target_os = "linux")]
fn assert_stage_evidence(
    evidence: &ConsumerArtifactEvidence,
    output_dir: &std::path::Path,
    stage_id: &str,
    leaf: &str,
    revision: u64,
) {
    assert_eq!(evidence.stage_id(), stage_id);
    assert_eq!(evidence.root_revision(), revision);
    assert_eq!(evidence.leaf().as_str(), leaf);
    assert_eq!(evidence.png_sha256().len(), SHA256_HEX_LENGTH);
    assert_eq!(evidence.pixel_hash().len(), SHA256_HEX_LENGTH);
    assert_eq!(evidence.root_record_hash().len(), SHA256_HEX_LENGTH);
    assert_eq!(evidence.accesskit_snapshot_hash().len(), SHA256_HEX_LENGTH);
    assert_eq!(evidence.unicode_evidence_hash().len(), SHA256_HEX_LENGTH);
    let unicode_evidence: serde_json::Value =
        serde_json::from_slice(evidence.unicode_evidence_json()).expect("Unicode evidence JSON");
    assert!(unicode_evidence["catalog_face"]["source_file_path"].is_null());
    assert!(output_dir.join(format!("{stage_id}.png")).is_file());
    assert!(
        output_dir
            .join(format!("{stage_id}.consumer-artifact.json"))
            .is_file()
    );
    assert!(
        output_dir
            .join(format!("{stage_id}.unicode-evidence.json"))
            .is_file()
    );
}

#[test]
#[cfg(target_os = "linux")]
fn issued_plan_executes_stages_and_collects_expected_artifact_evidence() {
    let mut bindings = complete_bindings(7, b"consumer-target");
    bindings[0].leaf = ConsumerArtifactLeafId::new("leaf-a").expect("leaf");
    bindings[1].leaf = ConsumerArtifactLeafId::new("leaf-b").expect("leaf");
    let mut plan = ConsumerArtifactPlanIssuer::new()
        .issue(ConsumerArtifactPlanV1::new(7, bindings))
        .expect("plan should issue");
    assert_eq!(plan.remaining_stage_count(), 10);

    let context = egui::Context::default();
    let output_dir = temp_dir("execute-success");
    let first = plan
        .execute_next(&context, output_dir.as_path())
        .expect("first stage should execute with the pinned Linux color emoji");
    assert_stage_evidence(
        &first,
        output_dir.as_path(),
        "consumer-stage-0000",
        "leaf-a",
        7,
    );
    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(output_dir.join("consumer-stage-0000.consumer-artifact.json"))
            .expect("manifest should read"),
    )
    .expect("manifest should parse");
    assert_eq!(
        manifest["stage_id"],
        serde_json::json!("consumer-stage-0000")
    );
    assert_eq!(plan.remaining_stage_count(), 9);

    let second = plan
        .execute_next(&context, output_dir.as_path())
        .expect("second stage should execute");
    assert_stage_evidence(
        &second,
        output_dir.as_path(),
        "consumer-stage-0001",
        "leaf-b",
        8,
    );
    assert_eq!(plan.remaining_stage_count(), 8);
    assert_eq!(
        plan.consume_forwarding_receipt_once(second.into_forwarding_receipt()),
        Ok(())
    );
}

#[test]
#[cfg(target_os = "linux")]
fn forwarding_receipts_reject_a_different_issued_plan() {
    let context = egui::Context::default();
    let mut first_plan = ConsumerArtifactPlanIssuer::new()
        .issue(ConsumerArtifactPlanV1::new(
            1,
            complete_bindings(1, b"shared-receipt-root"),
        ))
        .expect("first plan");
    let mut second_plan = ConsumerArtifactPlanIssuer::new()
        .issue(ConsumerArtifactPlanV1::new(
            1,
            complete_bindings(1, b"shared-receipt-root"),
        ))
        .expect("second plan");
    let first = first_plan
        .execute_next(&context, temp_dir("first-receipt-root").as_path())
        .expect("first evidence");
    second_plan
        .execute_next(&context, temp_dir("second-receipt-root").as_path())
        .expect("second evidence");

    assert!(matches!(
        second_plan.consume_forwarding_receipt_once(first.into_forwarding_receipt()),
        Err(ConsumerArtifactPlanError::ReceiptCrossBind)
    ));
}

#[test]
#[cfg(target_os = "linux")]
fn semantic_interaction_stages_use_the_retained_root_locator() {
    let context = egui::Context::default();
    let mut plan = ConsumerArtifactPlanIssuer::new()
        .issue(ConsumerArtifactPlanV1::new(
            1,
            complete_semantic_bindings(1, b"semantic-target"),
        ))
        .expect("semantic plan");
    let output_dir = temp_dir("semantic-sequence");
    for _ in GenericInteractionClass::FULL_EDITOR_SEQUENCE {
        plan.execute_next(&context, &output_dir)
            .expect("semantic stage follows the KUC-issued locator protocol");
    }
    assert_eq!(plan.remaining_stage_count(), 0);
    assert!(matches!(
        plan.execute_next(&context, &output_dir),
        Err(ConsumerArtifactPlanError::PlanComplete)
    ));
}

#[test]
#[cfg(target_os = "linux")]
fn semantic_plan_fails_closed_when_a_bound_action_is_unavailable() {
    let mut bindings = complete_semantic_bindings(1, b"unavailable-action-target");
    let binding = &mut bindings[4];
    let replacement = ConsumerArtifactStageBinding::new_with_action_target(
        binding.leaf.clone(),
        "kuc.consumer-artifact.missing-action",
        binding.interaction,
        binding.effect,
        binding.token.take().expect("toolbar token"),
    );
    bindings[4] = replacement;
    let mut plan = ConsumerArtifactPlanIssuer::new()
        .issue(ConsumerArtifactPlanV1::new(1, bindings))
        .expect("plan should issue before any frame is rendered");
    let context = egui::Context::default();
    let output_dir = temp_dir("unavailable-bound-action");
    for _ in 0..4 {
        plan.execute_next(&context, &output_dir)
            .expect("preceding KUC stages should execute");
    }
    assert!(matches!(
        plan.execute_next(&context, &output_dir),
        Err(ConsumerArtifactPlanError::Artifact(_))
    ));
    assert!(matches!(
        plan.execute_next(&context, &output_dir),
        Err(ConsumerArtifactPlanError::StageExecutionFailed(4))
    ));
}

#[test]
fn issuer_rejects_invalid_schema_empty_duplicate_and_overflow_plans() {
    let issuer = ConsumerArtifactPlanIssuer::new();
    assert!(matches!(
        issuer.issue(ConsumerArtifactPlanV1::with_schema_version(
            2,
            1,
            Vec::new()
        )),
        Err(ConsumerArtifactPlanError::UnsupportedSchemaVersion(2))
    ));
    assert!(matches!(
        issuer.issue(ConsumerArtifactPlanV1::new(1, Vec::new())),
        Err(ConsumerArtifactPlanError::EmptyPlan)
    ));
    assert!(matches!(
        issuer.issue(ConsumerArtifactPlanV1::new(
            1,
            vec![binding("leaf", 1), binding("leaf", 2)],
        )),
        Err(ConsumerArtifactPlanError::DuplicateLeaf(value)) if value == "leaf"
    ));
    assert!(matches!(
        issuer.issue(ConsumerArtifactPlanV1::new(
            u64::MAX,
            GenericInteractionClass::FULL_EDITOR_SEQUENCE
                .into_iter()
                .enumerate()
                .map(|(index, interaction)| {
                    ConsumerArtifactStageBinding::new(
                        ConsumerArtifactLeafId::new(format!("overflow-{index}")).expect("leaf"),
                        interaction,
                        GenericEffectClass::NoHostEffect,
                        token(if index == 0 { u64::MAX } else { 0 }),
                    )
                })
                .collect(),
        )),
        Err(ConsumerArtifactPlanError::RevisionOverflow)
    ));
}

#[test]
fn issuer_requires_the_complete_ordered_full_editor_sequence() {
    let issuer = ConsumerArtifactPlanIssuer::new();
    let complete = complete_bindings(1, b"complete-sequence-target");
    assert!(
        issuer
            .issue(ConsumerArtifactPlanV1::new(1, complete))
            .is_ok()
    );

    let mut missing = complete_bindings(1, b"complete-sequence-target");
    missing.pop();
    assert!(matches!(
        issuer.issue(ConsumerArtifactPlanV1::new(1, missing)),
        Err(ConsumerArtifactPlanError::IncompleteStageSequence)
    ));

    let mut reordered = complete_bindings(1, b"complete-sequence-target");
    reordered.swap(0, 1);
    assert!(matches!(
        issuer.issue(ConsumerArtifactPlanV1::new(1, reordered)),
        Err(ConsumerArtifactPlanError::IncompleteStageSequence)
    ));

    let mut repeated = complete_bindings(1, b"complete-sequence-target");
    repeated[1].interaction = repeated[0].interaction;
    assert!(matches!(
        issuer.issue(ConsumerArtifactPlanV1::new(1, repeated)),
        Err(ConsumerArtifactPlanError::IncompleteStageSequence)
    ));
}

#[test]
fn issuer_fails_closed_for_opaque_forwarding_without_a_transport_forwarder() {
    let opaque_binding = ConsumerArtifactStageBinding::new(
        ConsumerArtifactLeafId::new("opaque-forwarding").expect("leaf"),
        GenericInteractionClass::ToolbarActivation,
        GenericEffectClass::OpaqueForwarding,
        token(1),
    );

    assert!(matches!(
        ConsumerArtifactPlanIssuer::new()
            .issue(ConsumerArtifactPlanV1::new(1, vec![opaque_binding],)),
        Err(ConsumerArtifactPlanError::UnsupportedEffectClass(
            GenericEffectClass::OpaqueForwarding
        ))
    ));
}

#[test]
fn issuer_default_and_defensive_effect_display_remain_explicit() {
    assert!(matches!(
        ConsumerArtifactPlanIssuer::default().issue(ConsumerArtifactPlanV1::new(1, Vec::new())),
        Err(ConsumerArtifactPlanError::EmptyPlan)
    ));
    assert!(matches!(
        ConsumerArtifactPlanIssuer::new()
            .clone()
            .issue(ConsumerArtifactPlanV1::new(1, Vec::new())),
        Err(ConsumerArtifactPlanError::EmptyPlan)
    ));
    assert_eq!(
        ConsumerArtifactPlanError::UnsupportedEffectClass(GenericEffectClass::NoHostEffect)
            .to_string(),
        "consumer artifact effect class is unsupported"
    );
}

#[test]
fn text_interaction_requires_a_record_change() {
    assert_eq!(ensure_record_changed("before", "after"), Ok(()));
    assert!(matches!(
        ensure_record_changed("same", "same"),
        Err(ConsumerArtifactPlanError::Artifact(message)) if message == "KUC interaction protocol failed: text interaction did not change the retained text target"
    ));
}

#[test]
fn scroll_interaction_requires_a_retained_viewport_offset_change() {
    assert_eq!(ensure_scroll_changed(0, 24), Ok(()));
    assert!(matches!(
        ensure_scroll_changed(0, 0),
        Err(ConsumerArtifactPlanError::Artifact(message)) if message == "KUC interaction protocol failed: scroll interaction did not move the retained viewport"
    ));
}

#[test]
fn issuer_rejects_stale_or_consumed_stages_before_root_retention() {
    let mut stale = complete_bindings(4, b"stale-target");
    stale[0] = binding_from_encoder_with_interaction(
        "leaf",
        3,
        b"stale-target",
        GenericInteractionClass::TextInput,
    );
    assert!(matches!(
        ConsumerArtifactPlanIssuer::new().issue(ConsumerArtifactPlanV1::new(4, stale)),
        Err(ConsumerArtifactPlanError::StaleRevision {
            stage: 0,
            expected: 4,
            actual: 3,
        })
    ));
    let mut plan = ConsumerArtifactPlanV1::new(1, complete_bindings(1, b"consumed-target"));
    plan.bindings[1].token = None;
    assert!(matches!(
        ConsumerArtifactPlanIssuer::new().issue(plan),
        Err(ConsumerArtifactPlanError::StageAlreadyConsumed(1))
    ));
}

#[test]
fn root_errors_are_mapped_to_artifact_plan_errors() {
    for error in [
        EguiTextCommandSurfaceRootFactoryError::IdentityChanged,
        EguiTextCommandSurfaceRootFactoryError::StaleRevision {
            current: 4,
            received: 3,
        },
        EguiTextCommandSurfaceRootFactoryError::RevisionConflict { revision: 1 },
    ] {
        assert_eq!(
            map_root_error(error),
            ConsumerArtifactPlanError::TokenRootMismatch
        );
    }
    assert!(matches!(
        map_root_error(EguiTextCommandSurfaceRootFactoryError::Root("kuc root".to_string())),
        ConsumerArtifactPlanError::Root(reason) if reason == "kuc root"
    ));
}

#[test]
fn interaction_errors_are_mapped_to_artifact_plan_errors() {
    assert_eq!(
        interaction_error("semantic action was rejected"),
        ConsumerArtifactPlanError::Artifact(
            "KUC interaction protocol failed: semantic action was rejected".to_owned()
        )
    );
}

#[test]
#[cfg(target_os = "linux")]
fn execute_next_rejects_existing_output_and_issuer_rejects_changed_root_identity() {
    let mut collision_plan = ConsumerArtifactPlanIssuer::new()
        .issue(ConsumerArtifactPlanV1::new(
            11,
            complete_bindings(11, b"consumer-target-collision"),
        ))
        .expect("plan should issue");
    let context = egui::Context::default();
    let collision_dir = temp_dir("preflight");
    let existing = collision_dir.join("consumer-stage-0000.png");
    std::fs::write(&existing, b"occupied").expect("write marker");
    match collision_plan.execute_next(&context, collision_dir.as_path()) {
        Err(ConsumerArtifactPlanError::ExistingMedia(path)) => assert_eq!(path, existing),
        Err(other) => panic!("expected output collision, got {other}"),
        Ok(_) => panic!("existing output must reject execution"),
    }

    let mut changed_root = complete_bindings(1, b"same-target");
    changed_root[1] = binding_from_encoder("leaf-sync-2", 2, b"changed-target");
    assert!(matches!(
        ConsumerArtifactPlanIssuer::new().issue(ConsumerArtifactPlanV1::new(1, changed_root)),
        Err(ConsumerArtifactPlanError::TokenRootMismatch)
    ));
}

#[test]
#[cfg(target_os = "linux")]
fn execute_next_maps_artifact_write_failure() {
    let mut plan = ConsumerArtifactPlanIssuer::new()
        .issue(ConsumerArtifactPlanV1::new(
            1,
            complete_bindings(1, b"consumer-target"),
        ))
        .expect("plan should issue");
    let context = egui::Context::default();
    let output_file = temp_dir("write-failure").join("not-a-directory");
    std::fs::write(&output_file, b"file").expect("write output file");
    assert!(matches!(
        plan.execute_next(&context, output_file.as_path()),
        Err(ConsumerArtifactPlanError::Artifact(_))
    ));
    assert!(matches!(
        plan.execute_next(&context, output_file.as_path()),
        Err(ConsumerArtifactPlanError::StageExecutionFailed(0))
    ));
    assert_eq!(plan.remaining_stage_count(), 10);
}

#[test]
fn unicode_evidence_fails_closed_without_a_pinned_face() {
    let mut options = KucUnicodeColorGlyphEvidenceOptions::default();
    options.config.emoji_candidate_sha256.clear();
    assert!(matches!(
        capture_unicode_evidence(options),
        Err(ConsumerArtifactPlanError::UnicodeEvidence(_))
    ));
}

#[test]
fn unicode_evidence_commitment_binds_the_executed_stage_and_receipt() {
    let leaf = ConsumerArtifactLeafId::new("unicode-stage").expect("leaf");
    let unicode = r#"{"glyph":"日本語⭐️"}"#.as_bytes();
    let commitment = bind_unicode_evidence(
        unicode,
        "consumer-stage-0000",
        &leaf,
        7,
        "root-record",
        "accesskit-snapshot",
        "receipt",
    );
    assert_ne!(commitment, sha256(unicode));
    assert_ne!(
        commitment,
        bind_unicode_evidence(
            unicode,
            "consumer-stage-0001",
            &leaf,
            7,
            "root-record",
            "accesskit-snapshot",
            "receipt",
        )
    );
    assert_ne!(
        commitment,
        bind_unicode_evidence(
            unicode,
            "consumer-stage-0000",
            &leaf,
            8,
            "root-record-next",
            "accesskit-snapshot-next",
            "receipt-next",
        )
    );
}

#[test]
fn execute_next_requires_a_trusted_platform_color_emoji_pin() {
    let mut plan = ConsumerArtifactPlanIssuer::new()
        .issue(ConsumerArtifactPlanV1::new(
            1,
            complete_bindings(1, b"non-linux-target"),
        ))
        .expect("plan should issue");
    let context = egui::Context::default();
    let output_dir = temp_dir("unpinned-color-emoji");
    let has_trusted_pin = !artifact_unicode_evidence_options()
        .config
        .emoji_candidate_sha256
        .is_empty();
    let result = plan.execute_next(&context, output_dir.as_path());
    if has_trusted_pin {
        result.expect("a trusted platform pin must allow Unicode evidence capture");
        assert_eq!(plan.remaining_stage_count(), 9);
    } else {
        assert!(matches!(
            result,
            Err(ConsumerArtifactPlanError::UnicodeEvidence(_))
        ));
        assert_eq!(plan.remaining_stage_count(), 10);
    }
}

#[test]
fn issuer_uses_a_caller_supplied_color_emoji_pin() {
    let mut options = KucUnicodeColorGlyphEvidenceOptions::default();
    let Some((candidate, hash)) = options.config.emoji_candidates.iter().find_map(|path| {
        std::fs::read(path).ok().map(|bytes| {
            (
                path.clone(),
                crate::text_raster::PlatformFontSha256::digest(&bytes),
            )
        })
    }) else {
        return;
    };
    options.config.emoji_candidates = vec![candidate];
    options.config.emoji_candidate_sha256 = vec![hash];
    let mut plan = ConsumerArtifactPlanIssuer::with_unicode_evidence_options(options)
        .issue(ConsumerArtifactPlanV1::new(
            1,
            complete_bindings(1, b"consumer-target"),
        ))
        .expect("plan should issue");

    let result = plan.execute_next(
        &egui::Context::default(),
        temp_dir("caller-supplied-pin").as_path(),
    );
    assert!(
        result.is_ok(),
        "a caller-supplied pin for a readable platform emoji font must be accepted"
    );
}

#[test]
fn issuer_rejects_a_caller_supplied_color_emoji_pin_that_does_not_match_loaded_bytes() {
    let mut options = KucUnicodeColorGlyphEvidenceOptions::default();
    let Some(candidate) = options
        .config
        .emoji_candidates
        .iter()
        .find(|path| path.is_file())
        .cloned()
    else {
        return;
    };
    options.config.emoji_candidates = vec![candidate];
    options.config.emoji_candidate_sha256 = vec![crate::text_raster::PlatformFontSha256::digest(
        b"substituted color emoji font",
    )];
    let mut plan = ConsumerArtifactPlanIssuer::with_unicode_evidence_options(options)
        .issue(ConsumerArtifactPlanV1::new(
            1,
            complete_bindings(1, b"consumer-target"),
        ))
        .expect("plan should issue");

    let error = match plan.execute_next(
        &egui::Context::default(),
        temp_dir("mismatched-caller-pin").as_path(),
    ) {
        Ok(_) => panic!("a mismatched caller pin must not produce evidence"),
        Err(error) => error,
    };
    assert!(
        matches!(error, ConsumerArtifactPlanError::UnicodeEvidence(_)),
        "{error}"
    );
    assert_eq!(plan.remaining_stage_count(), 10);
}

#[test]
fn artifact_unicode_options_preserve_only_build_time_or_caller_supplied_pins() {
    let default_options = KucUnicodeColorGlyphEvidenceOptions::default();
    let options = artifact_unicode_evidence_options();
    assert_eq!(
        options.config.emoji_candidates,
        default_options.config.emoji_candidates
    );
    assert_eq!(
        options.config.emoji_candidate_sha256,
        default_options.config.emoji_candidate_sha256
    );
}
