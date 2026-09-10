#![cfg(feature = "egui")]
#![cfg(feature = "storybook-artifacts")]

use katana_ui_core::egui::text_command_surface::{
    ConsumerArtifactLeafId, ConsumerArtifactPlanError, EguiTextCommandSurfaceHostProjectionEncoder,
    EguiTextCommandSurfacePresentation, EguiTextCommandSurfacePresentationToken,
    TextCommandSurfaceStyle,
};
use katana_ui_core::text_surface::{
    TextSurface, TextSurfacePresentation, TextSurfaceProps, TextSurfaceViewport,
};

fn host_projected_token(revision: u64) -> EguiTextCommandSurfacePresentationToken {
    let surface = TextSurface::new(TextSurfaceProps::new(
        katana_ui_core::atom::TextArea::new("consumer-artifact-plan")
            .value("consumer artifact plan"),
        Vec::new(),
        TextSurfaceViewport::new(0, 0, 320, 180),
    ));
    EguiTextCommandSurfaceHostProjectionEncoder::token(
        revision,
        b"host-owned-target",
        EguiTextCommandSurfacePresentation {
            text_state_id: None,
            text: TextSurfacePresentation::from_props(surface.props()),
            toolbar: None,
            floating: None,
            search: None,
            context_menu: None,
        },
        TextCommandSurfaceStyle::standard().expect("standard style"),
    )
    .expect("opaque token")
}

mod foreign_consumer {
    use katana_ui_core::egui::text_command_surface::{
        ConsumerArtifactLeafId, ConsumerArtifactPlanError, ConsumerArtifactPlanIssuer,
        ConsumerArtifactPlanV1, ConsumerArtifactStageBinding,
        EguiTextCommandSurfacePresentationToken, FullTextCommandSurfaceScenarioSession,
        GenericEffectClass, GenericInteractionClass,
    };

    const KUC_CONSUMER_ARTIFACT_ACTION_TARGET: &str = "kuc.rich.inline-strong";

    pub(super) fn issue_stage_count(
        token: EguiTextCommandSurfacePresentationToken,
    ) -> Result<usize, ConsumerArtifactPlanError> {
        let plan = ConsumerArtifactPlanV1::new(
            1,
            vec![ConsumerArtifactStageBinding::new(
                ConsumerArtifactLeafId::new("source-derived-leaf")?,
                GenericInteractionClass::ImeCommit,
                GenericEffectClass::OpaqueForwarding,
                token,
            )],
        );
        ConsumerArtifactPlanIssuer::new()
            .issue(plan)
            .map(|issued| issued.remaining_stage_count())
    }

    pub(super) fn issue_full_plan_from_opaque_scenario_leases()
    -> Result<usize, Box<dyn std::error::Error>> {
        let session = FullTextCommandSurfaceScenarioSession::new_consumer_artifact();
        let mut leases = Vec::with_capacity(GenericInteractionClass::FULL_EDITOR_SEQUENCE.len());
        leases.push(session.retain_lease()?);
        for _ in 1..GenericInteractionClass::FULL_EDITOR_SEQUENCE.len() {
            leases.push(session.synchronize_lease()?);
        }
        let bindings = GenericInteractionClass::FULL_EDITOR_SEQUENCE
            .into_iter()
            .zip(leases)
            .enumerate()
            .map(|(index, (interaction, lease))| {
                Ok(ConsumerArtifactStageBinding::from_host_projection_lease(
                    ConsumerArtifactLeafId::new(format!("foreign-stage-{index}"))?,
                    KUC_CONSUMER_ARTIFACT_ACTION_TARGET,
                    interaction,
                    GenericEffectClass::NoHostEffect,
                    lease,
                ))
            })
            .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
        ConsumerArtifactPlanIssuer::new()
            .issue(ConsumerArtifactPlanV1::new(1, bindings))
            .map(|issued| issued.remaining_stage_count())
            .map_err(Into::into)
    }

    #[cfg(target_os = "linux")]
    pub(super) fn execute_full_plan_from_opaque_scenario_leases()
    -> Result<(), Box<dyn std::error::Error>> {
        let session = FullTextCommandSurfaceScenarioSession::new_consumer_artifact();
        let mut leases = Vec::with_capacity(GenericInteractionClass::FULL_EDITOR_SEQUENCE.len());
        leases.push(session.retain_lease()?);
        for _ in 1..GenericInteractionClass::FULL_EDITOR_SEQUENCE.len() {
            leases.push(session.synchronize_lease()?);
        }
        let bindings = GenericInteractionClass::FULL_EDITOR_SEQUENCE
            .into_iter()
            .zip(leases)
            .enumerate()
            .map(|(index, (interaction, lease))| {
                Ok(ConsumerArtifactStageBinding::from_host_projection_lease(
                    ConsumerArtifactLeafId::new(format!("foreign-stage-{index}"))?,
                    KUC_CONSUMER_ARTIFACT_ACTION_TARGET,
                    interaction,
                    GenericEffectClass::NoHostEffect,
                    lease,
                ))
            })
            .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
        let mut plan =
            ConsumerArtifactPlanIssuer::new().issue(ConsumerArtifactPlanV1::new(1, bindings))?;
        let output = tempfile::tempdir()?;
        let context = egui::Context::default();

        for _ in GenericInteractionClass::FULL_EDITOR_SEQUENCE {
            plan.execute_next(&context, output.path())?;
        }

        assert_eq!(0, plan.remaining_stage_count());
        Ok(())
    }
}

#[test]
fn foreign_consumer_is_rejected_before_opaque_forwarding_can_drop_events() {
    assert!(matches!(
        foreign_consumer::issue_stage_count(host_projected_token(1)),
        Err(ConsumerArtifactPlanError::UnsupportedEffectClass(
            katana_ui_core::egui::text_command_surface::GenericEffectClass::OpaqueForwarding
        ))
    ));
}

#[test]
fn consumer_plan_rejects_invalid_leaf_identifier() {
    assert!(matches!(
        ConsumerArtifactLeafId::new("\0"),
        Err(ConsumerArtifactPlanError::InvalidLeafId)
    ));
}

#[test]
fn foreign_consumer_can_issue_full_plan_from_opaque_scenario_leases() {
    assert_eq!(
        foreign_consumer::issue_full_plan_from_opaque_scenario_leases()
            .expect("opaque leases must create the full consumer plan"),
        10
    );
}

#[cfg(target_os = "linux")]
#[test]
fn foreign_consumer_executes_every_stage_from_opaque_scenario_leases() {
    foreign_consumer::execute_full_plan_from_opaque_scenario_leases()
        .expect("opaque leases must execute the complete consumer plan");
}
