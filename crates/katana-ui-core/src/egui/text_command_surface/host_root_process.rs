use super::super::EditorViewportProjectionLease;
use super::super::root::KucRootEffectRouter;
use super::super::root::{EguiTextCommandSurfaceRoot, EguiTextCommandSurfaceRootOutput};
use super::super::source_address_projection_lease::SourceAddressProjectionLease;
use super::super::status_diagnostics_projection_lease::StatusDiagnosticsProjectionLease;
use super::super::tab_strip_projection_lease::TabStripProjectionLease;
use super::super::types::TextCommandSurfaceStyle;
use super::EguiTextCommandSurfaceCommandFamilyProjection;
use super::host_root_surface::surface_from_presentation;
use super::host_root_token_codec::DecodedRootPresentation;
use super::host_root_types::EguiTextCommandSurfaceRootFactoryError;

pub(crate) struct HostRootProcess {
    root: EguiTextCommandSurfaceRoot,
    identity: String,
    style: TextCommandSurfaceStyle,
    presentation: super::super::types::EguiTextCommandSurfacePresentation,
    command_families: Option<EguiTextCommandSurfaceCommandFamilyProjection>,
    presentation_revision: u64,
    effect_router: Option<Box<dyn KucRootEffectRouter>>,
}

impl HostRootProcess {
    pub(super) fn retain(
        decoded: DecodedRootPresentation,
        presentation_revision: u64,
    ) -> Result<Self, EguiTextCommandSurfaceRootFactoryError> {
        if decoded.identity.trim().is_empty() {
            return Err(EguiTextCommandSurfaceRootFactoryError::InvalidToken(
                "host target identity is empty",
            ));
        }
        let surface = surface_from_presentation(
            &decoded.identity,
            &decoded.presentation,
            decoded.command_families.as_ref(),
        );
        let mut root =
            EguiTextCommandSurfaceRoot::with_identity(decoded.identity.clone(), surface)?;
        let _ = root.synchronize_presentation(decoded.presentation.clone());
        Ok(Self {
            root,
            identity: decoded.identity,
            style: decoded.style,
            presentation: decoded.presentation,
            command_families: decoded.command_families,
            presentation_revision,
            effect_router: None,
        })
    }

    pub(super) fn retain_with_router(
        decoded: DecodedRootPresentation,
        presentation_revision: u64,
        router: Box<dyn KucRootEffectRouter>,
        source_address: Option<SourceAddressProjectionLease>,
        tab_strip: Option<TabStripProjectionLease>,
        status_diagnostics: Option<StatusDiagnosticsProjectionLease>,
        editor_viewport: Option<EditorViewportProjectionLease>,
    ) -> Result<Self, EguiTextCommandSurfaceRootFactoryError> {
        let mut process = Self::retain(decoded, presentation_revision)?;
        process.effect_router = Some(router);
        if let Some(source_address) = source_address {
            process.root.attach_source_address(source_address);
        }
        if let Some(tab_strip) = tab_strip {
            let _ = process.root.attach_tab_strip(tab_strip)?;
        }
        if let Some(status_diagnostics) = status_diagnostics {
            let (status_bar, diagnostics_list) = status_diagnostics.into_parts();
            if let Some(status_bar) = status_bar {
                process.root.attach_status_bar(status_bar);
            }
            if let Some(diagnostics_list) = diagnostics_list {
                process.root.attach_diagnostics_list(diagnostics_list);
            }
        }
        if let Some(editor_viewport) = editor_viewport {
            process.root.attach_editor_viewport(editor_viewport);
        }
        Ok(process)
    }

    pub(super) fn synchronize(
        &mut self,
        revision: u64,
        decoded: DecodedRootPresentation,
    ) -> Result<bool, EguiTextCommandSurfaceRootFactoryError> {
        if decoded.identity != self.identity {
            return Err(EguiTextCommandSurfaceRootFactoryError::IdentityChanged);
        }
        if revision < self.presentation_revision {
            return Err(EguiTextCommandSurfaceRootFactoryError::StaleRevision {
                current: self.presentation_revision,
                received: revision,
            });
        }
        if revision == self.presentation_revision {
            if decoded.style != self.style
                || decoded.presentation != self.presentation
                || decoded.command_families != self.command_families
            {
                return Err(EguiTextCommandSurfaceRootFactoryError::RevisionConflict { revision });
            }
            return Ok(false);
        }
        let family_changed = decoded.command_families != self.command_families;
        let mut changed = self
            .root
            .synchronize_presentation(decoded.presentation.clone());
        if family_changed && let Some(command_families) = decoded.command_families.as_ref() {
            changed |= self.root.synchronize_command_families(
                command_families.primary().cloned(),
                command_families.floating().cloned(),
            );
        }
        /* WHY: A newer plain token has no tab lease, so it must not retain an
        earlier lease-owned tab strip behind the opaque root boundary. */
        changed |= self.root.clear_tab_strip();
        changed |= self.root.clear_status_diagnostics();
        changed |= self.root.clear_editor_viewport();
        self.style = decoded.style;
        self.presentation = decoded.presentation;
        self.command_families = decoded.command_families;
        self.presentation_revision = revision;
        Ok(changed)
    }

    pub(super) fn synchronize_with_router(
        &mut self,
        revision: u64,
        decoded: DecodedRootPresentation,
        router: Box<dyn KucRootEffectRouter>,
        source_address: Option<SourceAddressProjectionLease>,
        tab_strip: Option<TabStripProjectionLease>,
        status_diagnostics: Option<StatusDiagnosticsProjectionLease>,
        editor_viewport: Option<EditorViewportProjectionLease>,
    ) -> Result<bool, EguiTextCommandSurfaceRootFactoryError> {
        if revision <= self.presentation_revision {
            return Err(EguiTextCommandSurfaceRootFactoryError::DuplicateLease { revision });
        }
        let mut changed = self.synchronize(revision, decoded)?;
        self.effect_router = Some(router);
        if let Some(source_address) = source_address {
            self.root.attach_source_address(source_address);
        }
        if let Some(tab_strip) = tab_strip {
            changed |= self.root.attach_tab_strip(tab_strip)?;
        }
        if let Some(status_diagnostics) = status_diagnostics {
            let (status_bar, diagnostics_list) = status_diagnostics.into_parts();
            if let Some(status_bar) = status_bar {
                self.root.attach_status_bar(status_bar);
                changed = true;
            }
            if let Some(diagnostics_list) = diagnostics_list {
                self.root.attach_diagnostics_list(diagnostics_list);
                changed = true;
            }
        }
        if let Some(editor_viewport) = editor_viewport {
            self.root.attach_editor_viewport(editor_viewport);
            changed = true;
        }
        Ok(changed)
    }

    pub(super) fn show(
        &mut self,
        ui: &mut egui::Ui,
    ) -> Result<EguiTextCommandSurfaceRootOutput, EguiTextCommandSurfaceRootFactoryError> {
        let output = self.root.show(ui, &self.style)?;
        if let Some(router) = self.effect_router.as_mut() {
            let effect = router
                .route(output.events().current_context())
                .map_err(|_| EguiTextCommandSurfaceRootFactoryError::OpaqueHostEffect)?;
            if let Some(effect) = effect {
                output.events().attach_opaque_host_effect_batch(effect)?;
            }
        }
        Ok(output)
    }

    pub(super) fn identity(&self) -> &str {
        &self.identity
    }

    pub(super) const fn presentation_revision(&self) -> u64 {
        self.presentation_revision
    }
}
