use super::support::{map_root_error, next_issued_plan_identity};
use super::{
    ConsumerArtifactPlanError, ConsumerArtifactPlanV1, GenericEffectClass, GenericInteractionClass,
    IssuedConsumerArtifactPlan, SCHEMA_VERSION,
};
use crate::egui::text_command_surface::{
    EguiTextCommandSurfaceRootFactory, KucUnicodeColorGlyphEvidenceOptions,
};
use std::collections::{BTreeMap, BTreeSet};

/// KUC issuer for opaque consumer artifact plans.
#[derive(Debug, Clone)]
pub struct ConsumerArtifactPlanIssuer {
    pub(super) unicode_evidence_options: KucUnicodeColorGlyphEvidenceOptions,
}

impl ConsumerArtifactPlanIssuer {
    #[must_use]
    pub fn new() -> Self {
        Self::with_unicode_evidence_options(
            super::unicode_evidence::artifact_unicode_evidence_options(),
        )
    }

    /// Supplies a release-verified color-emoji pin for artifact Unicode evidence.
    #[must_use]
    pub fn with_unicode_evidence_options(
        unicode_evidence_options: KucUnicodeColorGlyphEvidenceOptions,
    ) -> Self {
        Self {
            unicode_evidence_options,
        }
    }
}

impl Default for ConsumerArtifactPlanIssuer {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsumerArtifactPlanIssuer {
    pub fn issue(
        &self,
        mut plan: ConsumerArtifactPlanV1,
    ) -> Result<IssuedConsumerArtifactPlan, ConsumerArtifactPlanError> {
        if plan.schema_version != SCHEMA_VERSION {
            return Err(ConsumerArtifactPlanError::UnsupportedSchemaVersion(
                plan.schema_version,
            ));
        }
        if plan.bindings.is_empty() {
            return Err(ConsumerArtifactPlanError::EmptyPlan);
        }
        let mut leaves = BTreeSet::new();
        for binding in &plan.bindings {
            if binding.effect != GenericEffectClass::NoHostEffect {
                return Err(ConsumerArtifactPlanError::UnsupportedEffectClass(
                    binding.effect,
                ));
            }
            if !leaves.insert(binding.leaf.clone()) {
                return Err(ConsumerArtifactPlanError::DuplicateLeaf(
                    binding.leaf.0.clone(),
                ));
            }
        }
        if plan.bindings.len() != GenericInteractionClass::FULL_EDITOR_SEQUENCE.len()
            || plan
                .bindings
                .iter()
                .zip(GenericInteractionClass::FULL_EDITOR_SEQUENCE)
                .any(|(binding, expected)| binding.interaction != expected)
        {
            return Err(ConsumerArtifactPlanError::IncompleteStageSequence);
        }
        for (index, binding) in plan.bindings.iter().enumerate() {
            let expected = plan
                .initial_revision
                .checked_add(index as u64)
                .ok_or(ConsumerArtifactPlanError::RevisionOverflow)?;
            let actual = binding
                .token()
                .ok_or(ConsumerArtifactPlanError::StageAlreadyConsumed(index))?
                .revision();
            if actual != expected {
                return Err(ConsumerArtifactPlanError::StaleRevision {
                    stage: index,
                    expected,
                    actual,
                });
            }
        }
        let factory = EguiTextCommandSurfaceRootFactory::new();
        let first_token = plan.bindings[0]
            .token()
            .ok_or(ConsumerArtifactPlanError::StageAlreadyConsumed(0))?;
        for (index, binding) in plan.bindings.iter().enumerate().skip(1) {
            let token = binding
                .token()
                .ok_or(ConsumerArtifactPlanError::StageAlreadyConsumed(index))?;
            if !factory
                .has_same_root_identity(first_token, token)
                .map_err(map_root_error)?
            {
                return Err(ConsumerArtifactPlanError::TokenRootMismatch);
            }
        }
        let root = if let Some(lease) = plan.bindings[0].take_root_lease() {
            factory.retain_with_lease(lease)
        } else {
            let token = plan.bindings[0]
                .take_token()
                .ok_or(ConsumerArtifactPlanError::StageAlreadyConsumed(0))?;
            factory.retain(token)
        }
        .map_err(map_root_error)?;
        Ok(IssuedConsumerArtifactPlan {
            root,
            bindings: plan.bindings,
            unicode_evidence_options: self.unicode_evidence_options.clone(),
            next_stage: 0,
            prepared_stage: None,
            failed_stage: None,
            root_revision: plan.initial_revision,
            receipt_root_identity_fingerprint: None,
            issuance_nonce: next_issued_plan_identity(),
            issued_receipts: BTreeMap::new(),
        })
    }
}
