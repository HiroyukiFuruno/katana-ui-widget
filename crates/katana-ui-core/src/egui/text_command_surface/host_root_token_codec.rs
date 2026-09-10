use super::super::types::{EguiTextCommandSurfacePresentation, TextCommandSurfaceStyle};
use super::EguiTextCommandSurfaceHostTargetToken;
use super::host_root_types::{RootPresentationWire, RootPresentationWireWithCommandFamilies};
use super::{
    EguiTextCommandSurfaceCommandFamilyProjection, EguiTextCommandSurfacePresentationToken,
    EguiTextCommandSurfaceRootFactoryError,
};
use sha2::{Digest, Sha256};

pub(super) struct DecodedRootPresentation {
    pub(super) identity: String,
    pub(super) presentation: EguiTextCommandSurfacePresentation,
    pub(super) style: TextCommandSurfaceStyle,
    pub(super) command_families: Option<EguiTextCommandSurfaceCommandFamilyProjection>,
}

pub(super) fn decode_token(
    token: &EguiTextCommandSurfacePresentationToken,
) -> Result<DecodedRootPresentation, EguiTextCommandSurfaceRootFactoryError> {
    if token.target.payload.is_empty() {
        return Err(EguiTextCommandSurfaceRootFactoryError::InvalidToken(
            "host target is empty",
        ));
    }
    let identity = hex::encode(Sha256::digest(&token.target.payload));
    if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&token.payload)
        && value.get("version").is_some()
    {
        let wire = serde_json::from_value::<RootPresentationWireWithCommandFamilies>(value)
            .map_err(|error| EguiTextCommandSurfaceRootFactoryError::Decode(error.to_string()))?;
        validate_version(wire.version)?;
        return Ok(DecodedRootPresentation {
            identity,
            presentation: wire.presentation,
            style: wire.style,
            command_families: Some(wire.command_families),
        });
    }
    let wire = serde_json::from_slice::<RootPresentationWire>(&token.payload)
        .map_err(|error| EguiTextCommandSurfaceRootFactoryError::Decode(error.to_string()))?;
    Ok(DecodedRootPresentation {
        identity,
        presentation: wire.presentation,
        style: wire.style,
        command_families: None,
    })
}

fn validate_version(version: u8) -> Result<(), EguiTextCommandSurfaceRootFactoryError> {
    if version == 1 {
        Ok(())
    } else {
        Err(EguiTextCommandSurfaceRootFactoryError::InvalidToken(
            "unsupported root presentation version",
        ))
    }
}

pub(super) fn encode_presentation_with_command_families(
    revision: u64,
    target: Vec<u8>,
    presentation: EguiTextCommandSurfacePresentation,
    style: TextCommandSurfaceStyle,
    command_families: EguiTextCommandSurfaceCommandFamilyProjection,
) -> Result<EguiTextCommandSurfacePresentationToken, serde_json::Error> {
    let identity = hex::encode(Sha256::digest(&target));
    let payload = serde_json::to_vec(&RootPresentationWireWithCommandFamilies {
        version: 1,
        presentation,
        style,
        command_families,
    })?;
    Ok(EguiTextCommandSurfacePresentationToken::from_encoded(
        revision,
        EguiTextCommandSurfaceHostTargetToken::from_opaque_bytes(identity),
        payload,
    ))
}

pub(super) fn encode_presentation(
    revision: u64,
    target: Vec<u8>,
    presentation: EguiTextCommandSurfacePresentation,
    style: TextCommandSurfaceStyle,
) -> Result<EguiTextCommandSurfacePresentationToken, serde_json::Error> {
    let identity = hex::encode(Sha256::digest(&target));
    let payload = serde_json::to_vec(&RootPresentationWire {
        presentation,
        style,
    })?;
    Ok(EguiTextCommandSurfacePresentationToken::from_encoded(
        revision,
        EguiTextCommandSurfaceHostTargetToken::from_opaque_bytes(identity),
        payload,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_model::UiTextSpan;
    use crate::text_surface::{
        TextSurface, TextSurfacePresentation, TextSurfaceProps, TextSurfaceViewport,
    };

    fn presentation() -> EguiTextCommandSurfacePresentation {
        let surface = TextSurface::new(TextSurfaceProps::new(
            crate::atom::TextArea::new("host-token-codec").value("opaque text"),
            Vec::<UiTextSpan>::new(),
            TextSurfaceViewport::new(0, 0, 320, 180),
        ));
        EguiTextCommandSurfacePresentation {
            text_state_id: None,
            text: TextSurfacePresentation::from_props(surface.props()),
            toolbar: None,
            floating: None,
            search: None,
            context_menu: None,
        }
    }

    #[test]
    fn malformed_versioned_and_legacy_payloads_are_typed_decode_failures() {
        for payload in [br#"{"version":1}"#.as_slice(), b"not-json".as_slice()] {
            let token = EguiTextCommandSurfacePresentationToken::from_encoded(
                1,
                EguiTextCommandSurfaceHostTargetToken::from_opaque_bytes("opaque-target"),
                payload.to_vec(),
            );
            assert!(matches!(
                decode_token(&token),
                Err(EguiTextCommandSurfaceRootFactoryError::Decode(_))
            ));
        }
    }

    #[test]
    fn unknown_version_fails_closed() {
        assert!(matches!(
            validate_version(2),
            Err(EguiTextCommandSurfaceRootFactoryError::InvalidToken(
                "unsupported root presentation version"
            ))
        ));
        assert!(validate_version(1).is_ok());
    }

    #[test]
    fn versioned_family_payload_decodes_in_the_unit_crate() {
        let token = encode_presentation_with_command_families(
            1,
            b"unit-family-target".to_vec(),
            presentation(),
            TextCommandSurfaceStyle::standard().expect("standard style"),
            EguiTextCommandSurfaceCommandFamilyProjection::default(),
        )
        .expect("versioned family token");

        let decoded = decode_token(&token).expect("versioned family payload");
        assert!(decoded.command_families.is_some());
    }
}
