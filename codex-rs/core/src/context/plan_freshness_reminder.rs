use super::ContextualUserFragment;
use crate::state::PlanStaleReason;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlanFreshnessReminder {
    reason: PlanStaleReason,
}

impl PlanFreshnessReminder {
    pub(crate) fn new(reason: PlanStaleReason) -> Self {
        Self { reason }
    }
}

impl ContextualUserFragment for PlanFreshnessReminder {
    fn role(&self) -> &'static str {
        "developer"
    }

    fn markers(&self) -> (&'static str, &'static str) {
        Self::type_markers()
    }

    fn type_markers() -> (&'static str, &'static str) {
        ("<plan_freshness_reminder>", "</plan_freshness_reminder>")
    }

    fn body(&self) -> String {
        format!(
            "\nThe active Plan may be stale because {}. Before continuing substantial work, review the visible/structured Plan and call `update_plan` with the current Plan/SubPlan state. For Hypo-Workflow projects, treat phase changes, worker handoffs, validation boundaries, final response boundaries, and changed `.pipeline` state files as Plan refresh boundaries. If the Plan is already accurate, still call `update_plan` with that accurate state so the runtime can clear this gate.\n",
            self.reason.model_facing_label()
        )
    }
}
