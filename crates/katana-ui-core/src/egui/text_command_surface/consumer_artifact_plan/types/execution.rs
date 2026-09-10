use super::stage_interactions::render_stage;
use super::support::{
    cleanup_stage_output, map_root_error, preflight_output, sha256, validate_decoded_png,
    write_manifest, write_stage_artifact,
};
use super::unicode_evidence::{bind_unicode_evidence, capture_unicode_evidence};
use super::{
    ConsumerArtifactEvidence, ConsumerArtifactForwardingReceipt, ConsumerArtifactLeafId,
    ConsumerArtifactPlanError, ConsumerArtifactStageBinding, EguiTextCommandSurfaceHostRoot,
};
use crate::egui::text_command_surface::KucUnicodeColorGlyphEvidenceOptions;
use std::collections::BTreeMap;
use std::path::Path;

/// Issued stages retain one KUC root and can only run in their KUC-defined order.
pub struct IssuedConsumerArtifactPlan {
    pub(super) root: EguiTextCommandSurfaceHostRoot,
    pub(super) bindings: Vec<ConsumerArtifactStageBinding>,
    pub(super) unicode_evidence_options: KucUnicodeColorGlyphEvidenceOptions,
    pub(super) next_stage: usize,
    pub(super) prepared_stage: Option<usize>,
    pub(super) failed_stage: Option<usize>,
    pub(super) root_revision: u64,
    pub(super) receipt_root_identity_fingerprint: Option<String>,
    pub(super) issuance_nonce: u64,
    pub(super) issued_receipts: BTreeMap<String, (ConsumerArtifactLeafId, String, u64)>,
}

impl IssuedConsumerArtifactPlan {
    #[must_use]
    pub fn remaining_stage_count(&self) -> usize {
        self.bindings.len().saturating_sub(self.next_stage)
    }

    /// Consumes one receipt only when it originated from this retained root.
    pub fn consume_forwarding_receipt_once(
        &self,
        receipt: ConsumerArtifactForwardingReceipt,
    ) -> Result<(), ConsumerArtifactPlanError> {
        let root_identity_fingerprint = self
            .receipt_root_identity_fingerprint
            .as_deref()
            .ok_or(ConsumerArtifactPlanError::ReceiptCrossBind)?;
        let (leaf, stage_id, root_revision) = self
            .issued_receipts
            .get(&receipt.fingerprint)
            .ok_or(ConsumerArtifactPlanError::ReceiptCrossBind)?;
        receipt.consume_once(root_identity_fingerprint, leaf, stage_id, *root_revision)
    }

    pub fn execute_next(
        &mut self,
        context: &egui::Context,
        output_dir: &Path,
    ) -> Result<ConsumerArtifactEvidence, ConsumerArtifactPlanError> {
        if let Some(stage) = self.failed_stage {
            return Err(ConsumerArtifactPlanError::StageExecutionFailed(stage));
        }
        let index = self.next_stage;
        let stage_id = format!("consumer-stage-{index:04}");
        preflight_output(output_dir, &stage_id)?;
        let (interaction, leaf, action_target) = {
            let binding = self
                .bindings
                .get_mut(index)
                .ok_or(ConsumerArtifactPlanError::PlanComplete)?;
            if index > 0 && self.prepared_stage != Some(index) {
                if let Some(lease) = binding.take_root_lease() {
                    self.root
                        .synchronize_with_lease(lease)
                        .map_err(map_root_error)?;
                } else {
                    let token = binding
                        .take_token()
                        .ok_or(ConsumerArtifactPlanError::StageAlreadyConsumed(index))?;
                    self.root.synchronize(token).map_err(map_root_error)?;
                }
                self.prepared_stage = Some(index);
            }
            (
                binding.interaction,
                binding.leaf.clone(),
                binding.action_target().to_owned(),
            )
        };
        self.failed_stage = Some(index);
        let result = (|| {
            let frame = render_stage(&mut self.root, context, interaction, action_target.as_str())?;
            let receipt = write_stage_artifact(&frame, output_dir, &stage_id)?;
            let artifact = receipt.artifact().clone();
            validate_decoded_png(&artifact)?;
            let root_revision = self.root_revision + index as u64;
            let root_identity_fingerprint = sha256(frame.record().identity().as_bytes());
            self.receipt_root_identity_fingerprint = Some(root_identity_fingerprint.clone());
            let receipt_fingerprint = sha256(
                format!(
                    "{}:{root_identity_fingerprint}:{}:{}:{}",
                    self.issuance_nonce,
                    leaf.0,
                    stage_id,
                    frame.record().record_hash()
                )
                .as_bytes(),
            );
            let unicode_json = capture_unicode_evidence(self.unicode_evidence_options.clone())?;
            let unicode_hash = bind_unicode_evidence(
                &unicode_json,
                &stage_id,
                &leaf,
                root_revision,
                frame.record().record_hash(),
                frame.record().accessibility_snapshot_hash(),
                &receipt_fingerprint,
            );
            let evidence = ConsumerArtifactEvidence {
                stage_id: stage_id.clone(),
                leaf: leaf.clone(),
                root_revision,
                png_sha256: artifact.png_sha256().to_owned(),
                pixel_hash: artifact.pixel_hash().to_owned(),
                root_record_hash: artifact.root_record_hash().to_owned(),
                accesskit_snapshot_hash: frame.record().accessibility_snapshot_hash().to_owned(),
                unicode_evidence_hash: unicode_hash,
                unicode_evidence_json: unicode_json,
                receipt: ConsumerArtifactForwardingReceipt {
                    leaf: leaf.clone(),
                    stage_id: stage_id.clone(),
                    root_revision: self.root_revision + index as u64,
                    root_identity_fingerprint,
                    consumed: false,
                    fingerprint: receipt_fingerprint,
                },
            };
            write_manifest(output_dir, &evidence)?;
            Ok(evidence)
        })();
        match result {
            Ok(evidence) => {
                self.issued_receipts.insert(
                    evidence.receipt.fingerprint.clone(),
                    (
                        evidence.receipt.leaf.clone(),
                        evidence.receipt.stage_id.clone(),
                        evidence.receipt.root_revision,
                    ),
                );
                self.next_stage += 1;
                self.prepared_stage = None;
                self.failed_stage = None;
                Ok(evidence)
            }
            Err(error) => {
                cleanup_stage_output(output_dir, &stage_id)?;
                Err(error)
            }
        }
    }
}

pub(super) fn show_frame(
    root: &mut EguiTextCommandSurfaceHostRoot,
    context: &egui::Context,
    input: egui::RawInput,
) -> Result<
    crate::egui::text_command_surface::EguiTextCommandSurfaceHostRootFrame,
    ConsumerArtifactPlanError,
> {
    let mut frame = None;
    let mut output = context.run_ui(input, |ui| {
        frame = Some(root.show(ui));
    });
    output.textures_delta.clear();
    frame
        .ok_or(ConsumerArtifactPlanError::MissingFrame)?
        .map_err(map_root_error)
}

pub(super) fn interaction_error(error: impl std::fmt::Display) -> ConsumerArtifactPlanError {
    ConsumerArtifactPlanError::Artifact(format!("KUC interaction protocol failed: {error}"))
}
