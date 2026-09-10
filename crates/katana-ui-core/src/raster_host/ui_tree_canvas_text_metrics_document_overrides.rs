use super::{
    UiTreeTextMetrics,
    metric_roles::{
        is_document_body_role, is_heading_1_role, is_heading_2_role, is_heading_3_role,
    },
    metric_scaling::{strikethrough_offset, underline_offset},
};
use crate::raster_host::document_typography::{
    UiTreeDocumentTypography, UiTreeTextRoleBaselineTypography, UiTreeTextRoleTypography,
};

pub(super) fn with_document_typography(
    mut metrics: UiTreeTextMetrics,
    role: &str,
    document_typography: UiTreeDocumentTypography,
) -> UiTreeTextMetrics {
    let Some(role_typography) = active_document_role_typography(role, document_typography) else {
        return metrics;
    };
    let (font_size, line_box_height) = match role_typography {
        ActiveRoleTypography::Legacy(role_typography) => {
            metrics.font_size = role_typography.font_size;
            metrics.line_box_height = role_typography.line_height as f32;
            metrics.line_height = role_typography.line_height;
            metrics.top_margin = role_typography.baseline_offset;
            metrics.baseline_from_line_box_top = None;
            (
                role_typography.font_size,
                role_typography.line_height as f32,
            )
        }
        ActiveRoleTypography::Baseline(role_typography) => {
            metrics.font_size = role_typography.font_size;
            metrics.line_box_height = role_typography.line_box_height;
            metrics.line_height = role_typography.line_box_height.ceil() as usize;
            metrics.top_margin = 0;
            metrics.baseline_from_line_box_top = Some(role_typography.baseline_from_line_box_top);
            (role_typography.font_size, role_typography.line_box_height)
        }
    };
    metrics.background_height = metrics.line_height;
    metrics.highlight_height = line_box_height.ceil().max(1.0) as usize;
    metrics.underline_offset = underline_offset(font_size);
    metrics.strikethrough_offset = strikethrough_offset(font_size);
    metrics
}

pub(super) fn has_active_document_role_typography(
    role: &str,
    document_typography: UiTreeDocumentTypography,
) -> bool {
    active_document_role_typography(role, document_typography).is_some()
}

fn active_document_role_typography(
    role: &str,
    document_typography: UiTreeDocumentTypography,
) -> Option<ActiveRoleTypography> {
    if is_heading_1_role(role) {
        document_typography
            .heading_1_baseline()
            .map(ActiveRoleTypography::Baseline)
            .or_else(|| {
                document_typography
                    .heading_1()
                    .map(ActiveRoleTypography::Legacy)
            })
    } else if is_heading_2_role(role) {
        document_typography
            .heading_2_baseline()
            .map(ActiveRoleTypography::Baseline)
            .or_else(|| {
                document_typography
                    .heading_2()
                    .map(ActiveRoleTypography::Legacy)
            })
    } else if is_heading_3_role(role) {
        document_typography
            .heading_3_baseline()
            .map(ActiveRoleTypography::Baseline)
            .or_else(|| {
                document_typography
                    .heading_3()
                    .map(ActiveRoleTypography::Legacy)
            })
    } else if is_document_body_role(role) {
        document_typography
            .body_baseline()
            .map(ActiveRoleTypography::Baseline)
            .or_else(|| document_typography.body().map(ActiveRoleTypography::Legacy))
    } else {
        None
    }
    .filter(|typography| match typography {
        ActiveRoleTypography::Legacy(typography) => typography.is_valid(),
        ActiveRoleTypography::Baseline(typography) => typography.is_valid(),
    })
}

enum ActiveRoleTypography {
    Legacy(UiTreeTextRoleTypography),
    Baseline(UiTreeTextRoleBaselineTypography),
}
