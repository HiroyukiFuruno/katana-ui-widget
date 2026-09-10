use super::ui_tree_canvas_text_table_types::TableLayoutRow;
use crate::raster_host::ui_tree_canvas_text_metrics::UiTreeTextMetrics;
use crate::raster_host::ui_tree_canvas_types::UiTreeRenderArea;

const TABLE_CELL_PADDING: usize = 16;
const TABLE_ROW_MIN_HEIGHT: usize = 52;
const TABLE_ROW_VERTICAL_PADDING: usize = 16;
const ASCII_CELL_CHAR_WIDTH: usize = 12;
const WIDE_CELL_CHAR_WIDTH: usize = 22;

pub(super) fn build_row_layout(
    row: &[String],
    column_widths: &[usize],
    metrics: UiTreeTextMetrics,
) -> TableLayoutRow {
    let lines = row
        .iter()
        .enumerate()
        .map(|(index, cell)| wrap_cell(cell, cell_content_width(column_widths, index)))
        .collect::<Vec<_>>();
    let height = lines
        .iter()
        .map(|value| value.len())
        .max()
        .unwrap_or(1)
        .max(1)
        .saturating_mul(metrics.line_height)
        .saturating_add(TABLE_ROW_VERTICAL_PADDING.saturating_mul(2))
        .max(TABLE_ROW_MIN_HEIGHT);
    TableLayoutRow { lines, height }
}

fn wrap_cell(cell: &str, max_width: usize) -> Vec<String> {
    let max_width = max_width.max(ASCII_CELL_CHAR_WIDTH);
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_width = 0usize;
    for character in cell.chars() {
        let character_width = estimated_cell_char_width(character);
        if !current.is_empty() && current_width.saturating_add(character_width) > max_width {
            lines.push(std::mem::take(&mut current));
            current_width = 0;
        }
        current.push(character);
        current_width = current_width.saturating_add(character_width);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn cell_content_width(widths: &[usize], index: usize) -> usize {
    widths
        .get(index)
        .copied()
        .unwrap_or(0)
        .saturating_sub(TABLE_CELL_PADDING.saturating_mul(2))
}

fn estimated_cell_char_width(character: char) -> usize {
    if character.is_ascii() {
        ASCII_CELL_CHAR_WIDTH
    } else {
        WIDE_CELL_CHAR_WIDTH
    }
}

pub(super) fn remaining_width(area: UiTreeRenderArea, x: usize) -> usize {
    area.width.saturating_sub(x.saturating_sub(area.x)).max(1)
}

#[cfg(test)]
mod tests {
    use super::{
        ASCII_CELL_CHAR_WIDTH, TABLE_ROW_MIN_HEIGHT, WIDE_CELL_CHAR_WIDTH, build_row_layout,
        estimated_cell_char_width, wrap_cell,
    };
    use crate::raster_host::ui_tree_canvas_text_metrics::UiTreeTextMetrics;

    #[test]
    fn table_row_layout_keeps_kdv_export_surface_min_height() {
        let metrics = UiTreeTextMetrics {
            font_size: 12.0,
            line_height: 19,
            line_box_height: 19.0,
            top_margin: 0,
            baseline_from_line_box_top: None,
            background_height: 19,
            highlight_height: 19,
            underline_offset: 14,
            strikethrough_offset: 8,
            raster_vertical_scale: 1.0,
        };

        let row = build_row_layout(&["Header".to_string()], &[1200], metrics);

        assert_eq!(TABLE_ROW_MIN_HEIGHT, row.height);
    }

    #[test]
    fn table_row_layout_keeps_fitting_ascii_cell_on_one_line() {
        let metrics = UiTreeTextMetrics {
            font_size: 12.0,
            line_height: 19,
            line_box_height: 19.0,
            top_margin: 0,
            baseline_from_line_box_top: None,
            background_height: 19,
            highlight_height: 19,
            underline_offset: 14,
            strikethrough_offset: 8,
            raster_vertical_scale: 1.0,
        };

        let row = build_row_layout(&["Full support".to_string()], &[241], metrics);

        assert_eq!(vec!["Full support".to_string()], row.lines[0]);
    }

    #[test]
    fn table_cell_wrap_handles_empty_and_wide_text() {
        assert_eq!(vec![String::new()], wrap_cell("", 0));
        assert_eq!(WIDE_CELL_CHAR_WIDTH, estimated_cell_char_width('界'));
        assert_eq!(ASCII_CELL_CHAR_WIDTH, estimated_cell_char_width('a'));
    }
}
