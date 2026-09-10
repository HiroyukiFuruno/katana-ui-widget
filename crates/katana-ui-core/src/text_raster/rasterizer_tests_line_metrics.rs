use super::*;

#[test]
fn line_metrics_preserve_the_requested_fractional_line_box() {
    let mut request =
        PlatformTextRasterRequest::from_text("Hg", font(FontFamily::Proportional), TEXT_COLOR);
    request.line_height_px = 31.5;
    let mut rasterizer = PlatformTextRasterizer::new(PlatformTextRasterConfig::default());

    let metrics = rasterizer
        .measure_line_metrics(&request)
        .expect("installed font catalog must measure a line box");

    assert_eq!(31.5, metrics.line_box_height_px);
    assert!(metrics.baseline_from_raster_origin_px.is_finite());
    assert!(metrics.baseline_from_raster_origin_px > 0.0);
}
