fn stages(id: FullTextCommandSurfaceScenarioId) -> Vec<FullTextCommandSurfaceRawInputStage> {
    let mut stages = vec![stage(Vec::new(), 1.0)];
    match id {
        FullTextCommandSurfaceScenarioId::Context => {
            stages.push(stage(
                vec![
                    egui::Event::PointerMoved(egui::pos2(260.0, 140.0)),
                    egui::Event::PointerButton {
                        pos: egui::pos2(260.0, 140.0),
                        button: egui::PointerButton::Secondary,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
                1.0,
            ));
            stages.push(stage(vec![key_press(egui::Key::Escape)], 1.0));
            /* WHY: Render one retained frame after dismissal so focus restoration is observable. */
            stages.push(stage(Vec::new(), 1.0));
        }
        FullTextCommandSurfaceScenarioId::ResizeScrollIme => {
            stages.push(stage(
                vec![
                    egui::Event::PointerMoved(egui::pos2(260.0, 140.0)),
                    primary_pointer(egui::pos2(260.0, 140.0), true),
                ],
                1.0,
            ));
            stages.push(stage(
                vec![primary_pointer(egui::pos2(260.0, 140.0), false)],
                1.0,
            ));
            stages.push(stage(
                vec![egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, 96.0),
                    phase: egui::TouchPhase::Move,
                    modifiers: egui::Modifiers::NONE,
                }],
                1.0,
            ));
            stages.push(stage_with_screen(
                Vec::new(),
                1.0,
                egui::vec2(RESIZED_WIDTH, RESIZED_HEIGHT),
            ));
            stages.push(stage(
                vec![egui::Event::Ime(egui::ImeEvent::Preedit {
                    text: String::from("かな"),
                    active_range_chars: None,
                })],
                IME_PIXELS_PER_POINT,
            ));
            stages.push(stage(
                vec![egui::Event::Ime(egui::ImeEvent::Commit(String::from(
                    "入力",
                )))],
                IME_PIXELS_PER_POINT,
            ));
        }
        FullTextCommandSurfaceScenarioId::Readonly => {
            stages.push(stage(vec![egui::Event::Text(String::from("blocked"))], 1.0));
        }
        FullTextCommandSurfaceScenarioId::NavigationInput => {
            stages.push(stage(
                vec![
                    egui::Event::PointerMoved(egui::pos2(80.0, 14.0)),
                    primary_pointer(egui::pos2(80.0, 14.0), true),
                    primary_pointer(egui::pos2(80.0, 14.0), false),
                ],
                1.0,
            ));
            stages.push(stage(
                vec![egui::Event::Text(String::from(NAVIGATION_INPUT_FIXTURE))],
                1.0,
            ));
            stages.push(stage(vec![key_press(egui::Key::Enter)], 1.0));
        }
        FullTextCommandSurfaceScenarioId::WorkspaceTabs => {
            let source = egui::pos2(TAB_SOURCE_X, TAB_Y);
            let end = egui::pos2(TAB_TARGET_X, TAB_Y);
            stages.push(stage(
                vec![
                    egui::Event::PointerMoved(source),
                    primary_pointer(source, true),
                ],
                1.0,
            ));
            stages.push(stage(vec![egui::Event::PointerMoved(end)], 1.0));
            stages.push(stage(
                vec![egui::Event::PointerMoved(end), primary_pointer(end, false)],
                1.0,
            ));
        }
        _ => {}
    }
    stages
}

fn consumer_artifact_stages() -> Vec<FullTextCommandSurfaceRawInputStage> {
    vec![
        stage(vec![egui::Event::Text(String::from("consumer artifact input"))], 1.0),
        stage(
            vec![egui::Event::Ime(egui::ImeEvent::Commit(String::from(
                "入力",
            )))],
            IME_PIXELS_PER_POINT,
        ),
        stage(
            vec![
                egui::Event::PointerMoved(egui::pos2(260.0, 140.0)),
                primary_pointer(egui::pos2(260.0, 140.0), true),
                primary_pointer(egui::pos2(300.0, 140.0), false),
            ],
            1.0,
        ),
        stage(
            vec![egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, 96.0),
                phase: egui::TouchPhase::Move,
                modifiers: egui::Modifiers::NONE,
            }],
            1.0,
        ),
        stage(
            vec![
                egui::Event::PointerMoved(egui::pos2(32.0, 24.0)),
                primary_pointer(egui::pos2(32.0, 24.0), true),
                primary_pointer(egui::pos2(32.0, 24.0), false),
            ],
            1.0,
        ),
        stage(
            vec![
                egui::Event::PointerMoved(egui::pos2(260.0, 96.0)),
                primary_pointer(egui::pos2(260.0, 96.0), true),
                primary_pointer(egui::pos2(260.0, 96.0), false),
            ],
            1.0,
        ),
        stage(vec![key_press(egui::Key::Tab)], 1.0),
        stage(
            vec![
                egui::Event::PointerMoved(egui::pos2(260.0, 140.0)),
                egui::Event::PointerButton {
                    pos: egui::pos2(260.0, 140.0),
                    button: egui::PointerButton::Secondary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            1.0,
        ),
        stage(vec![key_press(egui::Key::Enter)], 1.0),
        stage_with_screen(
            Vec::new(),
            1.0,
            egui::vec2(RESIZED_WIDTH, RESIZED_HEIGHT),
        ),
    ]
}
