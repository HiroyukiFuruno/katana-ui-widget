use super::super::canvas::Canvas;
use super::super::text::{RichTextStyle, TextRenderer};
use super::super::ui_tree_canvas_palette::UiTreeCanvasPalette;
use super::super::ui_tree_canvas_text_metrics::UiTreeTextMetrics;
use super::super::ui_tree_canvas_text_role::UiTreeTextRoleRenderer;
use super::super::ui_tree_canvas_types::UiTreeRenderArea;
use katana_ui_core::render_model::{UiNode, UiTextSpan};

#[path = "ui_tree_canvas_text_lines_decoration.rs"]
mod line_decoration;
#[path = "ui_tree_canvas_text_lines_rich_span.rs"]
mod rich_span;
#[path = "ui_tree_canvas_text_span_style.rs"]
mod span_style;
#[path = "ui_tree_canvas_text_wrap.rs"]
mod text_wrap;
#[path = "ui_tree_canvas_text_wrap_state.rs"]
mod wrap_state;

use crate::raster_host::ui_tree_canvas_text_line_width::{
    SpanTextRenderers, preserves_whitespace, span_line_width, span_part_width,
    span_visible_part_bounds,
};
pub(super) use line_decoration::underline_y_offset;
use line_decoration::{TextDecorationLine, decoration_y, underline_part_bounds};
use rich_span::rich_line_span;
use span_style::{draw_span_background, should_strikethrough, should_underline, span_color};
use text_wrap::UiTreeTextWrap;

const STRIKETHROUGH_LINE_THICKNESS: usize = 1;
const UNDERLINE_LINE_THICKNESS: usize = 1;
const UNDERLINE_HEIGHT_BOTTOM_PADDING: usize = 3;

pub(super) struct UiTreeTextLines;

#[derive(Clone, Copy)]
pub(super) struct UiTreeTextLineContext<'a> {
    pub(super) renderer: &'a TextRenderer,
    pub(super) code_renderer: &'a TextRenderer,
    pub(super) node: &'a UiNode,
    pub(super) area: UiTreeRenderArea,
    pub(super) palette: UiTreeCanvasPalette,
    pub(super) metrics: UiTreeTextMetrics,
}

impl UiTreeTextLines {
    pub(super) fn draw_plain(
        canvas: &mut Canvas,
        context: UiTreeTextLineContext<'_>,
        origin_x: usize,
        x: usize,
        y: usize,
    ) {
        for (index, line) in UiTreeTextWrap::plain_lines(
            context.renderer,
            context.node,
            x,
            context.area,
            context.metrics,
        )
        .iter()
        .enumerate()
        {
            let Some(line_box_top) = visible_line_y(index, y, context.area, context.metrics) else {
                continue;
            };
            let color = UiTreeTextRoleRenderer::line_color(context.node, context.palette, index);
            let bold = UiTreeTextRoleRenderer::line_bold(context.node, index);
            let style = RichTextStyle::new(context.metrics.font_size, color)
                .bold(bold)
                .raster_vertical_scale(context.metrics.raster_vertical_scale);
            let width = if bold {
                context.renderer.measure_width_rich(line, style)
            } else {
                context
                    .renderer
                    .measure_width(line, context.metrics.font_size)
            }
            .max(1);
            let line_x = UiTreeTextRoleRenderer::line_x(
                context.node,
                origin_x,
                x,
                context.area,
                width,
                index,
            );
            if let Some(baseline) = context.metrics.baseline_from_line_box_top {
                context.renderer.draw_signed_styled_in_line_box(
                    canvas,
                    line,
                    line_x,
                    line_box_top,
                    context.metrics.line_box_height,
                    baseline,
                    style,
                );
            } else {
                context.renderer.draw_signed_styled(
                    canvas,
                    line,
                    line_x,
                    line_box_top.round().max(0.0) as usize,
                    style,
                );
            }
        }
    }

    pub(super) fn draw_spans(
        canvas: &mut Canvas,
        context: UiTreeTextLineContext<'_>,
        x: usize,
        y: usize,
    ) {
        let lines = UiTreeTextWrap::span_lines(
            SpanTextRenderers::new(context.renderer, context.code_renderer),
            context.node,
            x,
            context.area,
            context.metrics,
        );
        let preserve_whitespace = preserves_whitespace(context.node);
        let renderers = SpanTextRenderers::new(context.renderer, context.code_renderer);
        for (line_index, line) in lines.iter().enumerate() {
            let Some(line_box_top) = visible_line_y(line_index, y, context.area, context.metrics)
            else {
                continue;
            };
            let line_bold = UiTreeTextRoleRenderer::line_bold(context.node, line_index);
            let bold_line_storage;
            let render_line = if line_bold {
                bold_line_storage = line
                    .iter()
                    .cloned()
                    .map(force_span_bold)
                    .collect::<Vec<_>>();
                bold_line_storage.as_slice()
            } else {
                line.as_slice()
            };
            let line_width =
                span_line_width(renderers, render_line, context.metrics, preserve_whitespace);
            let line_x = UiTreeTextRoleRenderer::line_x(
                context.node,
                x,
                x,
                context.area,
                line_width,
                line_index,
            );
            let mut cursor_x = line_x;
            let mut rich_line = Vec::with_capacity(render_line.len());
            let mut decorations = Vec::new();
            for span in render_line {
                let width = span_part_width(renderers, span, context.metrics, preserve_whitespace);
                if let Some(background_x) = canvas_x(cursor_x) {
                    draw_span_background(
                        canvas,
                        background_x,
                        line_box_top,
                        width,
                        span.style,
                        context.palette,
                        context.metrics,
                    );
                }
                let color = span_color(span, context.palette);
                rich_line.push(rich_line_span(context, renderers, span, color));
                if should_underline(span) {
                    let (decoration_x, decoration_width) = underline_part_bounds(
                        renderers,
                        span,
                        context.metrics,
                        preserve_whitespace,
                        context.node,
                        width,
                    );
                    decorations.push(TextDecorationLine {
                        x: cursor_x.saturating_add(decoration_x as isize),
                        legacy_offset: underline_y_offset(context.metrics, context.node),
                        width: decoration_width,
                        color,
                        thickness: underline_line_thickness(),
                    });
                }
                if should_strikethrough(span) {
                    let (decoration_x, decoration_width) = span_visible_part_bounds(
                        renderers,
                        span,
                        context.metrics,
                        preserve_whitespace,
                    );
                    decorations.push(TextDecorationLine {
                        x: cursor_x.saturating_add(decoration_x as isize),
                        legacy_offset: context.metrics.strikethrough_offset,
                        width: decoration_width,
                        color,
                        thickness: STRIKETHROUGH_LINE_THICKNESS,
                    });
                }
                cursor_x += width as isize;
            }
            let raster_baseline = if let Some(baseline) = context.metrics.baseline_from_line_box_top
            {
                context.renderer.draw_rich_line_signed_in_line_box(
                    canvas,
                    &rich_line,
                    line_x,
                    line_box_top,
                    context.metrics.line_box_height,
                    baseline,
                )
            } else {
                context.renderer.draw_rich_line_signed(
                    canvas,
                    &rich_line,
                    line_x,
                    line_box_top.round().max(0.0) as usize,
                );
                0.0
            };
            for decoration in decorations {
                let y = decoration_y(
                    line_box_top,
                    context.metrics.baseline_from_line_box_top,
                    raster_baseline,
                    decoration.legacy_offset,
                );
                decoration.draw(canvas, y);
            }
        }
    }

    pub(super) fn line_count(
        renderer: &TextRenderer,
        code_renderer: &TextRenderer,
        node: &UiNode,
        x: usize,
        area: UiTreeRenderArea,
        metrics: UiTreeTextMetrics,
    ) -> usize {
        if node.props().text.spans.is_empty() {
            return UiTreeTextWrap::plain_lines(renderer, node, x, area, metrics)
                .len()
                .max(1);
        }
        UiTreeTextWrap::span_lines(
            SpanTextRenderers::new(renderer, code_renderer),
            node,
            x,
            area,
            metrics,
        )
        .len()
        .max(1)
    }
}

fn force_span_bold(mut span: UiTextSpan) -> UiTextSpan {
    span.style.bold = true;
    span
}

fn canvas_x(x: isize) -> Option<usize> {
    usize::try_from(x).ok()
}

fn visible_line_y(
    line_index: usize,
    y: usize,
    area: UiTreeRenderArea,
    metrics: UiTreeTextMetrics,
) -> Option<f32> {
    let line_top = line_index as f32 * metrics.line_box_height;
    let line_bottom = line_top + metrics.line_box_height;
    let scroll_y = area.scroll_y.max(0.0);
    let viewport_bottom = scroll_y + area.height as f32;
    if line_bottom <= scroll_y {
        return None;
    }
    if line_top >= viewport_bottom {
        return None;
    }
    Some(y as f32 + (line_top - scroll_y).max(0.0))
}

const fn underline_line_thickness() -> usize {
    UNDERLINE_LINE_THICKNESS
}

#[cfg(test)]
#[path = "ui_tree_canvas_text_lines_tests.rs"]
mod tests;
