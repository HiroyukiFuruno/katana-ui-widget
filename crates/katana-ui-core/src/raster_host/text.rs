use crate::raster_host::canvas::Canvas;
use katana_ui_core::facade::UiCoreFacade;
use katana_ui_core::text_raster::{
    PlatformTextFaceSelection, PlatformTextRasterConfig, PlatformTextRasterizer,
};

#[path = "text_types.rs"]
mod text_types;
pub use text_types::TextRenderer;
pub(crate) use text_types::{RichTextLineSpan, RichTextStyle};
#[path = "text_line_box.rs"]
mod text_line_box;
#[path = "text_runtime.rs"]
mod text_runtime;
#[path = "text_style.rs"]
mod text_style;
use text_runtime::{default_line_height, resolve_font, ui_span};

impl TextRenderer {
    pub fn load(facade: &UiCoreFacade, role: &str) -> Self {
        Self::load_with_text_raster_config(
            facade,
            role,
            PlatformTextRasterConfig::default(),
            PlatformTextFaceSelection::System,
        )
    }

    pub fn load_with_text_raster_config(
        facade: &UiCoreFacade,
        role: &str,
        text_raster_config: PlatformTextRasterConfig,
        face_selection: PlatformTextFaceSelection,
    ) -> Self {
        let font = resolve_font(facade, role);
        Self {
            rasterizer: std::cell::RefCell::new(PlatformTextRasterizer::new_with_face_selection(
                text_raster_config,
                face_selection,
            )),
            font,
        }
    }

    pub fn draw(&self, canvas: &mut Canvas, text: &str, x: usize, y: usize, size: f32, color: u32) {
        self.draw_signed(canvas, text, x as isize, y, size, color);
    }

    pub(crate) fn draw_signed(
        &self,
        canvas: &mut Canvas,
        text: &str,
        x: isize,
        y: usize,
        size: f32,
        color: u32,
    ) {
        self.draw_layout(
            canvas,
            text,
            x,
            y,
            RichTextStyle::new(size, color),
            false,
            canvas.scale_factor(),
        );
    }

    pub(crate) fn draw_signed_styled(
        &self,
        canvas: &mut Canvas,
        text: &str,
        x: isize,
        y: usize,
        style: RichTextStyle,
    ) {
        self.draw_layout_with_line_height(
            canvas,
            text,
            x,
            y,
            style,
            canvas.scale_factor(),
            default_line_height(style.size),
        );
    }

    pub fn draw_emoji(
        &self,
        canvas: &mut Canvas,
        text: &str,
        x: usize,
        y: usize,
        size: f32,
        color: u32,
    ) {
        self.draw_layout(
            canvas,
            text,
            x as isize,
            y,
            RichTextStyle::new(size, color),
            true,
            canvas.scale_factor(),
        );
    }

    pub(crate) fn draw_rich_line_signed(
        &self,
        canvas: &mut Canvas,
        spans: &[RichTextLineSpan],
        x: isize,
        y: usize,
    ) {
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
        let line_height_px = spans
            .iter()
            .map(|span| default_line_height(span.style.size))
            .fold(default_line_height(font.size), f32::max);
        let spans = spans
            .iter()
            .map(|span| ui_span(&span.text, span.style))
            .collect();
        self.draw_request(
            canvas,
            spans,
            x,
            y as f32,
            y as f32,
            canvas.scale_factor(),
            raster_vertical_scale,
            font,
            line_height_px,
        );
    }

    pub fn measure_width(&self, text: &str, size: f32) -> usize {
        self.measure_width_with_emoji(text, size, false)
    }

    pub fn measure_emoji_width(&self, text: &str, size: f32) -> usize {
        self.measure_width_with_emoji(text, size, true)
    }

    pub(crate) fn measure_width_rich(&self, text: &str, style: RichTextStyle) -> usize {
        self.measure_request(vec![ui_span(text, style)], style.size, 1.0)
    }

    fn measure_width_with_emoji(&self, text: &str, size: f32, emoji: bool) -> usize {
        self.measure_request(
            vec![ui_span(text, RichTextStyle::new(size, 0).emoji(emoji))],
            size,
            1.0,
        )
    }

    fn draw_layout(
        &self,
        canvas: &mut Canvas,
        text: &str,
        x: isize,
        y: usize,
        style: RichTextStyle,
        emoji: bool,
        scale_factor: f32,
    ) {
        self.draw_layout_with_line_height(
            canvas,
            text,
            x,
            y,
            style.emoji(emoji),
            scale_factor,
            default_line_height(style.size),
        );
    }

    fn draw_layout_with_line_height(
        &self,
        canvas: &mut Canvas,
        text: &str,
        x: isize,
        y: usize,
        style: RichTextStyle,
        scale_factor: f32,
        line_height_px: f32,
    ) {
        self.draw_request(
            canvas,
            vec![ui_span(text, style)],
            x,
            y as f32,
            y as f32,
            scale_factor,
            style.raster_vertical_scale,
            self.font_with_size(style.size),
            line_height_px,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{Canvas, RichTextLineSpan, RichTextStyle, TextRenderer, ui_span};
    use katana_ui_core::facade::UiCoreFacade;

    #[test]
    fn target_baseline_maps_to_raster_origin_without_consumer_specific_offsets() {
        let origin = super::text_line_box::draw_origin_for_target_baseline(8.5, 12.5, 9.0);

        assert_eq!(12.0, origin);
        assert_eq!(21.0, origin + 9.0);
    }

    #[test]
    fn selection_runs_track_logical_line_box_top_when_paint_origin_is_offset() {
        let renderer = TextRenderer::load(&UiCoreFacade::new(ThemeSnapshot::light()), "body");
        let mut canvas = Canvas::new(180, 80, 0x101010);
        let style = RichTextStyle::new(14.0, 0xeeeeee);
        let line_box_top = 10.5;
        let line_box_height = 20.5;
        let baseline_from_line_box_top = {
            let font = renderer.font_with_size(style.size);
            let raster_baseline = renderer.raster_baseline(
                &[ui_span("selection", style)],
                font,
                line_box_height,
                canvas.scale_factor(),
            );
            raster_baseline + 3.5
        };
        let paint_origin = super::text_line_box::draw_origin_for_target_baseline(
            line_box_top,
            baseline_from_line_box_top,
            baseline_from_line_box_top - 3.5,
        );

        renderer.draw_signed_styled_in_line_box(
            &mut canvas,
            "selection",
            4,
            line_box_top,
            line_box_height,
            baseline_from_line_box_top,
            style,
        );
        let run = canvas
            .text_runs()
            .first()
            .expect("selectable text run should be recorded");

        assert_eq!(line_box_top.round() as usize, run.y());
        assert_ne!(paint_origin.round() as usize, run.y());
    }
    use katana_ui_core::theme::ThemeSnapshot;

    #[test]
    fn raster_host_text_renderer_draws_plain_emoji_signed_and_rich_text() {
        let renderer = TextRenderer::load(&UiCoreFacade::new(ThemeSnapshot::light()), "body");
        let mut canvas = Canvas::new(160, 80, 0x101010);
        let style = RichTextStyle::new(14.0, 0xeeeeee)
            .bold(true)
            .italic(true)
            .raster_vertical_scale(1.25);

        renderer.draw(&mut canvas, "plain", 4, 4, 14.0, 0xffffff);
        renderer.draw_emoji(&mut canvas, "😀", 4, 24, 14.0, 0xffffff);
        renderer.draw_signed(&mut canvas, "signed", -2, 40, 14.0, 0xffffff);
        renderer.draw_signed_styled(&mut canvas, "styled", 4, 52, style);
        renderer.draw_signed_styled_in_line_box(
            &mut canvas,
            "line box",
            4,
            56.5,
            20.5,
            14.0,
            style,
        );
        renderer.draw_rich_line_signed(
            &mut canvas,
            &[RichTextLineSpan {
                text: "rich".to_owned(),
                style,
            }],
            4,
            64,
        );
        renderer.draw_rich_line_signed_in_line_box(
            &mut canvas,
            &[RichTextLineSpan {
                text: "rich line box".to_owned(),
                style,
            }],
            4,
            68.5,
            20.5,
            14.0,
        );

        assert!(renderer.measure_width("plain", 14.0) > 0);
        assert!(renderer.measure_emoji_width("😀", 14.0) > 0);
        assert!(renderer.measure_width_rich("rich", style) > 0);
        assert!(canvas.non_background_pixels(0x101010) > 0);
        assert!(!canvas.text_runs().is_empty());
    }
}
