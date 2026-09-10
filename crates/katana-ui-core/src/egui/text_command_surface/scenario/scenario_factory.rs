use super::{
    FullTextCommandSurfaceScenario, FullTextCommandSurfaceScenarioError,
    FullTextCommandSurfaceScenarioId, NoopRouter, consumer_artifact_presentation, issue_lease,
    consumer_artifact_stages, presentation, stages,
};

/// Issues generic full-surface scenarios without exposing fixture geometry or semantics.
pub struct FullTextCommandSurfaceScenarioFactory;

impl FullTextCommandSurfaceScenarioFactory {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Creates a deterministic opaque scenario from a stable ID.
    pub fn issue(
        &self,
        id: FullTextCommandSurfaceScenarioId,
    ) -> Result<FullTextCommandSurfaceScenario, FullTextCommandSurfaceScenarioError> {
        self.issue_with_router(id, NoopRouter)
    }

    /// Creates the additive consumer-artifact scenario without extending the
    /// stable, exhaustively-matchable scenario ID enum.
    pub fn issue_consumer_artifact(
        &self,
    ) -> Result<FullTextCommandSurfaceScenario, FullTextCommandSurfaceScenarioError> {
        let lease = issue_lease(FullTextCommandSurfaceScenarioId::Resting, consumer_artifact_presentation(), NoopRouter)?;
        Ok(FullTextCommandSurfaceScenario {
            id: FullTextCommandSurfaceScenarioId::Resting,
            lease: Some(lease),
            stages: consumer_artifact_stages(),
        })
    }

    /// Creates a deterministic scenario and retains a caller-owned generic router opaquely.
    ///
    /// The router receives only the closed root event-batch context at render time. The
    /// scenario fixture, presentation, and encoded token remain private to KUC.
    pub fn issue_with_router<R>(
        &self,
        id: FullTextCommandSurfaceScenarioId,
        router: R,
    ) -> Result<FullTextCommandSurfaceScenario, FullTextCommandSurfaceScenarioError>
    where
        R: crate::egui::text_command_surface::KucRootEffectRouter + 'static,
    {
        let lease = issue_lease(id, presentation(id), router)?;
        Ok(FullTextCommandSurfaceScenario {
            id,
            lease: Some(lease),
            stages: stages(id),
        })
    }
}

impl Default for FullTextCommandSurfaceScenarioFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_issues_every_public_scenario_with_nonempty_stages() {
        let default_factory = FullTextCommandSurfaceScenarioFactory::default;
        let factory = default_factory();
        for id in [
            FullTextCommandSurfaceScenarioId::Resting,
            FullTextCommandSurfaceScenarioId::Selection,
            FullTextCommandSurfaceScenarioId::Find,
            FullTextCommandSurfaceScenarioId::Context,
            FullTextCommandSurfaceScenarioId::Readonly,
            FullTextCommandSurfaceScenarioId::ResizeScrollIme,
            FullTextCommandSurfaceScenarioId::NavigationInput,
            FullTextCommandSurfaceScenarioId::WorkspaceTabs,
        ] {
            let scenario = factory.issue(id).expect("public scenario remains issuable");
            assert_eq!(scenario.id(), id);
            assert!(!scenario.stages().is_empty());
        }
    }

    #[test]
    fn factory_issues_the_additive_consumer_artifact_scenario() {
        let factory = FullTextCommandSurfaceScenarioFactory::new();
        let scenario = factory
            .issue_consumer_artifact()
            .expect("consumer artifact scenario remains issuable");

        assert_eq!(scenario.id(), FullTextCommandSurfaceScenarioId::Resting);
        assert_eq!(scenario.stages().len(), 10);
        assert!(scenario.stages().iter().any(|stage| stage.event_count() > 0));
    }
}
