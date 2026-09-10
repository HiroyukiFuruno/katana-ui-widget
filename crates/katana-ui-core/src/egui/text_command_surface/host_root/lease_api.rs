use super::super::EditorViewportProjectionLease;
use super::super::root::KucRootEffectRouter;
use super::super::source_address_projection_lease::SourceAddressProjectionLease;
use super::super::status_diagnostics_projection_lease::StatusDiagnosticsProjectionLease;
use super::super::tab_strip_projection_lease::TabStripProjectionLease;
use super::host_root_types::{
    EguiTextCommandSurfaceHostProjectionLease, EguiTextCommandSurfacePresentationToken,
    HostProjectionParts,
};

impl EguiTextCommandSurfaceHostProjectionLease {
    pub(crate) fn token(&self) -> &EguiTextCommandSurfacePresentationToken {
        &self.token
    }

    #[must_use]
    pub fn new<R>(token: EguiTextCommandSurfacePresentationToken, router: R) -> Self
    where
        R: KucRootEffectRouter + 'static,
    {
        Self::from_router(token, Box::new(router))
    }

    #[must_use]
    pub fn from_router(
        token: EguiTextCommandSurfacePresentationToken,
        router: Box<dyn KucRootEffectRouter>,
    ) -> Self {
        Self {
            token,
            router,
            source_address: None,
            tab_strip: None,
            status_diagnostics: None,
            editor_viewport: None,
        }
    }

    #[must_use]
    pub fn with_source_address(mut self, lease: SourceAddressProjectionLease) -> Self {
        self.source_address = Some(lease);
        self
    }
    #[must_use]
    pub fn with_tab_strip(mut self, lease: TabStripProjectionLease) -> Self {
        self.tab_strip = Some(lease);
        self
    }
    #[must_use]
    pub fn with_status_diagnostics(mut self, lease: StatusDiagnosticsProjectionLease) -> Self {
        self.status_diagnostics = Some(lease);
        self
    }
    #[must_use]
    pub fn with_editor_viewport(mut self, lease: EditorViewportProjectionLease) -> Self {
        self.editor_viewport = Some(lease);
        self
    }

    pub(super) fn into_parts(self) -> HostProjectionParts {
        (
            self.token,
            self.router,
            self.source_address,
            self.tab_strip,
            self.status_diagnostics,
            self.editor_viewport,
        )
    }
}

impl std::fmt::Debug for EguiTextCommandSurfaceHostProjectionLease {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("EguiTextCommandSurfaceHostProjectionLease(..)")
    }
}
