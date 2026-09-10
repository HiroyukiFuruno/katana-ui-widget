use crate::render_model::UiTextSpan;
use crate::theme::FontToken;

mod types;
pub use types::*;

const DEFAULT_LINE_HEIGHT_RATIO: f32 = 1.45;
pub(crate) const RGBA_CHANNEL_COUNT: usize = 4;
pub(crate) const RGBA_ALPHA_INDEX: usize = 3;
pub(crate) const TRANSPARENT_RGBA: [u8; RGBA_CHANNEL_COUNT] = [0, 0, 0, 0];

#[derive(Debug, Clone, PartialEq)]
pub struct PlatformTextRasterRequest {
    pub spans: Vec<UiTextSpan>,
    pub font: FontToken,
    pub fallback_color_rgba: [u8; RGBA_CHANNEL_COUNT],
    pub line_height_px: f32,
    pub max_width_px: Option<f32>,
    pub scale_factor: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlatformTextMetricsRequest {
    pub text: String,
    pub font: FontToken,
    pub scale_factor: f32,
}

impl PlatformTextMetricsRequest {
    #[must_use]
    pub fn from_text(text: impl Into<String>, font: FontToken, scale_factor: f32) -> Self {
        Self {
            text: text.into(),
            font,
            scale_factor,
        }
    }

    #[must_use]
    pub fn normalized_scale_factor(&self) -> f32 {
        if self.scale_factor.is_finite() && self.scale_factor > 0.0 {
            self.scale_factor
        } else {
            1.0
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlatformTextMetrics {
    pub text: String,
    pub font_size_px: f32,
    pub scale_factor: f32,
    pub ascent_px: f32,
    pub descent_px: f32,
    pub baseline_px: f32,
    pub line_height_px: f32,
    pub advance_px: f32,
    pub grapheme_advances: Vec<PlatformTextGraphemeAdvance>,
}

/// Vertical metrics for a raster request in the raster's own coordinate system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlatformTextLineMetrics {
    /// Offset from the raster origin to the baseline, in logical pixels.
    pub baseline_from_raster_origin_px: f32,
    /// Total line-box height requested from the rasterizer, in logical pixels.
    pub line_box_height_px: f32,
}

/// Measurements collected during one adapter frame.
///
/// The record is intentionally keyed only by the generic text request.  It is
/// shared by all KUC text-bearing children so a repeated request in one frame
/// cannot drift between body, chrome, inputs, or overlays.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PlatformTextMetricsFrame {
    scale_factor: Option<u32>,
    requests: Vec<PlatformTextMetricsRequest>,
    records: Vec<PlatformTextMetrics>,
}

impl PlatformTextMetricsFrame {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn begin(&mut self, scale_factor: f32) -> Result<(), PlatformTextRasterError> {
        if !scale_factor.is_finite() || scale_factor <= 0.0 {
            return Err(PlatformTextRasterError::NonFiniteLayoutExtent);
        }
        self.scale_factor = Some(scale_factor.to_bits());
        self.requests.clear();
        self.records.clear();
        Ok(())
    }

    pub fn measure_text(
        &mut self,
        rasterizer: &mut crate::text_raster::PlatformTextRasterizer,
        request: &PlatformTextMetricsRequest,
    ) -> Result<PlatformTextMetrics, PlatformTextRasterError> {
        let scale = request.normalized_scale_factor();
        if let Some(expected) = self.scale_factor {
            if expected != scale.to_bits() {
                return Err(PlatformTextRasterError::MetricsFrameScaleMismatch {
                    expected_bits: expected,
                    actual_bits: scale.to_bits(),
                });
            }
        } else {
            self.scale_factor = Some(scale.to_bits());
        }
        if let Some(index) = self
            .requests
            .iter()
            .position(|existing| existing == request)
        {
            return Ok(self.records[index].clone());
        }
        let measured = rasterizer.measure_text(request)?;
        self.requests.push(request.clone());
        self.records.push(measured.clone());
        Ok(measured)
    }

    #[must_use]
    pub fn records(&self) -> &[PlatformTextMetrics] {
        &self.records
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlatformTextGraphemeAdvance {
    pub byte_start: usize,
    pub byte_end: usize,
    pub advance_px: f32,
}

impl PlatformTextRasterRequest {
    #[must_use]
    pub fn from_text(
        text: impl AsRef<str>,
        font: FontToken,
        color_rgba: [u8; RGBA_CHANNEL_COUNT],
    ) -> Self {
        let spans =
            UiTextSpan::emoji_marked_spans(text, crate::render_model::UiTextSpanStyle::default());
        Self {
            spans,
            line_height_px: font.size * DEFAULT_LINE_HEIGHT_RATIO,
            font,
            fallback_color_rgba: color_rgba,
            max_width_px: None,
            scale_factor: 1.0,
        }
    }

    #[must_use]
    pub fn text(&self) -> String {
        self.spans.iter().map(|span| span.text.as_str()).collect()
    }

    #[must_use]
    pub fn normalized_scale_factor(&self) -> f32 {
        if self.scale_factor.is_finite() && self.scale_factor > 0.0 {
            self.scale_factor
        } else {
            1.0
        }
    }

    #[must_use]
    pub fn normalized_line_height(&self) -> f32 {
        if self.line_height_px.is_finite() && self.line_height_px > 0.0 {
            self.line_height_px
        } else {
            self.font.size.max(1.0) * DEFAULT_LINE_HEIGHT_RATIO
        }
    }

    #[must_use]
    pub fn normalized_max_width(&self, fallback: f32, maximum: f32) -> f32 {
        self.max_width_px
            .filter(|width| width.is_finite() && *width > 0.0)
            .unwrap_or(fallback)
            .min(maximum)
    }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
