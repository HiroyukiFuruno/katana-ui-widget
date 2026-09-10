use super::super::types::{EguiTextCommandSurfacePresentation, TextCommandSurfaceStyle};
use super::EguiTextCommandSurfaceCommandFamilyProjection;
use super::host_root_token_codec::{
    encode_presentation, encode_presentation_with_command_families,
};
use super::host_root_types::{
    EguiTextCommandSurfaceHostProjectionEncoder, EguiTextCommandSurfaceHostTargetToken,
    EguiTextCommandSurfacePresentationToken,
};

impl EguiTextCommandSurfaceHostTargetToken {
    #[must_use]
    pub fn from_opaque_bytes(payload: impl Into<Vec<u8>>) -> Self {
        Self {
            payload: payload.into().into_boxed_slice(),
        }
    }
}
impl std::fmt::Debug for EguiTextCommandSurfaceHostTargetToken {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("EguiTextCommandSurfaceHostTargetToken(..)")
    }
}
impl EguiTextCommandSurfacePresentationToken {
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }
    #[must_use]
    pub fn from_opaque_bytes(
        revision: u64,
        target: EguiTextCommandSurfaceHostTargetToken,
        payload: impl Into<Vec<u8>>,
    ) -> Self {
        Self::from_encoded(revision, target, payload)
    }
    pub(super) fn from_encoded(
        revision: u64,
        target: EguiTextCommandSurfaceHostTargetToken,
        payload: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            revision,
            target,
            payload: payload.into().into_boxed_slice(),
        }
    }
}
impl std::fmt::Debug for EguiTextCommandSurfacePresentationToken {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EguiTextCommandSurfacePresentationToken")
            .field("revision", &self.revision)
            .field("target", &self.target)
            .field("payload", &"<opaque>")
            .finish()
    }
}
impl EguiTextCommandSurfaceHostProjectionEncoder {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
    pub fn token(
        revision: u64,
        target: impl Into<Vec<u8>>,
        presentation: EguiTextCommandSurfacePresentation,
        style: TextCommandSurfaceStyle,
    ) -> Result<EguiTextCommandSurfacePresentationToken, serde_json::Error> {
        encode_presentation(revision, target.into(), presentation, style)
    }
    pub fn encode(
        &self,
        revision: u64,
        target: impl Into<Vec<u8>>,
        presentation: EguiTextCommandSurfacePresentation,
        style: TextCommandSurfaceStyle,
    ) -> Result<EguiTextCommandSurfacePresentationToken, serde_json::Error> {
        Self::token(revision, target, presentation, style)
    }
    pub fn token_with_command_families(
        revision: u64,
        target: impl Into<Vec<u8>>,
        presentation: EguiTextCommandSurfacePresentation,
        style: TextCommandSurfaceStyle,
        command_families: EguiTextCommandSurfaceCommandFamilyProjection,
    ) -> Result<EguiTextCommandSurfacePresentationToken, serde_json::Error> {
        encode_presentation_with_command_families(
            revision,
            target.into(),
            presentation,
            style,
            command_families,
        )
    }
    pub fn encode_with_command_families(
        &self,
        revision: u64,
        target: impl Into<Vec<u8>>,
        presentation: EguiTextCommandSurfacePresentation,
        style: TextCommandSurfaceStyle,
        command_families: EguiTextCommandSurfaceCommandFamilyProjection,
    ) -> Result<EguiTextCommandSurfacePresentationToken, serde_json::Error> {
        Self::token_with_command_families(revision, target, presentation, style, command_families)
    }
}
impl Default for EguiTextCommandSurfaceHostProjectionEncoder {
    fn default() -> Self {
        Self::new()
    }
}
