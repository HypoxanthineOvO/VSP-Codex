use super::*;
use crate::agent::control::SpawnAgentForkMode;
use crate::agent::control::SpawnAgentOptions;
use crate::agent::control::render_input_preview;
use crate::agent::next_thread_spawn_depth;
use crate::agent::role::DEFAULT_ROLE_NAME;
use crate::agent::role::apply_role_to_config;
use crate::agent::role::apply_role_to_worker_routed_config;
use crate::agent::worker_routing::WorkerRoutingClass;
use crate::agent::worker_routing::WorkerRoutingSelection;
use crate::agent::worker_routing::load_worker_routing_policy;
use crate::agent::worker_routing::resolve_worker_routing;
use crate::agent_communication::AgentCommunicationContext;
use crate::agent_communication::AgentCommunicationKind;
use crate::tools::handlers::multi_agents_spec::SpawnAgentToolOptions;
use crate::tools::handlers::multi_agents_spec::create_spawn_agent_tool_v2;
use crate::tools::handlers::multi_agents_v2::message_tool::message_content;
use codex_protocol::AgentPath;
use codex_protocol::protocol::SessionSource;
use codex_tools::ToolSpec;

#[derive(Default)]
pub(crate) struct Handler {
    options: SpawnAgentToolOptions,
}

impl Handler {
    pub(crate) fn new(options: SpawnAgentToolOptions) -> Self {
        Self { options }
    }
}

impl ToolExecutor<ToolInvocation> for Handler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain("spawn_agent")
    }

    fn spec(&self) -> ToolSpec {
        create_spawn_agent_tool_v2(self.options.clone())
    }

    fn handle(&self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'_> {
        Box::pin(async move { handle_spawn_agent(invocation).await.map(boxed_tool_output) })
    }
}

async fn handle_spawn_agent(
    invocation: ToolInvocation,
) -> Result<SpawnAgentResult, FunctionCallError> {
    let ToolInvocation {
        session,
        turn,
        payload,
        call_id,
        ..
    } = invocation;
    let arguments = function_arguments(payload)?;
    let SpawnAgentArgs {
        message,
        task_name,
        agent_type,
        model,
        reasoning_effort,
        service_tier,
        routing_class,
        fork_turns,
        fork_context,
    } = parse_arguments(&arguments)?;
    let has_explicit_overrides = model
        .as_deref()
        .is_some_and(|model| !model.trim().is_empty())
        || reasoning_effort.is_some()
        || service_tier
            .as_deref()
            .is_some_and(|service_tier| !service_tier.trim().is_empty());
    if routing_class.is_some() && has_explicit_overrides {
        return Err(FunctionCallError::RespondToModel(
            "routing_class cannot be combined with model, reasoning_effort, or service_tier overrides"
                .to_string(),
        ));
    }
    let semantic_routing = (!has_explicit_overrides).then_some(routing_class.unwrap_or_default());
    let role_name = agent_type
        .as_deref()
        .map(str::trim)
        .filter(|role| !role.is_empty())
        .map(str::to_string);
    let role_name = role_name.as_deref();
    if fork_turns
        .as_deref()
        .is_some_and(|fork_turns| fork_turns.trim().eq_ignore_ascii_case("all"))
    {
        reject_full_fork_spawn_overrides(role_name, model.as_deref(), reasoning_effort.clone())?;
    }

    let initial_operation = parse_collab_input(Some(message.clone()), /*items*/ None)?;
    let _prompt = render_input_preview(&initial_operation);

    let message = message_content(message)?;
    let session_source = turn.session_source.clone();
    let child_depth = next_thread_spawn_depth(&session_source);
    let mut config =
        build_agent_spawn_config(&session.get_base_instructions().await, turn.as_ref())?;
    let parent_model_configuration = (
        config.effective_model_provider_id_for_persistence(),
        config.model.clone(),
        config.model_reasoning_effort.clone(),
    );
    let policy = load_worker_routing_policy(config.codex_home.as_path())
        .await
        .map_err(FunctionCallError::RespondToModel)?;
    if has_explicit_overrides && !policy.allows_explicit_overrides() {
        return Err(FunctionCallError::RespondToModel(
            "explicit model, reasoning_effort, and service_tier overrides are disabled by the trusted worker routing policy"
                .to_string(),
        ));
    }
    let mut routing = if let Some(routing_class) = semantic_routing {
        let routing = resolve_worker_routing(&session, &mut config, &policy, routing_class).await;
        if let Some(role_name) = role_name {
            apply_role_to_worker_routed_config(&mut config, role_name)
                .await
                .map_err(FunctionCallError::RespondToModel)?;
        }
        routing
    } else {
        let inherited_model = config.model.clone();
        let inherited_reasoning_effort = config.model_reasoning_effort.clone();
        if role_name.is_some() {
            apply_role_to_config(&mut config, role_name)
                .await
                .map_err(FunctionCallError::RespondToModel)?;
        }
        let requested_model = model
            .as_deref()
            .filter(|_| role_name.is_none() || config.model == inherited_model);
        let requested_reasoning_effort = reasoning_effort.clone().filter(|_| {
            role_name.is_none() || config.model_reasoning_effort == inherited_reasoning_effort
        });
        apply_requested_spawn_agent_model_overrides(
            &session,
            turn.as_ref(),
            &mut config,
            requested_model,
            requested_reasoning_effort,
        )
        .await?;
        WorkerRoutingSelection::explicit(&policy, &config)
    };
    if routing.candidate_id.is_none() {
        apply_spawn_agent_service_tier(
            &session,
            &mut config,
            turn.config.service_tier.as_deref(),
            service_tier.as_deref(),
        )
        .await?;
    }
    apply_spawn_agent_runtime_overrides(&mut config, turn.as_ref())?;
    routing.refresh_effective(&config);
    let effective_model_configuration = (
        config.effective_model_provider_id_for_persistence(),
        config.model.clone(),
        config.model_reasoning_effort.clone(),
    );
    let has_spawn_overrides =
        role_name.is_some() || effective_model_configuration != parent_model_configuration;
    let effective_fork_turns = if semantic_routing.is_some()
        && effective_model_configuration != parent_model_configuration
        && fork_turns
            .as_deref()
            .is_some_and(|fork_turns| fork_turns.trim().eq_ignore_ascii_case("all"))
    {
        if matches!(session_source, SessionSource::Exec) {
            Some("none")
        } else {
            None
        }
    } else {
        fork_turns.as_deref()
    };
    let fork_mode = spawn_agent_fork_mode(effective_fork_turns, fork_context, has_spawn_overrides)?;
    if matches!(fork_mode, Some(SpawnAgentForkMode::FullHistory)) && has_spawn_overrides {
        return Err(FunctionCallError::RespondToModel(
            "Full-history forked agents must inherit the parent agent type and model configuration; omit agent_type, routing_class, model, reasoning_effort, and service_tier, or use a partial/no-history fork."
                .to_string(),
        ));
    }

    let spawn_source = thread_spawn_source(
        session.thread_id,
        &turn.session_source,
        child_depth,
        role_name,
        Some(task_name),
    )?;
    let new_agent_path = spawn_source.get_agent_path().ok_or_else(|| {
        FunctionCallError::RespondToModel(
            "spawned agent is missing a canonical task name".to_string(),
        )
    })?;
    let author = turn
        .session_source
        .get_agent_path()
        .unwrap_or_else(AgentPath::root);
    let communication = communication_from_tool_message(author, new_agent_path.clone(), message);
    let context = AgentCommunicationContext::new(AgentCommunicationKind::Spawn, session.thread_id);
    let spawned_agent = Box::pin(
        session
            .services
            .agent_control
            .spawn_agent_with_communication(
                config,
                communication,
                context,
                Some(spawn_source),
                SpawnAgentOptions {
                    fork_parent_spawn_call_id: fork_mode.as_ref().map(|_| call_id.clone()),
                    fork_mode,
                    parent_thread_id: Some(session.thread_id),
                    environments: Some(turn.environments.to_selections()),
                },
            ),
    )
    .await
    .map_err(collab_spawn_error)?;
    let new_thread_id = spawned_agent.thread_id;
    let agent_snapshot = session
        .services
        .agent_control
        .get_agent_config_snapshot(new_thread_id)
        .await;
    let nickname = agent_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.session_source.get_nickname())
        .or(spawned_agent.metadata.agent_nickname);
    emit_sub_agent_activity(
        &session,
        &turn,
        SubAgentActivityItem {
            id: call_id,
            agent_thread_id: new_thread_id,
            agent_path: new_agent_path.clone(),
            kind: SubAgentActivityKind::Started,
        },
    )
    .await;
    let role_tag = role_name.unwrap_or(DEFAULT_ROLE_NAME);
    turn.session_telemetry.counter(
        "codex.multi_agent.spawn",
        /*inc*/ 1,
        &[("role", role_tag), ("version", "v2")],
    );
    let requested_class = routing
        .requested_class
        .map(|routing_class| routing_class.to_string())
        .unwrap_or_else(|| "explicit".to_string());
    let fallback = routing
        .fallback
        .as_ref()
        .map(|_| "inherited")
        .unwrap_or("none");
    turn.session_telemetry.counter(
        "codex.multi_agent.worker_routing",
        /*inc*/ 1,
        &[
            ("requested_class", requested_class.as_str()),
            ("fallback", fallback),
        ],
    );
    tracing::info!(
        target: "codex.worker_routing",
        requested_class = %requested_class,
        policy_version = %routing.policy_version,
        candidate_id = ?routing.candidate_id,
        effective_provider = %routing.effective_provider,
        effective_model = ?routing.effective_model,
        effective_reasoning = ?routing.effective_reasoning,
        effective_service_tier = ?routing.effective_service_tier,
        fallback,
        skipped_candidates = ?routing.skipped_candidates,
        "resolved worker routing for spawned agent"
    );
    let task_name = String::from(new_agent_path);

    let hide_agent_metadata = turn.config.multi_agent_v2.hide_spawn_agent_metadata;
    if hide_agent_metadata {
        Ok(SpawnAgentResult::HiddenMetadata { task_name })
    } else {
        Ok(SpawnAgentResult::WithNickname {
            task_name,
            nickname,
        })
    }
}

impl CoreToolRuntime for Handler {
    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        matches!(payload, ToolPayload::Function { .. })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SpawnAgentArgs {
    message: String,
    task_name: String,
    agent_type: Option<String>,
    model: Option<String>,
    reasoning_effort: Option<ReasoningEffort>,
    service_tier: Option<String>,
    routing_class: Option<WorkerRoutingClass>,
    fork_turns: Option<String>,
    fork_context: Option<bool>,
}

fn spawn_agent_fork_mode(
    fork_turns: Option<&str>,
    fork_context: Option<bool>,
    has_spawn_overrides: bool,
) -> Result<Option<SpawnAgentForkMode>, FunctionCallError> {
    if fork_context.is_some() {
        return Err(FunctionCallError::RespondToModel(
            "fork_context is not supported in MultiAgentV2; use fork_turns instead".to_string(),
        ));
    }

    let fork_turns = fork_turns
        .map(str::trim)
        .filter(|fork_turns| !fork_turns.is_empty());
    // Full-history forks must inherit their parent's model configuration. When the caller asks
    // for a different role/model/effort without choosing a fork mode, retain enough recent
    // context to make the delegation useful while keeping those overrides valid.
    let fork_turns = fork_turns.unwrap_or(if has_spawn_overrides { "3" } else { "all" });

    if fork_turns.eq_ignore_ascii_case("none") {
        return Ok(None);
    }
    if fork_turns.eq_ignore_ascii_case("all") {
        return Ok(Some(SpawnAgentForkMode::FullHistory));
    }

    let last_n_turns = fork_turns.parse::<usize>().map_err(|_| {
        FunctionCallError::RespondToModel(
            "fork_turns must be `none`, `all`, or a positive integer string".to_string(),
        )
    })?;
    if last_n_turns == 0 {
        return Err(FunctionCallError::RespondToModel(
            "fork_turns must be `none`, `all`, or a positive integer string".to_string(),
        ));
    }

    Ok(Some(SpawnAgentForkMode::LastNTurns(last_n_turns)))
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(crate) enum SpawnAgentResult {
    WithNickname {
        task_name: String,
        nickname: Option<String>,
    },
    HiddenMetadata {
        task_name: String,
    },
}

impl ToolOutput for SpawnAgentResult {
    fn log_preview(&self) -> String {
        tool_output_json_text(self, "spawn_agent")
    }

    fn success_for_logging(&self) -> bool {
        true
    }

    fn to_response_item(&self, call_id: &str, payload: &ToolPayload) -> ResponseInputItem {
        tool_output_response_item(call_id, payload, self, Some(true), "spawn_agent")
    }

    fn code_mode_result(&self, _payload: &ToolPayload) -> JsonValue {
        tool_output_code_mode_result(self, "spawn_agent")
    }
}
