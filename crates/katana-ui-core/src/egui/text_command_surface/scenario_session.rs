//! KUC-owned retained interaction state for a generic full-surface scenario.

use super::scenario::{
    self, FullTextCommandSurfaceScenarioError, FullTextCommandSurfaceScenarioId,
    consumer_artifact_presentation,
};
use super::{
    EguiTextCommandSurfaceHostProjectionLease, KucOpaqueHostEffectBatch, KucRootEventBatchContext,
};
use crate::atom::TextAreaEvent;
use crate::molecule::command_chrome::CommandChromeSearchEvent;
use crate::molecule::structured::{SearchControlStripEvent, SearchOptionKind, SearchOptions};
use crate::text_surface::TextSurfaceEvent;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

mod types;

pub use self::types::FullTextCommandSurfaceScenarioSession;
use self::types::{ScenarioSessionState, ScenarioSessionUpdate};

impl FullTextCommandSurfaceScenarioSession {
    #[must_use]
    pub fn new(id: FullTextCommandSurfaceScenarioId) -> Self {
        Self {
            id,
            state: Rc::new(RefCell::new(ScenarioSessionState::default())),
            next_revision: Cell::new(1),
            consumer_artifact: false,
        }
    }

    /// Creates the additive consumer-artifact session without extending the
    /// stable scenario ID enum.
    #[must_use]
    pub fn new_consumer_artifact() -> Self {
        Self {
            id: FullTextCommandSurfaceScenarioId::Resting,
            state: Rc::new(RefCell::new(ScenarioSessionState::default())),
            next_revision: Cell::new(1),
            consumer_artifact: true,
        }
    }

    /// Issues the initial opaque root lease.
    pub fn retain_lease(
        &self,
    ) -> Result<EguiTextCommandSurfaceHostProjectionLease, FullTextCommandSurfaceScenarioError>
    {
        self.issue_lease()
    }

    /// Issues the current opaque root lease after accepted one-shot input dispatch.
    pub fn synchronize_lease(
        &self,
    ) -> Result<EguiTextCommandSurfaceHostProjectionLease, FullTextCommandSurfaceScenarioError>
    {
        self.issue_lease()
    }

    /// Restores a search strip that this session previously closed.
    ///
    /// The retained session owns search visibility, so consumers do not need a
    /// parallel visible flag or child-model callback to reopen it.
    pub fn reopen_search(&self) {
        self.state.borrow_mut().search_visible = Some(true);
    }

    fn issue_lease(
        &self,
    ) -> Result<EguiTextCommandSurfaceHostProjectionLease, FullTextCommandSurfaceScenarioError>
    {
        let presentation = if self.consumer_artifact {
            self.state.borrow().consumer_artifact_presentation()
        } else {
            self.state.borrow().presentation(self.id)
        };
        let projected_text = presentation.text.value.clone();
        let revision = self.next_revision()?;
        let state = Rc::clone(&self.state);
        scenario::issue_lease_at_revision(self.id, presentation, revision, move |context| {
            let update = ScenarioSessionUpdate::from_context(&context);
            if update.is_empty() {
                return Ok(None);
            }
            let state = Rc::clone(&state);
            let projected_text = projected_text.clone();
            Ok(Some(KucOpaqueHostEffectBatch::from_handler(move || {
                state.borrow_mut().apply(update, &projected_text);
                Ok(())
            })))
        })
    }

    fn next_revision(&self) -> Result<u64, FullTextCommandSurfaceScenarioError> {
        let revision = self.next_revision.get();
        let next = revision
            .checked_add(1)
            .ok_or(FullTextCommandSurfaceScenarioError::RevisionExhausted)?;
        self.next_revision.set(next);
        Ok(revision)
    }
}

impl ScenarioSessionState {
    fn presentation(
        &self,
        id: FullTextCommandSurfaceScenarioId,
    ) -> super::EguiTextCommandSurfacePresentation {
        self.apply_to_presentation(scenario::presentation(id))
    }

    fn consumer_artifact_presentation(&self) -> super::EguiTextCommandSurfacePresentation {
        self.apply_to_presentation(consumer_artifact_presentation())
    }

    fn apply_to_presentation(
        &self,
        mut presentation: super::EguiTextCommandSurfacePresentation,
    ) -> super::EguiTextCommandSurfacePresentation {
        if let Some(text) = &self.text {
            presentation.text.value.clone_from(text);
            presentation.text.annotations.clear();
        }
        if let Some((start, end)) = self.selection
            && valid_selection(&presentation.text.value, start, end)
        {
            presentation.text.selection_start = start;
            presentation.text.selection_end = end;
        }
        if self.search_visible == Some(false) {
            presentation.search = None;
        } else if let Some(search) = &mut presentation.search {
            if let Some(query) = &self.search_query {
                search.value.query.clone_from(query);
            }
            if let Some(options) = self.search_options {
                search.value.options = options;
            }
            if let Some(mode) = self.replace_mode {
                search.value.replace_mode = mode;
            }
            if let Some(value) = &self.replace_value {
                search.value.replace_value.clone_from(value);
            }
            if let Some((result_count, active_index)) = self.result_position {
                search.value.result_count = Some(result_count);
                search.value.active_index = active_index;
            }
        }
        presentation
    }

    fn apply(&mut self, update: ScenarioSessionUpdate, projected_text: &str) {
        let selection_source = update.text.as_deref().unwrap_or(projected_text);
        let accepted_selection = update
            .selection
            .filter(|selection| valid_selection(selection_source, selection.0, selection.1));
        if let Some(text) = update.text {
            self.text = Some(text);
        }
        if let Some(selection) = accepted_selection {
            self.selection = Some(selection);
        }
        if let Some(visible) = update.search_visible {
            self.search_visible = Some(visible);
        }
        if let Some(query) = update.search_query {
            self.search_query = Some(query);
        }
        if !update.search_option_changes.is_empty() {
            let options = self
                .search_options
                .get_or_insert_with(SearchOptions::default);
            for (option, enabled) in update.search_option_changes {
                set_search_option(options, option, enabled);
            }
        }
        if let Some(mode) = update.replace_mode {
            self.replace_mode = Some(mode);
        }
        if let Some(value) = update.replace_value {
            self.replace_value = Some(value);
        }
        if let Some(position) = update.result_position {
            self.result_position = Some(position);
        }
    }
}

impl ScenarioSessionUpdate {
    fn from_context(context: &KucRootEventBatchContext) -> Self {
        let mut update = Self::default();
        for event in context.text_events() {
            match event {
                TextSurfaceEvent::TextArea(TextAreaEvent::Change(value)) => {
                    update.text = Some(value.clone());
                }
                TextSurfaceEvent::SelectionChanged {
                    selection_start,
                    selection_end,
                } => {
                    update.selection = Some((*selection_start, *selection_end));
                }
                _ => {}
            }
        }
        for event in context.search_events() {
            update.apply_search_event(event);
        }
        update
    }

    fn apply_search_event(&mut self, event: &CommandChromeSearchEvent) {
        match event {
            CommandChromeSearchEvent::CloseRequested => self.search_visible = Some(false),
            CommandChromeSearchEvent::Strip { event } => match event {
                SearchControlStripEvent::SearchQueryChanged(value) => {
                    self.search_query = Some(value.clone());
                }
                SearchControlStripEvent::SearchOptionChanged { option, enabled } => {
                    self.search_option_changes.push((*option, *enabled));
                }
                SearchControlStripEvent::ReplaceModeChanged(value) => {
                    self.replace_mode = Some(*value);
                }
                SearchControlStripEvent::ReplaceValueChanged(value) => {
                    self.replace_value = Some(value.clone());
                }
                SearchControlStripEvent::SearchResultPositionChanged {
                    result_count,
                    active_index,
                } => {
                    self.result_position = Some((*result_count, *active_index));
                }
                SearchControlStripEvent::SearchNavigationRequested { .. }
                | SearchControlStripEvent::ReplaceRequested { .. } => {}
            },
        }
    }

    const fn is_empty(&self) -> bool {
        self.text.is_none()
            && self.selection.is_none()
            && self.search_visible.is_none()
            && self.search_query.is_none()
            && self.search_option_changes.is_empty()
            && self.replace_mode.is_none()
            && self.replace_value.is_none()
            && self.result_position.is_none()
    }
}

fn set_search_option(options: &mut SearchOptions, option: SearchOptionKind, enabled: bool) {
    match option {
        SearchOptionKind::MatchCase => options.match_case = enabled,
        SearchOptionKind::WholeWord => options.whole_word = enabled,
        SearchOptionKind::UseRegex => options.use_regex = enabled,
    }
}

fn valid_selection(value: &str, start: usize, end: usize) -> bool {
    start <= end
        && end <= value.len()
        && value.is_char_boundary(start)
        && value.is_char_boundary(end)
}

#[cfg(test)]
#[path = "scenario_session_tests.rs"]
mod tests;
