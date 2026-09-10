//! Consumer-defined full-editor artifacts without consumer-owned input or rendering.

use super::super::EguiTextCommandSurfaceHostRoot;
use std::path::PathBuf;

pub(super) const SCHEMA_VERSION: u16 = 1;
pub(super) const MAX_LEAF_IDENTIFIER_LENGTH: usize = 256;

/// KUC-owned generic interaction categories accepted by artifact plans.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GenericInteractionClass {
    TextInput,
    ImeCommit,
    Selection,
    Scroll,
    ToolbarActivation,
    FloatingToolbar,
    Search,
    ContextMenu,
    AccessibilityActivation,
    ViewportResize,
}

impl GenericInteractionClass {
    /// The complete KUC-owned full-editor sequence accepted by v1 artifact plans.
    pub const FULL_EDITOR_SEQUENCE: [Self; 10] = [
        Self::TextInput,
        Self::ImeCommit,
        Self::Selection,
        Self::Scroll,
        Self::ToolbarActivation,
        Self::FloatingToolbar,
        Self::Search,
        Self::ContextMenu,
        Self::AccessibilityActivation,
        Self::ViewportResize,
    ];
}

/// KUC-owned generic effect categories. They deliberately carry no host semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GenericEffectClass {
    NoHostEffect,
    /// Requires a generic opaque event-transport forwarder, which artifact plans do not expose.
    OpaqueForwarding,
}

/// Opaque consumer leaf identifier used only to bind a KUC-issued artifact stage.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConsumerArtifactLeafId(String);

impl ConsumerArtifactLeafId {
    pub fn new(value: impl Into<String>) -> Result<Self, ConsumerArtifactPlanError> {
        let value = value.into();
        if value.trim().is_empty()
            || value.len() > MAX_LEAF_IDENTIFIER_LENGTH
            || value.contains('\0')
        {
            return Err(ConsumerArtifactPlanError::InvalidLeafId);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for ConsumerArtifactLeafId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("ConsumerArtifactLeafId")
            .field(&self.0)
            .finish()
    }
}

pub use stage_binding::ConsumerArtifactStageBinding;

/// Versioned input to the KUC artifact issuer.
pub struct ConsumerArtifactPlanV1 {
    schema_version: u16,
    initial_revision: u64,
    bindings: Vec<ConsumerArtifactStageBinding>,
}

impl ConsumerArtifactPlanV1 {
    #[must_use]
    pub fn new(initial_revision: u64, bindings: Vec<ConsumerArtifactStageBinding>) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            initial_revision,
            bindings,
        }
    }

    /// Allows a consumer to declare its wire schema explicitly; unsupported values fail at issue.
    #[must_use]
    pub fn with_schema_version(
        schema_version: u16,
        initial_revision: u64,
        bindings: Vec<ConsumerArtifactStageBinding>,
    ) -> Self {
        Self {
            schema_version,
            initial_revision,
            bindings,
        }
    }
}
/// Stage evidence that excludes host targets and token bytes while publishing Unicode observations.
pub struct ConsumerArtifactEvidence {
    stage_id: String,
    leaf: ConsumerArtifactLeafId,
    root_revision: u64,
    png_sha256: String,
    pixel_hash: String,
    root_record_hash: String,
    accesskit_snapshot_hash: String,
    unicode_evidence_hash: String,
    unicode_evidence_json: Vec<u8>,
    receipt: ConsumerArtifactForwardingReceipt,
}

impl ConsumerArtifactEvidence {
    #[must_use]
    pub fn stage_id(&self) -> &str {
        &self.stage_id
    }
    #[must_use]
    pub fn leaf(&self) -> &ConsumerArtifactLeafId {
        &self.leaf
    }
    #[must_use]
    pub const fn root_revision(&self) -> u64 {
        self.root_revision
    }
    #[must_use]
    pub fn png_sha256(&self) -> &str {
        &self.png_sha256
    }
    #[must_use]
    pub fn pixel_hash(&self) -> &str {
        &self.pixel_hash
    }
    #[must_use]
    pub fn root_record_hash(&self) -> &str {
        &self.root_record_hash
    }
    #[must_use]
    pub fn accesskit_snapshot_hash(&self) -> &str {
        &self.accesskit_snapshot_hash
    }
    #[must_use]
    pub fn unicode_evidence_hash(&self) -> &str {
        &self.unicode_evidence_hash
    }
    #[must_use]
    pub fn unicode_evidence_json(&self) -> &[u8] {
        &self.unicode_evidence_json
    }
    pub fn into_forwarding_receipt(self) -> ConsumerArtifactForwardingReceipt {
        self.receipt
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsumerArtifactPlanError {
    UnsupportedSchemaVersion(u16),
    UnsupportedEffectClass(GenericEffectClass),
    EmptyPlan,
    InvalidLeafId,
    DuplicateLeaf(String),
    RevisionOverflow,
    StaleRevision {
        stage: usize,
        expected: u64,
        actual: u64,
    },
    StageAlreadyConsumed(usize),
    IncompleteStageSequence,
    StageExecutionFailed(usize),
    PlanComplete,
    TokenRootMismatch,
    Root(String),
    ExistingMedia(PathBuf),
    MissingFrame,
    Artifact(String),
    InvalidPng(PathBuf),
    UnicodeEvidence(String),
    ReceiptReuse,
    ReceiptCrossBind,
}

impl std::fmt::Display for ConsumerArtifactPlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion(value) => {
                write!(f, "unsupported consumer artifact schema version {value}")
            }
            Self::UnsupportedEffectClass(GenericEffectClass::OpaqueForwarding) => {
                f.write_str("opaque forwarding is unsupported by consumer artifact plans")
            }
            Self::UnsupportedEffectClass(GenericEffectClass::NoHostEffect) => {
                f.write_str("consumer artifact effect class is unsupported")
            }
            Self::EmptyPlan => f.write_str("consumer artifact plan is empty"),
            Self::InvalidLeafId => f.write_str("consumer artifact leaf id is invalid"),
            Self::DuplicateLeaf(value) => write!(f, "duplicate consumer artifact leaf {value}"),
            Self::RevisionOverflow => f.write_str("consumer artifact revision overflow"),
            Self::StaleRevision {
                stage,
                expected,
                actual,
            } => write!(
                f,
                "stage {stage} revision {actual} does not match expected {expected}"
            ),
            Self::StageAlreadyConsumed(stage) => {
                write!(f, "consumer artifact stage {stage} was already consumed")
            }
            Self::IncompleteStageSequence => {
                f.write_str("consumer artifact stages must be executed in full sequence")
            }
            Self::StageExecutionFailed(stage) => {
                write!(
                    f,
                    "consumer artifact stage {stage} failed after root mutation"
                )
            }
            Self::PlanComplete => f.write_str("consumer artifact plan is complete"),
            Self::TokenRootMismatch => {
                f.write_str("consumer artifact token does not match retained root")
            }
            Self::Root(error) => write!(f, "consumer artifact root failed: {error}"),
            Self::ExistingMedia(path) => write!(
                f,
                "consumer artifact media already exists: {}",
                path.display()
            ),
            Self::MissingFrame => f.write_str("consumer artifact stage did not produce a frame"),
            Self::Artifact(error) => write!(f, "consumer artifact write failed: {error}"),
            Self::InvalidPng(path) => {
                write!(f, "consumer artifact PNG is invalid: {}", path.display())
            }
            Self::UnicodeEvidence(error) => {
                write!(f, "consumer artifact unicode evidence failed: {error}")
            }
            Self::ReceiptReuse => f.write_str("consumer artifact receipt was reused"),
            Self::ReceiptCrossBind => {
                f.write_str("consumer artifact receipt is bound to another stage")
            }
        }
    }
}
impl std::error::Error for ConsumerArtifactPlanError {}

mod execution;
mod issuer;
mod receipt;
mod stage_binding;
mod stage_interactions;
mod support;
#[cfg(test)]
mod tests;
mod text_interactions;
mod unicode_evidence;

pub use execution::IssuedConsumerArtifactPlan;
pub use issuer::ConsumerArtifactPlanIssuer;
pub use receipt::ConsumerArtifactForwardingReceipt;
