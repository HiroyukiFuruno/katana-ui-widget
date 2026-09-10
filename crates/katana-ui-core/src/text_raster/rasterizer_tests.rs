use super::*;
use crate::text_raster::{
    PlatformFontCatalog, PlatformTextFaceSelection, PlatformTextRasterConfig,
    PlatformTextRasterRequest,
};
use crate::theme::{FontFamily, FontToken};
use cosmic_text::{
    Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Style,
    fontdb::{Database, Source},
};
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

const TEXT_COLOR: [u8; 4] = [245, 245, 245, 255];
const TEST_FONT_SIZE_PX: f32 = 18.0;
const TEST_FONT_WEIGHT: u16 = 400;
const SOURCE_IDENTITY_TEXT: &str = "Candidate source";

static NEXT_TEST_PATH: AtomicU64 = AtomicU64::new(0);

#[path = "rasterizer_tests_line_metrics.rs"]
mod line_metrics;

fn installed_font_candidate() -> io::Result<(PathBuf, String)> {
    let mut candidate_paths = HashSet::new();
    let (path, family) = FontSystem::new()
        .db()
        .faces()
        .filter_map(|face| match &face.source {
            Source::File(path) => candidate_paths.insert(path.clone()).then(|| path.clone()),
            _ => None,
        })
        .find_map(loaded_candidate_with_identity_text)
        .ok_or_else(|| io::Error::other("a Latin-capable file-backed system font is required"))?;
    Ok((path, family))
}

fn loaded_candidate_with_identity_text(path: PathBuf) -> Option<(PathBuf, String)> {
    let mut database = Database::new();
    database.load_font_file(&path).ok()?;
    let mut font_system = FontSystem::new_with_locale_and_db(String::new(), database);
    let (face_id, weight, style, family) = font_system.db().faces().next().and_then(|face| {
        face.families
            .first()
            .map(|(family, _)| (face.id, face.weight, face.style, family.clone()))
    })?;
    if weight.0 != TEST_FONT_WEIGHT || style != Style::Normal {
        return None;
    }
    font_system
        .get_font(face_id, weight)
        .is_some_and(|font| {
            SOURCE_IDENTITY_TEXT
                .chars()
                .all(|character| font.as_swash().charmap().map(character) != 0)
        })
        .then_some((path, family))
}

fn missing_font_path() -> PathBuf {
    let serial = NEXT_TEST_PATH.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "kuc-first-candidate-{serial}-{}.ttf",
        std::process::id()
    ))
}

fn font(family: FontFamily) -> FontToken {
    FontToken {
        name: "candidate-selection".to_owned(),
        family,
        size: TEST_FONT_SIZE_PX,
        weight: TEST_FONT_WEIGHT,
    }
}

fn first_candidate_config(candidate: PathBuf) -> PlatformTextRasterConfig {
    first_candidate_config_for_faces(candidate.clone(), candidate)
}

fn first_candidate_config_for_faces(
    proportional_candidate: PathBuf,
    monospace_candidate: PathBuf,
) -> PlatformTextRasterConfig {
    let missing = missing_font_path();
    PlatformTextRasterConfig {
        proportional_candidates: vec![missing.clone(), proportional_candidate],
        monospace_candidates: vec![missing, monospace_candidate],
        emoji_candidates: Vec::new(),
        emoji_candidate_sha256: Vec::new(),
        cache_capacity: 4,
    }
}

fn copy_font_candidate(source: &Path) -> io::Result<PathBuf> {
    let serial = NEXT_TEST_PATH.fetch_add(1, Ordering::Relaxed);
    let extension = source
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("ttf");
    let candidate = std::env::temp_dir().join(format!(
        "kuc-first-candidate-source-{serial}-{}.{}",
        std::process::id(),
        extension
    ));
    fs::copy(source, &candidate)?;
    Ok(candidate)
}

fn source_path(source: &Source) -> Option<&Path> {
    match source {
        Source::File(path) | Source::SharedFile(path, _) => Some(path),
        Source::Binary(_) => None,
    }
}

fn first_shaped_font_source(
    rasterizer: &PlatformTextRasterizer,
    family: &str,
) -> io::Result<PathBuf> {
    rasterizer
        .catalog()
        .with_font_system_for_face_selection(
            PlatformTextFaceSelection::FirstCandidate,
            |font_system| {
                let font_id = {
                    let mut buffer = Buffer::new(font_system, Metrics::new(18.0, 18.0));
                    let mut buffer = buffer.borrow_with(font_system);
                    buffer.set_size(Some(1024.0), Some(1024.0));
                    buffer.set_rich_text(
                        [(
                            "Candidate source",
                            Attrs::new().family(Family::Name(family)),
                        )],
                        &Attrs::new(),
                        Shaping::Advanced,
                        None,
                    );
                    buffer
                        .layout_runs()
                        .flat_map(|run| run.glyphs.iter())
                        .map(|glyph| glyph.font_id)
                        .next()
                        .ok_or_else(|| io::Error::other("candidate text did not shape"))?
                };
                let face = font_system
                    .db()
                    .face(font_id)
                    .ok_or_else(|| io::Error::other("shaped glyph face is unavailable"))?;
                source_path(&face.source)
                    .map(Path::to_path_buf)
                    .ok_or_else(|| io::Error::other("shaped glyph is not file-backed"))
            },
        )
        .map_err(|error| io::Error::other(format!("{error:?}")))?
}

#[test]
fn first_candidate_selection_reaches_regular_and_monospace_rasterization()
-> Result<(), Box<dyn std::error::Error>> {
    let (candidate, family) = installed_font_candidate()?;
    let config = first_candidate_config(candidate);
    let catalog = Arc::new(PlatformFontCatalog::new(config.catalog_policy()));
    let resolved_faces = catalog.regular_font_faces();
    let mut rasterizer = PlatformTextRasterizer::with_catalog_and_face_selection(
        catalog,
        config,
        PlatformTextFaceSelection::FirstCandidate,
    )?;

    assert_eq!(
        resolved_faces
            .proportional
            .as_ref()
            .map(|face| face.family.as_str()),
        Some(family.as_str())
    );
    assert_eq!(
        resolved_faces
            .monospace
            .as_ref()
            .map(|face| face.family.as_str()),
        Some(family.as_str())
    );

    let regular = rasterizer.rasterize(&PlatformTextRasterRequest::from_text(
        "Regular candidate",
        font(FontFamily::Proportional),
        TEXT_COLOR,
    ))?;
    let monospace = rasterizer.rasterize(&PlatformTextRasterRequest::from_text(
        "Monospace candidate",
        font(FontFamily::Monospace),
        TEXT_COLOR,
    ))?;

    assert!(regular.width > 0 && regular.height > 0);
    assert!(monospace.width > 0 && monospace.height > 0);
    assert!(!regular.rgba_pixels.is_empty());
    assert!(!monospace.rgba_pixels.is_empty());
    Ok(())
}

#[test]
fn first_candidate_selection_preserves_the_candidate_source_through_shaping()
-> Result<(), Box<dyn std::error::Error>> {
    let (source, family) = installed_font_candidate()?;
    let proportional_candidate = copy_font_candidate(&source)?;
    let monospace_candidate = copy_font_candidate(&source)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let config = first_candidate_config_for_faces(
            proportional_candidate.clone(),
            monospace_candidate.clone(),
        );
        let catalog = Arc::new(PlatformFontCatalog::new(config.catalog_policy()));
        let mut rasterizer = PlatformTextRasterizer::with_catalog_and_face_selection(
            catalog,
            config,
            PlatformTextFaceSelection::FirstCandidate,
        )?;

        let raster = rasterizer.rasterize(&PlatformTextRasterRequest::from_text(
            "Candidate source",
            font(FontFamily::Proportional),
            TEXT_COLOR,
        ))?;
        assert!(raster.width > 0 && raster.height > 0);
        let selected_family = rasterizer
            .text_faces
            .proportional()
            .expect("first candidate alias")
            .to_owned();
        assert_ne!(selected_family, family);
        assert_eq!(
            first_shaped_font_source(&rasterizer, &selected_family)?,
            proportional_candidate
        );

        let monospace_raster = rasterizer.rasterize(&PlatformTextRasterRequest::from_text(
            "Candidate source",
            font(FontFamily::Monospace),
            TEXT_COLOR,
        ))?;
        assert!(monospace_raster.width > 0 && monospace_raster.height > 0);
        let monospace_family = rasterizer
            .text_faces
            .monospace()
            .expect("first candidate monospace alias")
            .to_owned();
        assert_eq!(
            first_shaped_font_source(&rasterizer, &monospace_family)?,
            monospace_candidate
        );
        Ok(())
    })();
    let _ = fs::remove_file(proportional_candidate);
    let _ = fs::remove_file(monospace_candidate);
    result
}

#[test]
fn unresolved_first_candidate_selection_keeps_generic_fallback_faces()
-> Result<(), Box<dyn std::error::Error>> {
    let missing = missing_font_path();
    let config = PlatformTextRasterConfig {
        proportional_candidates: vec![missing.clone()],
        monospace_candidates: vec![missing],
        emoji_candidates: Vec::new(),
        emoji_candidate_sha256: Vec::new(),
        cache_capacity: 4,
    };
    let catalog = Arc::new(PlatformFontCatalog::new(config.catalog_policy()));
    let mut rasterizer = PlatformTextRasterizer::with_catalog_and_face_selection(
        catalog,
        config,
        PlatformTextFaceSelection::FirstCandidate,
    )?;

    assert_eq!(rasterizer.text_faces, ResolvedTextFaces::default());
    let raster = rasterizer.rasterize(&PlatformTextRasterRequest::from_text(
        "System fallback",
        font(FontFamily::Proportional),
        TEXT_COLOR,
    ))?;

    assert!(raster.width > 0 && raster.height > 0);
    Ok(())
}
