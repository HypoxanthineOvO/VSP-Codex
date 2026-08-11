use super::*;
use codex_app_server_protocol::ModelProvider;
use codex_app_server_protocol::ModelProviderModel;
use codex_protocol::openai_models::ReasoningEffort;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn server_provider_catalog_uses_server_credential_state_and_current_provider() {
    let codex_home = tempfile::tempdir().expect("tempdir");
    let mut config = Config::load_default_with_cli_overrides_for_codex_home(
        codex_home.path().to_path_buf(),
        Vec::new(),
    )
    .await
    .expect("config");
    let response = ModelProviderListResponse {
        data: vec![
            ModelProvider {
                id: "openai".to_string(),
                name: "Remote Base".to_string(),
                base_url: None,
                credential_env_key: None,
                credential_available: true,
                is_addon: false,
                models: vec![ModelProviderModel {
                    id: "remote-base-model".to_string(),
                    display_name: "Remote Base Model".to_string(),
                    context_window: 256_000,
                    effective_context_window_percent: 95,
                    supported_reasoning_efforts: vec![ReasoningEffort::High],
                    default_reasoning_effort: Some(ReasoningEffort::High),
                }],
            },
            ModelProvider {
                id: "remote-addon".to_string(),
                name: "Remote Addon".to_string(),
                base_url: Some("https://remote.example/v1".to_string()),
                credential_env_key: Some("REMOTE_ADDON_KEY".to_string()),
                credential_available: true,
                is_addon: true,
                models: vec![ModelProviderModel {
                    id: "remote-model".to_string(),
                    display_name: "Remote Model".to_string(),
                    context_window: 128_000,
                    effective_context_window_percent: 95,
                    supported_reasoning_efforts: vec![ReasoningEffort::High],
                    default_reasoning_effort: Some(ReasoningEffort::High),
                }],
            },
        ],
        next_cursor: None,
        base_provider_id: "openai".to_string(),
        current_provider_id: "remote-addon".to_string(),
    };

    apply_server_provider_catalog(&mut config, &response, "remote-model");

    let addon = config
        .addon_model_providers
        .as_ref()
        .expect("remote addon catalog should be installed");
    assert_eq!(
        addon.providers["remote-addon"].credential_available,
        Some(true)
    );
    assert_eq!(
        addon.default_selection,
        Some(AddonModelSelection {
            provider_id: "remote-addon".to_string(),
            model_id: "remote-model".to_string(),
        })
    );
    assert_eq!(config.model_provider_id, "remote-addon");
    assert_eq!(config.base_model_provider_id, "openai");
    assert_eq!(config.model.as_deref(), Some("remote-model"));
    assert_eq!(
        config.model_catalog.as_ref().unwrap().models[0].slug,
        "remote-base-model"
    );
}
