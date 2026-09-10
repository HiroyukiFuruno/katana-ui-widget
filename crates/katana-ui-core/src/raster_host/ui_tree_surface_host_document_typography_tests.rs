use super::{
    Canvas, UiTreeDocumentTypography, UiTreeRenderArea, UiTreeSurfaceHost,
    UiTreeTextRoleBaselineTypography, UiTreeTextRoleTypography,
};
use crate::test_assert::KucTestExpect;
use katana_ui_core::atom::Text;
use katana_ui_core::molecule::Accordion;
use katana_ui_core::render_model::{UiHostActionSpec, UiNode, UiNodeId, UiNodeKind, UiTextProps};
use katana_ui_core::theme::ThemeSnapshot;

const TEST_AREA_WIDTH: usize = 240;
const TEST_AREA_HEIGHT: usize = 80;

#[test]
fn surface_host_document_typography_shares_raster_and_node_hit_metrics() {
    let document_typography = UiTreeDocumentTypography::new()
        .with_body_baseline(UiTreeTextRoleBaselineTypography::new(16.5, 23.0, 18.5))
        .with_heading_1_baseline(UiTreeTextRoleBaselineTypography::new(24.75, 40.0, 30.0));
    let body: UiNode = Text::new("WWWW").text_role("body").into();
    let heading: UiNode = Text::new("WWWW").text_role("heading").into();
    let root = UiNode::new(UiNodeKind::Column, "")
        .child(body.stable_node_id(UiNodeId::new("body")))
        .child(heading.stable_node_id(UiNodeId::new("heading")));
    let host =
        UiTreeSurfaceHost::with_document_typography(ThemeSnapshot::dark(), document_typography);
    let mut canvas = Canvas::new(TEST_AREA_WIDTH, TEST_AREA_HEIGHT, 0);

    host.render(&mut canvas, &root, test_area());
    let hits = host.document_node_hits(&root, test_area());
    let body_hit = hits
        .iter()
        .find(|hit| hit.node_id.as_str() == "body")
        .kuc_expect("body node hit");
    let heading_hit = hits
        .iter()
        .find(|hit| hit.node_id.as_str() == "heading")
        .kuc_expect("heading node hit");

    assert_eq!(23, body_hit.rect.height);
    assert_eq!(23, heading_hit.rect.y);
    assert_eq!(40, heading_hit.rect.height);
    assert!(non_background_width(&canvas, 0, 23) > 0);
    assert!(non_background_width(&canvas, 23, 63) > non_background_width(&canvas, 0, 23));
}

#[test]
fn surface_host_document_typography_keeps_sub_legacy_line_heights_in_action_layout() {
    let document_typography =
        UiTreeDocumentTypography::new().with_body(UiTreeTextRoleTypography::new(10.0, 12, 0));
    let body: UiNode = Text::new("Body").text_role("body").into();
    let body = body
        .stable_node_id(UiNodeId::new("body"))
        .host_action(UiHostActionSpec::command("body", "Body"));
    let child: UiNode = Text::new("Child").text_role("body").into();
    let child = child
        .stable_node_id(UiNodeId::new("child"))
        .host_action(UiHostActionSpec::command("child", "Child"));
    let accordion = UiNode::from(Accordion::new("Details").open(true).child(child))
        .text(UiTextProps {
            role: "html-accordion".to_owned(),
            ..UiTextProps::default()
        })
        .stable_node_id(UiNodeId::new("accordion"));
    let root = UiNode::new(UiNodeKind::Column, "")
        .child(body)
        .child(accordion);
    let host =
        UiTreeSurfaceHost::with_document_typography(ThemeSnapshot::dark(), document_typography);
    let action_hits = host.document_host_action_hits(&root, test_area());
    let node_hits = host.document_node_hits(&root, test_area());
    let body_action = action_hits
        .iter()
        .find(|hit| hit.action.action_id == "body")
        .kuc_expect("body action hit");
    let accordion_action = action_hits
        .iter()
        .find(|hit| hit.action.action_id == "ui.disclosure.toggle")
        .kuc_expect("accordion action hit");
    let child_action = action_hits
        .iter()
        .find(|hit| hit.action.action_id == "child")
        .kuc_expect("accordion child action hit");
    let body_node = node_hits
        .iter()
        .find(|hit| hit.node_id.as_str() == "body")
        .kuc_expect("body node hit");
    let child_node = node_hits
        .iter()
        .find(|hit| hit.node_id.as_str() == "child")
        .kuc_expect("accordion child node hit");

    assert_eq!(12, body_action.rect.height);
    assert_eq!(12, accordion_action.rect.y);
    assert_eq!(12, accordion_action.rect.height);
    assert_eq!(24, child_action.rect.y);
    assert_eq!(12, child_action.rect.height);
    assert_eq!(body_action.rect, body_node.rect);
    assert_eq!(child_action.rect, child_node.rect);
}

#[test]
fn surface_host_document_typography_applies_preview_accordion_hit_metrics() {
    let document_typography =
        UiTreeDocumentTypography::new().with_body(UiTreeTextRoleTypography::new(10.0, 12, 0));
    let body: UiNode = Text::new("Body").text_role("body").into();
    let body = body
        .stable_node_id(UiNodeId::new("body"))
        .host_action(UiHostActionSpec::command("body", "Body"));
    let child: UiNode = Text::new("Child").text_role("body").into();
    let child = child
        .stable_node_id(UiNodeId::new("child"))
        .host_action(UiHostActionSpec::command("child", "Child"));
    let accordion = UiNode::from(Accordion::new("Details").open(true).child(child))
        .text(UiTextProps {
            role: "html-accordion-preview".to_owned(),
            ..UiTextProps::default()
        })
        .stable_node_id(UiNodeId::new("accordion"));
    let root = UiNode::new(UiNodeKind::Column, "")
        .child(body)
        .child(accordion);
    let host =
        UiTreeSurfaceHost::with_document_typography(ThemeSnapshot::dark(), document_typography);
    let action_hits = host.document_host_action_hits(&root, test_area());
    let node_hits = host.document_node_hits(&root, test_area());
    let accordion_action = action_hits
        .iter()
        .find(|hit| hit.action.action_id == "ui.disclosure.toggle")
        .kuc_expect("preview accordion action hit");
    let child_action = action_hits
        .iter()
        .find(|hit| hit.action.action_id == "child")
        .kuc_expect("preview accordion child action hit");
    let child_node = node_hits
        .iter()
        .find(|hit| hit.node_id.as_str() == "child")
        .kuc_expect("preview accordion child node hit");

    assert_eq!(12, accordion_action.rect.y);
    assert_eq!(12, accordion_action.rect.height);
    assert_eq!(24, child_action.rect.y);
    assert_eq!(12, child_action.rect.height);
    assert_eq!(child_action.rect, child_node.rect);
}

fn test_area() -> UiTreeRenderArea {
    UiTreeRenderArea {
        x: 0,
        y: 0,
        width: TEST_AREA_WIDTH,
        height: TEST_AREA_HEIGHT,
        scroll_y: 0.0,
    }
}

fn non_background_width(canvas: &Canvas, start_y: usize, end_y: usize) -> usize {
    let mut min_x = canvas.width();
    let mut max_x = 0;
    for (index, pixel) in canvas.pixels().iter().enumerate() {
        let x = index % canvas.width();
        let y = index / canvas.width();
        if *pixel != 0 && y >= start_y && y < end_y {
            min_x = min_x.min(x);
            max_x = max_x.max(x);
        }
    }
    if min_x == canvas.width() {
        return 0;
    }
    max_x.saturating_sub(min_x).saturating_add(1)
}
