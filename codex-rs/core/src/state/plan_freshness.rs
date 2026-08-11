use codex_protocol::plan_tool::StepStatus;
use codex_protocol::plan_tool::UpdatePlanArgs;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlanStaleReason {
    Resume,
    Compact,
    ToolBurst,
    WorkflowStateChanged,
}

impl PlanStaleReason {
    pub(crate) fn model_facing_label(self) -> &'static str {
        match self {
            Self::Resume => "this thread was resumed",
            Self::Compact => "conversation context was compacted",
            Self::ToolBurst => "multiple tools ran after the last Plan update",
            Self::WorkflowStateChanged => {
                "Hypo-Workflow state files changed after the last Plan update"
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PlanFreshness {
    active_plan: Option<UpdatePlanArgs>,
    stale_reason: Option<PlanStaleReason>,
    reminder_injected: bool,
    initial_subagent_reminder_injected: bool,
    plan_update_recorded: bool,
    tools_since_update: u32,
}

impl PlanFreshness {
    pub(crate) fn record_plan_update(&mut self, update: UpdatePlanArgs) {
        self.plan_update_recorded = true;
        self.active_plan = plan_has_active_work(&update).then_some(update);
        self.stale_reason = None;
        self.reminder_injected = false;
        self.tools_since_update = 0;
    }

    pub(crate) fn mark_stale(&mut self, reason: PlanStaleReason) {
        if self.active_plan.is_some() {
            self.stale_reason = Some(reason);
            self.reminder_injected = false;
        }
    }

    pub(crate) fn note_tool_call(&mut self) {
        if self.active_plan.is_none() || self.stale_reason.is_some() {
            return;
        }
        self.tools_since_update = self.tools_since_update.saturating_add(1);
        if self.tools_since_update >= 3 {
            self.mark_stale(PlanStaleReason::ToolBurst);
        }
    }

    pub(crate) fn take_pending_reminder(&mut self) -> Option<PlanStaleReason> {
        let reason = self.stale_reason?;
        if self.reminder_injected {
            return None;
        }
        self.reminder_injected = true;
        Some(reason)
    }

    pub(crate) fn claim_initial_subagent_reminder(&mut self) -> bool {
        if self.plan_update_recorded || self.initial_subagent_reminder_injected {
            return false;
        }
        self.initial_subagent_reminder_injected = true;
        true
    }
}

fn plan_has_active_work(update: &UpdatePlanArgs) -> bool {
    if update
        .plan
        .iter()
        .any(|item| item.status != StepStatus::Completed)
    {
        return true;
    }
    update.state.as_ref().is_some_and(|state| {
        state.items.iter().any(|item| {
            item.status != StepStatus::Completed
                || item.subplan.as_ref().is_some_and(|subplan| {
                    subplan
                        .items
                        .iter()
                        .any(|subitem| subitem.status != StepStatus::Completed)
                })
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_subagent_reminder_is_claimed_once() {
        let mut freshness = PlanFreshness::default();

        assert!(freshness.claim_initial_subagent_reminder());
        assert!(!freshness.claim_initial_subagent_reminder());
    }

    #[test]
    fn plan_update_suppresses_initial_subagent_reminder() {
        let mut freshness = PlanFreshness::default();
        freshness.record_plan_update(UpdatePlanArgs {
            explanation: None,
            plan: Vec::new(),
            state: None,
        });

        assert!(!freshness.claim_initial_subagent_reminder());
    }
}
