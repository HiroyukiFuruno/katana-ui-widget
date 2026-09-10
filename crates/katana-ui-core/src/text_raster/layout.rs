use crate::render_model::UiTextSpan;
use crate::text_raster::catalog::PlatformRegularFontFaces;
use crate::text_raster::catalog_types::PlatformColorEmojiFaceRecord;
use crate::text_raster::model::{
    PlatformTextGraphemeBounds, PlatformTextLineMetrics, PlatformTextMetrics,
    PlatformTextMetricsRequest, PlatformTextRasterError, PlatformTextRasterRequest,
    RGBA_CHANNEL_COUNT,
};
use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, SwashCache, Wrap};

mod attributes;
mod metrics;
mod raster;

use attributes::{attrs_for_span, normalized_runs};
use metrics::collect_grapheme_advances;
use raster::{collect_grapheme_bounds, collect_pixels, raster_extent};

const FALLBACK_LAYOUT_WIDTH: f32 = 4096.0;
const FALLBACK_LAYOUT_HEIGHT: f32 = 4096.0;
const MAX_LAYOUT_WIDTH: f32 = 8192.0;
const MAX_RASTER_DIMENSION: usize = 8192;
const MAX_RASTER_PIXELS: usize = 16_777_216;
const MIN_FONT_SIZE_PX: f32 = 1.0;
const MIN_GRAPHEME_EXTENT_PX: f32 = 1.0;
const MIN_RASTER_DIMENSION: f32 = 1.0;
const MIN_GRAPHEME_COUNT: usize = 1;
const REGULAR_WEIGHT: u16 = 400;
const BOLD_WEIGHT: u16 = 700;
const OPAQUE_COLOR_CHANNEL: u8 = u8::MAX;

pub(crate) struct TextLayoutRasterizer;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ResolvedTextFaces {
    proportional: Option<String>,
    monospace: Option<String>,
}

impl ResolvedTextFaces {
    pub(crate) fn from_first_candidates(
        proportional: Option<String>,
        monospace: Option<String>,
    ) -> Self {
        Self {
            proportional,
            monospace,
        }
    }

    pub(crate) fn from_candidate_faces(candidates: PlatformRegularFontFaces) -> Self {
        Self::from_first_candidates(
            candidates.proportional.map(|face| face.selection_family),
            candidates.monospace.map(|face| face.selection_family),
        )
    }

    pub(crate) fn proportional(&self) -> Option<&str> {
        self.proportional.as_deref()
    }

    pub(crate) fn monospace(&self) -> Option<&str> {
        self.monospace.as_deref()
    }
}

pub(crate) struct LayoutRaster {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) rgba_pixels: Vec<[u8; RGBA_CHANNEL_COUNT]>,
    pub(crate) grapheme_bounds: Vec<PlatformTextGraphemeBounds>,
}

impl TextLayoutRasterizer {
    pub(crate) fn line_metrics(
        font_system: &mut FontSystem,
        request: &PlatformTextRasterRequest,
        emoji_face: &PlatformColorEmojiFaceRecord,
        text_faces: &ResolvedTextFaces,
    ) -> Result<PlatformTextLineMetrics, PlatformTextRasterError> {
        if request.spans.iter().all(|span| span.text.is_empty()) {
            return Err(PlatformTextRasterError::EmptyText);
        }
        if !request.scale_factor.is_finite() || !request.font.size.is_finite() {
            return Err(PlatformTextRasterError::NonFiniteLayoutExtent);
        }
        let scale = request.normalized_scale_factor();
        let metrics = Metrics::new(
            request.font.size.max(MIN_FONT_SIZE_PX) * scale,
            request.normalized_line_height() * scale,
        );
        let mut buffer = Buffer::new(font_system, metrics);
        let mut buffer = buffer.borrow_with(font_system);
        buffer.set_wrap(Wrap::Word);
        buffer.set_size(
            Some(request.normalized_max_width(FALLBACK_LAYOUT_WIDTH, MAX_LAYOUT_WIDTH) * scale),
            Some(FALLBACK_LAYOUT_HEIGHT * scale),
        );
        let runs = normalized_runs(&request.spans);
        let rich_text = runs
            .iter()
            .map(|span| {
                attrs_for_span(
                    &request.font,
                    span,
                    request.fallback_color_rgba,
                    emoji_face,
                    text_faces,
                )
                .map(|attrs| (span.text.as_str(), attrs))
            })
            .collect::<Result<Vec<_>, _>>()?;
        buffer.set_rich_text(rich_text, &Attrs::new(), Shaping::Advanced, None);
        let max_ascent = buffer
            .line_layout(0)
            .and_then(|lines| lines.first())
            .map(|line| line.max_ascent)
            .ok_or(PlatformTextRasterError::EmptyText)?;
        let line_top = buffer
            .layout_runs()
            .next()
            .map(|run| run.line_top)
            .ok_or(PlatformTextRasterError::EmptyText)?;
        Ok(PlatformTextLineMetrics {
            baseline_from_raster_origin_px: (line_top + max_ascent) / scale,
            line_box_height_px: request.normalized_line_height(),
        })
    }

    pub(crate) fn measure(
        font_system: &mut FontSystem,
        request: &PlatformTextMetricsRequest,
        emoji_face: &PlatformColorEmojiFaceRecord,
        text_faces: &ResolvedTextFaces,
    ) -> Result<PlatformTextMetrics, PlatformTextRasterError> {
        if request.text.is_empty() {
            return Err(PlatformTextRasterError::EmptyText);
        }
        if !request.scale_factor.is_finite() || !request.font.size.is_finite() {
            return Err(PlatformTextRasterError::NonFiniteLayoutExtent);
        }
        let scale = request.normalized_scale_factor();
        let font_size_px = request.font.size.max(MIN_FONT_SIZE_PX);
        let metrics = Metrics::new(font_size_px * scale, font_size_px * scale);
        let mut buffer = Buffer::new(font_system, metrics);
        let mut buffer = buffer.borrow_with(font_system);
        buffer.set_wrap(Wrap::None);
        buffer.set_size(
            Some(FALLBACK_LAYOUT_WIDTH * scale),
            Some(FALLBACK_LAYOUT_HEIGHT * scale),
        );
        let spans = [UiTextSpan::plain(request.text.clone())];
        let rich_text = spans
            .iter()
            .map(|span| {
                attrs_for_span(
                    &request.font,
                    span,
                    [OPAQUE_COLOR_CHANNEL; RGBA_CHANNEL_COUNT],
                    emoji_face,
                    text_faces,
                )
                .map(|attrs| (span.text.as_str(), attrs))
            })
            .collect::<Result<Vec<_>, _>>();
        let rich_text = rich_text?;
        buffer.set_rich_text(rich_text, &Attrs::new(), Shaping::Advanced, None);
        let first_line = buffer
            .line_layout(0)
            .and_then(|lines| lines.first())
            .map(|line| {
                (
                    line.max_ascent,
                    line.max_descent,
                    line.max_ascent + line.max_descent,
                )
            })
            .ok_or(PlatformTextRasterError::EmptyText);
        let (ascent, descent, line_height) = first_line?;
        let advance_px = buffer.layout_runs().map(|run| run.line_w).sum();
        let grapheme_advances = collect_grapheme_advances(&mut buffer, &request.text);
        Ok(PlatformTextMetrics {
            text: request.text.clone(),
            font_size_px,
            scale_factor: scale,
            ascent_px: ascent,
            descent_px: descent,
            baseline_px: ascent,
            line_height_px: line_height,
            advance_px,
            grapheme_advances,
        })
    }

    pub(crate) fn rasterize(
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
        request: &PlatformTextRasterRequest,
        emoji_face: &PlatformColorEmojiFaceRecord,
        text_faces: &ResolvedTextFaces,
    ) -> Result<LayoutRaster, PlatformTextRasterError> {
        let scale = request.normalized_scale_factor();
        let metrics = Metrics::new(
            request.font.size.max(MIN_FONT_SIZE_PX) * scale,
            request.normalized_line_height() * scale,
        );
        let mut buffer = Buffer::new(font_system, metrics);
        let mut buffer = buffer.borrow_with(font_system);
        buffer.set_wrap(Wrap::Word);
        buffer.set_size(
            Some(request.normalized_max_width(FALLBACK_LAYOUT_WIDTH, MAX_LAYOUT_WIDTH) * scale),
            Some(FALLBACK_LAYOUT_HEIGHT * scale),
        );
        let runs = normalized_runs(&request.spans);
        let rich_text = runs
            .iter()
            .map(|span| {
                attrs_for_span(
                    &request.font,
                    span,
                    request.fallback_color_rgba,
                    emoji_face,
                    text_faces,
                )
                .map(|attrs| (span.text.as_str(), attrs))
            })
            .collect::<Result<Vec<_>, _>>();
        let rich_text = rich_text?;
        buffer.set_rich_text(rich_text, &Attrs::new(), Shaping::Advanced, None);
        let source_text = request.text();
        let grapheme_bounds = collect_grapheme_bounds(&mut buffer, &source_text, scale);
        let (width, height) = raster_extent(&grapheme_bounds, scale)?;
        let rgba_pixels = collect_pixels(&mut buffer, swash_cache, width, height);
        Ok(LayoutRaster {
            width,
            height,
            rgba_pixels,
            grapheme_bounds,
        })
    }
}

#[cfg(test)]
#[path = "layout_tests.rs"]
mod tests;
