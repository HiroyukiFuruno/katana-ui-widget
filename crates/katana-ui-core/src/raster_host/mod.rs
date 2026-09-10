//! Framework-neutral KUC raster and UI-tree host.
//!
//! This is the registry-safe rendering and interaction subset shared with the
//! private Storybook. It intentionally excludes windowing and GUI runtimes.

pub(crate) mod canvas;
pub(crate) mod canvas_blit;
pub(crate) mod canvas_clip;
pub(crate) mod canvas_color;
pub(crate) mod canvas_model;
pub(crate) mod canvas_physical;
pub(crate) mod canvas_png;
pub(crate) mod canvas_rendering;
pub(crate) mod canvas_round_rect;
pub(crate) mod canvas_scale;
pub(crate) mod canvas_scroll;
pub(crate) mod canvas_text_selection;
pub(crate) mod canvas_viewport;
pub mod document_typography;
pub(crate) mod layout_metrics;
pub(crate) mod palette;
pub(crate) mod presentation;
pub(crate) mod presentation_frame;
pub(crate) mod presentation_frame_scale;
pub(crate) mod presentation_frame_scale_average;
pub(crate) mod switch_control;
pub(crate) mod text;
pub(crate) mod text_selection;
pub(crate) mod text_selection_overlay;
pub(crate) mod text_selection_types;
pub(crate) mod ui_tree_canvas;
pub(crate) mod ui_tree_canvas_checkbox;
pub(crate) mod ui_tree_canvas_choice_control;
pub(crate) mod ui_tree_canvas_context_menu;
pub(crate) mod ui_tree_canvas_control;
pub(crate) mod ui_tree_canvas_entry;
pub(crate) mod ui_tree_canvas_extensions;
pub(crate) mod ui_tree_canvas_grid;
pub(crate) mod ui_tree_canvas_grid_border;
pub(crate) mod ui_tree_canvas_hit;
pub(crate) mod ui_tree_canvas_hit_api;
pub(crate) mod ui_tree_canvas_hit_metrics;
pub(crate) mod ui_tree_canvas_hover;
pub(crate) mod ui_tree_canvas_image_blit;
pub(crate) mod ui_tree_canvas_image_cache;
pub(crate) mod ui_tree_canvas_image_cache_entry;
pub(crate) mod ui_tree_canvas_image_cache_key;
pub(crate) mod ui_tree_canvas_image_metrics;
pub(crate) mod ui_tree_canvas_image_raster_cache;
pub(crate) mod ui_tree_canvas_layout;
pub(crate) mod ui_tree_canvas_loading;
pub(crate) mod ui_tree_canvas_palette;
pub(crate) mod ui_tree_canvas_renderer_image;
pub(crate) mod ui_tree_canvas_rgba;
pub(crate) mod ui_tree_canvas_row_layout;
pub(crate) mod ui_tree_canvas_scroll;
pub(crate) mod ui_tree_canvas_scroll_height_cache;
pub(crate) mod ui_tree_canvas_scroll_measure;
pub(crate) mod ui_tree_canvas_scroll_partial;
pub(crate) mod ui_tree_canvas_separator;
pub(crate) mod ui_tree_canvas_settings;
pub(crate) mod ui_tree_canvas_svg_icon;
pub(crate) mod ui_tree_canvas_text;
pub(crate) mod ui_tree_canvas_text_line_width;
pub(crate) mod ui_tree_canvas_text_metrics;
pub(crate) mod ui_tree_canvas_text_role;
pub(crate) mod ui_tree_canvas_tree;
pub(crate) mod ui_tree_canvas_tree_parts;
pub(crate) mod ui_tree_canvas_types;
pub(crate) mod ui_tree_interaction_surface;
pub(crate) mod ui_tree_storybook_host;
pub(crate) mod ui_tree_surface_host;

#[cfg(test)]
mod canvas_extensions_regression_tests;
#[cfg(test)]
mod canvas_fractional_y_tests;
#[cfg(test)]
mod canvas_regression_tests;
#[cfg(test)]
mod canvas_retina_regression_tests;
#[cfg(test)]
mod canvas_text_selection_regression_tests;
#[cfg(test)]
mod ui_tree_canvas_grid_regression_tests;
#[cfg(test)]
mod ui_tree_canvas_hit_tests;
#[cfg(test)]
mod ui_tree_canvas_image_cache_regression_tests;
#[cfg(test)]
mod ui_tree_canvas_image_cache_tests;
#[cfg(test)]
mod ui_tree_canvas_interactive_hover_tests;
#[cfg(test)]
mod ui_tree_canvas_row_layout_tests;
#[cfg(test)]
mod ui_tree_canvas_scroll_offset_tests;
#[cfg(test)]
mod ui_tree_canvas_text_wrap_support;
#[cfg(test)]
mod ui_tree_canvas_text_wrap_tests;
#[cfg(test)]
mod ui_tree_canvas_tree_context_tests;
#[cfg(test)]
mod ui_tree_canvas_tree_katana_tests;
#[cfg(test)]
mod ui_tree_canvas_tree_test_support;
#[cfg(test)]
mod ui_tree_canvas_tree_tests;
#[cfg(test)]
mod ui_tree_storybook_host_tests;
#[cfg(test)]
mod ui_tree_surface_host_document_typography_tests;
#[cfg(test)]
mod ui_tree_surface_host_tests;

pub use canvas::Canvas;
pub use document_typography::{
    UiTreeDocumentTypography, UiTreeTextRoleBaselineTypography, UiTreeTextRoleTypography,
};
pub use presentation::StorybookPresentation;
pub use text::TextRenderer;
pub use text_selection::SelectableTextRun;
pub use ui_tree_canvas::UiTreeCanvasRenderer;
pub use ui_tree_canvas_types::{
    CanvasBlitRequest, RgbaBlitRequest, RgbaSourceRect, UiTreeHitRect, UiTreeHostActionHit,
    UiTreeHostActionHitQuery, UiTreeInteractionTarget, UiTreeNodeHit, UiTreeRenderArea,
};
pub use ui_tree_interaction_surface::UiTreeInteractionSurface;
pub use ui_tree_storybook_host::UiTreeStorybookHost;
pub use ui_tree_surface_host::UiTreeSurfaceHost;
