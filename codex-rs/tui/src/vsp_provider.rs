use codex_app_server_protocol::ConfigEdit;
use codex_model_provider_info::AMAZON_BEDROCK_PROVIDER_ID;
use codex_model_provider_info::LMSTUDIO_OSS_PROVIDER_ID;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::OLLAMA_OSS_PROVIDER_ID;
use codex_model_provider_info::OPENAI_PROVIDER_ID;
use serde_json::json;

use crate::config_update::replace_config_value;

pub(crate) const VSP_PROVIDER_ID: &str = "vsp";
pub(crate) const VSP_PROVIDER_NAME: &str = "VSP Provider";

const PROVIDER_KEY_PREFIX: &str = "model_providers.vsp";
const OFFICIAL_SERVICE_DISABLED_MESSAGE: &str =
    "此官方服务入口已在 VSP Codex 中禁用。请使用 /provider 配置内部 Provider。";

pub(crate) fn build_vsp_provider_setup_edits(base_url: &str, api_key: &str) -> Vec<ConfigEdit> {
    vec![
        replace_config_value("model_provider", json!(VSP_PROVIDER_ID)),
        replace_config_value(
            format!("{PROVIDER_KEY_PREFIX}.name"),
            json!(VSP_PROVIDER_NAME),
        ),
        replace_config_value(format!("{PROVIDER_KEY_PREFIX}.base_url"), json!(base_url)),
        replace_config_value(
            format!("{PROVIDER_KEY_PREFIX}.experimental_bearer_token"),
            json!(api_key),
        ),
        replace_config_value(
            format!("{PROVIDER_KEY_PREFIX}.wire_api"),
            json!("responses"),
        ),
        replace_config_value(
            format!("{PROVIDER_KEY_PREFIX}.requires_openai_auth"),
            json!(false),
        ),
        replace_config_value(
            format!("{PROVIDER_KEY_PREFIX}.supports_websockets"),
            json!(false),
        ),
    ]
}

pub(crate) fn build_provider_selection_edits(provider_id: &str) -> Vec<ConfigEdit> {
    vec![replace_config_value("model_provider", json!(provider_id))]
}

pub(crate) fn official_services_disabled() -> bool {
    true
}

pub(crate) fn official_service_disabled_message() -> String {
    OFFICIAL_SERVICE_DISABLED_MESSAGE.to_string()
}

pub(crate) fn is_provider_allowed(provider_id: &str, provider: &ModelProviderInfo) -> bool {
    provider_id == VSP_PROVIDER_ID || is_user_configured_provider(provider_id, provider)
}

fn is_user_configured_provider(provider_id: &str, provider: &ModelProviderInfo) -> bool {
    !matches!(
        provider_id,
        OPENAI_PROVIDER_ID
            | AMAZON_BEDROCK_PROVIDER_ID
            | OLLAMA_OSS_PROVIDER_ID
            | LMSTUDIO_OSS_PROVIDER_ID
    ) && !provider.requires_openai_auth
}

pub(crate) fn provider_display_name(provider_id: &str, provider: &ModelProviderInfo) -> String {
    let name = provider.name.trim();
    if name.is_empty() || name == provider_id {
        provider_id.to_string()
    } else {
        format!("{name} ({provider_id})")
    }
}

pub(crate) fn provider_description(provider: &ModelProviderInfo) -> String {
    let endpoint = provider
        .base_url
        .as_deref()
        .map(str::trim)
        .filter(|base_url| !base_url.is_empty())
        .unwrap_or("default endpoint");
    let credential = if provider.experimental_bearer_token.is_some() {
        "API Key configured"
    } else if let Some(env_key) = provider.env_key.as_deref() {
        return format!("{endpoint} | API Key from {env_key}");
    } else if provider.auth.is_some() {
        "command auth"
    } else if provider.requires_openai_auth {
        "OpenAI auth"
    } else {
        "no credential configured"
    };

    format!("{endpoint} | {credential}")
}

pub(crate) fn is_provider_setup_complete(provider: &ModelProviderInfo) -> bool {
    provider
        .base_url
        .as_deref()
        .is_some_and(|base_url| !base_url.trim().is_empty())
        && provider
            .experimental_bearer_token
            .as_deref()
            .is_some_and(|api_key| !api_key.trim().is_empty())
}

pub(crate) fn normalize_base_url(input: &str) -> Result<String, String> {
    let trimmed = input.trim().trim_end_matches('/').to_string();
    if trimmed.is_empty() {
        return Err("Base URL 不能为空".to_string());
    }

    let parsed = url::Url::parse(&trimmed).map_err(|_| "Base URL 格式不正确".to_string())?;
    match parsed.scheme() {
        "http" | "https" => Ok(trimmed),
        _ => Err("Base URL 需要以 http:// 或 https:// 开头".to_string()),
    }
}

pub(crate) fn default_provider_base_url() -> String {
    std::env::var("VSP_CODEX_BASE_URL")
        .ok()
        .or_else(|| std::env::var("OPENAI_BASE_URL").ok())
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_default()
}

pub(crate) fn masked_secret(value: &str) -> String {
    let len = value.chars().count();
    if len == 0 {
        return String::new();
    }
    "*".repeat(len.clamp(8, 32))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn provider_description_does_not_include_bearer_token() {
        let provider = ModelProviderInfo {
            name: VSP_PROVIDER_NAME.to_string(),
            base_url: Some("https://provider.example/v1".to_string()),
            experimental_bearer_token: Some("sk-secret".to_string()),
            ..Default::default()
        };

        let description = provider_description(&provider);

        assert_eq!(
            description,
            "https://provider.example/v1 | API Key configured"
        );
        assert!(!description.contains("sk-secret"));
    }

    #[test]
    fn normalize_base_url_trims_trailing_slash() {
        assert_eq!(
            normalize_base_url(" https://provider.example/v1/ ").unwrap(),
            "https://provider.example/v1"
        );
    }

    #[test]
    fn built_in_providers_are_hidden_from_vsp_selection() {
        let provider = ModelProviderInfo {
            name: "OpenAI".to_string(),
            requires_openai_auth: true,
            ..Default::default()
        };

        assert!(!is_provider_allowed("openai", &provider));
        assert!(!is_provider_allowed(
            AMAZON_BEDROCK_PROVIDER_ID,
            &ModelProviderInfo {
                name: "Amazon Bedrock".to_string(),
                ..Default::default()
            }
        ));
        assert!(!is_provider_allowed(
            OLLAMA_OSS_PROVIDER_ID,
            &ModelProviderInfo {
                name: "Ollama".to_string(),
                ..Default::default()
            }
        ));
        assert!(!is_provider_allowed(
            LMSTUDIO_OSS_PROVIDER_ID,
            &ModelProviderInfo {
                name: "LM Studio".to_string(),
                ..Default::default()
            }
        ));
    }

    #[test]
    fn custom_provider_is_allowed() {
        let provider = ModelProviderInfo {
            name: "Custom Provider".to_string(),
            base_url: Some("https://provider.example/v1".to_string()),
            ..Default::default()
        };

        assert!(is_provider_allowed("custom", &provider));
    }

    #[test]
    fn vsp_provider_is_allowed_even_when_complete() {
        let provider = ModelProviderInfo {
            name: VSP_PROVIDER_NAME.to_string(),
            requires_openai_auth: false,
            ..Default::default()
        };

        assert!(is_provider_allowed(VSP_PROVIDER_ID, &provider));
    }
}
