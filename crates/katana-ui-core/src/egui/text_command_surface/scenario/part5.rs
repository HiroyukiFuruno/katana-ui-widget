pub(super) fn presentation(
    id: FullTextCommandSurfaceScenarioId,
) -> EguiTextCommandSurfacePresentation {
    presentation_for(id, false)
}

pub(super) fn consumer_artifact_presentation() -> EguiTextCommandSurfacePresentation {
    presentation_for(FullTextCommandSurfaceScenarioId::Resting, true)
}

fn presentation_for(
    id: FullTextCommandSurfaceScenarioId,
    consumer_artifact: bool,
) -> EguiTextCommandSurfacePresentation {
    let mut text = TextSurfacePresentation::from_props(
        TextSurface::new(
            TextSurfaceProps::new(
                TextArea::new("kuc-scenario-text").value(FIXTURE_TEXT),
                Vec::new(),
                TextSurfaceViewport::new(0, 0, WIDTH as u32, HEIGHT as u32),
            )
            .accessibility_label("Generic text surface"),
        )
        .props(),
    );
    text.automatic_gutter = Some(TextSurfaceAutomaticGutterPresentation::new());
    if consumer_artifact
        || matches!(
            id,
            FullTextCommandSurfaceScenarioId::Find
                | FullTextCommandSurfaceScenarioId::WorkspaceTabs
        )
    {
        text.annotations = generic_find_annotations(&text.value);
    }
    text.readonly = matches!(id, FullTextCommandSurfaceScenarioId::Readonly);
    let readonly = text.readonly;
    EguiTextCommandSurfacePresentation {
        text_state_id: Some(UiStateId::new("kuc-scenario-text")),
        text,
        toolbar: Some(toolbar(readonly)),
        floating: (consumer_artifact
            || matches!(id, FullTextCommandSurfaceScenarioId::Selection))
        .then(|| {
            EguiTextCommandSurfaceFloatingPresentation {
                toolbar: toolbar(false),
                visibility: FloatingCommandToolbarVisibility::Visible,
            }
        }),
        search: (consumer_artifact
            || matches!(
                id,
                FullTextCommandSurfaceScenarioId::Find
                    | FullTextCommandSurfaceScenarioId::WorkspaceTabs
            ))
        .then(search),
        context_menu: (consumer_artifact
            || matches!(id, FullTextCommandSurfaceScenarioId::Context))
        .then(|| context_menu(true)),
    }
}

fn scenario_style() -> Result<TextCommandSurfaceStyle, FullTextCommandSurfaceScenarioError> {
    let mut style = TextCommandSurfaceStyle::standard()
        .map_err(|_| FullTextCommandSurfaceScenarioError::InvalidProjection)?;
    style.text_paint.annotation_paints = vec![
        TextSurfaceAnnotationPaint::new(GENERIC_SEARCH_MATCH_ROLE, style.text_paint.selection_rgba),
        TextSurfaceAnnotationPaint::new(GENERIC_SEARCH_CURRENT_ROLE, style.text_paint.preedit_rgba),
    ];
    Ok(style)
}

fn generic_find_annotations(value: &str) -> Vec<TextSurfaceAnnotation> {
    value
        .match_indices(FIND_FIXTURE_QUERY)
        .enumerate()
        .map(|(index, (start, _))| {
            let range = UiTextSelectionRange::new(start, start + FIND_FIXTURE_QUERY.len());
            if index == 0 {
                TextSurfaceAnnotation::new(
                    format!("kuc.fixture.find.current-{index}"),
                    range,
                    GENERIC_SEARCH_CURRENT_ROLE,
                    TextSurfaceAnnotationStyle::Fill,
                )
                .priority(1)
            } else {
                TextSurfaceAnnotation::new(
                    format!("kuc.fixture.find.match-{index}"),
                    range,
                    GENERIC_SEARCH_MATCH_ROLE,
                    TextSurfaceAnnotationStyle::Outline,
                )
            }
        })
        .collect()
}

const RICH_AUTHORING_AFFORDANCES: [(CommandChromeIcon, &str, &str); 12] = [
    (
        CommandChromeIcon::EmphasisStrong,
        "Inline emphasis",
        "inline-strong",
    ),
    (
        CommandChromeIcon::EmphasisItalic,
        "Inline slant",
        "inline-italic",
    ),
    (
        CommandChromeIcon::Strike,
        "Inline crossing",
        "inline-strike",
    ),
    (CommandChromeIcon::InlineCode, "Inline code", "inline-code"),
    (
        CommandChromeIcon::HeadingOne,
        "Heading level one",
        "heading-one",
    ),
    (
        CommandChromeIcon::HeadingTwo,
        "Heading level two",
        "heading-two",
    ),
    (
        CommandChromeIcon::HeadingThree,
        "Heading level three",
        "heading-three",
    ),
    (
        CommandChromeIcon::ListUnordered,
        "Unordered list",
        "list-unordered",
    ),
    (
        CommandChromeIcon::ListOrdered,
        "Ordered list",
        "list-ordered",
    ),
    (CommandChromeIcon::Quote, "Block quotation", "blockquote"),
    (CommandChromeIcon::CodeBlock, "Block code", "block-code"),
    (CommandChromeIcon::Image, "Media image", "media-image"),
];

fn toolbar(disabled: bool) -> CommandChromeToolbarPresentation {
    CommandChromeToolbarPresentation {
        actions: RICH_AUTHORING_AFFORDANCES
            .into_iter()
            .map(|(icon, accessible_name, id)| {
                let action = CommandChromeAction::new(format!("kuc.rich.{id}"), accessible_name)
                    .icon(icon.icon_props())
                    .tooltip(accessible_name)
                    .accessibility_label(accessible_name)
                    .disabled(disabled);
                (id == "block-code")
                    .then(generic_language_choice_dropdown)
                    .map_or(action.clone(), |dropdown| action.dropdown(dropdown))
            })
            .collect(),
        groups: Vec::new(),
        display_mode: CommandChromeDisplayMode::IconOnly,
        density: Default::default(),
        overflow_strategy: Default::default(),
    }
}

fn generic_language_choice_dropdown() -> CommandChromeDropdown {
    GENERIC_LANGUAGE_CHOICE_LABELS.into_iter().enumerate().fold(
        CommandChromeDropdown::new(CommandChromeDropdownTrigger::Primary),
        |dropdown, (index, label)| {
            dropdown.item(CommandChromeDropdownItem::new(
                format!("kuc.generic-language-{index:02}"),
                label,
            ))
        },
    )
}

fn search() -> EguiTextCommandSurfaceSearchPresentation {
    let text = |value: &str| CommandChromeText::new(value, value, value);
    EguiTextCommandSurfaceSearchPresentation {
        state_id: UiStateId::new("kuc-scenario-search"),
        label: String::from("Search"),
        value: CommandChromeSearchPresentation {
            query: String::from(FIND_FIXTURE_QUERY),
            options: SearchOptions::default(),
            result_count: Some(generic_find_annotations(FIXTURE_TEXT).len()),
            active_index: Some(0),
            replace_mode: ReplaceMode::Disabled,
            replace_value: String::new(),
            strings: SearchControlStrings {
                strip: text("Search"),
                query: text("Query"),
                replace: text("Replace"),
                match_case: text("Match case"),
                whole_word: text("Whole word"),
                use_regex: text("Regex"),
                previous: text("Previous"),
                next: text("Next"),
                replace_one: text("Replace"),
                replace_all: text("Replace all"),
                close: text("Close"),
                result_summary: SearchResultSummaryTemplate {
                    empty: String::new(),
                    zero_results: String::from("0"),
                    single_result: String::from("1 / 1"),
                    indexed_result: String::from("{active} / {count}"),
                    count_results: String::from("{count}"),
                },
            },
            capabilities: SearchControlCapabilities {
                replace: CommandChromeCapability::unavailable("replace unavailable"),
                ..SearchControlCapabilities::default()
            },
            icons: SearchControlIcons::default(),
        },
    }
}

fn context_menu(visible: bool) -> ContextMenuPresentation {
    ContextMenuPresentation {
        visible,
        items: vec![
            ContextMenuPresentationItem::action("kuc.context.copy", "Copy"),
            ContextMenuPresentationItem::action("kuc.context.paste", "Paste"),
            ContextMenuPresentationItem {
                id: String::from("kuc.context.more"),
                label: String::from("More"),
                accessibility_label: String::new(),
                icon: None,
                enabled: true,
                checked: false,
                kind: ContextMenuItemKind::Submenu,
                children: vec![ContextMenuPresentationItem::action(
                    "kuc.context.more.generic",
                    "Generic action",
                )],
            },
        ],
    }
}
