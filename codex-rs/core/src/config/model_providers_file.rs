use codex_model_provider_info::ModelProviderInfo;
use codex_protocol::openai_models::ModelVisibility;
use codex_protocol::openai_models::ModelsResponse;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::openai_models::ReasoningEffortPreset;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::ErrorKind;
use std::path::Path;
use toml_edit::DocumentMut;
use url::Url;

pub const MODEL_PROVIDERS_TOML_FILE: &str = "model-providers.toml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddonModelProviders {
    pub version: u32,
    pub default_selection: Option<AddonModelSelection>,
    pub providers: BTreeMap<String, AddonProvider>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddonModelSelection {
    pub provider_id: String,
    pub model_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddonProvider {
    pub name: String,
    pub base_url: String,
    pub env_key: String,
    /// Remote app-server credential state when the catalog crossed a process boundary.
    pub credential_available: Option<bool>,
    pub models: Vec<AddonModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddonModel {
    pub id: String,
    pub display_name: String,
    pub context_window: i64,
    pub effective_context_window_percent: i64,
    pub supported_reasoning_efforts: Vec<ReasoningEffort>,
    pub default_reasoning_effort: Option<ReasoningEffort>,
}

impl AddonModelProviders {
    pub fn model_catalog(&self, provider_id: &str) -> Option<ModelsResponse> {
        let provider = self.providers.get(provider_id)?;
        let models = provider
            .models
            .iter()
            .enumerate()
            .map(|(priority, model)| {
                let mut info = codex_models_manager::model_info::model_info_from_slug(&model.id);
                info.display_name.clone_from(&model.display_name);
                info.description = Some(format!("Context window: {} tokens", model.context_window));
                info.default_reasoning_level = model.default_reasoning_effort.clone();
                info.supported_reasoning_levels = model
                    .supported_reasoning_efforts
                    .iter()
                    .cloned()
                    .map(|effort| ReasoningEffortPreset {
                        effort,
                        description: String::new(),
                    })
                    .collect();
                info.visibility = ModelVisibility::List;
                info.priority = i32::try_from(priority).unwrap_or(i32::MAX);
                info.context_window = Some(model.context_window);
                info.max_context_window = Some(model.context_window);
                info.effective_context_window_percent = model.effective_context_window_percent;
                info.used_fallback_model_metadata = false;
                info
            })
            .collect();
        Some(ModelsResponse { models })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub(crate) struct ModelProvidersFileError {
    message: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAddonModelProviders {
    version: u32,
    default_provider: Option<String>,
    default_model: Option<String>,
    providers: BTreeMap<String, RawAddonProvider>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAddonProvider {
    name: String,
    base_url: String,
    env_key: String,
    wire_api: AddonWireApi,
    models: Vec<RawAddonModel>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum AddonWireApi {
    Responses,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAddonModel {
    id: String,
    display_name: String,
    context_window: i64,
    #[serde(default = "default_effective_context_window_percent")]
    effective_context_window_percent: i64,
    #[serde(default)]
    supported_reasoning_efforts: Vec<ReasoningEffort>,
    default_reasoning_effort: Option<ReasoningEffort>,
}

pub(crate) fn parse_model_providers_toml(
    contents: &str,
) -> Result<Option<AddonModelProviders>, ModelProvidersFileError> {
    if contents.trim().is_empty() {
        return Ok(None);
    }

    let raw: RawAddonModelProviders = toml::from_str(contents)
        .map_err(|error| invalid_config(format!("invalid TOML: {error}")))?;
    if raw.version != 1 {
        return Err(invalid_config(format!(
            "unsupported version {}; expected version 1",
            raw.version
        )));
    }
    if raw.providers.is_empty() {
        return Err(invalid_config("providers must not be empty"));
    }

    let default_selection = match (raw.default_provider, raw.default_model) {
        (Some(provider_id), Some(model_id)) => Some(AddonModelSelection {
            provider_id,
            model_id,
        }),
        (None, None) => None,
        _ => {
            return Err(invalid_config(
                "default_provider and default_model must be set together",
            ));
        }
    };

    let mut providers = BTreeMap::new();
    for (provider_id, provider) in raw.providers {
        validate_provider_id(&provider_id)?;
        validate_nonempty("provider name", &provider.name)?;
        validate_env_key(&provider.env_key)?;
        validate_base_url(&provider.base_url)?;
        if provider.models.is_empty() {
            return Err(invalid_config(format!(
                "provider `{provider_id}` must define at least one model"
            )));
        }

        let mut model_ids = HashSet::new();
        let mut models = Vec::with_capacity(provider.models.len());
        for model in provider.models {
            validate_nonempty("model id", &model.id)?;
            validate_nonempty("model display_name", &model.display_name)?;
            if !model_ids.insert(model.id.clone()) {
                return Err(invalid_config(format!(
                    "duplicate model id `{}` in provider `{provider_id}`",
                    model.id
                )));
            }
            if model.context_window <= 0 {
                return Err(invalid_config(format!(
                    "provider `{provider_id}` model `{}` context_window must be greater than zero",
                    model.id
                )));
            }
            if !(1..=100).contains(&model.effective_context_window_percent) {
                return Err(invalid_config(format!(
                    "provider `{provider_id}` model `{}` effective_context_window_percent must be between 1 and 100",
                    model.id
                )));
            }
            let unique_efforts = model
                .supported_reasoning_efforts
                .iter()
                .collect::<HashSet<_>>();
            if unique_efforts.len() != model.supported_reasoning_efforts.len() {
                return Err(invalid_config(format!(
                    "provider `{provider_id}` model `{}` has duplicate supported_reasoning_efforts",
                    model.id
                )));
            }
            if let Some(default_effort) = model.default_reasoning_effort.as_ref()
                && !model.supported_reasoning_efforts.contains(default_effort)
            {
                return Err(invalid_config(format!(
                    "provider `{provider_id}` model `{}` default_reasoning_effort `{default_effort}` is not listed in supported_reasoning_efforts",
                    model.id
                )));
            }
            models.push(AddonModel {
                id: model.id,
                display_name: model.display_name,
                context_window: model.context_window,
                effective_context_window_percent: model.effective_context_window_percent,
                supported_reasoning_efforts: model.supported_reasoning_efforts,
                default_reasoning_effort: model.default_reasoning_effort,
            });
        }

        let RawAddonProvider {
            name,
            base_url,
            env_key,
            wire_api: AddonWireApi::Responses,
            models: _,
        } = provider;
        providers.insert(
            provider_id,
            AddonProvider {
                name,
                base_url,
                env_key,
                credential_available: None,
                models,
            },
        );
    }

    if let Some(selection) = default_selection.as_ref()
        && let Some(provider) = providers.get(&selection.provider_id)
        && !provider
            .models
            .iter()
            .any(|model| model.id == selection.model_id)
    {
        return Err(invalid_config(format!(
            "default model `{}` is not defined by provider `{}`",
            selection.model_id, selection.provider_id
        )));
    }

    Ok(Some(AddonModelProviders {
        version: raw.version,
        default_selection,
        providers,
    }))
}

pub(crate) async fn load_model_providers_file(
    codex_home: &Path,
) -> std::io::Result<Option<AddonModelProviders>> {
    let path = codex_home.join(MODEL_PROVIDERS_TOML_FILE);
    let contents = match tokio::fs::read_to_string(&path).await {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    parse_model_providers_toml(&contents)
        .map_err(|error| std::io::Error::new(ErrorKind::InvalidData, error))
}

pub(crate) fn merge_addon_model_providers(
    model_providers: &mut HashMap<String, ModelProviderInfo>,
    addon: Option<&AddonModelProviders>,
) -> std::io::Result<()> {
    let Some(addon) = addon else {
        return Ok(());
    };
    for (provider_id, provider) in &addon.providers {
        if model_providers.contains_key(provider_id) {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                format!(
                    "provider `{provider_id}` in {MODEL_PROVIDERS_TOML_FILE} conflicts with the effective config.toml provider registry"
                ),
            ));
        }
        model_providers.insert(
            provider_id.clone(),
            ModelProviderInfo {
                name: provider.name.clone(),
                base_url: Some(provider.base_url.clone()),
                env_key: Some(provider.env_key.clone()),
                requires_openai_auth: false,
                supports_websockets: false,
                ..Default::default()
            },
        );
    }
    if let Some(selection) = addon.default_selection.as_ref()
        && !model_providers.contains_key(&selection.provider_id)
    {
        return Err(std::io::Error::new(
            ErrorKind::InvalidData,
            format!(
                "default provider `{}` in {MODEL_PROVIDERS_TOML_FILE} is not in the effective provider registry",
                selection.provider_id
            ),
        ));
    }
    Ok(())
}

pub fn persist_addon_default_selection(
    codex_home: &Path,
    selection: Option<&AddonModelSelection>,
) -> std::io::Result<()> {
    let path = codex_home.join(MODEL_PROVIDERS_TOML_FILE);
    let contents = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound && selection.is_none() => return Ok(()),
        Err(error) => return Err(error),
    };
    let parsed = parse_model_providers_toml(&contents)
        .map_err(|error| std::io::Error::new(ErrorKind::InvalidData, error))?;
    if let Some(selection) = selection {
        let addon = parsed.as_ref().ok_or_else(|| {
            std::io::Error::new(
                ErrorKind::InvalidData,
                "cannot persist an addon selection to an empty model-providers.toml",
            )
        })?;
        validate_default_selection(addon, selection)?;
    }

    let mut document = contents.parse::<DocumentMut>().map_err(|error| {
        std::io::Error::new(
            ErrorKind::InvalidData,
            format!("invalid {MODEL_PROVIDERS_TOML_FILE}: {error}"),
        )
    })?;
    match selection {
        Some(selection) => {
            document["default_provider"] = toml_edit::value(&selection.provider_id);
            document["default_model"] = toml_edit::value(&selection.model_id);
        }
        None => {
            document.remove("default_provider");
            document.remove("default_model");
        }
    }
    crate::path_utils::write_atomically(&path, &document.to_string())
}

fn validate_default_selection(
    addon: &AddonModelProviders,
    selection: &AddonModelSelection,
) -> std::io::Result<()> {
    let Some(provider) = addon.providers.get(&selection.provider_id) else {
        return Ok(());
    };
    if provider
        .models
        .iter()
        .any(|model| model.id == selection.model_id)
    {
        Ok(())
    } else {
        Err(std::io::Error::new(
            ErrorKind::InvalidInput,
            format!(
                "unknown addon model `{}` for provider `{}`",
                selection.model_id, selection.provider_id
            ),
        ))
    }
}

fn default_effective_context_window_percent() -> i64 {
    95
}

fn validate_provider_id(provider_id: &str) -> Result<(), ModelProvidersFileError> {
    let valid = !provider_id.is_empty()
        && provider_id.len() <= 64
        && provider_id
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && provider_id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        });
    if valid {
        Ok(())
    } else {
        Err(invalid_config(format!(
            "provider id `{provider_id}` must match ^[a-z0-9][a-z0-9_-]{{0,63}}$"
        )))
    }
}

fn validate_nonempty(label: &str, value: &str) -> Result<(), ModelProvidersFileError> {
    if !value.is_empty() && value.trim() == value {
        Ok(())
    } else {
        Err(invalid_config(format!(
            "{label} must be nonempty and have no leading or trailing whitespace"
        )))
    }
}

fn validate_env_key(env_key: &str) -> Result<(), ModelProvidersFileError> {
    let mut bytes = env_key.bytes();
    let valid_start = bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_');
    if valid_start && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
        Ok(())
    } else {
        Err(invalid_config(format!(
            "env_key `{env_key}` must be an environment variable name"
        )))
    }
}

fn validate_base_url(base_url: &str) -> Result<(), ModelProvidersFileError> {
    let parsed = Url::parse(base_url)
        .map_err(|error| invalid_config(format!("invalid base_url `{base_url}`: {error}")))?;
    let valid = matches!(parsed.scheme(), "http" | "https")
        && parsed.host_str().is_some()
        && parsed.username().is_empty()
        && parsed.password().is_none()
        && parsed.query().is_none()
        && parsed.fragment().is_none();
    if valid {
        Ok(())
    } else {
        Err(invalid_config(format!(
            "base_url `{base_url}` must be HTTP(S) and must not contain userinfo, query, or fragment"
        )))
    }
}

fn invalid_config(message: impl Into<String>) -> ModelProvidersFileError {
    ModelProvidersFileError {
        message: message.into(),
    }
}

#[cfg(test)]
#[path = "model_providers_file_tests.rs"]
mod tests;
