use super::canvas::Canvas;
use super::text::TextRenderer;
use super::ui_tree_canvas_text_metrics::UiTreeTextMetrics;
use super::ui_tree_canvas_text_role::UiTreeTextRoleRenderer;
use super::ui_tree_canvas_types::UiTreeRenderArea;
use katana_ui_core::render_model::{UiDimension, UiNode};

#[path = "ui_tree_canvas_text_lines.rs"]
mod text_lines;
#[path = "ui_tree_canvas_text_table.rs"]
mod text_table;
#[path = "ui_tree_canvas_text_types.rs"]
mod text_types;

use text_lines::{UiTreeTextLineContext, UiTreeTextLines};
use text_table::{UiTreeTextTable, UiTreeTextTableContext};
pub(super) use text_types::{UiTreeTextContext, UiTreeTextRenderer};

const TEXT_CLIP_VERTICAL_GUARD_RATIO: f32 = 0.35;
impl UiTreeTextRenderer {
    pub(super) fn measure_node_height(
        context: UiTreeTextContext<'_>,
        node: &UiNode,
        x: usize,
        area: UiTreeRenderArea,
    ) -> usize {
        let renderer = renderer_for_role(
            context.text,
            context.export_text,
            context.code_text,
            &node.props().font_role,
        );
        let metrics = UiTreeTextMetrics::for_node_with_typography(node, context.typography);
        let content_x = text_content_x(node, x);
        let requested_height = dimension_px(&node.props().common.height);
        if node.props().text.role == "table" {
            let content_height =
                UiTreeTextTable::content_height(renderer, node, content_x, area, metrics);
            return explicit_or_content_height(requested_height, content_height);
        }
        let line_count = UiTreeTextLines::line_count(
            renderer,
            context.code_text,
            node,
            content_x,
            area,
            metrics,
        );
        text_advance_height(requested_height, line_count, metrics)
    }

    pub(super) fn draw_node(
        canvas: &mut Canvas,
        context: UiTreeTextContext<'_>,
        node: &UiNode,
        x: usize,
        y: &mut usize,
        area: UiTreeRenderArea,
    ) {
        let renderer = renderer_for_role(
            context.text,
            context.export_text,
            context.code_text,
            &node.props().font_role,
        );
        let mut metrics = UiTreeTextMetrics::for_node_with_typography(node, context.typography);
        let requested_height = dimension_px(&node.props().common.height);
        let content_x = text_content_x(node, x);
        if node.props().text.role == "table" {
            let content_height =
                UiTreeTextTable::content_height(renderer, node, content_x, area, metrics);
            let advance_height = explicit_or_content_height(requested_height, content_height);
            canvas.with_clip(
                content_x,
                *y,
                text_clip_width(node, area, content_x),
                advance_height,
                &mut |canvas| {
                    UiTreeTextTable::draw(
                        canvas,
                        UiTreeTextTableContext {
                            renderer,
                            node,
                            area,
                            palette: context.palette,
                            metrics,
                        },
                        content_x,
                        *y,
                    );
                },
            );
            *y = y.saturating_add(advance_height);
            return;
        }
        let line_count = UiTreeTextLines::line_count(
            renderer,
            context.code_text,
            node,
            content_x,
            area,
            metrics,
        );
        if requested_height > 0 {
            metrics.background_height = requested_height;
        } else {
            metrics.background_height = metrics
                .background_height
                .max(text_line_box_height(line_count, metrics));
        }
        let clip_height = text_clip_height(requested_height, line_count, metrics);

        let clip_width = text_clip_width(node, area, content_x);
        let draw_metrics = draw_metrics_for_node(node, metrics);
        UiTreeTextRoleRenderer::draw_background(
            canvas,
            node,
            x,
            *y,
            area,
            context.palette,
            metrics,
        );
        let text_y = y.saturating_add(metrics.top_margin);
        if node.props().text.spans.is_empty() {
            canvas.with_clip(content_x, *y, clip_width, clip_height, &mut |canvas| {
                UiTreeTextLines::draw_plain(
                    canvas,
                    UiTreeTextLineContext {
                        renderer,
                        code_renderer: context.code_text,
                        node,
                        area,
                        palette: context.palette,
                        metrics: draw_metrics,
                    },
                    content_x,
                    content_x,
                    text_y,
                );
            });
            *y = y.saturating_add(text_advance_height(requested_height, line_count, metrics));
            return;
        }
        canvas.with_clip(content_x, *y, clip_width, clip_height, &mut |canvas| {
            UiTreeTextLines::draw_spans(
                canvas,
                UiTreeTextLineContext {
                    renderer,
                    code_renderer: context.code_text,
                    node,
                    area,
                    palette: context.palette,
                    metrics: draw_metrics,
                },
                content_x,
                text_y,
            );
        });
        *y = y.saturating_add(text_advance_height(requested_height, line_count, metrics));
    }
}

fn remaining_width(area: UiTreeRenderArea, x: usize) -> usize {
    area.width.saturating_sub(x.saturating_sub(area.x)).max(1)
}

fn dimension_px(value: &UiDimension) -> usize {
    match value {
        UiDimension::Px(value) => usize::from(*value),
        _ => 0,
    }
}

fn padding_right(node: &UiNode) -> usize {
    dimension_px(&node.props().common.padding.right)
}

fn margin_left(node: &UiNode) -> usize {
    dimension_px(&node.props().common.margin.left)
}

fn margin_right(node: &UiNode) -> usize {
    dimension_px(&node.props().common.margin.right)
}

fn text_content_x(node: &UiNode, x: usize) -> usize {
    x.saturating_add(margin_left(node))
}

fn text_clip_width(node: &UiNode, area: UiTreeRenderArea, content_x: usize) -> usize {
    remaining_width(area, content_x)
        .saturating_sub(padding_right(node))
        .saturating_sub(margin_right(node))
        .max(1)
}

fn text_advance_height(
    requested_height: usize,
    line_count: usize,
    metrics: UiTreeTextMetrics,
) -> usize {
    if requested_height > 0 {
        return requested_height;
    }
    text_line_box_height(line_count, metrics)
}

fn text_line_box_height(line_count: usize, metrics: UiTreeTextMetrics) -> usize {
    (line_count as f32 * metrics.line_box_height)
        .ceil()
        .max(1.0) as usize
}

fn explicit_or_content_height(requested_height: usize, content_height: usize) -> usize {
    if requested_height > 0 {
        return requested_height.max(content_height);
    }
    content_height
}

fn draw_metrics_for_node(_node: &UiNode, metrics: UiTreeTextMetrics) -> UiTreeTextMetrics {
    metrics
}

fn text_clip_height(
    requested_height: usize,
    line_count: usize,
    metrics: UiTreeTextMetrics,
) -> usize {
    let line_height = metrics.top_margin.saturating_add(
        text_line_box_height(line_count, metrics).saturating_add(text_clip_guard(metrics)),
    );
    if requested_height > 0 {
        return requested_height.max(line_height);
    }
    metrics.background_height.max(line_height)
}

fn text_clip_guard(metrics: UiTreeTextMetrics) -> usize {
    (metrics.font_size * TEXT_CLIP_VERTICAL_GUARD_RATIO)
        .ceil()
        .max(0.0) as usize
}

fn renderer_for_role<'a>(
    text: &'a TextRenderer,
    export_text: &'a TextRenderer,
    code_text: &'a TextRenderer,
    role: &str,
) -> &'a TextRenderer {
    if role == "code" || role == "document-code" {
        return code_text;
    }
    if role == "document-export-body" {
        return export_text;
    }
    text
}

#[cfg(test)]
mod tests {
    use super::{
        UiTreeRenderArea, UiTreeTextContext, UiTreeTextMetrics, UiTreeTextRenderer,
        explicit_or_content_height, renderer_for_role, text_clip_height,
    };
    use crate::raster_host::canvas::Canvas;
    use crate::raster_host::text::TextRenderer;
    use crate::raster_host::ui_tree_canvas_palette::UiTreeCanvasPalette;
    use crate::raster_host::ui_tree_canvas_text_metrics::UiTreeDocumentTypography;
    use crate::test_assert::KucTestExpect;
    use katana_ui_core::atom::Text;
    use katana_ui_core::facade::UiCoreFacade;
    use katana_ui_core::render_model::{UiCommonProps, UiDimension, UiEdgeInsets, UiNode};
    use katana_ui_core::theme::ThemeSnapshot;

    #[test]
    fn explicit_document_height_does_not_clip_taller_raster_line() {
        let metrics = UiTreeTextMetrics {
            font_size: 24.79,
            line_height: 36,
            line_box_height: 36.0,
            top_margin: 0,
            baseline_from_line_box_top: None,
            background_height: 34,
            highlight_height: 34,
            underline_offset: 30,
            strikethrough_offset: 17,
            raster_vertical_scale: 1.0,
        };

        assert_eq!(45, text_clip_height(34, 1, metrics));
    }

    #[test]
    fn explicit_document_height_includes_top_margin_in_clip_height() {
        let metrics = UiTreeTextMetrics {
            font_size: 19.83,
            line_height: 34,
            line_box_height: 34.0,
            top_margin: 29,
            baseline_from_line_box_top: None,
            background_height: 34,
            highlight_height: 5,
            underline_offset: 24,
            strikethrough_offset: 14,
            raster_vertical_scale: 1.0,
        };

        assert_eq!(70, text_clip_height(47, 1, metrics));
    }

    #[test]
    fn explicit_text_height_never_truncates_measured_content() {
        let resolve_height =
            std::hint::black_box(explicit_or_content_height as fn(usize, usize) -> usize);
        assert_eq!(
            42,
            resolve_height(std::hint::black_box(20), std::hint::black_box(42))
        );
        assert_eq!(
            42,
            resolve_height(std::hint::black_box(42), std::hint::black_box(20))
        );
        assert_eq!(
            20,
            resolve_height(std::hint::black_box(20), std::hint::black_box(5))
        );
        assert_eq!(
            42,
            resolve_height(std::hint::black_box(0), std::hint::black_box(42))
        );
    }

    #[test]
    fn table_measurement_expands_past_underestimated_viewer_height() {
        let context = text_context();
        let node = table_node_with_height(std::hint::black_box(80_u16));
        let area = render_area();

        let height = UiTreeTextRenderer::measure_node_height(context, &node, 56, area);

        assert_eq!(132, height);
    }

    #[test]
    fn table_draw_advances_past_underestimated_viewer_height() {
        let context = text_context();
        let node = table_node_with_height(std::hint::black_box(80_u16));
        let area = render_area();
        let mut canvas = Canvas::new(320, 220, 0xffffff);
        let mut y = 12;

        UiTreeTextRenderer::draw_node(&mut canvas, context, &node, 56, &mut y, area);

        assert_eq!(144, y);
    }

    #[test]
    fn text_node_left_margin_moves_rendered_ink_origin() {
        let context = text_context();
        let node = Text::new("Margin")
            .common(UiCommonProps::default().margin(UiEdgeInsets {
                left: UiDimension::Px(40),
                ..UiEdgeInsets::default()
            }))
            .into();
        let area = render_area();
        let mut canvas = Canvas::new(320, 120, 0xffffff);
        let mut y = 12;

        UiTreeTextRenderer::draw_node(&mut canvas, context, &node, 56, &mut y, area);

        let (min_x, _) = horizontal_non_background_bounds(&canvas, 0xffffff).kuc_unwrap();
        assert!(
            min_x >= 96,
            "text ink must start after the 40px left margin: min_x={min_x}"
        );
    }

    #[test]
    fn text_role_selects_body_export_and_code_renderers() {
        let context = text_context();

        assert!(std::ptr::eq(
            context.text,
            renderer_for_role(context.text, context.export_text, context.code_text, "body")
        ));
        assert!(std::ptr::eq(
            context.export_text,
            renderer_for_role(
                context.text,
                context.export_text,
                context.code_text,
                "document-export-body"
            )
        ));
        for role in ["code", "document-code"] {
            assert!(std::ptr::eq(
                context.code_text,
                renderer_for_role(context.text, context.export_text, context.code_text, role)
            ));
        }
    }

    fn text_context() -> UiTreeTextContext<'static> {
        let facade = Box::leak(Box::new(UiCoreFacade::default()));
        let text = Box::leak(Box::new(TextRenderer::load(facade, "body")));
        let export_text = Box::leak(Box::new(TextRenderer::load(facade, "body")));
        let code_text = Box::leak(Box::new(TextRenderer::load(facade, "code")));
        UiTreeTextContext {
            text,
            export_text,
            code_text,
            palette: UiTreeCanvasPalette::from_theme(&ThemeSnapshot::light()),
            typography: UiTreeDocumentTypography::default(),
        }
    }

    fn table_node_with_height(height: u16) -> UiNode {
        let node: UiNode = Text::new("Header\nTable after list")
            .text_role("table")
            .into();
        node.height(UiDimension::Px(height))
    }

    fn render_area() -> UiTreeRenderArea {
        UiTreeRenderArea {
            x: 0,
            y: 0,
            width: 320,
            height: 220,
            scroll_y: 0.0,
        }
    }

    fn horizontal_non_background_bounds(
        canvas: &Canvas,
        background: u32,
    ) -> Option<(usize, usize)> {
        let mut min_x = usize::MAX;
        let mut max_x = 0usize;
        for (index, pixel) in canvas.pixels().iter().enumerate() {
            if *pixel == background {
                continue;
            }
            let x = index % canvas.width();
            min_x = min_x.min(x);
            max_x = max_x.max(x);
        }
        (min_x <= max_x).then_some((min_x, max_x))
    }
}
