//! Public document-role typography contracts for raster hosts.

/// Typography values for one document text role.
///
/// `baseline_offset` is the vertical offset from the role line box origin to
/// the raster draw origin. This field and the constructor signature are kept
/// stable for consumers of the 0.3.x API.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiTreeTextRoleTypography {
    /// Raster font size in logical pixels.
    pub font_size: f32,
    /// Total logical line-box height in pixels.
    pub line_height: usize,
    /// Vertical offset from the line-box origin before raster drawing.
    pub baseline_offset: usize,
}

impl UiTreeTextRoleTypography {
    /// Creates one role's independent raster typography values.
    #[must_use]
    pub const fn new(font_size: f32, line_height: usize, baseline_offset: usize) -> Self {
        Self {
            font_size,
            line_height,
            baseline_offset,
        }
    }

    /// Creates an additive fractional baseline contract while preserving the
    /// legacy [`Self::new`] fields and meaning.
    #[must_use]
    pub const fn with_baseline_from_line_box_top(
        self,
        line_box_height: f32,
        baseline_from_line_box_top: f32,
    ) -> UiTreeTextRoleBaselineTypography {
        UiTreeTextRoleBaselineTypography::new(
            self.font_size,
            line_box_height,
            baseline_from_line_box_top,
        )
    }

    pub(in crate::raster_host) fn is_valid(self) -> bool {
        self.font_size.is_finite()
            && self.font_size > 0.0
            && self.line_height > 0
            && self.baseline_offset < self.line_height
    }
}

/// Additive fractional baseline contract for a document text role.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiTreeTextRoleBaselineTypography {
    pub font_size: f32,
    pub line_box_height: f32,
    pub baseline_from_line_box_top: f32,
}

impl UiTreeTextRoleBaselineTypography {
    #[must_use]
    pub const fn new(
        font_size: f32,
        line_box_height: f32,
        baseline_from_line_box_top: f32,
    ) -> Self {
        Self {
            font_size,
            line_box_height,
            baseline_from_line_box_top,
        }
    }

    pub(in crate::raster_host) fn is_valid(self) -> bool {
        self.font_size.is_finite()
            && self.font_size > 0.0
            && self.line_box_height.is_finite()
            && self.line_box_height > 0.0
            && self.baseline_from_line_box_top.is_finite()
            && self.baseline_from_line_box_top >= 0.0
            && self.baseline_from_line_box_top < self.line_box_height
    }
}

/// Optional document-role typography overrides for a raster host.
///
/// Roles that are not configured retain the metrics derived from the supplied
/// [`ThemeSnapshot`](crate::theme::ThemeSnapshot). Invalid role values are
/// ignored so the existing theme-derived metrics remain active.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct UiTreeDocumentTypography {
    body: Option<UiTreeTextRoleTypography>,
    body_baseline: Option<UiTreeTextRoleBaselineTypography>,
    heading_1: Option<UiTreeTextRoleTypography>,
    heading_1_baseline: Option<UiTreeTextRoleBaselineTypography>,
    heading_2: Option<UiTreeTextRoleTypography>,
    heading_2_baseline: Option<UiTreeTextRoleBaselineTypography>,
    heading_3: Option<UiTreeTextRoleTypography>,
    heading_3_baseline: Option<UiTreeTextRoleBaselineTypography>,
}

impl UiTreeDocumentTypography {
    /// Creates an override set that preserves all theme-derived metrics.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            body: None,
            body_baseline: None,
            heading_1: None,
            heading_1_baseline: None,
            heading_2: None,
            heading_2_baseline: None,
            heading_3: None,
            heading_3_baseline: None,
        }
    }

    /// Overrides document body text metrics.
    #[must_use]
    pub const fn with_body(mut self, typography: UiTreeTextRoleTypography) -> Self {
        self.body = Some(typography);
        self.body_baseline = None;
        self
    }

    #[must_use]
    pub const fn with_body_baseline(
        mut self,
        typography: UiTreeTextRoleBaselineTypography,
    ) -> Self {
        self.body_baseline = Some(typography);
        self.body = None;
        self
    }

    /// Overrides first-level document heading metrics.
    #[must_use]
    pub const fn with_heading_1(mut self, typography: UiTreeTextRoleTypography) -> Self {
        self.heading_1 = Some(typography);
        self.heading_1_baseline = None;
        self
    }

    #[must_use]
    pub const fn with_heading_1_baseline(
        mut self,
        typography: UiTreeTextRoleBaselineTypography,
    ) -> Self {
        self.heading_1_baseline = Some(typography);
        self.heading_1 = None;
        self
    }

    /// Overrides second-level document heading metrics.
    #[must_use]
    pub const fn with_heading_2(mut self, typography: UiTreeTextRoleTypography) -> Self {
        self.heading_2 = Some(typography);
        self.heading_2_baseline = None;
        self
    }

    #[must_use]
    pub const fn with_heading_2_baseline(
        mut self,
        typography: UiTreeTextRoleBaselineTypography,
    ) -> Self {
        self.heading_2_baseline = Some(typography);
        self.heading_2 = None;
        self
    }

    /// Overrides third-level document heading metrics.
    #[must_use]
    pub const fn with_heading_3(mut self, typography: UiTreeTextRoleTypography) -> Self {
        self.heading_3 = Some(typography);
        self.heading_3_baseline = None;
        self
    }

    #[must_use]
    pub const fn with_heading_3_baseline(
        mut self,
        typography: UiTreeTextRoleBaselineTypography,
    ) -> Self {
        self.heading_3_baseline = Some(typography);
        self.heading_3 = None;
        self
    }

    pub(in crate::raster_host) const fn body(self) -> Option<UiTreeTextRoleTypography> {
        self.body
    }

    pub(in crate::raster_host) const fn body_baseline(
        self,
    ) -> Option<UiTreeTextRoleBaselineTypography> {
        self.body_baseline
    }

    pub(in crate::raster_host) const fn heading_1(self) -> Option<UiTreeTextRoleTypography> {
        self.heading_1
    }

    pub(in crate::raster_host) const fn heading_1_baseline(
        self,
    ) -> Option<UiTreeTextRoleBaselineTypography> {
        self.heading_1_baseline
    }

    pub(in crate::raster_host) const fn heading_2(self) -> Option<UiTreeTextRoleTypography> {
        self.heading_2
    }

    pub(in crate::raster_host) const fn heading_2_baseline(
        self,
    ) -> Option<UiTreeTextRoleBaselineTypography> {
        self.heading_2_baseline
    }

    pub(in crate::raster_host) const fn heading_3(self) -> Option<UiTreeTextRoleTypography> {
        self.heading_3
    }

    pub(in crate::raster_host) const fn heading_3_baseline(
        self,
    ) -> Option<UiTreeTextRoleBaselineTypography> {
        self.heading_3_baseline
    }
}

#[cfg(test)]
mod tests {
    use super::{
        UiTreeDocumentTypography, UiTreeTextRoleBaselineTypography, UiTreeTextRoleTypography,
    };

    #[test]
    fn role_overrides_are_optional_and_keep_independent_metrics() {
        let body = UiTreeTextRoleTypography::new(16.5, 23, 0);
        let heading = UiTreeTextRoleTypography::new(24.75, 40, 9);
        let typography = UiTreeDocumentTypography::new()
            .with_body(body)
            .with_heading_1(heading);

        assert_eq!(Some(body), typography.body());
        assert_eq!(Some(heading), typography.heading_1());
        assert_eq!(None, typography.heading_2());
        assert_eq!(None, typography.heading_3());
    }

    #[test]
    fn additive_baseline_api_preserves_role_selection_and_legacy_conversion() {
        let legacy = UiTreeTextRoleTypography::new(16.5, 23, 4);
        let baseline = legacy.with_baseline_from_line_box_top(23.5, 18.5);
        assert_eq!(
            baseline,
            UiTreeTextRoleBaselineTypography::new(16.5, 23.5, 18.5)
        );

        let body = UiTreeTextRoleBaselineTypography::new(16.5, 23.5, 18.5);
        let heading_1 = UiTreeTextRoleBaselineTypography::new(24.75, 40.0, 30.0);
        let heading_2 = UiTreeTextRoleBaselineTypography::new(22.0, 34.0, 25.0);
        let heading_3 = UiTreeTextRoleBaselineTypography::new(20.0, 30.0, 22.0);
        let typography = UiTreeDocumentTypography::new()
            .with_body_baseline(body)
            .with_heading_1_baseline(heading_1)
            .with_heading_2_baseline(heading_2)
            .with_heading_3_baseline(heading_3);

        assert_eq!(Some(body), typography.body_baseline());
        assert_eq!(Some(heading_1), typography.heading_1_baseline());
        assert_eq!(Some(heading_2), typography.heading_2_baseline());
        assert_eq!(Some(heading_3), typography.heading_3_baseline());
        assert_eq!(None, typography.body());
        assert_eq!(None, typography.heading_1());
        assert_eq!(None, typography.heading_2());
        assert_eq!(None, typography.heading_3());

        let legacy_heading_2 = UiTreeTextRoleTypography::new(22.0, 34, 5);
        let legacy_heading_3 = UiTreeTextRoleTypography::new(20.0, 30, 4);
        let legacy_typography = typography
            .with_heading_2(legacy_heading_2)
            .with_heading_3(legacy_heading_3);

        assert_eq!(Some(legacy_heading_2), legacy_typography.heading_2());
        assert_eq!(Some(legacy_heading_3), legacy_typography.heading_3());
        assert_eq!(None, legacy_typography.heading_2_baseline());
        assert_eq!(None, legacy_typography.heading_3_baseline());
    }

    #[test]
    fn invalid_role_values_are_rejected_by_the_raster_host_boundary() {
        assert!(!UiTreeTextRoleTypography::new(0.0, 23, 0).is_valid());
        assert!(!UiTreeTextRoleTypography::new(f32::NAN, 23, 0).is_valid());
        assert!(!UiTreeTextRoleTypography::new(16.5, 0, 0).is_valid());
        assert!(!UiTreeTextRoleTypography::new(16.5, 23, 23).is_valid());
        assert!(UiTreeTextRoleBaselineTypography::new(16.5, 23.5, 0.5).is_valid());
    }
}
