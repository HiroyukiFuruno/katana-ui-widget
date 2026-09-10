use katana_ui_core::render_model::UiNode;

const UI_FONT_SIZE: f32 = 14.0;
const UI_LINE_HEIGHT: usize = 20;
const UI_TOP_MARGIN: usize = 0;
const UI_HIGHLIGHT_HEIGHT: usize = 12;

const HEADING_FONT_SIZE: f32 = 36.0;
const HEADING_LINE_HEIGHT: usize = 68;
const HEADING_TOP_MARGIN: usize = 16;
const EXPORT_HEADING_FONT_SIZE: f32 = 40.0;
const EXPORT_HEADING_2_FONT_SIZE: f32 = 34.0;
const EXPORT_HEADING_3_FONT_SIZE: f32 = 28.0;
const EXPORT_HEADING_LINE_HEIGHT: usize = 92;
const EXPORT_HEADING_2_LINE_HEIGHT: usize = 78;
const EXPORT_HEADING_3_LINE_HEIGHT: usize = 66;
const EXPORT_HEADING_2_TOP_MARGIN: usize = 14;
const EXPORT_HEADING_3_TOP_MARGIN: usize = 12;
const HEADING_2_FONT_SIZE: f32 = 34.0;
const HEADING_2_LINE_HEIGHT: usize = 58;
const HEADING_2_TOP_MARGIN: usize = 14;
const HEADING_3_FONT_SIZE: f32 = 32.0;
const HEADING_3_LINE_HEIGHT: usize = 52;
const HEADING_3_TOP_MARGIN: usize = 12;

const BODY_FONT_SIZE: f32 = 24.0;
const BODY_LINE_HEIGHT: usize = 34;
const BODY_TOP_MARGIN: usize = 0;
const COMPACT_BODY_FONT_SIZE: f32 = 14.0;
const COMPACT_BODY_LINE_HEIGHT: usize = 23;
const COMPACT_HTML_BODY_LINE_HEIGHT: usize = 21;
const COMPACT_ALERT_LINE_HEIGHT: usize = 28;
const HTML_EXPORT_SURFACE_TOP_MARGIN: usize = 5;
const ALERT_TOP_MARGIN: usize = 16;

const CODE_FONT_SIZE: f32 = 22.0;
const CODE_LINE_HEIGHT: usize = 34;
const CODE_TOP_MARGIN: usize = 0;
const TABLE_FONT_SIZE: f32 = 22.0;
const TABLE_LINE_HEIGHT: usize = 34;

const DOCUMENT_BODY_FONT_ROLE: &str = "document-body";
const DOCUMENT_CODE_FONT_ROLE: &str = "document-code";
const MARKDOWN_HEADING_1_RASTER_VERTICAL_SCALE: f32 = 7.0 / 5.0;
const KATANA_LONG_HEADING_2_TOP_MARGIN_PX: usize = 15;

#[path = "ui_tree_canvas_text_metrics_document_overrides.rs"]
mod document_overrides;
#[path = "ui_tree_canvas_text_document_typography.rs"]
mod document_typography;
#[path = "ui_tree_canvas_text_metric_roles.rs"]
mod metric_roles;
#[path = "ui_tree_canvas_text_metric_scaling.rs"]
mod metric_scaling;

use document_overrides::{has_active_document_role_typography, with_document_typography};
pub(in crate::raster_host) use document_typography::UiTreeDocumentTypography;
use metric_roles::{
    compact_heading_font_size, compact_heading_line_height, html_body_font_size,
    is_document_body_role, is_export_heading_1_role, is_export_heading_2_role,
    is_export_heading_3_role, is_heading_1_role, is_heading_2_role, is_heading_3_role,
    is_heading_role, is_html_role, is_long_heading_2_role, is_preview_html_body_role,
};
use metric_scaling::{
    dimension_px, scale_usize, scaled_document_text_line_height, strikethrough_offset,
    underline_offset,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct UiTreeTextMetrics {
    pub(super) font_size: f32,
    pub(super) line_height: usize,
    pub(super) line_box_height: f32,
    pub(super) top_margin: usize,
    pub(super) baseline_from_line_box_top: Option<f32>,
    pub(super) background_height: usize,
    pub(super) highlight_height: usize,
    pub(super) underline_offset: usize,
    pub(super) strikethrough_offset: usize,
    pub(super) raster_vertical_scale: f32,
}

impl UiTreeTextMetrics {
    pub(super) fn has_active_document_role_typography(
        role: &str,
        typography: UiTreeDocumentTypography,
    ) -> bool {
        has_active_document_role_typography(role, typography.document_typography)
    }

    pub(super) fn highlight_box_height(&self) -> f32 {
        self.baseline_from_line_box_top
            .map_or(self.highlight_height as f32, |_| self.line_box_height)
    }

    #[cfg(test)]
    pub(super) fn for_node(node: &UiNode) -> Self {
        Self::for_node_with_typography(node, UiTreeDocumentTypography::default())
    }

    pub(super) fn for_node_with_typography(
        node: &UiNode,
        typography: UiTreeDocumentTypography,
    ) -> Self {
        let mut metrics = Self::for_role(&node.props().text.role);
        metrics = metrics.with_typography(&node.props().text.role, typography);
        metrics = with_document_typography(
            metrics,
            &node.props().text.role,
            typography.document_typography,
        );
        let top_padding = dimension_px(&node.props().common.padding.top);
        if node.props().text.role == "code" && top_padding > 0 {
            metrics.top_margin = top_padding;
        }
        let explicit_height = dimension_px(&node.props().common.height);
        if explicit_height > 0 {
            metrics.background_height = explicit_height;
        }
        metrics
    }

    fn for_role(role: &str) -> Self {
        if is_export_heading_1_role(role) {
            return Self::document(
                EXPORT_HEADING_FONT_SIZE,
                EXPORT_HEADING_LINE_HEIGHT,
                HEADING_TOP_MARGIN,
            );
        }
        if is_export_heading_2_role(role) {
            return Self::document(
                EXPORT_HEADING_2_FONT_SIZE,
                EXPORT_HEADING_2_LINE_HEIGHT,
                EXPORT_HEADING_2_TOP_MARGIN,
            );
        }
        if is_export_heading_3_role(role) {
            return Self::document(
                EXPORT_HEADING_3_FONT_SIZE,
                EXPORT_HEADING_3_LINE_HEIGHT,
                EXPORT_HEADING_3_TOP_MARGIN,
            );
        }
        if is_heading_2_role(role) {
            return Self::document(
                HEADING_2_FONT_SIZE,
                HEADING_2_LINE_HEIGHT,
                HEADING_2_TOP_MARGIN,
            );
        }
        if is_heading_3_role(role) {
            return Self::document(
                HEADING_3_FONT_SIZE,
                HEADING_3_LINE_HEIGHT,
                HEADING_3_TOP_MARGIN,
            );
        }
        if is_heading_1_role(role) {
            return Self::document(HEADING_FONT_SIZE, HEADING_LINE_HEIGHT, HEADING_TOP_MARGIN);
        }
        match role {
            "code" => Self::document(CODE_FONT_SIZE, CODE_LINE_HEIGHT, CODE_TOP_MARGIN),
            "table" => Self::document(TABLE_FONT_SIZE, TABLE_LINE_HEIGHT, CODE_TOP_MARGIN),
            "alert" => Self::document(BODY_FONT_SIZE, BODY_LINE_HEIGHT, ALERT_TOP_MARGIN),
            role if is_document_body_role(role) => {
                Self::document(BODY_FONT_SIZE, BODY_LINE_HEIGHT, BODY_TOP_MARGIN)
            }
            _ => Self::ui(),
        }
    }

    fn with_typography(mut self, role: &str, typography: UiTreeDocumentTypography) -> Self {
        if is_heading_role(role) {
            self.scale_document_text_font(
                compact_heading_font_size(role, typography.body_font_size),
                compact_heading_line_height(role),
            );
            if role == "heading" {
                self.raster_vertical_scale = MARKDOWN_HEADING_1_RASTER_VERTICAL_SCALE;
            }
            if is_long_heading_2_role(role) {
                self.add_scaled_long_heading_2_top_margin(typography.body_font_size);
            }
            if is_html_role(role) {
                self.add_scaled_top_margin(typography.body_font_size);
            }
            return self;
        }
        match role {
            "alert" => {
                self.scale_document_text_font(typography.body_font_size, COMPACT_ALERT_LINE_HEIGHT);
            }
            "code" | "table" => {
                self.scale_document_font(typography.code_font_size / CODE_FONT_SIZE);
            }
            role if is_document_body_role(role) => {
                if is_preview_html_body_role(role) {
                    self.scale_document_text_font(
                        html_body_font_size(typography.body_font_size),
                        COMPACT_HTML_BODY_LINE_HEIGHT,
                    );
                } else {
                    self.scale_document_text_font(
                        typography.body_font_size,
                        COMPACT_BODY_LINE_HEIGHT,
                    );
                }
            }
            _ => {}
        }
        self
    }

    fn add_scaled_top_margin(&mut self, font_size: f32) {
        if !font_size.is_finite() || font_size <= 0.0 {
            return;
        }
        self.top_margin = self.top_margin.saturating_add(scale_usize(
            HTML_EXPORT_SURFACE_TOP_MARGIN,
            font_size / BODY_FONT_SIZE,
        ));
    }

    fn add_scaled_long_heading_2_top_margin(&mut self, font_size: f32) {
        if !font_size.is_finite() || font_size <= 0.0 {
            return;
        }
        self.top_margin = self.top_margin.saturating_add(scale_usize(
            KATANA_LONG_HEADING_2_TOP_MARGIN_PX,
            font_size / COMPACT_BODY_FONT_SIZE,
        ));
        self.highlight_height = self.line_height.saturating_sub(self.top_margin);
    }

    fn scale_document_text_font(&mut self, font_size: f32, compact_line_height: usize) {
        if !font_size.is_finite() || font_size <= 0.0 {
            return;
        }
        let scale = font_size / BODY_FONT_SIZE;
        self.font_size *= scale;
        self.line_height =
            scaled_document_text_line_height(self.line_height, compact_line_height, font_size);
        self.line_box_height = self.line_height as f32;
        self.top_margin = scale_usize(self.top_margin, scale);
        self.background_height = self.line_height;
        self.highlight_height = self.line_height.saturating_sub(self.top_margin);
        self.underline_offset = underline_offset(self.font_size);
        self.strikethrough_offset = strikethrough_offset(self.font_size);
    }

    fn scale_document_font(&mut self, scale: f32) {
        if !scale.is_finite() || scale <= 0.0 {
            return;
        }
        self.font_size *= scale;
        self.line_height = scale_usize(self.line_height, scale);
        self.line_box_height = self.line_height as f32;
        self.top_margin = scale_usize(self.top_margin, scale);
        self.background_height = scale_usize(self.background_height, scale);
        self.highlight_height = self.line_height.saturating_sub(self.top_margin);
        self.underline_offset = underline_offset(self.font_size);
        self.strikethrough_offset = strikethrough_offset(self.font_size);
    }

    const fn document(font_size: f32, line_height: usize, top_margin: usize) -> Self {
        Self {
            font_size,
            line_height,
            line_box_height: line_height as f32,
            top_margin,
            baseline_from_line_box_top: None,
            background_height: line_height,
            highlight_height: line_height.saturating_sub(top_margin),
            underline_offset: underline_offset(font_size),
            strikethrough_offset: strikethrough_offset(font_size),
            raster_vertical_scale: 1.0,
        }
    }

    const fn ui() -> Self {
        Self {
            font_size: UI_FONT_SIZE,
            line_height: UI_LINE_HEIGHT,
            line_box_height: UI_LINE_HEIGHT as f32,
            top_margin: UI_TOP_MARGIN,
            baseline_from_line_box_top: None,
            background_height: UI_LINE_HEIGHT,
            highlight_height: UI_HIGHLIGHT_HEIGHT,
            underline_offset: underline_offset(UI_FONT_SIZE),
            strikethrough_offset: strikethrough_offset(UI_FONT_SIZE),
            raster_vertical_scale: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{UiTreeDocumentTypography, UiTreeTextMetrics};
    use crate::raster_host::{
        UiTreeDocumentTypography as UiTreeDocumentTypographyOverrides,
        UiTreeTextRoleBaselineTypography, UiTreeTextRoleTypography,
    };
    use katana_ui_core::atom::Text;
    use katana_ui_core::render_model::{
        UiCommonProps, UiDimension, UiEdgeInsets, UiNode, UiNodeKind, UiTextProps,
    };
    use katana_ui_core::theme::{FontFamily, FontToken, ThemeSnapshot};

    #[test]
    fn body_text_metrics_match_document_surface() {
        let node: UiNode = Text::new("body").text_role("body").into();
        let metrics = UiTreeTextMetrics::for_node(&node);

        assert_eq!(24.0, metrics.font_size);
        assert_eq!(34, metrics.line_height);
        assert_eq!(31, metrics.underline_offset);
        assert_eq!(17, metrics.strikethrough_offset);
        assert!(metrics.strikethrough_offset < metrics.underline_offset);
    }

    #[test]
    fn alert_text_metrics_keep_export_surface_top_padding() {
        let node: UiNode = Text::new("Tip\nbody").text_role("alert").into();
        let metrics = UiTreeTextMetrics::for_node(&node);

        assert_eq!(16, metrics.top_margin);
    }

    #[test]
    fn table_text_metrics_use_export_surface_table_line_height() {
        let mut theme = ThemeSnapshot::dark();
        theme.fonts.push(FontToken {
            name: "document-code".to_string(),
            family: FontFamily::Monospace,
            size: 12.0,
            weight: 400,
        });
        let node: UiNode = Text::new("A | B\n1 | 2").text_role("table").into();
        let typography = UiTreeDocumentTypography::from_theme(&theme);
        let metrics = UiTreeTextMetrics::for_node_with_typography(&node, typography);

        assert_eq!(12.0, metrics.font_size);
        assert_eq!(19, metrics.line_height);
    }

    #[test]
    fn code_text_metrics_keep_raster_line_height_at_compact_font() {
        let mut theme = ThemeSnapshot::dark();
        theme.fonts.push(FontToken {
            name: "document-code".to_string(),
            family: FontFamily::Monospace,
            size: 12.0,
            weight: 400,
        });
        let node: UiNode = Text::new("fn main() {\n    println!(\"Hello\");\n}")
            .text_role("code")
            .into();
        let typography = UiTreeDocumentTypography::from_theme(&theme);
        let metrics = UiTreeTextMetrics::for_node_with_typography(&node, typography);

        assert_eq!(12.0, metrics.font_size);
        assert_eq!(19, metrics.line_height);
    }

    #[test]
    fn code_padding_and_explicit_height_override_the_document_metric_defaults() {
        let node = UiNode::new(UiNodeKind::Text, "let value = 1;")
            .text(UiTextProps {
                role: "code".to_string(),
                ..UiTextProps::default()
            })
            .common(
                UiCommonProps::default()
                    .padding(UiEdgeInsets {
                        top: UiDimension::px(9),
                        ..UiEdgeInsets::default()
                    })
                    .height(UiDimension::px(28)),
            );

        let metrics = UiTreeTextMetrics::for_node(&node);

        assert_eq!(9, metrics.top_margin);
        assert_eq!(28, metrics.background_height);
    }

    #[test]
    fn document_metrics_can_follow_theme_document_font_tokens() {
        let mut theme = ThemeSnapshot::dark();
        theme.fonts.push(FontToken {
            name: "document-body".to_string(),
            family: FontFamily::Proportional,
            size: 18.0,
            weight: 400,
        });
        let node: UiNode = Text::new("body").text_role("body").into();
        let typography = UiTreeDocumentTypography::from_theme(&theme);
        let metrics = UiTreeTextMetrics::for_node_with_typography(&node, typography);

        assert_eq!(18.0, metrics.font_size);
        assert_eq!(27, metrics.line_height);
    }

    #[test]
    fn document_body_metrics_use_compact_export_surface_height_at_preview_font_14() {
        let mut theme = ThemeSnapshot::dark();
        theme.fonts.push(FontToken {
            name: "document-body".to_string(),
            family: FontFamily::Proportional,
            size: 14.0,
            weight: 400,
        });
        let node: UiNode = Text::new("body").text_role("body").into();
        let typography = UiTreeDocumentTypography::from_theme(&theme);
        let metrics = UiTreeTextMetrics::for_node_with_typography(&node, typography);

        assert!((metrics.font_size - 14.0).abs() < 0.01);
        assert_eq!(23, metrics.line_height);
        assert_eq!(0, metrics.top_margin);
    }

    #[test]
    fn compact_alert_metrics_keep_export_surface_body_spacing() {
        let mut theme = ThemeSnapshot::dark();
        theme.fonts.push(FontToken {
            name: "document-body".to_string(),
            family: FontFamily::Proportional,
            size: 14.0,
            weight: 400,
        });
        let node: UiNode = Text::new("Tip\nbody").text_role("alert").into();
        let typography = UiTreeDocumentTypography::from_theme(&theme);
        let metrics = UiTreeTextMetrics::for_node_with_typography(&node, typography);

        assert_eq!(28, metrics.line_height);
        assert_eq!(9, metrics.top_margin);
    }

    #[test]
    fn compact_html_body_metrics_match_export_surface_origin() {
        let mut theme = ThemeSnapshot::dark();
        theme.fonts.push(FontToken {
            name: "document-body".to_string(),
            family: FontFamily::Proportional,
            size: 14.0,
            weight: 400,
        });
        let node: UiNode = Text::new("html").text_role("html-centered").into();
        let typography = UiTreeDocumentTypography::from_theme(&theme);
        let metrics = UiTreeTextMetrics::for_node_with_typography(&node, typography);

        assert_eq!(14.0, metrics.font_size);
        assert_eq!(23, metrics.line_height);
        assert_eq!(0, metrics.top_margin);
    }

    #[test]
    fn compact_html_preview_body_metrics_match_katana_preview_origin() {
        let mut theme = ThemeSnapshot::dark();
        theme.fonts.push(FontToken {
            name: "document-body".to_string(),
            family: FontFamily::Proportional,
            size: 14.0,
            weight: 400,
        });
        let node: UiNode = Text::new("html").text_role("html-centered-preview").into();
        let typography = UiTreeDocumentTypography::from_theme(&theme);
        let metrics = UiTreeTextMetrics::for_node_with_typography(&node, typography);

        assert_eq!(13.0, metrics.font_size);
        assert_eq!(21, metrics.line_height);
        assert_eq!(0, metrics.top_margin);
    }

    #[test]
    fn compact_html_heading_metrics_match_katana_preview_origin() {
        let mut theme = ThemeSnapshot::dark();
        theme.fonts.push(FontToken {
            name: "document-body".to_string(),
            family: FontFamily::Proportional,
            size: 14.0,
            weight: 400,
        });
        let node: UiNode = Text::new("Heading")
            .text_role("heading-html-centered")
            .into();
        let typography = UiTreeDocumentTypography::from_theme(&theme);
        let metrics = UiTreeTextMetrics::for_node_with_typography(&node, typography);

        assert!((metrics.font_size - 21.0).abs() < 0.01);
        assert_eq!(40, metrics.line_height);
        assert_eq!(12, metrics.top_margin);
    }

    #[test]
    fn document_heading_metrics_keep_katana_scale_at_preview_font_14() {
        let mut theme = ThemeSnapshot::dark();
        theme.fonts.push(FontToken {
            name: "document-body".to_string(),
            family: FontFamily::Proportional,
            size: 14.0,
            weight: 400,
        });
        let node: UiNode = Text::new("Heading").text_role("heading-2").into();
        let typography = UiTreeDocumentTypography::from_theme(&theme);
        let metrics = UiTreeTextMetrics::for_node_with_typography(&node, typography);

        assert!((metrics.font_size - 19.83).abs() < 0.02);
        assert_eq!(34, metrics.line_height);
    }

    #[test]
    fn long_heading_2_metrics_keep_katana_preview_vertical_origin() {
        let mut theme = ThemeSnapshot::dark();
        theme.fonts.push(FontToken {
            name: "document-body".to_string(),
            family: FontFamily::Proportional,
            size: 14.0,
            weight: 400,
        });
        let node: UiNode = Text::new("Long Heading").text_role("heading-2-long").into();
        let typography = UiTreeDocumentTypography::from_theme(&theme);
        let metrics = UiTreeTextMetrics::for_node_with_typography(&node, typography);

        assert!((metrics.font_size - 19.83).abs() < 0.02);
        assert_eq!(34, metrics.line_height);
        assert_eq!(23, metrics.top_margin);
    }

    #[test]
    fn document_heading_1_metrics_match_katana_markdown_preview_at_preview_font_14() {
        let mut theme = ThemeSnapshot::dark();
        theme.fonts.push(FontToken {
            name: "document-body".to_string(),
            family: FontFamily::Proportional,
            size: 14.0,
            weight: 400,
        });
        let node: UiNode = Text::new("Heading").text_role("heading").into();
        let typography = UiTreeDocumentTypography::from_theme(&theme);
        let metrics = UiTreeTextMetrics::for_node_with_typography(&node, typography);

        assert!((metrics.font_size - 21.0).abs() < 0.02);
        assert_eq!(40, metrics.line_height);
    }

    #[test]
    fn explicit_document_role_typography_keeps_font_line_height_and_baseline_independent() {
        let body = UiTreeTextRoleBaselineTypography::new(16.5, 23.0, 0.0);
        let heading = UiTreeTextRoleBaselineTypography::new(23.4, 34.0, 8.0);
        let document_typography = UiTreeDocumentTypographyOverrides::new()
            .with_body_baseline(body)
            .with_heading_2_baseline(heading)
            .with_heading_3_baseline(UiTreeTextRoleBaselineTypography::new(22.0, 30.0, 7.0));
        let body_node: UiNode = Text::new("body").text_role("body").into();
        let heading_node: UiNode = Text::new("Long Heading").text_role("heading-2-long").into();
        let heading_3_node: UiNode = Text::new("Heading").text_role("heading-3").into();

        let theme = ThemeSnapshot::dark();
        let typography = UiTreeDocumentTypography::from_theme_with_document_typography(
            &theme,
            document_typography,
        );
        let body_metrics = UiTreeTextMetrics::for_node_with_typography(&body_node, typography);
        let heading_metrics =
            UiTreeTextMetrics::for_node_with_typography(&heading_node, typography);
        let heading_3_metrics =
            UiTreeTextMetrics::for_node_with_typography(&heading_3_node, typography);

        assert_eq!(16.5, body_metrics.font_size);
        assert_eq!(23, body_metrics.line_height);
        assert_eq!(23.0, body_metrics.line_box_height);
        assert_eq!(0, body_metrics.top_margin);
        assert_eq!(Some(0.0), body_metrics.baseline_from_line_box_top);
        assert_eq!(23.4, heading_metrics.font_size);
        assert_eq!(34, heading_metrics.line_height);
        assert_eq!(34.0, heading_metrics.line_box_height);
        assert_eq!(0, heading_metrics.top_margin);
        assert_eq!(Some(8.0), heading_metrics.baseline_from_line_box_top);
        assert_eq!(22.0, heading_3_metrics.font_size);
        assert_eq!(30, heading_3_metrics.line_height);
        assert_eq!(30.0, heading_3_metrics.line_box_height);
        assert_eq!(0, heading_3_metrics.top_margin);
        assert_eq!(Some(7.0), heading_3_metrics.baseline_from_line_box_top);
    }

    #[test]
    fn invalid_document_role_typography_keeps_theme_derived_metrics() {
        let node: UiNode = Text::new("body").text_role("body").into();
        let theme = ThemeSnapshot::dark();
        let theme_typography = UiTreeDocumentTypography::from_theme(&theme);
        let invalid_override = UiTreeDocumentTypographyOverrides::new()
            .with_body(UiTreeTextRoleTypography::new(0.0, 23, 0));
        let invalid_typography =
            UiTreeDocumentTypography::from_theme_with_document_typography(&theme, invalid_override);

        assert_eq!(
            UiTreeTextMetrics::for_node_with_typography(&node, theme_typography),
            UiTreeTextMetrics::for_node_with_typography(&node, invalid_typography)
        );
    }

    #[test]
    fn export_surface_heading_3_metrics_match_kdv_export_surface_at_preview_font_14() {
        let mut theme = ThemeSnapshot::dark();
        theme.fonts.push(FontToken {
            name: "document-body".to_string(),
            family: FontFamily::Proportional,
            size: 14.0,
            weight: 400,
        });
        let node: UiNode = Text::new("Heading").text_role("heading-3-export").into();
        let typography = UiTreeDocumentTypography::from_theme(&theme);
        let metrics = UiTreeTextMetrics::for_node_with_typography(&node, typography);

        assert!((metrics.font_size - 16.33).abs() < 0.02);
        assert_eq!(30, metrics.line_height);
    }

    #[test]
    fn export_surface_heading_1_and_2_roles_use_their_dedicated_metrics() {
        let heading_1: UiNode = Text::new("Heading").text_role("heading-export").into();
        let heading_2: UiNode = Text::new("Heading").text_role("heading-2-export").into();

        let heading_1_metrics = UiTreeTextMetrics::for_node(&heading_1);
        let heading_2_metrics = UiTreeTextMetrics::for_node(&heading_2);

        assert!(heading_1_metrics.font_size > heading_2_metrics.font_size);
        assert!(heading_1_metrics.line_height > heading_2_metrics.line_height);
    }

    #[test]
    fn invalid_typography_scales_leave_text_metrics_unchanged() {
        let mut metrics = UiTreeTextMetrics::for_role("body");
        let original = metrics;

        metrics.add_scaled_top_margin(f32::NAN);
        metrics.add_scaled_long_heading_2_top_margin(0.0);
        metrics.scale_document_text_font(-1.0, 20);
        metrics.scale_document_font(f32::INFINITY);

        assert_eq!(original.font_size, metrics.font_size);
        assert_eq!(original.line_height, metrics.line_height);
        assert_eq!(original.top_margin, metrics.top_margin);
        assert_eq!(original.background_height, metrics.background_height);
    }

    #[test]
    fn compact_heading_metrics_keep_raster_line_height_inside_clip() {
        let mut theme = ThemeSnapshot::dark();
        theme.fonts.push(FontToken {
            name: "document-body".to_string(),
            family: FontFamily::Proportional,
            size: 14.0,
            weight: 400,
        });
        let typography = UiTreeDocumentTypography::from_theme(&theme);

        for role in ["heading-2", "heading-3"] {
            let node: UiNode = Text::new("Heading").text_role(role).into();
            let metrics = UiTreeTextMetrics::for_node_with_typography(&node, typography);
            let raster_line_height = (metrics.font_size * 1.45).ceil() as usize;

            assert!(
                metrics.line_height >= raster_line_height,
                "{role} line height {} must contain raster height {raster_line_height}",
                metrics.line_height
            );
        }
    }
}
