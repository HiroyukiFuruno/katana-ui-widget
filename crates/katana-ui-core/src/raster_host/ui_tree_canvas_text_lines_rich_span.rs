use super::super::super::text::RichTextLineSpan;
use super::UiTreeTextLineContext;
use crate::raster_host::ui_tree_canvas_text_line_width::{SpanTextRenderers, span_rich_text_style};
use katana_ui_core::render_model::UiTextSpan;

pub(super) fn rich_line_span(
    context: UiTreeTextLineContext<'_>,
    renderers: SpanTextRenderers<'_>,
    span: &UiTextSpan,
    color: u32,
) -> RichTextLineSpan {
    renderers.for_span(span).rich_line_span(
        span.text.clone(),
        span_rich_text_style(span, context.metrics, color),
    )
}
