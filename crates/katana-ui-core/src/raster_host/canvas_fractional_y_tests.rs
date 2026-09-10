use super::Canvas;

const BACKGROUND: u32 = 0x000000;
const FILL: u32 = 0xffffff;
const BLEND: u32 = 0xff0000;

#[test]
fn fractional_y_fill_preserves_physical_boundaries_and_canvas_clipping() {
    let mut canvas = Canvas::new_scaled(4, 4, 2.0, BACKGROUND);

    canvas.fill_rect_at_logical_y(0, 1.5, 4, 1.0, FILL);
    canvas.fill_rect_at_logical_y(0, f32::NAN, 4, 1.0, BLEND);
    canvas.fill_rect_at_logical_y(0, 0.0, 4, 0.0, BLEND);
    canvas.fill_rect_at_logical_y(0, 0.0, 0, 1.0, BLEND);
    canvas.fill_rect_at_logical_y(4, 0.0, 1, 1.0, BLEND);
    canvas.fill_rect_at_logical_y(0, 4.0, 1, 1.0, BLEND);
    canvas.fill_rect_at_logical_y(0, 3.5, 1, 1.0, BLEND);
    canvas.fill_rect_at_logical_y(0, -1.0, 1, 0.5, BLEND);
    canvas.with_clip(1, 2, 2, 1, &mut |canvas| {
        canvas.fill_rect_at_logical_y(0, 2.0, 4, 1.0, BLEND);
    });

    assert_eq!(Some(BACKGROUND), pixel_at(&canvas, 0, 2));
    assert_eq!(Some(FILL), pixel_at(&canvas, 0, 3));
    assert_eq!(Some(FILL), pixel_at(&canvas, 0, 4));
    assert_eq!(Some(BACKGROUND), pixel_at(&canvas, 0, 5));
    assert_eq!(Some(BLEND), pixel_at(&canvas, 2, 4));
    assert_eq!(Some(BLEND), pixel_at(&canvas, 5, 5));
    assert_eq!(Some(BACKGROUND), pixel_at(&canvas, 0, 6));
}

#[test]
fn fill_rect_at_logical_y_skips_fully_clipped_negative_rectangles() {
    let mut canvas = Canvas::new_scaled(4, 4, 2.0, BACKGROUND);

    canvas.fill_rect_at_logical_y(0, -1.0, 4, 1.0, FILL);

    assert!(canvas.pixels().iter().all(|pixel| *pixel == BACKGROUND));

    canvas.fill_rect_at_logical_y(0, 0.0, 4, 1.0, FILL);
    assert_eq!(Some(FILL), pixel_at(&canvas, 0, 0));
}

fn pixel_at(canvas: &Canvas, x: usize, y: usize) -> Option<u32> {
    canvas
        .pixels()
        .get(y.checked_mul(canvas.width())?.checked_add(x)?)
        .copied()
}
