use crate::function_tool::FunctionCallError;
use crate::goals::CreateGoalRequest;
use crate::goals::SetGoalRequest;
use crate::tools::context::FunctionToolOutput;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolPayload;
use crate::tools::context::boxed_tool_output;
use crate::tools::handlers::goal_spec::CREATE_GOAL_TOOL_NAME;
use crate::tools::handlers::goal_spec::UPDATE_GOAL_TOOL_NAME;
use crate::tools::handlers::goal_spec::create_create_goal_tool;
use crate::tools::handlers::goal_spec::create_update_goal_tool;
use crate::tools::handlers::parse_arguments;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolExecutor;
use codex_features::Feature;
use codex_protocol::protocol::ThreadGoal;
use codex_protocol::protocol::ThreadGoalStatus;
use codex_tools::ToolName;
use codex_tools::ToolSpec;
use serde::Deserialize;
use serde_json::json;

pub struct CreateGoalHandler;
pub struct UpdateGoalHandler;

#[derive(Deserialize)]
struct CreateGoalArgs {
    objective: String,
    #[serde(default)]
    token_budget: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum UpdateGoalStatus {
    Complete,
    Blocked,
}

#[derive(Deserialize)]
struct UpdateGoalArgs {
    status: UpdateGoalStatus,
}

impl ToolExecutor<ToolInvocation> for CreateGoalHandler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain(CREATE_GOAL_TOOL_NAME)
    }

    fn spec(&self) -> ToolSpec {
        create_create_goal_tool()
    }

    fn handle(&self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'_> {
        Box::pin(self.handle_call(invocation))
    }
}

impl ToolExecutor<ToolInvocation> for UpdateGoalHandler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain(UPDATE_GOAL_TOOL_NAME)
    }

    fn spec(&self) -> ToolSpec {
        create_update_goal_tool()
    }

    fn handle(&self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'_> {
        Box::pin(self.handle_call(invocation))
    }
}

impl CreateGoalHandler {
    async fn handle_call(
        &self,
        invocation: ToolInvocation,
    ) -> Result<Box<dyn crate::tools::context::ToolOutput>, FunctionCallError> {
        let ToolInvocation {
            session,
            turn,
            payload,
            ..
        } = invocation;
        if !turn.config.features.enabled(Feature::Goals) {
            return Err(FunctionCallError::RespondToModel(
                "goals feature is disabled".to_string(),
            ));
        }

        let arguments = function_arguments(payload, CREATE_GOAL_TOOL_NAME)?;
        let args: CreateGoalArgs = parse_arguments(&arguments)?;
        let goal = session
            .create_thread_goal(
                turn.as_ref(),
                CreateGoalRequest {
                    objective: args.objective,
                    token_budget: args.token_budget,
                },
            )
            .await
            .map_err(|err| FunctionCallError::RespondToModel(err.to_string()))?;
        goal_response(Some(goal), false)
    }
}

impl UpdateGoalHandler {
    async fn handle_call(
        &self,
        invocation: ToolInvocation,
    ) -> Result<Box<dyn crate::tools::context::ToolOutput>, FunctionCallError> {
        let ToolInvocation {
            session,
            turn,
            payload,
            ..
        } = invocation;
        if !turn.config.features.enabled(Feature::Goals) {
            return Err(FunctionCallError::RespondToModel(
                "goals feature is disabled".to_string(),
            ));
        }

        let arguments = function_arguments(payload, UPDATE_GOAL_TOOL_NAME)?;
        let args: UpdateGoalArgs = parse_arguments(&arguments)?;
        let status = match args.status {
            UpdateGoalStatus::Complete => ThreadGoalStatus::Complete,
            UpdateGoalStatus::Blocked => ThreadGoalStatus::Blocked,
        };
        let goal = session
            .set_thread_goal(
                turn.as_ref(),
                SetGoalRequest {
                    objective: None,
                    status: Some(status),
                    token_budget: None,
                },
            )
            .await
            .map_err(|err| FunctionCallError::RespondToModel(err.to_string()))?;
        goal_response(Some(goal), status == ThreadGoalStatus::Complete)
    }
}

impl CoreToolRuntime for CreateGoalHandler {}
impl CoreToolRuntime for UpdateGoalHandler {}

fn function_arguments(payload: ToolPayload, tool_name: &str) -> Result<String, FunctionCallError> {
    match payload {
        ToolPayload::Function { arguments } => Ok(arguments),
        _ => Err(FunctionCallError::RespondToModel(format!(
            "{tool_name} handler received unsupported payload"
        ))),
    }
}

fn goal_response(
    goal: Option<ThreadGoal>,
    include_completion_report: bool,
) -> Result<Box<dyn crate::tools::context::ToolOutput>, FunctionCallError> {
    let remaining_tokens = goal
        .as_ref()
        .and_then(|goal| goal.token_budget.map(|budget| budget - goal.tokens_used));
    let completion_budget_report = include_completion_report.then(|| {
        "Goal achieved. Report final usage from this tool result's structured goal fields."
            .to_string()
    });
    let value = json!({
        "goal": goal,
        "remaining_tokens": remaining_tokens,
        "completion_budget_report": completion_budget_report,
    });
    let content = serde_json::to_string(&value).map_err(|err| {
        FunctionCallError::Fatal(format!("failed to serialize goal response: {err}"))
    })?;
    Ok(boxed_tool_output(FunctionToolOutput::from_text(
        content,
        Some(true),
    )))
}
