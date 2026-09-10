use super::canvas_clip::CanvasClip;
use super::canvas_color::blend_color;
pub use super::canvas_model::Canvas;
use super::canvas_model::CanvasImageSurfaceExtentMode;
use super::canvas_scale::{normalized_scale, physical_size};
const RECT_BORDER_WIDTH: usize = 1;

fn physical_fractional_position(logical: f32, scale_factor: f32) -> usize {
    (f64::from(logical.max(0.0)) * f64::from(scale_factor)).round() as usize
}

impl Canvas {
    #[must_use]
    pub fn new(width: usize, height: usize, color: u32) -> Self {
        Self::new_scaled(width, height, 1.0, color)
    }

    #[must_use]
    pub fn new_scaled(width: usize, height: usize, scale: f32, color: u32) -> Self {
        Self::new_scaled_with_raster_scale(width, height, scale, scale, color)
    }

    #[must_use]
    pub fn new_scaled_with_raster_scale(
        width: usize,
        height: usize,
        scale: f32,
        raster_scale: f32,
        color: u32,
    ) -> Self {
        let scale = normalized_scale(scale);
        let raster_scale = normalized_scale(raster_scale);
        Self {
            width: physical_size(width, scale),
            height: physical_size(height, scale),
            logical_width: width,
            logical_height: height,
            scale_factor: scale,
            raster_scale_factor: raster_scale,
            image_surface_extent_mode: CanvasImageSurfaceExtentMode::LogicalDisplay,
            pixels: vec![color; physical_size(width, scale) * physical_size(height, scale)],
            clip: None,
            text_runs: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_reference_capture_image_surface_extents(mut self) -> Self {
        self.image_surface_extent_mode = CanvasImageSurfaceExtentMode::RasterPresentation;
        self
    }

    #[must_use]
    pub fn width(&self) -> usize {
        self.width
    }

    #[must_use]
    pub fn height(&self) -> usize {
        self.height
    }

    #[must_use]
    pub fn logical_width(&self) -> usize {
        self.logical_width
    }

    #[must_use]
    pub fn logical_height(&self) -> usize {
        self.logical_height
    }

    #[must_use]
    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    #[must_use]
    pub fn raster_scale_factor(&self) -> f32 {
        self.raster_scale_factor
    }

    pub(super) fn uses_reference_capture_image_surface_extents(&self) -> bool {
        self.image_surface_extent_mode == CanvasImageSurfaceExtentMode::RasterPresentation
    }

    #[must_use]
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    #[must_use]
    pub fn non_background_pixels(&self, background: u32) -> usize {
        self.pixels.iter().filter(|it| **it != background).count()
    }

    pub fn fill_rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: u32) {
        let Some(rect) = self.visible_rect(x, y, width, height) else {
            return;
        };
        for current_y in rect.y..rect.bottom() {
            let start = current_y * self.width + rect.x;
            let end = current_y * self.width + rect.right();
            self.pixels[start..end].fill(color);
        }
    }

    pub(crate) fn fill_rect_at_logical_y(
        &mut self,
        x: usize,
        y: f32,
        width: usize,
        height: f32,
        color: u32,
    ) {
        let Some(rect) = self.visible_rect_at_logical_y(x, y, width, height) else {
            return;
        };
        for current_y in rect.y..rect.bottom() {
            let start = current_y * self.width + rect.x;
            let end = current_y * self.width + rect.right();
            self.pixels[start..end].fill(color);
        }
    }

    pub(crate) fn with_clip(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        draw: &mut dyn FnMut(&mut Self),
    ) {
        let Some(next) = self.to_physical_clip(x, y, width, height) else {
            return;
        };
        let previous = self.clip;
        self.clip = match previous {
            Some(current) => current.intersect(next),
            None => Some(next),
        };
        if self.clip.is_some() {
            draw(self);
        }
        self.clip = previous;
    }

    #[must_use]
    pub(super) fn visible_rect(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Option<CanvasClip> {
        let rect = self.to_physical_clip(x, y, width, height)?;
        match self.clip {
            Some(clip) => rect.intersect(clip),
            None => Some(rect),
        }
    }

    fn visible_rect_at_logical_y(
        &self,
        x: usize,
        y: f32,
        width: usize,
        height: f32,
    ) -> Option<CanvasClip> {
        if !y.is_finite() || !height.is_finite() || height <= 0.0 || width == 0 {
            return None;
        }
        let left = self.to_physical_x(x);
        let top = physical_fractional_position(y, self.scale_factor()).min(self.height());
        let bottom =
            physical_fractional_position(y + height, self.scale_factor()).min(self.height());
        if left >= self.width() || top >= self.height() {
            return None;
        }
        let right = self.to_physical_x(x.saturating_add(width)).max(left + 1);
        if bottom <= top {
            return None;
        }
        let rect = CanvasClip {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        };
        match self.clip {
            Some(clip) => rect.intersect(clip),
            None => Some(rect),
        }
    }

    pub fn stroke_rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.fill_rect(x, y, width, RECT_BORDER_WIDTH, color);
        self.fill_rect(
            x,
            y + height - RECT_BORDER_WIDTH,
            width,
            RECT_BORDER_WIDTH,
            color,
        );
        self.fill_rect(x, y, RECT_BORDER_WIDTH, height, color);
        self.fill_rect(
            x + width - RECT_BORDER_WIDTH,
            y,
            RECT_BORDER_WIDTH,
            height,
            color,
        );
    }

    pub fn set(&mut self, x: usize, y: usize, color: u32) {
        let Some((left, right)) = self.physical_span_x(x) else {
            return;
        };
        let Some((top, bottom)) = self.physical_span_y(y) else {
            return;
        };
        for current_y in top..bottom {
            for current_x in left..right {
                self.set_physical(current_x, current_y, color);
            }
        }
    }

    pub fn blend(&mut self, x: usize, y: usize, color: u32, alpha: u8) {
        let Some((left, right)) = self.physical_span_x(x) else {
            return;
        };
        let Some((top, bottom)) = self.physical_span_y(y) else {
            return;
        };
        for current_y in top..bottom {
            for current_x in left..right {
                self.blend_physical(current_x, current_y, color, alpha)
            }
        }
    }

    pub(crate) fn blend_physical(&mut self, x: usize, y: usize, color: u32, alpha: u8) {
        if x >= self.width || y >= self.height || !self.clip.is_none_or(|clip| clip.contains(x, y))
        {
            return;
        }
        let index = y * self.width + x;
        self.pixels[index] = blend_color(self.pixels[index], color, alpha);
    }

    pub(crate) fn set_physical(&mut self, x: usize, y: usize, color: u32) {
        if x >= self.width || y >= self.height || !self.clip.is_none_or(|clip| clip.contains(x, y))
        {
            return;
        }
        self.pixels[y * self.width + x] = color;
    }

    pub fn blend_rect(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        color: u32,
        alpha: u8,
    ) {
        let Some(rect) = self.visible_rect(x, y, width, height) else {
            return;
        };
        for current_y in rect.y..rect.bottom() {
            let left = current_y * self.width + rect.x;
            let right = current_y * self.width + rect.right();
            for index in left..right {
                let destination = self.pixels[index];
                self.pixels[index] = blend_color(destination, color, alpha);
            }
        }
    }
}
