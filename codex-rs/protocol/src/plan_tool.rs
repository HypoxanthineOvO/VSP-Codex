use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;

// Types for the TODO tool arguments matching codex-vscode/todo-mcp/src/main.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct PlanItemArg {
    pub step: String,
    pub status: StepStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct PlanState {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub items: Vec<PlanNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct PlanNode {
    #[serde(default)]
    pub id: Option<String>,
    pub title: String,
    #[serde(default)]
    pub summary: Option<String>,
    pub status: StepStatus,
    #[serde(default)]
    pub subplan: Option<SubPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct SubPlan {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub items: Vec<PlanSubItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct PlanSubItem {
    #[serde(default)]
    pub id: Option<String>,
    pub title: String,
    #[serde(default)]
    pub summary: Option<String>,
    pub status: StepStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct UpdatePlanArgs {
    /// Arguments for the `update_plan` todo/checklist tool (not plan mode).
    #[serde(default)]
    pub explanation: Option<String>,
    #[serde(default)]
    pub plan: Vec<PlanItemArg>,
    #[serde(default)]
    pub state: Option<PlanState>,
}

impl UpdatePlanArgs {
    pub fn normalize(mut self) -> Self {
        if self.plan.is_empty() && self.state.is_none() {
            return self;
        }

        if self.plan.is_empty()
            && let Some(state) = &self.state
        {
            self.plan = state
                .items
                .iter()
                .map(|item| PlanItemArg {
                    step: item.title.clone(),
                    status: item.status.clone(),
                })
                .collect();
        }

        if self.state.is_none() {
            self.state = Some(PlanState::from_legacy_plan(&self.plan));
        }

        self
    }
}

impl PlanState {
    pub fn from_legacy_plan(plan: &[PlanItemArg]) -> Self {
        Self {
            id: None,
            title: None,
            summary: None,
            items: plan
                .iter()
                .map(|item| PlanNode {
                    id: None,
                    title: item.step.clone(),
                    summary: None,
                    status: item.status.clone(),
                    subplan: None,
                })
                .collect(),
        }
    }
}
