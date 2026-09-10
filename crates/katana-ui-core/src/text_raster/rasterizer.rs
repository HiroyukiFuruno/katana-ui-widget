use crate::text_raster::catalog::PlatformFontCatalog;
use crate::text_raster::config::{PlatformTextFaceSelection, PlatformTextRasterConfig};
use crate::text_raster::layout::{ResolvedTextFaces, TextLayoutRasterizer};
use crate::text_raster::model::{
    PlatformTextLineMetrics, PlatformTextMetrics, PlatformTextMetricsRequest, PlatformTextRaster,
    PlatformTextRasterError, PlatformTextRasterReport, PlatformTextRasterRequest,
    PlatformTextRasterStats,
};
use cosmic_text::SwashCache;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

const MIN_CACHE_CAPACITY: usize = 1;
const INITIAL_FONT_DATABASE_LOADS: usize = 1;

pub struct PlatformTextRasterizer {
    catalog: Arc<PlatformFontCatalog>,
    swash_cache: SwashCache,
    cache: HashMap<String, PlatformTextRaster>,
    cache_order: VecDeque<String>,
    cache_capacity: usize,
    face_selection: PlatformTextFaceSelection,
    text_faces: ResolvedTextFaces,
    stats: PlatformTextRasterStats,
}

impl PlatformTextRasterizer {
    #[must_use]
    pub fn new(config: PlatformTextRasterConfig) -> Self {
        Self::new_with_face_selection(config, PlatformTextFaceSelection::System)
    }

    #[must_use]
    pub fn new_with_face_selection(
        config: PlatformTextRasterConfig,
        face_selection: PlatformTextFaceSelection,
    ) -> Self {
        let catalog = Arc::new(PlatformFontCatalog::new(config.catalog_policy()));
        Self::from_matching_catalog_with_face_selection(catalog, config, face_selection)
    }

    pub fn with_catalog(
        catalog: Arc<PlatformFontCatalog>,
        config: PlatformTextRasterConfig,
    ) -> Result<Self, PlatformTextRasterError> {
        Self::with_catalog_and_face_selection(catalog, config, PlatformTextFaceSelection::System)
    }

    pub fn with_catalog_and_face_selection(
        catalog: Arc<PlatformFontCatalog>,
        config: PlatformTextRasterConfig,
        face_selection: PlatformTextFaceSelection,
    ) -> Result<Self, PlatformTextRasterError> {
        if catalog.policy() != &config.catalog_policy() {
            return Err(PlatformTextRasterError::CatalogConfigurationMismatch);
        }
        Ok(Self::from_matching_catalog_with_face_selection(
            catalog,
            config,
            face_selection,
        ))
    }

    #[must_use]
    pub fn catalog(&self) -> Arc<PlatformFontCatalog> {
        Arc::clone(&self.catalog)
    }

    pub(crate) fn from_matching_catalog_with_face_selection(
        catalog: Arc<PlatformFontCatalog>,
        config: PlatformTextRasterConfig,
        face_selection: PlatformTextFaceSelection,
    ) -> Self {
        let regular_font_faces = catalog.regular_font_faces();
        let text_faces = match face_selection {
            PlatformTextFaceSelection::System => ResolvedTextFaces::default(),
            PlatformTextFaceSelection::FirstCandidate => {
                ResolvedTextFaces::from_candidate_faces(regular_font_faces)
            }
        };
        Self {
            catalog,
            swash_cache: SwashCache::new(),
            cache: HashMap::new(),
            cache_order: VecDeque::new(),
            cache_capacity: config.cache_capacity.max(MIN_CACHE_CAPACITY),
            face_selection,
            text_faces,
            stats: PlatformTextRasterStats {
                font_database_loads: INITIAL_FONT_DATABASE_LOADS,
                ..PlatformTextRasterStats::default()
            },
        }
    }

    fn request_contains_emoji(request: &PlatformTextRasterRequest) -> bool {
        request.spans.iter().any(|span| span.style.emoji)
    }

    #[must_use]
    pub const fn stats(&self) -> PlatformTextRasterStats {
        self.stats
    }

    pub fn rasterize(
        &mut self,
        request: &PlatformTextRasterRequest,
    ) -> Result<PlatformTextRaster, PlatformTextRasterError> {
        if request.spans.iter().all(|span| span.text.is_empty()) {
            return Err(PlatformTextRasterError::EmptyText);
        }
        if !request.scale_factor.is_finite() {
            return Err(PlatformTextRasterError::NonFiniteLayoutExtent);
        }
        if Self::request_contains_emoji(request) && !self.catalog.emoji_face().is_available() {
            return Err(PlatformTextRasterError::ColorEmojiUnavailable {
                face: Box::new(self.catalog.emoji_face().clone()),
            });
        }
        let key = cache_key(request);
        if let Some(cached) = self.cache.get(&key) {
            self.stats.cache_hits += 1;
            return Ok(self.cached_result(cached));
        }
        self.stats.cache_misses += 1;
        let mut raster = self.rasterize_uncached(request)?;
        self.insert_cache(key, raster.clone());
        raster.report = self.report(false);
        Ok(raster)
    }

    pub fn measure_text(
        &mut self,
        request: &PlatformTextMetricsRequest,
    ) -> Result<PlatformTextMetrics, PlatformTextRasterError> {
        if request.text.is_empty() {
            return Err(PlatformTextRasterError::EmptyText);
        }
        let emoji_face = self.catalog.emoji_face().clone();
        self.catalog
            .with_font_system_for_face_selection(self.face_selection, |font_system| {
                TextLayoutRasterizer::measure(font_system, request, &emoji_face, &self.text_faces)
            })
            .map_err(|_| PlatformTextRasterError::CatalogAccess)?
    }

    pub fn measure_line_metrics(
        &mut self,
        request: &PlatformTextRasterRequest,
    ) -> Result<PlatformTextLineMetrics, PlatformTextRasterError> {
        let emoji_face = self.catalog.emoji_face().clone();
        self.catalog
            .with_font_system_for_face_selection(self.face_selection, |font_system| {
                TextLayoutRasterizer::line_metrics(
                    font_system,
                    request,
                    &emoji_face,
                    &self.text_faces,
                )
            })
            .map_err(|_| PlatformTextRasterError::CatalogAccess)?
    }

    fn cached_result(&self, cached: &PlatformTextRaster) -> PlatformTextRaster {
        let mut result = cached.clone();
        result.report = self.report(true);
        result
    }

    fn rasterize_uncached(
        &mut self,
        request: &PlatformTextRasterRequest,
    ) -> Result<PlatformTextRaster, PlatformTextRasterError> {
        let emoji_face = self.catalog.emoji_face().clone();
        let raster = self
            .catalog
            .with_font_system_for_face_selection(self.face_selection, |font_system| {
                TextLayoutRasterizer::rasterize(
                    font_system,
                    &mut self.swash_cache,
                    request,
                    &emoji_face,
                    &self.text_faces,
                )
            })
            .map_err(|_| PlatformTextRasterError::CatalogAccess)??;
        Ok(PlatformTextRaster {
            text: request.text(),
            width: raster.width,
            height: raster.height,
            rgba_pixels: raster.rgba_pixels,
            grapheme_bounds: raster.grapheme_bounds,
            report: self.report(false),
        })
    }

    fn insert_cache(&mut self, key: String, raster: PlatformTextRaster) {
        if self.cache.len() >= self.cache_capacity
            && let Some(oldest) = self.cache_order.pop_front()
        {
            self.cache.remove(&oldest);
        }
        self.cache_order.push_back(key.clone());
        self.cache.insert(key, raster);
        self.stats.cache_entries = self.cache.len();
    }

    fn report(&self, cache_hit: bool) -> PlatformTextRasterReport {
        let emoji_face = self.catalog.emoji_face().clone();
        PlatformTextRasterReport {
            resolved_emoji_font_family: emoji_face.resolved_family().map(str::to_string),
            color_emoji_font_available: emoji_face.is_available(),
            emoji_face,
            cache_hit,
            stats: self.stats,
        }
    }
}

fn cache_key(request: &PlatformTextRasterRequest) -> String {
    format!("{request:?}")
}

#[cfg(test)]
#[path = "rasterizer_tests.rs"]
mod tests;
