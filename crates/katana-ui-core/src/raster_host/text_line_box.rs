use super::text_runtime::ui_span;
use super::{Canvas, RichTextLineSpan, RichTextStyle, TextRenderer};
use katana_ui_core::render_model::UiTextSpan;
use katana_ui_core::theme::FontToken;

impl TextRenderer {
    pub(crate) fn draw_signed_styled_in_line_box(
        &self,
        canvas: &mut Canvas,
        text: &str,
        x: isize,
        line_box_top: f32,
        line_box_height: f32,
        baseline_from_line_box_top: f32,
        style: RichTextStyle,
    ) {
        self.draw_layout_in_line_box(
            canvas,
            vec![ui_span(text, style)],
            x,
            line_box_top,
            line_box_height,
            baseline_from_line_box_top,
            style.raster_vertical_scale,
            self.font_with_size(style.size),
        );
    }

    pub(crate) fn draw_rich_line_signed_in_line_box(
        &self,
        canvas: &mut Canvas,
        spans: &[RichTextLineSpan],
        x: isize,
        line_box_top: f32,
        line_box_height: f32,
        baseline_from_line_box_top: f32,
    ) -> f32 {
        let raster_vertical_scale = spans
            .iter()
            .map(|span| span.style.raster_vertical_scale)
            .fold(1.0_f32, f32::max);
        let font = self.font_with_size(
            spans
                .first()
                .map(|span| span.style.size)
                .unwrap_or(self.font.size),
        );
        let spans = spans
            .iter()
            .map(|span| ui_span(&span.text, span.style))
            .collect();
        self.draw_layout_in_line_box(
            canvas,
            spans,
            x,
            line_box_top,
            line_box_height,
            baseline_from_line_box_top,
            raster_vertical_scale,
            font,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_layout_in_line_box(
        &self,
        canvas: &mut Canvas,
        spans: Vec<UiTextSpan>,
        x: isize,
        line_box_top: f32,
        line_box_height: f32,
        baseline_from_line_box_top: f32,
        raster_vertical_scale: f32,
        font: FontToken,
    ) -> f32 {
        let scale = canvas.scale_factor();
        let raster_baseline = self.raster_baseline(&spans, font.clone(), line_box_height, scale);
        let origin_y = draw_origin_for_target_baseline(
            line_box_top,
            baseline_from_line_box_top,
            raster_baseline,
        );
        self.draw_request(
            canvas,
            spans,
            x,
            origin_y,
            line_box_top,
            scale,
            raster_vertical_scale,
            font,
            line_box_height,
        );
        raster_baseline
    }
}

pub(super) fn draw_origin_for_target_baseline(
    line_box_top: f32,
    baseline_from_line_box_top: f32,
    baseline_from_raster_origin: f32,
) -> f32 {
    line_box_top + baseline_from_line_box_top - baseline_from_raster_origin
}
