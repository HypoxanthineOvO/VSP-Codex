use crate::config::Config;
use crate::session::session::Session;
use crate::thread_manager::build_models_manager;
use codex_models_manager::manager::RefreshStrategy;
use codex_protocol::config_types::SERVICE_TIER_DEFAULT_REQUEST_VALUE;
use codex_protocol::openai_models::ModelInfo;
use codex_protocol::openai_models::ReasoningEffort;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use std::fmt;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::Arc;

const WORKER_ROUTING_FILE: &str = "worker-routing.toml";
const BUILT_IN_POLICY_VERSION: &str = "worker-routing-v1";

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorkerRoutingClass {
    Mechanical,
    LongContext,
    #[default]
    Standard,
    Explore,
    Critical,
    Escalation,
}

impl fmt::Display for WorkerRoutingClass {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Mechanical => "mechanical",
            Self::LongContext => "long_context",
            Self::Standard => "standard",
            Self::Explore => "explore",
            Self::Critical => "critical",
            Self::Escalation => "escalation",
        })
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkerRoutingCandidate {
    id: String,
    model_provider: String,
    model: String,
    reasoning_effort: ReasoningEffort,
    service_tier: Option<String>,
    min_context_window: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkerRoutingPolicy {
    enabled: bool,
    policy_version: String,
    allow_explicit_overrides: bool,
    mechanical: Vec<WorkerRoutingCandidate>,
    long_context: Vec<WorkerRoutingCandidate>,
    standard: Vec<WorkerRoutingCandidate>,
    explore: Vec<WorkerRoutingCandidate>,
    critical: Vec<WorkerRoutingCandidate>,
    escalation: Vec<WorkerRoutingCandidate>,
}

impl WorkerRoutingPolicy {
    pub(crate) fn allows_explicit_overrides(&self) -> bool {
        self.allow_explicit_overrides
    }

    fn candidates(&self, routing_class: WorkerRoutingClass) -> &[WorkerRoutingCandidate] {
        match routing_class {
            WorkerRoutingClass::Mechanical => &self.mechanical,
            WorkerRoutingClass::LongContext => &self.long_context,
            WorkerRoutingClass::Standard => &self.standard,
            WorkerRoutingClass::Explore => &self.explore,
            WorkerRoutingClass::Critical => &self.critical,
            WorkerRoutingClass::Escalation => &self.escalation,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorkerRoutingFallback {
    Inherited,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorkerRoutingSkipReason {
    PolicyDisabled,
    UnknownProvider,
    UnknownModel,
    UnsupportedReasoningEffort,
    ContextWindowUnknown,
    ContextWindowTooSmall,
    UnsupportedServiceTier,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct SkippedWorkerRoutingCandidate {
    pub(crate) candidate_id: Option<String>,
    pub(crate) reason: WorkerRoutingSkipReason,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct WorkerRoutingSelection {
    pub(crate) requested_class: Option<WorkerRoutingClass>,
    pub(crate) policy_version: String,
    pub(crate) candidate_id: Option<String>,
    pub(crate) effective_provider: String,
    pub(crate) effective_model: Option<String>,
    pub(crate) effective_reasoning: Option<ReasoningEffort>,
    pub(crate) effective_service_tier: Option<String>,
    pub(crate) fallback: Option<WorkerRoutingFallback>,
    pub(crate) skipped_candidates: Vec<SkippedWorkerRoutingCandidate>,
}

impl WorkerRoutingSelection {
    pub(crate) fn explicit(policy: &WorkerRoutingPolicy, config: &Config) -> Self {
        Self::effective(
            /*requested_class*/ None,
            policy.policy_version.clone(),
            /*candidate_id*/ None,
            config,
            /*fallback*/ None,
            Vec::new(),
        )
    }

    pub(crate) fn refresh_effective(&mut self, config: &Config) {
        self.effective_provider = config.effective_model_provider_id_for_persistence();
        self.effective_model.clone_from(&config.model);
        self.effective_reasoning
            .clone_from(&config.model_reasoning_effort);
        self.effective_service_tier.clone_from(&config.service_tier);
    }

    fn inherited(
        routing_class: WorkerRoutingClass,
        policy: &WorkerRoutingPolicy,
        config: &Config,
        skipped_candidates: Vec<SkippedWorkerRoutingCandidate>,
    ) -> Self {
        Self::effective(
            Some(routing_class),
            policy.policy_version.clone(),
            /*candidate_id*/ None,
            config,
            Some(WorkerRoutingFallback::Inherited),
            skipped_candidates,
        )
    }

    fn selected(
        routing_class: WorkerRoutingClass,
        policy: &WorkerRoutingPolicy,
        candidate_id: String,
        config: &Config,
        skipped_candidates: Vec<SkippedWorkerRoutingCandidate>,
    ) -> Self {
        Self::effective(
            Some(routing_class),
            policy.policy_version.clone(),
            Some(candidate_id),
            config,
            /*fallback*/ None,
            skipped_candidates,
        )
    }

    fn effective(
        requested_class: Option<WorkerRoutingClass>,
        policy_version: String,
        candidate_id: Option<String>,
        config: &Config,
        fallback: Option<WorkerRoutingFallback>,
        skipped_candidates: Vec<SkippedWorkerRoutingCandidate>,
    ) -> Self {
        Self {
            requested_class,
            policy_version,
            candidate_id,
            effective_provider: config.effective_model_provider_id_for_persistence(),
            effective_model: config.model.clone(),
            effective_reasoning: config.model_reasoning_effort.clone(),
            effective_service_tier: config.service_tier.clone(),
            fallback,
            skipped_candidates,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkerRoutingPolicyFile {
    worker_routing: WorkerRoutingPolicyOverlay,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkerRoutingPolicyOverlay {
    enabled: Option<bool>,
    policy_version: Option<String>,
    allow_explicit_overrides: Option<bool>,
    mechanical: Option<Vec<WorkerRoutingCandidate>>,
    long_context: Option<Vec<WorkerRoutingCandidate>>,
    standard: Option<Vec<WorkerRoutingCandidate>>,
    explore: Option<Vec<WorkerRoutingCandidate>>,
    critical: Option<Vec<WorkerRoutingCandidate>>,
    escalation: Option<Vec<WorkerRoutingCandidate>>,
}

pub(crate) async fn load_worker_routing_policy(
    codex_home: &Path,
) -> Result<WorkerRoutingPolicy, String> {
    let policy_path = codex_home.join(WORKER_ROUTING_FILE);
    let contents = match tokio::fs::read_to_string(&policy_path).await {
        Ok(contents) => contents,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(built_in_policy()),
        Err(err) => {
            return Err(format!(
                "failed to read trusted worker routing policy {}: {err}",
                policy_path.display()
            ));
        }
    };
    let overlay = toml::from_str::<WorkerRoutingPolicyFile>(&contents)
        .map_err(|err| invalid_policy_message(&policy_path, err.span().map(|span| span.start)))?
        .worker_routing;
    let policy = apply_overlay(built_in_policy(), overlay);
    validate_policy(&policy)?;
    Ok(policy)
}

pub(crate) async fn resolve_worker_routing(
    session: &Session,
    config: &mut Config,
    policy: &WorkerRoutingPolicy,
    routing_class: WorkerRoutingClass,
) -> WorkerRoutingSelection {
    if !policy.enabled {
        return WorkerRoutingSelection::inherited(
            routing_class,
            policy,
            config,
            vec![SkippedWorkerRoutingCandidate {
                candidate_id: None,
                reason: WorkerRoutingSkipReason::PolicyDisabled,
            }],
        );
    }

    let mut skipped_candidates = Vec::new();
    for candidate in policy.candidates(routing_class) {
        let Some(provider) = config
            .model_providers
            .get(&candidate.model_provider)
            .cloned()
        else {
            skip_candidate(
                &mut skipped_candidates,
                candidate,
                WorkerRoutingSkipReason::UnknownProvider,
            );
            continue;
        };
        let mut candidate_config = config.clone();
        candidate_config.model_provider_id = candidate.model_provider.clone();
        candidate_config.model_provider = provider;
        let models_manager = build_models_manager(
            &candidate_config,
            Arc::clone(&session.services.auth_manager),
        );
        let available_models = models_manager
            .list_models(
                RefreshStrategy::Offline,
                candidate_config.http_client_factory(),
            )
            .await;
        if !available_models
            .iter()
            .any(|model| model.model == candidate.model)
        {
            skip_candidate(
                &mut skipped_candidates,
                candidate,
                WorkerRoutingSkipReason::UnknownModel,
            );
            continue;
        }

        let model_info = models_manager
            .get_model_info(
                &candidate.model,
                &candidate_config.to_models_manager_config(),
            )
            .await;
        if let Some(reason) = incompatible_candidate_reason(candidate, &model_info) {
            skip_candidate(&mut skipped_candidates, candidate, reason);
            continue;
        }

        candidate_config.model = Some(candidate.model.clone());
        candidate_config.model_reasoning_effort = Some(candidate.reasoning_effort.clone());
        if let Some(service_tier) = candidate.service_tier.as_deref() {
            candidate_config.service_tier = (service_tier != SERVICE_TIER_DEFAULT_REQUEST_VALUE)
                .then(|| service_tier.to_string());
        } else {
            candidate_config.service_tier = candidate_config
                .service_tier
                .filter(|service_tier| model_info.supports_service_tier(service_tier));
        }
        *config = candidate_config;
        return WorkerRoutingSelection::selected(
            routing_class,
            policy,
            candidate.id.clone(),
            config,
            skipped_candidates,
        );
    }

    WorkerRoutingSelection::inherited(routing_class, policy, config, skipped_candidates)
}

fn incompatible_candidate_reason(
    candidate: &WorkerRoutingCandidate,
    model_info: &ModelInfo,
) -> Option<WorkerRoutingSkipReason> {
    if !model_info.supported_reasoning_levels.is_empty()
        && !model_info
            .supported_reasoning_levels
            .iter()
            .any(|preset| preset.effort == candidate.reasoning_effort)
    {
        return Some(WorkerRoutingSkipReason::UnsupportedReasoningEffort);
    }
    if let Some(min_context_window) = candidate.min_context_window {
        match model_info.resolved_context_window() {
            None => return Some(WorkerRoutingSkipReason::ContextWindowUnknown),
            Some(context_window) if context_window < min_context_window => {
                return Some(WorkerRoutingSkipReason::ContextWindowTooSmall);
            }
            Some(_) => {}
        }
    }
    if let Some(service_tier) = candidate.service_tier.as_deref()
        && service_tier != SERVICE_TIER_DEFAULT_REQUEST_VALUE
        && !model_info.supports_service_tier(service_tier)
    {
        return Some(WorkerRoutingSkipReason::UnsupportedServiceTier);
    }
    None
}

fn skip_candidate(
    skipped_candidates: &mut Vec<SkippedWorkerRoutingCandidate>,
    candidate: &WorkerRoutingCandidate,
    reason: WorkerRoutingSkipReason,
) {
    skipped_candidates.push(SkippedWorkerRoutingCandidate {
        candidate_id: Some(candidate.id.clone()),
        reason,
    });
}

fn invalid_policy_message(path: &Path, byte_offset: Option<usize>) -> String {
    byte_offset.map_or_else(
        || {
            format!(
                "trusted worker routing policy {} is invalid",
                path.display()
            )
        },
        |byte_offset| {
            format!(
                "trusted worker routing policy {} is invalid near byte {byte_offset}",
                path.display()
            )
        },
    )
}

fn apply_overlay(
    mut policy: WorkerRoutingPolicy,
    overlay: WorkerRoutingPolicyOverlay,
) -> WorkerRoutingPolicy {
    policy.enabled = overlay.enabled.unwrap_or(policy.enabled);
    policy.policy_version = overlay.policy_version.unwrap_or(policy.policy_version);
    policy.allow_explicit_overrides = overlay
        .allow_explicit_overrides
        .unwrap_or(policy.allow_explicit_overrides);
    policy.mechanical = overlay.mechanical.unwrap_or(policy.mechanical);
    policy.long_context = overlay.long_context.unwrap_or(policy.long_context);
    policy.standard = overlay.standard.unwrap_or(policy.standard);
    policy.explore = overlay.explore.unwrap_or(policy.explore);
    policy.critical = overlay.critical.unwrap_or(policy.critical);
    policy.escalation = overlay.escalation.unwrap_or(policy.escalation);
    policy
}

fn validate_policy(policy: &WorkerRoutingPolicy) -> Result<(), String> {
    if policy.policy_version.trim().is_empty() {
        return Err("worker routing policy_version must not be empty".to_string());
    }
    let mut candidate_ids = HashSet::new();
    for routing_class in [
        WorkerRoutingClass::Mechanical,
        WorkerRoutingClass::LongContext,
        WorkerRoutingClass::Standard,
        WorkerRoutingClass::Explore,
        WorkerRoutingClass::Critical,
        WorkerRoutingClass::Escalation,
    ] {
        for candidate in policy.candidates(routing_class) {
            if candidate.id.trim().is_empty()
                || candidate.model_provider.trim().is_empty()
                || candidate.model.trim().is_empty()
            {
                return Err(format!(
                    "worker routing class {routing_class} has a candidate with an empty id, model_provider, or model"
                ));
            }
            if !candidate_ids.insert(candidate.id.as_str()) {
                return Err(format!(
                    "worker routing candidate id '{}' is duplicated",
                    candidate.id
                ));
            }
            if candidate
                .min_context_window
                .is_some_and(|window| window <= 0)
            {
                return Err(format!(
                    "worker routing candidate '{}' has a non-positive min_context_window",
                    candidate.id
                ));
            }
        }
    }
    Ok(())
}

fn built_in_policy() -> WorkerRoutingPolicy {
    WorkerRoutingPolicy {
        enabled: true,
        policy_version: BUILT_IN_POLICY_VERSION.to_string(),
        allow_explicit_overrides: true,
        mechanical: vec![candidate(
            "mechanical-luna-medium",
            "gpt-5.6-luna",
            ReasoningEffort::Medium,
        )],
        long_context: vec![WorkerRoutingCandidate {
            min_context_window: Some(300_000),
            ..candidate(
                "long-context-luna-medium",
                "gpt-5.6-luna",
                ReasoningEffort::Medium,
            )
        }],
        standard: vec![candidate(
            "standard-luna-max",
            "gpt-5.6-luna",
            ReasoningEffort::Max,
        )],
        explore: vec![
            candidate("explore-luna-xhigh", "gpt-5.6-luna", ReasoningEffort::XHigh),
            candidate("explore-sol-medium", "gpt-5.6-sol", ReasoningEffort::Medium),
        ],
        critical: vec![candidate(
            "critical-sol-high",
            "gpt-5.6-sol",
            ReasoningEffort::High,
        )],
        escalation: vec![candidate(
            "escalation-sol-max",
            "gpt-5.6-sol",
            ReasoningEffort::Max,
        )],
    }
}

fn candidate(id: &str, model: &str, reasoning_effort: ReasoningEffort) -> WorkerRoutingCandidate {
    WorkerRoutingCandidate {
        id: id.to_string(),
        model_provider: "vsp".to_string(),
        model: model.to_string(),
        reasoning_effort,
        service_tier: None,
        min_context_window: None,
    }
}

#[cfg(test)]
#[path = "worker_routing_tests.rs"]
mod tests;
