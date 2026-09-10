use super::*;
use crate::text_raster::{
    PlatformColorEmojiAvailability, PlatformColorEmojiUnavailableReason,
    PlatformFontCatalogFingerprint, PlatformFontProfile, PlatformTextRasterError,
};

const TEST_FONT_SIZE_PX: f32 = 16.0;
const TEST_SHA256_BYTE_COUNT: usize = 32;

fn font() -> crate::theme::FontToken {
    crate::theme::FontToken {
        name: "coverage".to_string(),
        family: crate::theme::FontFamily::Monospace,
        size: TEST_FONT_SIZE_PX,
        weight: REGULAR_WEIGHT,
    }
}

fn unavailable_emoji_face() -> PlatformColorEmojiFaceRecord {
    PlatformColorEmojiFaceRecord {
        platform_profile: PlatformFontProfile::Unsupported,
        family_identity: String::new(),
        source_file_path: None,
        raw_file_sha256: None,
        catalog_fingerprint: PlatformFontCatalogFingerprint::from_bytes(
            [0; TEST_SHA256_BYTE_COUNT],
        ),
        availability: PlatformColorEmojiAvailability::Unavailable(
            PlatformColorEmojiUnavailableReason::NoCandidates,
        ),
    }
}

#[test]
fn direct_layout_measure_rejects_empty_text_before_shaping() {
    let mut font_system = cosmic_text::FontSystem::new();
    let request = PlatformTextMetricsRequest::from_text("", font(), 1.0);

    assert_eq!(
        TextLayoutRasterizer::measure(
            &mut font_system,
            &request,
            &unavailable_emoji_face(),
            &ResolvedTextFaces::default(),
        ),
        Err(PlatformTextRasterError::EmptyText)
    );
}

#[test]
fn direct_layout_measure_rejects_nonfinite_scale_factor_before_shaping() {
    let mut font_system = cosmic_text::FontSystem::new();
    let request = PlatformTextMetricsRequest::from_text("coverage", font(), f32::NAN);

    assert_eq!(
        TextLayoutRasterizer::measure(
            &mut font_system,
            &request,
            &unavailable_emoji_face(),
            &ResolvedTextFaces::default(),
        ),
        Err(PlatformTextRasterError::NonFiniteLayoutExtent)
    );
}

#[test]
fn direct_line_metrics_rejects_empty_and_nonfinite_requests_before_shaping() {
    let mut font_system = cosmic_text::FontSystem::new();
    let empty = PlatformTextRasterRequest::from_text("", font(), [255; RGBA_CHANNEL_COUNT]);

    assert_eq!(
        TextLayoutRasterizer::line_metrics(
            &mut font_system,
            &empty,
            &unavailable_emoji_face(),
            &ResolvedTextFaces::default(),
        ),
        Err(PlatformTextRasterError::EmptyText)
    );

    let mut nonfinite =
        PlatformTextRasterRequest::from_text("coverage", font(), [255; RGBA_CHANNEL_COUNT]);
    nonfinite.font.size = f32::NAN;

    assert_eq!(
        TextLayoutRasterizer::line_metrics(
            &mut font_system,
            &nonfinite,
            &unavailable_emoji_face(),
            &ResolvedTextFaces::default(),
        ),
        Err(PlatformTextRasterError::NonFiniteLayoutExtent)
    );
}
