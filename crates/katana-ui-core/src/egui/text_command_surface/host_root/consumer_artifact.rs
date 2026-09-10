use super::{EguiTextCommandSurfaceHostProjectionLease, EguiTextCommandSurfacePresentationToken};

impl EguiTextCommandSurfaceHostProjectionLease {
    pub(crate) fn into_consumer_artifact_token(self) -> EguiTextCommandSurfacePresentationToken {
        self.token
    }
}
