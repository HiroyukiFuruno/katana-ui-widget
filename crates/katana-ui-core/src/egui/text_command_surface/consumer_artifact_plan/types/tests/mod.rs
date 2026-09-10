use super::*;
use crate::atom::TextArea;
#[cfg(target_os = "linux")]
use crate::egui::context_menu::{ContextMenuPresentation, ContextMenuPresentationItem};
#[cfg(target_os = "linux")]
use crate::egui::text_command_surface::EguiTextCommandSurfaceFloatingPresentation;
#[cfg(target_os = "linux")]
use crate::egui::text_command_surface::EguiTextCommandSurfaceSearchPresentation;
use crate::egui::text_command_surface::{
    EguiTextCommandSurfaceHostProjectionEncoder, EguiTextCommandSurfaceHostProjectionLease,
    EguiTextCommandSurfaceHostTargetToken, EguiTextCommandSurfacePresentation,
    EguiTextCommandSurfacePresentationToken, TextCommandSurfaceStyle,
};
#[cfg(target_os = "linux")]
use crate::molecule::command_chrome::{
    CommandChromeAction, CommandChromeCapability, CommandChromeSearchPresentation,
    CommandChromeText, CommandChromeToolbarPresentation, FloatingCommandToolbarVisibility,
    SearchControlCapabilities, SearchControlIcons, SearchControlStrings,
    SearchResultSummaryTemplate,
};
#[cfg(target_os = "linux")]
use crate::molecule::structured::{ReplaceMode, SearchOptions};
#[cfg(target_os = "linux")]
use crate::render_model::UiStateId;
use crate::text_surface::{
    TextSurface, TextSurfacePresentation, TextSurfaceProps, TextSurfaceViewport,
};
use std::time::{SystemTime, UNIX_EPOCH};

const TEST_SURFACE_WIDTH: u32 = 320;
const TEST_SURFACE_HEIGHT: u32 = 180;

mod execution;
mod support;

pub(super) fn token(revision: u64) -> EguiTextCommandSurfacePresentationToken {
    EguiTextCommandSurfacePresentationToken::from_opaque_bytes(
        revision,
        EguiTextCommandSurfaceHostTargetToken::from_opaque_bytes("opaque-target"),
        "opaque-presentation",
    )
}

pub(super) fn binding(leaf: &str, revision: u64) -> ConsumerArtifactStageBinding {
    ConsumerArtifactStageBinding::new(
        ConsumerArtifactLeafId::new(leaf).expect("leaf"),
        GenericInteractionClass::ImeCommit,
        GenericEffectClass::NoHostEffect,
        token(revision),
    )
}

#[test]
fn lease_binding_retains_the_complete_host_projection_lease() {
    let token = token(1);
    let lease = EguiTextCommandSurfaceHostProjectionLease::new(token, |_context| Ok(None));
    let binding = ConsumerArtifactStageBinding::from_host_projection_lease(
        ConsumerArtifactLeafId::new("lease-binding").expect("leaf"),
        "host-action",
        GenericInteractionClass::TextInput,
        GenericEffectClass::NoHostEffect,
        lease,
    );

    assert!(binding.token.is_none());
    assert!(binding.lease.is_some());
    assert_eq!(binding.token().expect("lease token").revision(), 1);
}

#[cfg(target_os = "linux")]
pub(super) fn binding_from_encoder(
    leaf: &str,
    revision: u64,
    target: impl Into<Vec<u8>>,
) -> ConsumerArtifactStageBinding {
    binding_from_encoder_with_interaction(
        leaf,
        revision,
        target,
        GenericInteractionClass::ImeCommit,
    )
}

pub(super) fn binding_from_encoder_with_interaction(
    leaf: &str,
    revision: u64,
    target: impl Into<Vec<u8>>,
    interaction: GenericInteractionClass,
) -> ConsumerArtifactStageBinding {
    ConsumerArtifactStageBinding::new(
        ConsumerArtifactLeafId::new(leaf).expect("leaf"),
        interaction,
        GenericEffectClass::NoHostEffect,
        EguiTextCommandSurfaceHostProjectionEncoder::token(
            revision,
            target,
            presentation(),
            TextCommandSurfaceStyle::standard().expect("standard style"),
        )
        .expect("token should encode"),
    )
}

pub(super) fn complete_bindings(
    initial_revision: u64,
    target: &[u8],
) -> Vec<ConsumerArtifactStageBinding> {
    GenericInteractionClass::FULL_EDITOR_SEQUENCE
        .into_iter()
        .enumerate()
        .map(|(index, interaction)| {
            binding_from_encoder_with_interaction(
                &format!("full-editor-stage-{index}"),
                initial_revision + index as u64,
                target.to_vec(),
                interaction,
            )
        })
        .collect()
}

#[cfg(target_os = "linux")]
pub(super) fn complete_semantic_bindings(
    initial_revision: u64,
    target: &[u8],
) -> Vec<ConsumerArtifactStageBinding> {
    GenericInteractionClass::FULL_EDITOR_SEQUENCE
        .into_iter()
        .enumerate()
        .map(|(index, interaction)| {
            let leaf = match interaction {
                GenericInteractionClass::AccessibilityActivation => {
                    "kuc.consumer-artifact.activate".to_owned()
                }
                _ => format!("semantic-stage-{index}"),
            };
            let mut presentation = semantic_presentation();
            if interaction == GenericInteractionClass::ContextMenu {
                presentation
                    .context_menu
                    .as_mut()
                    .expect("semantic context menu")
                    .visible = true;
            }
            let token = EguiTextCommandSurfaceHostProjectionEncoder::token(
                initial_revision + index as u64,
                target.to_vec(),
                presentation,
                TextCommandSurfaceStyle::standard().expect("standard style"),
            )
            .expect("token should encode");
            match interaction {
                GenericInteractionClass::ToolbarActivation
                | GenericInteractionClass::FloatingToolbar => {
                    ConsumerArtifactStageBinding::new_with_action_target(
                        ConsumerArtifactLeafId::new(leaf).expect("leaf"),
                        "kuc.consumer-artifact.activate",
                        interaction,
                        GenericEffectClass::NoHostEffect,
                        token,
                    )
                }
                _ => ConsumerArtifactStageBinding::new(
                    ConsumerArtifactLeafId::new(leaf).expect("leaf"),
                    interaction,
                    GenericEffectClass::NoHostEffect,
                    token,
                ),
            }
        })
        .collect()
}

fn presentation() -> EguiTextCommandSurfacePresentation {
    let scroll_targets = (1..=80)
        .map(|index| format!("scroll target {index:02}"))
        .collect::<Vec<_>>()
        .join("\n");
    let surface = TextSurface::new(TextSurfaceProps::new(
        TextArea::new("consumer-artifact-plan")
            .value(format!("consumer artifact\n{scroll_targets}")),
        Vec::new(),
        TextSurfaceViewport::new(0, 0, TEST_SURFACE_WIDTH, TEST_SURFACE_HEIGHT),
    ));
    EguiTextCommandSurfacePresentation {
        text_state_id: None,
        text: TextSurfacePresentation::from_props(surface.props()),
        toolbar: None,
        floating: None,
        search: None,
        context_menu: None,
    }
}

#[cfg(target_os = "linux")]
pub(super) fn semantic_presentation() -> EguiTextCommandSurfacePresentation {
    let toolbar = CommandChromeToolbarPresentation {
        actions: vec![
            CommandChromeAction::new("kuc.consumer-artifact.other", "Other")
                .accessibility_label("Other"),
            CommandChromeAction::new("kuc.consumer-artifact.activate", "Activate")
                .accessibility_label("Activate"),
        ],
        groups: Vec::new(),
        display_mode: Default::default(),
        density: Default::default(),
        overflow_strategy: Default::default(),
    };
    let mut presentation = presentation();
    presentation.toolbar = Some(toolbar.clone());
    presentation.floating = Some(EguiTextCommandSurfaceFloatingPresentation {
        toolbar,
        visibility: FloatingCommandToolbarVisibility::Visible,
    });
    presentation.search = Some(search_presentation());
    presentation.context_menu = Some(ContextMenuPresentation {
        visible: false,
        items: vec![ContextMenuPresentationItem::action(
            "kuc.consumer-artifact.context-action",
            "Context action",
        )],
    });
    presentation
}

#[cfg(target_os = "linux")]
fn search_text(value: &str) -> CommandChromeText {
    CommandChromeText::new(value, value, value)
}

#[cfg(target_os = "linux")]
fn search_presentation() -> EguiTextCommandSurfaceSearchPresentation {
    EguiTextCommandSurfaceSearchPresentation {
        state_id: UiStateId::new("consumer-artifact-search"),
        label: String::from("検索と置換"),
        value: CommandChromeSearchPresentation {
            query: String::from("needle"),
            options: SearchOptions::default(),
            result_count: Some(1),
            active_index: Some(0),
            replace_mode: ReplaceMode::Visible,
            replace_value: String::from("replacement"),
            strings: SearchControlStrings {
                strip: search_text("検索と置換"),
                query: search_text("検索語"),
                replace: search_text("置換"),
                match_case: search_text("大文字小文字"),
                whole_word: search_text("単語"),
                use_regex: search_text("正規表現"),
                previous: search_text("前へ"),
                next: search_text("次へ"),
                replace_one: search_text("置換"),
                replace_all: search_text("すべて置換"),
                close: search_text("閉じる"),
                result_summary: SearchResultSummaryTemplate {
                    empty: String::from("検索待機"),
                    zero_results: String::from("一致なし"),
                    single_result: String::from("1件"),
                    indexed_result: String::from("{active} / {count}"),
                    count_results: String::from("{count}件"),
                },
            },
            capabilities: SearchControlCapabilities {
                regex: CommandChromeCapability::available(),
                replace: CommandChromeCapability::available(),
                navigation: CommandChromeCapability::available(),
                close: CommandChromeCapability::available(),
            },
            icons: SearchControlIcons::default(),
        },
    }
}

pub(super) fn temp_dir(tag: &str) -> std::path::PathBuf {
    let pid = std::process::id();
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should work")
        .as_millis();
    let path = std::env::temp_dir().join(format!("kuc-consumer-plan-{tag}-{pid}-{millis}"));
    std::fs::create_dir_all(&path).expect("temp output dir should create");
    path
}
