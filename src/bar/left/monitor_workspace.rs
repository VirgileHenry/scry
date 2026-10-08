/// Duration of the highlight change animation
const ANIM_DURATION: std::time::Duration = std::time::Duration::from_millis(400);

/// Stores for each monitor the current workspace.
/// This also holds the animation progress of each workspace highlight
pub struct MonitorWorkspace {
    workspace: hipc::types::WorkspaceId,
    highlights: [amane::Animation; super::WORKSPACE_COUNT],
}

impl MonitorWorkspace {
    pub fn new(workspace: hipc::types::WorkspaceId) -> Self {
        let index = usize::try_from(workspace.raw() - 1).unwrap_or(usize::MAX);
        Self {
            workspace,
            highlights: std::array::from_fn(|i| {
                amane::Animation::new(if i == index { 1.0 } else { 0.0 })
                    .duration(ANIM_DURATION)
                    .easing(amane::Easing::InOut)
            }),
        }
    }

    pub fn set_workspace(&mut self, workspace: hipc::types::WorkspaceId) {
        self.highlights.iter_mut().for_each(|highlight| highlight.to(0.0));
        if let Ok(index) = usize::try_from(workspace.raw() - 1) {
            if let Some(highlight) = self.highlights.get_mut(index) {
                highlight.to(1.0);
            }
        }
        self.workspace = workspace;
    }

    pub fn anim_value(&self, workspace_index: usize) -> f32 {
        self.highlights[workspace_index].value()
    }
}
