mod catalog;
mod catalog_types;
mod config;
mod font_candidates;
mod layout;
mod model;
mod rasterizer;
mod resources;
mod surface_layout;

pub use catalog::{PlatformFontCatalog, PlatformFontCatalogStats};
pub use catalog_types::{
    PlatformColorEmojiAvailability, PlatformColorEmojiError, PlatformColorEmojiFaceRecord,
    PlatformColorEmojiFaceResolver, PlatformColorEmojiUnavailableReason,
    PlatformEmojiFontCandidate, PlatformEmojiFontLoadError, PlatformEmojiFontLoader,
    PlatformEmojiFontObservation, PlatformFontCatalogError, PlatformFontCatalogFingerprint,
    PlatformFontCatalogPolicy, PlatformFontProfile, PlatformFontSha256,
};
pub use config::{PlatformTextFaceSelection, PlatformTextRasterConfig};
pub use model::{
    PlatformTextGraphemeAdvance, PlatformTextGraphemeBounds, PlatformTextGraphemeRange,
    PlatformTextHit, PlatformTextLineMetrics, PlatformTextMetrics, PlatformTextMetricsFrame,
    PlatformTextMetricsRequest, PlatformTextRaster, PlatformTextRasterCrop,
    PlatformTextRasterError, PlatformTextRasterReport, PlatformTextRasterRequest,
    PlatformTextRasterStats,
};
pub use rasterizer::PlatformTextRasterizer;
pub use resources::PlatformTextRasterResources;

#[cfg(test)]
mod catalog_contract_tests;
#[cfg(test)]
mod surface_layout_tests;
#[cfg(test)]
mod tests;
