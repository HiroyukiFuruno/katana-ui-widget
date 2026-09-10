use super::{
    SpanTextRenderers, UNDERLINE_HEIGHT_BOTTOM_PADDING, UiTreeTextMetrics, canvas_x,
    span_visible_part_bounds,
};
use crate::raster_host::canvas::Canvas;
use katana_ui_core::render_model::{UiDimension, UiNode, UiTextSpan};

pub(super) struct TextDecorationLine {
    pub(super) x: isize,
    pub(super) legacy_offset: usize,
    pub(super) width: usize,
    pub(super) color: u32,
    pub(super) thickness: usize,
}

impl TextDecorationLine {
    pub(super) fn draw(self, canvas: &mut Canvas, y: f32) {
        draw_decoration_line(canvas, self.x, y, self.width, self.color, self.thickness);
    }
}

pub(super) fn decoration_y(
    line_box_top: f32,
    baseline_from_line_box_top: Option<f32>,
    raster_baseline: f32,
    legacy_offset: usize,
) -> f32 {
    match baseline_from_line_box_top {
        Some(target_baseline) => {
            line_box_top + target_baseline + legacy_offset as f32 - raster_baseline
        }
        None => line_box_top.round().max(0.0) + legacy_offset as f32,
    }
}

pub(in super::super) fn underline_y_offset(metrics: UiTreeTextMetrics, node: &UiNode) -> usize {
    let underline = metrics.underline_offset;
    if let UiDimension::Px(height) = node.props().common.height {
        return underline.min(
            usize::from(height)
                .saturating_sub(metrics.top_margin)
                .saturating_sub(UNDERLINE_HEIGHT_BOTTOM_PADDING),
        );
    }
    underline
}

pub(super) fn underline_part_bounds(
    renderers: SpanTextRenderers<'_>,
    span: &UiTextSpan,
    metrics: UiTreeTextMetrics,
    preserve_whitespace: bool,
    _node: &UiNode,
    _full_width: usize,
) -> (usize, usize) {
    span_visible_part_bounds(renderers, span, metrics, preserve_whitespace)
}

fn draw_decoration_line(
    canvas: &mut Canvas,
    x: isize,
    y: f32,
    width: usize,
    color: u32,
    thickness: usize,
) {
    let Some(decoration_x) = canvas_x(x) else {
        return;
    };
    canvas.fill_rect_at_logical_y(decoration_x, y, width, thickness as f32, color);
}

#[cfg(test)]
mod tests {
    use super::{
        Canvas, UiTreeTextMetrics, decoration_y, draw_decoration_line, underline_y_offset,
    };
    use katana_ui_core::render_model::{UiDimension, UiNode, UiNodeKind};

    #[test]
    fn decoration_line_ignores_negative_horizontal_positions() {
        let mut canvas = Canvas::new(2, 2, 0);

        draw_decoration_line(&mut canvas, -1, 0.0, 1, 1, 1);

        assert!(canvas.pixels().iter().all(|pixel| *pixel == 0));
    }

    #[test]
    fn underline_offset_is_clipped_by_explicit_row_height() {
        let node = UiNode::new(UiNodeKind::Text, "short").height(UiDimension::Px(10));
        let metrics = UiTreeTextMetrics::for_node(&node);

        assert_eq!(7, underline_y_offset(metrics, &node));
        assert_eq!(
            metrics.underline_offset,
            underline_y_offset(metrics, &UiNode::new(UiNodeKind::Text, "auto"),)
        );
    }

    #[test]
    fn decoration_y_tracks_a_configured_text_baseline() {
        assert_eq!(21.0, decoration_y(8.5, Some(12.5), 9.0, 9));
        assert_eq!(18.0, decoration_y(8.5, None, 9.0, 9));
    }
}
