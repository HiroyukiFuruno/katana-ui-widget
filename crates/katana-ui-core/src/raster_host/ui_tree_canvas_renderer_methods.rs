use super::{
    Canvas, ContainerPadding, INDENT, NODE_GAP, TEXT_HEIGHT, UiNode, UiNodeKind,
    UiTreeCanvasPalette, UiTreeCanvasRenderer, UiTreeRenderArea, UiTreeSettingsContext,
    UiTreeTextMetrics, UiVisualRole, absolute_child_rect, child_container_x, child_render_area,
    dimension_px, draw_hover_surface, draw_label, gap_after_child, has_absolute_child, is_absolute,
    is_outside_vertical_viewport, remaining_width, should_draw_container_label, stack_frame_height,
};
use crate::raster_host::text::RichTextStyle;
use crate::raster_host::ui_tree_canvas_scroll_measure::measured_node_height;

impl UiTreeCanvasRenderer {
    pub(super) fn draw_container(
        &self,
        canvas: &mut Canvas,
        node: &UiNode,
        x: usize,
        y: &mut usize,
        area: UiTreeRenderArea,
        palette: UiTreeCanvasPalette,
    ) {
        if node.kind() == UiNodeKind::Stack && has_absolute_child(node) {
            self.draw_overlay_stack(canvas, node, x, y, area, palette);
            return;
        }
        let hover_surface_y = *y;
        let hover_surface_height = self.hover_surface_height(node, x, area, palette);
        if should_draw_container_label(node) {
            draw_label(canvas, &self.text, node, x, y, palette);
        }
        let padding = ContainerPadding::from_node(node);
        let child_x = child_container_x(node, x).saturating_add(padding.left);
        let child_area = child_render_area(area, node, child_x, padding);
        *y = y.saturating_add(padding.top);
        let requested_height = dimension_px(&node.props().common.height);
        if requested_height > 0 {
            let clip_height = hover_surface_child_clip_height(node, requested_height);
            canvas.with_clip(
                x,
                *y,
                remaining_width(area, x),
                clip_height,
                &mut |canvas| {
                    for (index, child) in node.children().iter().enumerate() {
                        self.draw_container_child(canvas, child, child_x, y, child_area, palette);
                        if index + 1 < node.children().len() {
                            *y = y.saturating_add(gap_after_child(
                                node,
                                child,
                                &node.children()[index + 1],
                            ));
                        }
                    }
                },
            );
        } else {
            for (index, child) in node.children().iter().enumerate() {
                self.draw_container_child(canvas, child, child_x, y, child_area, palette);
                if index + 1 < node.children().len() {
                    *y =
                        y.saturating_add(gap_after_child(node, child, &node.children()[index + 1]));
                }
            }
        }
        *y = y.saturating_add(padding.bottom);
        draw_hover_surface(
            canvas,
            node,
            x,
            hover_surface_y,
            area,
            palette,
            hover_surface_height,
        );
    }

    fn draw_container_child(
        &self,
        canvas: &mut Canvas,
        child: &UiNode,
        child_x: usize,
        y: &mut usize,
        area: UiTreeRenderArea,
        palette: UiTreeCanvasPalette,
    ) {
        let height =
            self.measured_scroll_node_height(child, self.text_context(palette), child_x, area);
        if is_outside_vertical_viewport(*y, height, area) {
            *y = y.saturating_add(height);
            return;
        }
        self.render_node(canvas, child, child_x, y, area, palette);
    }

    fn hover_surface_height(
        &self,
        node: &UiNode,
        x: usize,
        area: UiTreeRenderArea,
        palette: UiTreeCanvasPalette,
    ) -> usize {
        let requested_height = dimension_px(&node.props().common.height);
        if requested_height > 0 {
            return requested_height;
        }
        if node.props().visual_role != UiVisualRole::HoverSurface {
            return TEXT_HEIGHT;
        }
        measured_node_height(node, self.text_context(palette), x, area)
    }

    pub(super) fn draw_overlay_stack(
        &self,
        canvas: &mut Canvas,
        node: &UiNode,
        x: usize,
        y: &mut usize,
        area: UiTreeRenderArea,
        palette: UiTreeCanvasPalette,
    ) {
        let frame_top = *y;
        let frame_width = remaining_width(area, x);
        let frame_height = stack_frame_height(node).max(TEXT_HEIGHT);
        let visible_top = frame_top.max(area.y);
        let visible_bottom = frame_top
            .saturating_add(frame_height)
            .min(area.y.saturating_add(area.height));
        let visible_height = visible_bottom.saturating_sub(visible_top);
        if visible_height == 0 {
            *y = frame_top.saturating_add(frame_height);
            return;
        }
        let scroll_y = area.scroll_y.round().max(0.0) as usize;
        canvas.with_clip(x, visible_top, frame_width, visible_height, &mut |canvas| {
            if matches!(
                node.props().visual_role,
                UiVisualRole::MediaFrame | UiVisualRole::ExportMediaFrame
            ) {
                canvas.fill_rect(x, frame_top, frame_width, frame_height, palette.background);
            }
            let stack_area = UiTreeRenderArea {
                x,
                y: frame_top,
                width: frame_width,
                height: visible_height,
                scroll_y: area.scroll_y,
            };
            for child in node.children().iter().filter(|child| !is_absolute(child)) {
                let mut child_y = frame_top;
                self.render_node(canvas, child, x, &mut child_y, stack_area, palette);
            }
            for child in node.children().iter().filter(|child| is_absolute(child)) {
                let rect = absolute_child_rect(x, frame_top, frame_width, frame_height, child);
                let Some(child_y) = rect.y.checked_sub(scroll_y) else {
                    continue;
                };
                let mut child_y = child_y;
                self.render_node(canvas, child, rect.x, &mut child_y, stack_area, palette);
            }
        });
        *y = frame_top.saturating_add(frame_height);
    }

    pub(super) fn draw_accordion(
        &self,
        canvas: &mut Canvas,
        node: &UiNode,
        x: usize,
        y: &mut usize,
        area: UiTreeRenderArea,
        palette: UiTreeCanvasPalette,
    ) {
        let document_accordion = node.props().text.role == "html-accordion";
        let metrics = UiTreeTextMetrics::for_node_with_typography(node, self.typography);
        let label = if document_accordion {
            node.props().label.clone()
        } else if node.props().interaction.open {
            format!("v {}", node.props().label)
        } else {
            format!("> {}", node.props().label)
        };
        if let Some(baseline_from_line_box_top) = metrics.baseline_from_line_box_top {
            self.text.draw_signed_styled_in_line_box(
                canvas,
                &label,
                x as isize,
                (y.saturating_add(metrics.top_margin)) as f32,
                metrics.line_box_height,
                baseline_from_line_box_top,
                RichTextStyle::new(metrics.font_size, palette.text),
            );
        } else {
            self.text.draw(
                canvas,
                &label,
                x,
                y.saturating_add(metrics.top_margin),
                metrics.font_size,
                palette.text,
            );
        }
        *y = y.saturating_add(metrics.line_height);
        if node.props().interaction.open {
            let child_x = if document_accordion {
                x
            } else {
                x.saturating_add(INDENT)
            };
            for child in node.children() {
                self.render_node(canvas, child, child_x, y, area, palette);
            }
            if !document_accordion {
                *y = y.saturating_add(NODE_GAP);
            }
        }
    }

    pub(super) fn settings_context(
        &self,
        area: UiTreeRenderArea,
        palette: UiTreeCanvasPalette,
    ) -> UiTreeSettingsContext<'_> {
        UiTreeSettingsContext {
            renderer: self,
            text: &self.text,
            area,
            palette,
        }
    }
}

fn hover_surface_child_clip_height(node: &UiNode, requested_height: usize) -> usize {
    if node.props().visual_role == UiVisualRole::HoverSurface {
        return requested_height.saturating_add(TEXT_HEIGHT);
    }
    requested_height
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raster_host::{UiTreeDocumentTypography, UiTreeTextRoleBaselineTypography};
    use katana_ui_core::render_model::{UiDimension, UiInteractionState, UiPosition, UiTextProps};
    use katana_ui_core::theme::ThemeSnapshot;

    #[test]
    fn container_renderer_covers_fixed_clip_offscreen_child_and_invisible_overlay() {
        let theme = ThemeSnapshot::dark();
        let palette = UiTreeCanvasPalette::from_theme(&theme);
        let renderer = UiTreeCanvasRenderer::new(theme);
        let mut canvas = Canvas::new(180, 80, palette.background);
        let fixed = UiNode::new(UiNodeKind::Column, "fixed")
            .height(UiDimension::px(24))
            .child(UiNode::new(UiNodeKind::Text, "first"))
            .child(UiNode::new(UiNodeKind::Text, "second"));
        let mut y = 0;
        renderer.draw_container(
            &mut canvas,
            &fixed,
            0,
            &mut y,
            UiTreeRenderArea {
                x: 0,
                y: 0,
                width: 180,
                height: 24,
                scroll_y: 0.0,
            },
            palette,
        );

        let labeled = UiNode::new(UiNodeKind::Card, "labeled");
        let mut labeled_y = 0;
        renderer.draw_container(
            &mut canvas,
            &labeled,
            0,
            &mut labeled_y,
            UiTreeRenderArea {
                x: 0,
                y: 0,
                width: 180,
                height: 40,
                scroll_y: 0.0,
            },
            palette,
        );
        assert!(labeled_y >= TEXT_HEIGHT);

        let offscreen =
            UiNode::new(UiNodeKind::Column, "").child(UiNode::new(UiNodeKind::Text, "outside"));
        let mut offscreen_y = 0;
        renderer.draw_container(
            &mut canvas,
            &offscreen,
            0,
            &mut offscreen_y,
            UiTreeRenderArea {
                x: 0,
                y: 70,
                width: 180,
                height: 5,
                scroll_y: 0.0,
            },
            palette,
        );
        assert!(offscreen_y > 0);

        let overlay = UiNode::new(UiNodeKind::Stack, "")
            .height(UiDimension::px(20))
            .child(UiNode::new(UiNodeKind::Button, "absolute").position(UiPosition::Absolute));
        let mut overlay_y = 100;
        renderer.draw_overlay_stack(
            &mut canvas,
            &overlay,
            0,
            &mut overlay_y,
            UiTreeRenderArea {
                x: 0,
                y: 0,
                width: 180,
                height: 20,
                scroll_y: 0.0,
            },
            palette,
        );
        assert_eq!(120, overlay_y);

        let mut clipped_canvas = Canvas::new(180, 24, palette.background);
        let mut clipped_y = 0;
        renderer.draw_overlay_stack(
            &mut clipped_canvas,
            &overlay,
            0,
            &mut clipped_y,
            UiTreeRenderArea {
                x: 0,
                y: 0,
                width: 180,
                height: 20,
                scroll_y: 20.0,
            },
            palette,
        );
        assert_eq!(20, clipped_y);
        assert!(
            clipped_canvas
                .pixels()
                .iter()
                .all(|pixel| *pixel == palette.background)
        );

        let mut empty_overlay_y = 0;
        renderer.draw_overlay_stack(
            &mut canvas,
            &UiNode::new(UiNodeKind::Stack, ""),
            0,
            &mut empty_overlay_y,
            UiTreeRenderArea {
                x: 0,
                y: 0,
                width: 180,
                height: 40,
                scroll_y: 0.0,
            },
            palette,
        );
        assert_eq!(TEXT_HEIGHT, empty_overlay_y);
    }

    #[test]
    fn accordion_renderer_covers_document_and_storybook_open_states() {
        let theme = ThemeSnapshot::dark();
        let palette = UiTreeCanvasPalette::from_theme(&theme);
        let renderer = UiTreeCanvasRenderer::new(theme);
        let mut canvas = Canvas::new(240, 180, palette.background);
        let area = UiTreeRenderArea {
            x: 0,
            y: 0,
            width: 240,
            height: 180,
            scroll_y: 0.0,
        };
        let open = UiInteractionState {
            open: true,
            ..UiInteractionState::default()
        };
        let document = UiNode::new(UiNodeKind::Accordion, "Document")
            .text(UiTextProps {
                role: "html-accordion".to_string(),
                ..UiTextProps::default()
            })
            .interaction(open.clone())
            .child(UiNode::new(UiNodeKind::Text, "Body"));
        let storybook_open = UiNode::new(UiNodeKind::Accordion, "Story")
            .interaction(open)
            .child(UiNode::new(UiNodeKind::Text, "Body"));
        let storybook_closed = UiNode::new(UiNodeKind::Accordion, "Closed");
        let mut y = 0;

        renderer.draw_accordion(&mut canvas, &document, 0, &mut y, area, palette);
        renderer.draw_accordion(&mut canvas, &storybook_open, 0, &mut y, area, palette);
        renderer.draw_accordion(&mut canvas, &storybook_closed, 0, &mut y, area, palette);
        let _ = renderer.settings_context(area, palette);

        assert!(
            canvas
                .pixels()
                .iter()
                .any(|pixel| *pixel != palette.background)
        );
    }

    #[test]
    fn accordion_renderer_covers_document_label_baseline_override() {
        let theme = ThemeSnapshot::dark();
        let palette = UiTreeCanvasPalette::from_theme(&theme);
        let area = UiTreeRenderArea {
            x: 0,
            y: 0,
            width: 240,
            height: 160,
            scroll_y: 0.0,
        };
        let low_baseline = UiTreeDocumentTypography::new()
            .with_body_baseline(UiTreeTextRoleBaselineTypography::new(24.0, 40.0, 8.0));
        let high_baseline = UiTreeDocumentTypography::new()
            .with_body_baseline(UiTreeTextRoleBaselineTypography::new(24.0, 40.0, 18.0));
        let low_renderer =
            UiTreeCanvasRenderer::with_document_typography(theme.clone(), low_baseline);
        let high_renderer =
            UiTreeCanvasRenderer::with_document_typography(theme.clone(), high_baseline);

        for role in ["html-accordion", "html-accordion-preview"] {
            let accordion = UiNode::new(UiNodeKind::Accordion, "Document").text(UiTextProps {
                role: role.to_string(),
                ..UiTextProps::default()
            });
            let mut low_canvas = Canvas::new(240, 80, palette.background);
            let mut low_y = 0;
            low_renderer.draw_accordion(&mut low_canvas, &accordion, 0, &mut low_y, area, palette);
            assert!(low_canvas.non_background_pixels(palette.background) > 0);
            let baseline_low_top = first_non_background_row(&low_canvas, palette.background);

            let mut high_canvas = Canvas::new(240, 80, palette.background);
            let mut high_y = 0;
            high_renderer.draw_accordion(
                &mut high_canvas,
                &accordion,
                0,
                &mut high_y,
                area,
                palette,
            );
            assert!(high_canvas.non_background_pixels(palette.background) > 0);
            let baseline_high_top = first_non_background_row(&high_canvas, palette.background);

            assert_ne!(
                baseline_high_top, baseline_low_top,
                "public baseline override should move {role} label rendering position",
            );
            assert!(baseline_high_top > baseline_low_top);
        }
    }

    fn first_non_background_row(canvas: &Canvas, background: u32) -> usize {
        canvas
            .pixels()
            .iter()
            .position(|pixel| *pixel != background)
            .map(|index| index / canvas.width())
            .unwrap_or(0)
    }

    #[test]
    fn hover_surface_container_extends_child_clip_only_for_the_hover_contract() {
        let hover = UiNode::new(UiNodeKind::Column, "").visual_role(UiVisualRole::HoverSurface);
        let plain = UiNode::new(UiNodeKind::Column, "");

        assert_eq!(
            24 + TEXT_HEIGHT,
            hover_surface_child_clip_height(&hover, 24)
        );
        assert_eq!(24, hover_surface_child_clip_height(&plain, 24));
    }
}
