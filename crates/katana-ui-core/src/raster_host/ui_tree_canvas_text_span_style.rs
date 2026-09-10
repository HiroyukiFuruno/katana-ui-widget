use super::super::super::canvas::Canvas;
use super::super::super::ui_tree_canvas_palette::UiTreeCanvasPalette;
use super::super::super::ui_tree_canvas_text_metrics::UiTreeTextMetrics;
use katana_ui_core::render_model::{UiTextSpan, UiTextSpanStyle};

const HIGHLIGHT_BACKGROUND: u32 = 0x4a4620;
const CURRENT_HIGHLIGHT_BACKGROUND: u32 = 0x654100;
const INLINE_CODE_LEFT_PADDING: usize = 4;
const INLINE_CODE_EXTRA_WIDTH: usize = 8;
const INLINE_CODE_Y_OFFSET_SCALE: f32 = 0.08;
const INLINE_CODE_HEIGHT_SCALE: f32 = 1.24;
const ALPHA_CHANNEL_INDEX: usize = 3;
const RED_CHANNEL_INDEX: usize = 0;
const GREEN_CHANNEL_INDEX: usize = 1;
const BLUE_CHANNEL_INDEX: usize = 2;
const RGBA_CHANNEL_COUNT: usize = 4;
const TRANSPARENT_ALPHA: u8 = 0;
const RED_SHIFT: u32 = 16;
const GREEN_SHIFT: u32 = 8;

pub(super) fn draw_span_background(
    canvas: &mut Canvas,
    x: usize,
    y: f32,
    width: usize,
    style: UiTextSpanStyle,
    palette: UiTreeCanvasPalette,
    metrics: UiTreeTextMetrics,
) {
    if style.current_highlight {
        canvas.fill_rect_at_logical_y(
            x,
            y,
            width,
            metrics.highlight_box_height(),
            CURRENT_HIGHLIGHT_BACKGROUND,
        );
        return;
    }
    if style.highlight {
        canvas.fill_rect_at_logical_y(
            x,
            y,
            width,
            metrics.highlight_box_height(),
            HIGHLIGHT_BACKGROUND,
        );
        return;
    }
    if style.inline_code {
        canvas.fill_rect_at_logical_y(
            x.saturating_sub(INLINE_CODE_LEFT_PADDING),
            y + inline_code_y_offset(metrics) as f32,
            width.saturating_add(INLINE_CODE_EXTRA_WIDTH),
            inline_code_height(metrics) as f32,
            palette.inline_code_background,
        );
    }
}

fn inline_code_y_offset(metrics: UiTreeTextMetrics) -> usize {
    (metrics.font_size * INLINE_CODE_Y_OFFSET_SCALE)
        .floor()
        .max(0.0) as usize
}

fn inline_code_height(metrics: UiTreeTextMetrics) -> usize {
    (metrics.font_size * INLINE_CODE_HEIGHT_SCALE)
        .floor()
        .max(1.0) as usize
}

pub(super) fn span_color(span: &UiTextSpan, palette: UiTreeCanvasPalette) -> u32 {
    if span.style.color_rgba[ALPHA_CHANNEL_INDEX] > TRANSPARENT_ALPHA {
        return rgba_to_rgb(span.style.color_rgba);
    }
    if !span.link_target.is_empty() {
        return palette.link;
    }
    palette.text
}

pub(super) fn should_underline(span: &UiTextSpan) -> bool {
    !span.link_target.is_empty() || span.style.underline
}

pub(super) fn should_strikethrough(span: &UiTextSpan) -> bool {
    span.style.strikethrough
}

fn rgba_to_rgb(value: [u8; RGBA_CHANNEL_COUNT]) -> u32 {
    (u32::from(value[RED_CHANNEL_INDEX]) << RED_SHIFT)
        | (u32::from(value[GREEN_CHANNEL_INDEX]) << GREEN_SHIFT)
        | u32::from(value[BLUE_CHANNEL_INDEX])
}

#[cfg(test)]
mod tests {
    use super::{draw_span_background, inline_code_height, inline_code_y_offset};
    use crate::raster_host::canvas::Canvas;
    use crate::raster_host::ui_tree_canvas_palette::UiTreeCanvasPalette;
    use crate::raster_host::ui_tree_canvas_text_metrics::UiTreeTextMetrics;
    use katana_ui_core::render_model::UiTextSpanStyle;
    use katana_ui_core::theme::ThemeSnapshot;

    #[test]
    fn inline_code_background_matches_export_surface_padding_and_height() {
        let palette = UiTreeCanvasPalette::from_theme(&ThemeSnapshot::light());
        let metrics = metrics_for_test();
        let mut canvas = Canvas::new(80, 40, palette.background);

        draw_span_background(
            &mut canvas,
            20,
            10.0,
            24,
            UiTextSpanStyle {
                inline_code: true,
                ..UiTextSpanStyle::default()
            },
            palette,
            metrics,
        );

        assert_eq!(1, inline_code_y_offset(metrics));
        assert_eq!(17, inline_code_height(metrics));
        assert_eq!(
            Some(palette.inline_code_background),
            pixel_at(&canvas, 16, 11)
        );
        assert_eq!(
            Some(palette.inline_code_background),
            pixel_at(&canvas, 47, 27)
        );
        assert_eq!(Some(palette.background), pixel_at(&canvas, 15, 11));
        assert_eq!(Some(palette.background), pixel_at(&canvas, 16, 28));
    }

    #[test]
    fn highlight_backgrounds_take_priority_over_inline_code() {
        let palette = UiTreeCanvasPalette::from_theme(&ThemeSnapshot::dark());
        let metrics = metrics_for_test();
        let mut canvas = Canvas::new(80, 40, palette.background);

        draw_span_background(
            &mut canvas,
            10,
            8.0,
            20,
            UiTextSpanStyle {
                current_highlight: true,
                highlight: true,
                inline_code: true,
                ..UiTextSpanStyle::default()
            },
            palette,
            metrics,
        );
        assert_eq!(
            Some(super::CURRENT_HIGHLIGHT_BACKGROUND),
            pixel_at(&canvas, 10, 8)
        );

        draw_span_background(
            &mut canvas,
            40,
            8.0,
            20,
            UiTextSpanStyle {
                highlight: true,
                inline_code: true,
                ..UiTextSpanStyle::default()
            },
            palette,
            metrics,
        );
        assert_eq!(Some(super::HIGHLIGHT_BACKGROUND), pixel_at(&canvas, 40, 8));
    }

    #[test]
    fn span_background_keeps_fractional_line_tops_until_physical_rasterization() {
        let palette = UiTreeCanvasPalette::from_theme(&ThemeSnapshot::dark());
        let metrics = metrics_for_test();
        let mut canvas = Canvas::new_scaled(80, 80, 2.0, palette.background);

        draw_span_background(
            &mut canvas,
            10,
            31.5,
            20,
            UiTextSpanStyle {
                highlight: true,
                ..UiTextSpanStyle::default()
            },
            palette,
            metrics,
        );

        assert_eq!(Some(palette.background), pixel_at(&canvas, 20, 62));
        assert_eq!(Some(super::HIGHLIGHT_BACKGROUND), pixel_at(&canvas, 20, 63));
    }

    #[test]
    fn document_highlight_backgrounds_preserve_fractional_height_between_lines() {
        let palette = UiTreeCanvasPalette::from_theme(&ThemeSnapshot::dark());
        let mut metrics = metrics_for_test();
        metrics.line_height = 32;
        metrics.line_box_height = 31.5;
        metrics.baseline_from_line_box_top = Some(12.0);
        metrics.highlight_height = 32;
        let mut canvas = Canvas::new_scaled(80, 64, 2.0, palette.background);

        draw_span_background(
            &mut canvas,
            10,
            0.0,
            20,
            UiTextSpanStyle {
                highlight: true,
                ..UiTextSpanStyle::default()
            },
            palette,
            metrics,
        );
        draw_span_background(
            &mut canvas,
            10,
            31.5,
            20,
            UiTextSpanStyle {
                current_highlight: true,
                ..UiTextSpanStyle::default()
            },
            palette,
            metrics,
        );

        assert_eq!(Some(super::HIGHLIGHT_BACKGROUND), pixel_at(&canvas, 20, 62));
        assert_eq!(
            Some(super::CURRENT_HIGHLIGHT_BACKGROUND),
            pixel_at(&canvas, 20, 63)
        );
    }

    fn metrics_for_test() -> UiTreeTextMetrics {
        UiTreeTextMetrics {
            font_size: 14.0,
            line_height: 23,
            line_box_height: 23.0,
            top_margin: 0,
            baseline_from_line_box_top: None,
            background_height: 23,
            highlight_height: 23,
            underline_offset: 17,
            strikethrough_offset: 10,
            raster_vertical_scale: 1.0,
        }
    }

    fn pixel_at(canvas: &Canvas, x: usize, y: usize) -> Option<u32> {
        canvas
            .pixels()
            .get(y.checked_mul(canvas.width())?.checked_add(x)?)
            .copied()
    }
}
