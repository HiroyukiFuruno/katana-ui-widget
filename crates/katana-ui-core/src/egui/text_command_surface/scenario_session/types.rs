use super::super::scenario::FullTextCommandSurfaceScenarioId;
use crate::molecule::structured::{ReplaceMode, SearchOptionKind, SearchOptions};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// Retains generic Storybook input without exposing its presentation to consumers.
///
/// This session is deliberately limited to KUC-owned visible input state. It never
/// resolves document, Markdown, search-result, replacement, file, or undo effects.
pub struct FullTextCommandSurfaceScenarioSession {
    pub(super) id: FullTextCommandSurfaceScenarioId,
    pub(super) state: Rc<RefCell<ScenarioSessionState>>,
    pub(super) next_revision: Cell<u64>,
    pub(super) consumer_artifact: bool,
}

#[derive(Default)]
pub(super) struct ScenarioSessionState {
    pub(super) text: Option<String>,
    pub(super) selection: Option<(usize, usize)>,
    pub(super) search_visible: Option<bool>,
    pub(super) search_query: Option<String>,
    pub(super) search_options: Option<SearchOptions>,
    pub(super) replace_mode: Option<ReplaceMode>,
    pub(super) replace_value: Option<String>,
    pub(super) result_position: Option<(usize, Option<usize>)>,
}

#[derive(Default)]
pub(super) struct ScenarioSessionUpdate {
    pub(super) text: Option<String>,
    pub(super) selection: Option<(usize, usize)>,
    pub(super) search_visible: Option<bool>,
    pub(super) search_query: Option<String>,
    pub(super) search_option_changes: Vec<(SearchOptionKind, bool)>,
    pub(super) replace_mode: Option<ReplaceMode>,
    pub(super) replace_value: Option<String>,
    pub(super) result_position: Option<(usize, Option<usize>)>,
}
