//! Consumer-safe retained root facade.

#[cfg(test)]
use super::EditorViewportProjectionLease;
#[cfg(test)]
use super::source_address_projection_lease::SourceAddressProjectionLease;
#[cfg(test)]
use super::status_diagnostics_projection_lease::StatusDiagnosticsProjectionLease;
#[cfg(test)]
use super::tab_strip_projection_lease::TabStripProjectionLease;
#[cfg(test)]
use super::types::EguiTextCommandSurfacePresentation;

#[path = "host_root/command_families.rs"]
mod host_root_command_families;
#[path = "host_root/errors.rs"]
mod host_root_errors;
#[path = "host_root_facade.rs"]
mod host_root_facade;
#[path = "host_root/factory_api.rs"]
mod host_root_factory_api;
#[path = "host_root/frame.rs"]
mod host_root_frame;
#[path = "host_root/lease_api.rs"]
mod host_root_lease_api;
#[path = "host_root_process.rs"]
mod host_root_process;
#[path = "host_root_record.rs"]
mod host_root_record;
#[path = "host_root_surface.rs"]
mod host_root_surface;
#[path = "host_root/token_api.rs"]
mod host_root_token_api;
#[path = "host_root_token_codec.rs"]
mod host_root_token_codec;
#[path = "host_root/types.rs"]
mod host_root_types;

#[cfg(test)]
use super::types::TextCommandSurfaceStyle;
#[cfg(test)]
use host_root_process::HostRootProcess;

pub use host_root_types::{
    EguiTextCommandSurfaceCommandFamilyProjection, EguiTextCommandSurfaceHostProjectionEncoder,
    EguiTextCommandSurfaceHostProjectionLease, EguiTextCommandSurfaceHostRoot,
    EguiTextCommandSurfaceHostRootFrame, EguiTextCommandSurfaceHostTargetToken,
    EguiTextCommandSurfacePresentationToken, EguiTextCommandSurfaceRootFactory,
    EguiTextCommandSurfaceRootFactoryError,
};

pub use host_root_record::{
    EguiTextCommandSurfaceHostRootRecord, EguiTextCommandSurfaceHostRootRecordDimensions,
};

#[cfg(test)]
#[path = "host_root_tests.rs"]
mod tests;
