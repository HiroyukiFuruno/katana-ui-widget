#![cfg(feature = "egui")]
use katana_ui_core::egui::context_menu::{ContextMenuPresentation, ContextMenuPresentationItem};
use katana_ui_core::egui::text_command_surface::{
    EguiTextCommandSurfaceCommandFamilyProjection, EguiTextCommandSurfaceFloatingPresentation,
    EguiTextCommandSurfaceHostProjectionEncoder, EguiTextCommandSurfaceHostRoot,
    EguiTextCommandSurfaceHostTargetToken, EguiTextCommandSurfacePresentationToken,
    EguiTextCommandSurfaceRootEventTransport, EguiTextCommandSurfaceRootFactory,
    EguiTextCommandSurfaceRootFactoryError, EguiTextCommandSurfaceSearchPresentation,
    KucOpaqueHostEffectBatch, KucRootEventBatchContext, KucRootEventBatchForwarder,
    TabStripCorrelation, TabStripGroupCapabilities, TabStripGroupDescriptor,
    TabStripGroupPopupPresentation, TabStripGroupTarget, TabStripMenuEntry, TabStripMenuOperation,
    TabStripProjection, TabStripProjectionLease, TabStripTabCapabilities, TabStripTabDescriptor,
    TabStripTabTarget, TabStripText,
};
use katana_ui_core::egui::text_command_surface::{
    EguiTextCommandSurfacePresentation, TextCommandSurfaceStyle,
};
use katana_ui_core::molecule::command_chrome::{
    CommandChromeAction, CommandChromeFamilyId, CommandChromeSearchPresentation, CommandChromeText,
    CommandChromeToolbarPresentation, FloatingCommandToolbarVisibility, SearchControlIcons,
    SearchControlStrings, SearchResultSummaryTemplate,
};
use katana_ui_core::molecule::selection::ContextMenuItemKind;
use katana_ui_core::molecule::structured::{ReplaceMode, SearchOptions};
use katana_ui_core::render_model::UiStateId;
use katana_ui_core::render_model::UiTextSpan;
use katana_ui_core::text_surface::{
    TextSurface, TextSurfaceAutomaticGutterPresentation, TextSurfacePresentation, TextSurfaceProps,
    TextSurfaceViewport,
};

fn host_context_presentation() -> EguiTextCommandSurfacePresentation {
    let text = TextSurface::new(
        TextSurfaceProps::new(
            katana_ui_core::atom::TextArea::new("host-root-context-text")
                .value("# KLE / KUC\n\n日本語 ⭐️"),
            Vec::<UiTextSpan>::new(),
            TextSurfaceViewport::new(0, 0, 1280, 720),
        )
        .accessibility_label("KLE Storybook text surface")
        .context_target_label("KLE Storybook context target"),
    );
    let mut text = TextSurfacePresentation::from_props(text.props());
    text.selection_start = "# KLE / KUC\n\n".len();
    text.selection_end = text.selection_start + "日本語 ⭐️".len();
    EguiTextCommandSurfacePresentation {
        text_state_id: Some(UiStateId::new("host-root-context-text")),
        text,
        toolbar: Some(CommandChromeToolbarPresentation {
            actions: vec![CommandChromeAction::new(
                "generic-command",
                "Generic command",
            )],
            groups: Vec::new(),
            display_mode: Default::default(),
            density: Default::default(),
            overflow_strategy: Default::default(),
        }),
        floating: Some(EguiTextCommandSurfaceFloatingPresentation {
            toolbar: CommandChromeToolbarPresentation {
                actions: vec![CommandChromeAction::new(
                    "generic-selection",
                    "Generic selection",
                )],
                groups: Vec::new(),
                display_mode: Default::default(),
                density: Default::default(),
                overflow_strategy: Default::default(),
            },
            visibility: FloatingCommandToolbarVisibility::Visible,
        }),
        search: Some(EguiTextCommandSurfaceSearchPresentation {
            state_id: UiStateId::new("generic-search"),
            label: String::from("Generic search"),
            value: CommandChromeSearchPresentation {
                query: String::from("query"),
                options: SearchOptions::default(),
                result_count: Some(0),
                active_index: None,
                replace_mode: ReplaceMode::Hidden,
                replace_value: String::from("replacement"),
                strings: generic_search_strings(),
                capabilities: Default::default(),
                icons: SearchControlIcons::default(),
            },
        }),
        context_menu: Some(ContextMenuPresentation {
            visible: true,
            items: vec![
                ContextMenuPresentationItem::action("context.save", "保存 ⭐️"),
                ContextMenuPresentationItem {
                    id: String::from("context.authoring"),
                    label: String::from("Markdown editing ⭐️"),
                    accessibility_label: String::new(),
                    icon: None,
                    enabled: true,
                    checked: false,
                    kind: ContextMenuItemKind::Submenu,
                    children: vec![
                        ContextMenuPresentationItem::action("context.author.bold", "Bold ⭐️"),
                        ContextMenuPresentationItem::action(
                            "context.author.code-block",
                            "Code block ⭐️",
                        )
                        .child(ContextMenuPresentationItem::action(
                            "context.author.code-block.markdown",
                            "Markdown ⭐️",
                        )),
                    ],
                },
                ContextMenuPresentationItem {
                    id: String::from("context.ingest"),
                    label: String::from("Image ingest ⭐️"),
                    accessibility_label: String::new(),
                    icon: None,
                    enabled: true,
                    checked: false,
                    kind: ContextMenuItemKind::Submenu,
                    children: vec![ContextMenuPresentationItem::action(
                        "context.ingest.image-file",
                        "Image file ⭐️",
                    )],
                },
            ],
        }),
    }
}

fn generic_search_strings() -> SearchControlStrings {
    let text = |value: &str| CommandChromeText::new(value, value, value);
    SearchControlStrings {
        strip: text("Generic search"),
        query: text("Query"),
        replace: text("Replace"),
        match_case: text("Match case"),
        whole_word: text("Whole word"),
        use_regex: text("Regex"),
        previous: text("Previous"),
        next: text("Next"),
        replace_one: text("Replace one"),
        replace_all: text("Replace all"),
        close: text("Close"),
        result_summary: SearchResultSummaryTemplate {
            empty: String::new(),
            zero_results: String::from("0"),
            single_result: String::from("1 / 1"),
            indexed_result: String::from("{active} / {count}"),
            count_results: String::from("{count}"),
        },
    }
}

fn host_context_lease(
    revision: u64,
) -> katana_ui_core::egui::text_command_surface::EguiTextCommandSurfaceHostProjectionLease {
    let token = EguiTextCommandSurfaceHostProjectionEncoder::token(
        revision,
        b"host-root-context-target".to_vec(),
        host_context_presentation(),
        TextCommandSurfaceStyle::standard().expect("standard style"),
    )
    .expect("host context token");
    katana_ui_core::egui::text_command_surface::EguiTextCommandSurfaceHostProjectionLease::new(
        token,
        |context: KucRootEventBatchContext| {
            if context.context_menu_events().is_empty() {
                Ok(None)
            } else {
                Ok(Some(KucOpaqueHostEffectBatch::from_handler(|| Ok(()))))
            }
        },
    )
}

fn complex_host_context_presentation() -> EguiTextCommandSurfacePresentation {
    let mut presentation = host_context_presentation();
    presentation.search = None;
    presentation.toolbar.as_mut().expect("toolbar").actions = (0..12)
        .map(|index| {
            CommandChromeAction::new(format!("toolbar.{index}"), format!("Toolbar {index}"))
        })
        .collect();
    presentation
        .floating
        .as_mut()
        .expect("floating toolbar")
        .toolbar
        .actions = (0..12)
        .map(|index| {
            CommandChromeAction::new(format!("floating.{index}"), format!("Floating {index}"))
        })
        .collect();
    presentation.text.automatic_gutter = Some(TextSurfaceAutomaticGutterPresentation::new());

    let authoring_leafs = (0..14)
        .map(|index| {
            ContextMenuPresentationItem::action(
                format!("context.authoring.leaf-{index}"),
                format!("Markdown leaf {index} ⭐️"),
            )
        })
        .collect::<Vec<_>>();
    let code_kinds = [
        "text",
        "markdown",
        "bash",
        "zsh",
        "mermaid",
        "drawio",
        "plantuml",
        "json",
        "yaml",
        "toml",
        "rust",
        "typescript",
        "javascript",
        "python",
        "html",
        "css",
        "sql",
    ]
    .into_iter()
    .map(|kind| {
        ContextMenuPresentationItem::action(
            format!("context.authoring.code.{kind}"),
            format!("{kind} ⭐️"),
        )
    })
    .collect();
    let mut authoring = ContextMenuPresentationItem {
        id: String::from("context.authoring"),
        label: String::from("Markdown authoring ⭐️"),
        accessibility_label: String::new(),
        icon: None,
        enabled: true,
        checked: false,
        kind: ContextMenuItemKind::Submenu,
        children: authoring_leafs,
    };
    authoring.children.push(ContextMenuPresentationItem {
        id: String::from("context.authoring.code"),
        label: String::from("Code kind ⭐️"),
        accessibility_label: String::new(),
        icon: None,
        enabled: true,
        checked: false,
        kind: ContextMenuItemKind::Submenu,
        children: code_kinds,
    });
    presentation.context_menu = Some(ContextMenuPresentation {
        visible: true,
        items: vec![
            ContextMenuPresentationItem::action("context.save", "保存 ⭐️"),
            ContextMenuPresentationItem::action("context.format", "Format ⭐️"),
            authoring,
            ContextMenuPresentationItem {
                id: String::from("context.ingest"),
                label: String::from("Image ingest ⭐️"),
                accessibility_label: String::new(),
                icon: None,
                enabled: true,
                checked: false,
                kind: ContextMenuItemKind::Submenu,
                children: vec![ContextMenuPresentationItem {
                    id: String::from("context.ingest.image-file"),
                    label: String::from("Image file ⭐️"),
                    accessibility_label: String::new(),
                    icon: None,
                    enabled: false,
                    checked: false,
                    kind: ContextMenuItemKind::Action,
                    children: Vec::new(),
                }],
            },
        ],
    });
    presentation
}

fn complex_host_context_lease(
    revision: u64,
) -> katana_ui_core::egui::text_command_surface::EguiTextCommandSurfaceHostProjectionLease {
    let token = EguiTextCommandSurfaceHostProjectionEncoder::token(
        revision,
        b"complex-host-root-context-target".to_vec(),
        complex_host_context_presentation(),
        TextCommandSurfaceStyle::standard().expect("standard style"),
    )
    .expect("complex host context token");
    katana_ui_core::egui::text_command_surface::EguiTextCommandSurfaceHostProjectionLease::new(
        token,
        |context: KucRootEventBatchContext| {
            if context.context_menu_events().is_empty() {
                Ok(None)
            } else {
                Ok(Some(KucOpaqueHostEffectBatch::from_handler(|| Ok(()))))
            }
        },
    )
}

fn host_frame(
    context: &egui::Context,
    root: &mut EguiTextCommandSurfaceHostRoot,
    events: Vec<egui::Event>,
) -> katana_ui_core::egui::text_command_surface::EguiTextCommandSurfaceHostRootFrame {
    let mut frame = None;
    let mut full_output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 720.0),
            )),
            events,
            ..egui::RawInput::default()
        },
        |ui| frame = Some(root.show(ui).expect("host root show")),
    );
    full_output.textures_delta.clear();
    frame.expect("host root frame")
}

#[test]
fn host_root_fresh_lease_context_open_then_leaf_capture_preserves_context_state() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut root = EguiTextCommandSurfaceRootFactory::new()
        .retain_with_lease(host_context_lease(1))
        .expect("retain host root");

    root.synchronize_with_lease(host_context_lease(2))
        .expect("synchronize initial lease");
    let initial = host_frame(&context, &mut root, Vec::new());
    initial
        .forward_events_once(&mut CompileOnlyForwarder)
        .expect("forward initial frame once");
    let mut open_request = initial
        .interaction_locator()
        .request_context_open()
        .expect("initial context target");
    let mut open_input = egui::RawInput::default();
    open_request
        .apply_to_raw_input_once(&mut open_input)
        .expect("queue context opener");

    root.synchronize_with_lease(host_context_lease(3))
        .expect("synchronize opener lease");
    let opened = host_frame(&context, &mut root, open_input.events);
    opened
        .forward_events_once(&mut CompileOnlyForwarder)
        .expect("forward opener frame once");
    let mut leaf_request = opened
        .interaction_locator()
        .request(katana_ui_core::egui::text_command_surface::KucInteractionSelector::new(
            "context.save",
            katana_ui_core::egui::text_command_surface::KucInteractionActionClass::ContextMenuItem,
        ))
        .expect("context.save on opener frame");

    root.synchronize_with_lease(host_context_lease(4))
        .expect("synchronize leaf lease");
    let mut leaf_input = egui::RawInput::default();
    leaf_request
        .apply_to_raw_input_once(&mut leaf_input)
        .expect("queue context leaf");
    let captured = host_frame(&context, &mut root, leaf_input.events);
    captured
        .forward_events_once(&mut CompileOnlyForwarder)
        .expect("forward leaf frame once");
}

#[test]
fn host_root_complex_kle_context_fixture_preserves_current_frame_context_leaf() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut root = EguiTextCommandSurfaceRootFactory::new()
        .retain_with_lease(complex_host_context_lease(1))
        .expect("retain complex host root");

    root.synchronize_with_lease(complex_host_context_lease(2))
        .expect("synchronize initial complex lease");
    let initial = host_frame(&context, &mut root, Vec::new());
    let mut open_request = initial
        .interaction_locator()
        .request_context_open()
        .expect("complex context target");
    let mut open_input = egui::RawInput::default();
    open_request
        .apply_to_raw_input_once(&mut open_input)
        .expect("queue complex context opener");

    root.synchronize_with_lease(complex_host_context_lease(3))
        .expect("synchronize complex opener lease");
    let opened = host_frame(&context, &mut root, open_input.events);
    assert!(opened
        .interaction_locator()
        .request(katana_ui_core::egui::text_command_surface::KucInteractionSelector::new(
            "context.save",
            katana_ui_core::egui::text_command_surface::KucInteractionActionClass::ContextMenuItem,
        ))
        .is_ok());
}

fn compile_only_root_factory_signature(
    factory: &EguiTextCommandSurfaceRootFactory,
    token: EguiTextCommandSurfacePresentationToken,
) -> Result<EguiTextCommandSurfaceHostRoot, EguiTextCommandSurfaceRootFactoryError> {
    factory.retain(token)
}

struct CompileOnlyForwarder;

impl KucRootEventBatchForwarder for CompileOnlyForwarder {
    type Error = ();

    fn forward_root_event_batch(
        &mut self,
        _transport: EguiTextCommandSurfaceRootEventTransport,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[test]
fn public_facade_compiles_with_opaque_tokens_and_one_shot_transport() {
    let _factory = EguiTextCommandSurfaceRootFactory::new();
    let _encoder = EguiTextCommandSurfaceHostProjectionEncoder::new();
    let _target = EguiTextCommandSurfaceHostTargetToken::from_opaque_bytes(b"host-target");
    let _forwarder = CompileOnlyForwarder;
    let _factory_signature = compile_only_root_factory_signature;
}

#[test]
fn opaque_tokens_and_transport_have_no_clone_or_serialize_derives() {
    let source = include_str!("../src/egui/text_command_surface/host_root/types.rs");
    for type_name in [
        "EguiTextCommandSurfaceHostTargetToken",
        "EguiTextCommandSurfacePresentationToken",
    ] {
        let marker = format!("pub struct {type_name}");
        let (prefix, declaration) = source
            .split_once(&marker)
            .expect("opaque token declaration was not found");
        let attributes = prefix
            .rsplit_once("\n\n")
            .map_or(prefix, |(_, value)| value);
        let declaration = declaration
            .split_once("\n}")
            .map_or(declaration, |(value, _)| value);
        assert!(!attributes.contains("Clone"), "{type_name} became Clone");
        assert!(
            !attributes.contains("Serialize") && !declaration.contains("Serialize"),
            "{type_name} became Serialize"
        );
    }

    let event_source = include_str!("../src/egui/text_command_surface/root_event.rs");
    let (transport_prefix, transport) = event_source
        .split_once("pub struct EguiTextCommandSurfaceRootEventTransport")
        .expect("opaque event transport declaration was not found");
    let transport_attributes = transport_prefix
        .rsplit_once("\n\n")
        .map_or(transport_prefix, |(_, value)| value);
    let transport = transport
        .split_once("\n}")
        .map_or(transport, |(value, _)| value);
    assert!(!transport_attributes.contains("Clone"));
    assert!(!transport_attributes.contains("Serialize") && !transport.contains("Serialize"));
}

#[test]
fn public_facade_signatures_reject_child_and_presentation_concrete_types() {
    let factory_source = include_str!("../src/egui/text_command_surface/host_root/factory_api.rs");
    let facade_source = include_str!("../src/egui/text_command_surface/host_root_facade.rs");
    let types_source = include_str!("../src/egui/text_command_surface/host_root/types.rs");
    let public_sections = [
        factory_source
            .split_once("impl EguiTextCommandSurfaceRootFactory")
            .and_then(|(_, value)| {
                value.split_once("impl Default for EguiTextCommandSurfaceRootFactory")
            })
            .map(|(value, _)| value),
        facade_source
            .split_once("impl EguiTextCommandSurfaceHostRoot")
            .and_then(|(_, value)| value.split_once("impl EguiTextCommandSurfaceHostRootFrame"))
            .map(|(value, _)| value),
        facade_source
            .split_once("impl EguiTextCommandSurfaceHostRootFrame")
            .and_then(|(_, value)| value.split_once("impl std::fmt::Debug"))
            .map(|(value, _)| value),
        types_source
            .split_once("/// Errors raised")
            .and_then(|(_, value)| value.split_once("#[derive(Deserialize, Serialize)]"))
            .map(|(value, _)| value),
    ];
    let forbidden = [
        "EguiTextCommandSurfacePresentation,",
        "EguiTextCommandSurfacePresentation>",
        "EguiTextCommandSurfacePresentation)",
        "EguiTextCommandSurfaceFloatingPresentation,",
        "EguiTextCommandSurfaceFloatingPresentation>",
        "EguiTextCommandSurfaceSearchPresentation,",
        "EguiTextCommandSurfaceSearchPresentation>",
        "TextCommandSurfaceStyle",
        "TextSurfaceEvent",
        "CommandChromeToolbarEvent",
        "FloatingCommandToolbarEvent",
        "CommandChromeSearchEvent",
        "ContextMenuEvent",
        "PaintPlan",
        "TextureId",
        "egui::Id",
        "PlatformTextRasterConfig",
        "EguiTextCommandSurfaceRootError",
    ];
    for section in public_sections {
        let section = section.expect("facade public section was not found");
        for forbidden_name in forbidden {
            assert!(
                !section.contains(forbidden_name),
                "public facade section leaked `{forbidden_name}`"
            );
        }
    }
    assert!(!factory_source.contains("pub fn with_text_raster_config"));
    assert!(!facade_source.contains("pub fn with_text_raster_config"));
}

#[test]
fn compatibility_types_are_hidden() {
    let module_source = include_str!("../src/egui/text_command_surface.rs");
    assert!(module_source.contains("#[doc(hidden)]\npub use root"));
    assert!(module_source.contains("#[doc(hidden)]\npub use types"));
    assert!(module_source.contains("EguiTextCommandSurfaceHostProjectionEncoder"));
    assert!(!module_source.contains("EguiTextCommandSurfaceRootStorybookBuilder"));
}

#[test]
fn host_projection_boundary_keeps_target_bytes_and_rgba_private() {
    let source = include_str!("../src/egui/text_command_surface/host_root/types.rs");
    assert!(source.contains("pub struct EguiTextCommandSurfaceHostProjectionEncoder"));
    assert!(!source.contains("String::from_utf8"));
    assert!(
        !include_str!("../src/egui/text_command_surface/host_root_facade.rs")
            .contains("pub fn rgba_pixels")
    );
    assert!(
        !include_str!("../src/egui/text_command_surface/host_root_token_codec.rs")
            .contains("target.payload.to_vec()")
    );
}

#[test]
fn family_projection_round_trips_and_distinct_families_render() {
    let token = EguiTextCommandSurfaceHostProjectionEncoder::token_with_command_families(
        1,
        b"family-target".to_vec(),
        host_context_presentation(),
        TextCommandSurfaceStyle::standard().expect("standard style"),
        EguiTextCommandSurfaceCommandFamilyProjection::new(
            Some(CommandChromeFamilyId::new("primary-family")),
            Some(CommandChromeFamilyId::new("floating-family")),
        ),
    )
    .expect("family token");
    let context = egui::Context::default();
    let mut root = EguiTextCommandSurfaceRootFactory::new()
        .retain(token)
        .expect("distinct family root");
    let frame = host_frame(&context, &mut root, Vec::new());
    let record = frame.record();
    assert!(!record.identity().is_empty());
    assert_eq!(record.presentation_revision(), 1);
    assert_eq!(record.state_revision(), 0);
    assert!(record.dimensions().width() > 0);
    assert!(record.dimensions().height() > 0);
    assert!(!record.rgba_hash().is_empty());
    assert!(!record.paint_plan_hash().is_empty());
    assert!(!record.record_hash().is_empty());
    assert!(!record.accessibility_snapshot_hash().is_empty());
    assert!(format!("{frame:?}").contains("EguiTextCommandSurfaceHostRootFrame"));

    let legacy = EguiTextCommandSurfaceHostProjectionEncoder::token(
        2,
        b"family-target".to_vec(),
        host_context_presentation(),
        TextCommandSurfaceStyle::standard().expect("standard style"),
    )
    .expect("legacy synchronization token");
    let _ = root
        .synchronize(legacy)
        .expect("legacy synchronization must preserve compatibility");
}

#[test]
fn host_lease_tab_strip_reaches_the_public_root_composite() {
    let token = EguiTextCommandSurfaceHostProjectionEncoder::token(
        1,
        b"host-tab-composite-target",
        host_context_presentation(),
        TextCommandSurfaceStyle::standard().expect("standard style"),
    )
    .expect("host token");
    let projection = TabStripProjection::new(
        1,
        TabStripCorrelation::from_opaque_bytes(b"host-tab-correlation"),
    )
    .tab(
        TabStripTabDescriptor::new(
            TabStripTabTarget::from_opaque_bytes(b"host-tab"),
            TabStripText::new("日本語 ⭐️"),
        )
        .capabilities(TabStripTabCapabilities::new().active(true).selectable(true)),
    );
    let lease =
        katana_ui_core::egui::text_command_surface::EguiTextCommandSurfaceHostProjectionLease::new(
            token,
            |_context| Ok(None),
        )
        .with_tab_strip(TabStripProjectionLease::new(projection));
    let mut root = EguiTextCommandSurfaceRootFactory::new()
        .retain_with_lease(lease)
        .expect("tab-strip lease retain");
    let context = egui::Context::default();

    let frame = host_frame(&context, &mut root, Vec::new());

    assert!(!frame.record().rgba_hash().is_empty());
    assert!(!frame.record().paint_plan_hash().is_empty());
}

#[test]
fn host_tab_strip_group_rename_retains_focus_through_pointer_and_egui_blur() {
    let token = EguiTextCommandSurfaceHostProjectionEncoder::token(
        1,
        b"host-tab-rename-target",
        host_context_presentation(),
        TextCommandSurfaceStyle::standard().expect("standard style"),
    )
    .expect("host token");
    let projection = TabStripProjection::new(
        1,
        TabStripCorrelation::from_opaque_bytes(b"host-tab-rename-correlation"),
    )
    .group(
        TabStripGroupDescriptor::new(
            TabStripGroupTarget::from_opaque_bytes(b"host-tab-rename-group"),
            TabStripText::new("Group"),
        )
        .capabilities(
            TabStripGroupCapabilities::new()
                .menu_available(true)
                .renamable(true),
        )
        .popup(
            TabStripGroupPopupPresentation::new()
                .rename_placeholder(TabStripText::new("Group name"))
                .entry(TabStripMenuEntry::action(
                    TabStripText::new("Ungroup"),
                    TabStripText::new("Ungroup tabs"),
                    TabStripMenuOperation::Ungroup,
                )),
        ),
    );
    let lease =
        katana_ui_core::egui::text_command_surface::EguiTextCommandSurfaceHostProjectionLease::new(
            token,
            |_context| Ok(None),
        )
        .with_tab_strip(TabStripProjectionLease::new(projection));
    let mut root = EguiTextCommandSurfaceRootFactory::new()
        .retain_with_lease(lease)
        .expect("tab-strip lease retain");
    let context = egui::Context::default();

    let closed = host_frame(&context, &mut root, Vec::new());
    let closed_hash = closed.record().paint_plan_hash().to_owned();
    let opened = host_frame(
        &context,
        &mut root,
        vec![
            egui::Event::PointerMoved(egui::pos2(20.0, 18.0)),
            egui::Event::PointerButton {
                pos: egui::pos2(20.0, 18.0),
                button: egui::PointerButton::Secondary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Event::PointerButton {
                pos: egui::pos2(20.0, 18.0),
                button: egui::PointerButton::Secondary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    assert_ne!(opened.record().paint_plan_hash(), closed_hash);

    let pressed = host_frame(
        &context,
        &mut root,
        vec![
            egui::Event::PointerMoved(egui::pos2(20.0, 50.0)),
            egui::Event::PointerButton {
                pos: egui::pos2(20.0, 50.0),
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Event::PointerButton {
                pos: egui::pos2(20.0, 50.0),
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    assert!(!pressed.record().paint_plan_hash().is_empty());

    let mut reclaimed = None;
    let mut full_output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 720.0),
            )),
            ..egui::RawInput::default()
        },
        |ui| {
            ui.memory_mut(|memory| memory.request_focus(egui::Id::new("external-focus-owner")));
            reclaimed = Some(root.show(ui).expect("host root show after focus transfer"));
        },
    );
    full_output.textures_delta.clear();
    assert!(
        !reclaimed
            .expect("reclaimed host root frame")
            .record()
            .paint_plan_hash()
            .is_empty()
    );
}

#[test]
fn versioned_same_family_token_fails_closed_at_root_render() {
    let family = CommandChromeFamilyId::new("same-family");
    let token = EguiTextCommandSurfaceHostProjectionEncoder::token_with_command_families(
        1,
        b"duplicate-family-target".to_vec(),
        host_context_presentation(),
        TextCommandSurfaceStyle::standard().expect("standard style"),
        EguiTextCommandSurfaceCommandFamilyProjection::new(Some(family.clone()), Some(family)),
    )
    .expect("duplicate family token");
    let context = egui::Context::default();
    let mut root = EguiTextCommandSurfaceRootFactory::new()
        .retain(token)
        .expect("token decode");
    let mut result = None;
    let mut full_output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 720.0),
            )),
            ..egui::RawInput::default()
        },
        |ui| result = Some(root.show(ui)),
    );
    full_output.textures_delta.clear();
    assert!(matches!(
        result.expect("root frame"),
        Err(EguiTextCommandSurfaceRootFactoryError::Root(error))
            if error.contains("command family is mounted")
    ));
}

#[test]
fn family_token_debug_does_not_expose_semantic_payload() {
    let token = EguiTextCommandSurfaceHostProjectionEncoder::token_with_command_families(
        1,
        b"opaque-target".to_vec(),
        host_context_presentation(),
        TextCommandSurfaceStyle::standard().expect("standard style"),
        EguiTextCommandSurfaceCommandFamilyProjection::new(
            Some(CommandChromeFamilyId::new("primary-family")),
            Some(CommandChromeFamilyId::new("floating-family")),
        ),
    )
    .expect("family token");
    let debug = format!("{token:?}");
    assert!(!debug.contains("primary-family"));
    assert!(!debug.contains("floating-family"));
    assert!(debug.contains("payload: \"<opaque>\""));
}

#[test]
fn unknown_family_wire_version_fails_closed() {
    let token = EguiTextCommandSurfacePresentationToken::from_opaque_bytes(
        1,
        EguiTextCommandSurfaceHostTargetToken::from_opaque_bytes(b"target".to_vec()),
        br#"{"version":99}"#.to_vec(),
    );
    let result = EguiTextCommandSurfaceRootFactory::new().retain(token);
    assert!(matches!(
        result,
        Err(EguiTextCommandSurfaceRootFactoryError::Decode(_))
            | Err(EguiTextCommandSurfaceRootFactoryError::InvalidToken(_))
    ));
}
