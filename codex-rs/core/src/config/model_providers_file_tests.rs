use super::*;
use crate::config::ConfigBuilder;
use codex_model_provider_info::WireApi;
use pretty_assertions::assert_eq;
use tempfile::tempdir;

fn valid_config() -> &'static str {
    r#"
version = 1
default_provider = "deepseek"
default_model = "deepseek-v4-flash"

[providers.deepseek]
name = "DeepSeek"
base_url = "https://api.deepseek.com"
env_key = "DEEPSEEK_API_KEY"
wire_api = "responses"

[[providers.deepseek.models]]
id = "deepseek-v4-flash"
display_name = "DeepSeek V4 Flash"
context_window = 1048576
supported_reasoning_efforts = ["low", "high", "max"]
default_reasoning_effort = "high"
"#
}

#[test]
fn parses_versioned_responses_provider_catalog() {
    let parsed = parse_model_providers_toml(valid_config()).expect("valid config");

    assert_eq!(
        parsed,
        Some(AddonModelProviders {
            version: 1,
            default_selection: Some(AddonModelSelection {
                provider_id: "deepseek".to_string(),
                model_id: "deepseek-v4-flash".to_string(),
            }),
            providers: BTreeMap::from([(
                "deepseek".to_string(),
                AddonProvider {
                    name: "DeepSeek".to_string(),
                    base_url: "https://api.deepseek.com".to_string(),
                    env_key: "DEEPSEEK_API_KEY".to_string(),
                    credential_available: None,
                    models: vec![AddonModel {
                        id: "deepseek-v4-flash".to_string(),
                        display_name: "DeepSeek V4 Flash".to_string(),
                        context_window: 1_048_576,
                        effective_context_window_percent: 95,
                        supported_reasoning_efforts: vec![
                            ReasoningEffort::Low,
                            ReasoningEffort::High,
                            ReasoningEffort::Max,
                        ],
                        default_reasoning_effort: Some(ReasoningEffort::High),
                    }],
                },
            )]),
        })
    );
}

#[test]
fn whitespace_only_file_is_no_addon_layer() {
    assert_eq!(parse_model_providers_toml(" \n\t").unwrap(), None);
}

#[test]
fn same_model_id_is_allowed_across_providers() {
    let config = r#"
version = 1

[providers.alpha]
name = "Alpha"
base_url = "https://alpha.example/v1"
env_key = "ALPHA_API_KEY"
wire_api = "responses"

[[providers.alpha.models]]
id = "shared-model"
display_name = "Alpha Shared"
context_window = 100000

[providers.beta]
name = "Beta"
base_url = "https://beta.example/v1"
env_key = "BETA_API_KEY"
wire_api = "responses"

[[providers.beta.models]]
id = "shared-model"
display_name = "Beta Shared"
context_window = 200000
"#;

    let parsed = parse_model_providers_toml(config)
        .expect("cross-provider model ids are scoped by provider")
        .expect("addon layer");

    assert_eq!(
        parsed
            .providers
            .values()
            .map(|provider| provider.models[0].context_window)
            .collect::<Vec<_>>(),
        vec![100_000, 200_000]
    );
    assert_eq!(
        parsed
            .model_catalog("alpha")
            .and_then(|catalog| catalog.models[0].context_window),
        Some(100_000)
    );
    assert_eq!(
        parsed
            .model_catalog("beta")
            .and_then(|catalog| catalog.models[0].context_window),
        Some(200_000)
    );
}

#[test]
fn rejects_unsupported_version_and_unknown_fields() {
    for (config, expected) in [
        (
            valid_config().replacen("version = 1", "version = 2", 1),
            "unsupported version 2",
        ),
        (
            format!("{}\nunexpected = true\n", valid_config()),
            "unknown field `unexpected`",
        ),
    ] {
        let error = parse_model_providers_toml(&config).unwrap_err();
        assert!(
            error.to_string().contains(expected),
            "expected {expected:?} in {error}"
        );
    }
}

#[test]
fn rejects_incomplete_or_dangling_default_selection() {
    for (config, expected) in [
        (
            valid_config().replace("default_model = \"deepseek-v4-flash\"\n", ""),
            "default_provider and default_model must be set together",
        ),
        (
            valid_config().replace(
                "default_model = \"deepseek-v4-flash\"",
                "default_model = \"missing\"",
            ),
            "default model `missing` is not defined by provider `deepseek`",
        ),
    ] {
        let error = parse_model_providers_toml(&config).unwrap_err();
        assert!(
            error.to_string().contains(expected),
            "expected {expected:?} in {error}"
        );
    }
}

#[test]
fn external_effective_provider_default_is_resolved_during_merge() {
    let config = valid_config()
        .replace(
            "default_provider = \"deepseek\"",
            "default_provider = \"vsp\"",
        )
        .replace(
            "default_model = \"deepseek-v4-flash\"",
            "default_model = \"gpt-main\"",
        );
    let parsed = parse_model_providers_toml(&config).unwrap().unwrap();
    let mut registry = HashMap::from([(
        "vsp".to_string(),
        ModelProviderInfo {
            name: "VSP".to_string(),
            ..Default::default()
        },
    )]);

    merge_addon_model_providers(&mut registry, Some(&parsed)).unwrap();

    assert!(registry.contains_key("vsp"));
    assert!(registry.contains_key("deepseek"));
}

#[test]
fn merge_rejects_unknown_external_default_provider() {
    let config = valid_config().replace(
        "default_provider = \"deepseek\"",
        "default_provider = \"missing\"",
    );
    let parsed = parse_model_providers_toml(&config).unwrap().unwrap();
    let mut registry = HashMap::new();

    let error = merge_addon_model_providers(&mut registry, Some(&parsed)).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("default provider `missing` in model-providers.toml"),
        "{error}"
    );
}

#[test]
fn rejects_ambiguous_or_unsafe_provider_contracts() {
    for (config, expected) in [
        (
            valid_config().replace("wire_api = \"responses\"", "wire_api = \"chat\""),
            "wire_api",
        ),
        (
            valid_config().replace(
                "https://api.deepseek.com",
                "https://user@example.com/v1?key=value#fragment",
            ),
            "base_url",
        ),
        (
            valid_config().replace("DEEPSEEK_API_KEY", "not an env key"),
            "env_key",
        ),
        (
            valid_config().replace("context_window = 1048576", "context_window = 0"),
            "context_window",
        ),
        (
            valid_config().replace(
                "default_reasoning_effort = \"high\"",
                "default_reasoning_effort = \"ultra\"",
            ),
            "default_reasoning_effort",
        ),
    ] {
        let error = parse_model_providers_toml(&config).unwrap_err();
        assert!(
            error.to_string().contains(expected),
            "expected {expected:?} in {error}"
        );
    }
}

#[test]
fn rejects_duplicate_model_ids_within_provider() {
    let duplicate = r#"
[[providers.deepseek.models]]
id = "deepseek-v4-flash"
display_name = "Duplicate"
context_window = 1048576
"#;
    let config = format!("{}\n{duplicate}", valid_config());

    let error = parse_model_providers_toml(&config).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("duplicate model id `deepseek-v4-flash` in provider `deepseek`"),
        "{error}"
    );
}

#[tokio::test]
async fn missing_file_loads_as_no_addon_layer() {
    let codex_home = tempdir().unwrap();

    assert_eq!(
        load_model_providers_file(codex_home.path()).await.unwrap(),
        None
    );
}

#[tokio::test]
async fn config_builder_merges_addon_provider_and_applies_atomic_default() {
    let codex_home = tempdir().unwrap();
    std::fs::write(
        codex_home.path().join(MODEL_PROVIDERS_TOML_FILE),
        valid_config(),
    )
    .unwrap();

    let config = ConfigBuilder::without_managed_config_for_tests()
        .codex_home(codex_home.path().to_path_buf())
        .build()
        .await
        .unwrap();

    assert_eq!(config.model_provider_id, "deepseek");
    assert_eq!(config.model.as_deref(), Some("deepseek-v4-flash"));
    assert_eq!(
        config.model_providers.get("deepseek"),
        Some(&ModelProviderInfo {
            name: "DeepSeek".to_string(),
            base_url: Some("https://api.deepseek.com".to_string()),
            env_key: Some("DEEPSEEK_API_KEY".to_string()),
            wire_api: WireApi::Responses,
            requires_openai_auth: false,
            supports_websockets: false,
            ..Default::default()
        })
    );
    assert_eq!(
        config
            .addon_model_providers
            .as_ref()
            .and_then(|addon| addon.default_selection.as_ref()),
        Some(&AddonModelSelection {
            provider_id: "deepseek".to_string(),
            model_id: "deepseek-v4-flash".to_string(),
        })
    );
}

#[tokio::test]
async fn config_builder_rejects_addon_provider_id_collision() {
    let codex_home = tempdir().unwrap();
    std::fs::write(
        codex_home.path().join("config.toml"),
        r#"
[model_providers.deepseek]
name = "Official-layer DeepSeek"
base_url = "https://official-layer.example/v1"
env_key = "OFFICIAL_LAYER_API_KEY"
wire_api = "responses"
"#,
    )
    .unwrap();
    std::fs::write(
        codex_home.path().join(MODEL_PROVIDERS_TOML_FILE),
        valid_config(),
    )
    .unwrap();

    let error = ConfigBuilder::without_managed_config_for_tests()
        .codex_home(codex_home.path().to_path_buf())
        .build()
        .await
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("provider `deepseek` in model-providers.toml conflicts"),
        "{error}"
    );
}

#[test]
fn persistence_only_updates_addon_defaults_and_preserves_catalog() {
    let codex_home = tempdir().unwrap();
    let config_path = codex_home.path().join("config.toml");
    let addon_path = codex_home.path().join(MODEL_PROVIDERS_TOML_FILE);
    let official_config = "model = \"gpt-official\"\n";
    let addon_config = valid_config()
        .replace("default_provider = \"deepseek\"\n", "")
        .replace("default_model = \"deepseek-v4-flash\"\n", "")
        .replace(
            "[providers.deepseek]",
            "# provider comment\n[providers.deepseek]",
        );
    std::fs::write(&config_path, official_config).unwrap();
    std::fs::write(&addon_path, addon_config).unwrap();
    let selection = AddonModelSelection {
        provider_id: "deepseek".to_string(),
        model_id: "deepseek-v4-flash".to_string(),
    };

    persist_addon_default_selection(codex_home.path(), Some(&selection)).unwrap();

    assert_eq!(
        std::fs::read_to_string(&config_path).unwrap(),
        official_config
    );
    let updated = std::fs::read_to_string(&addon_path).unwrap();
    assert!(updated.contains("# provider comment"));
    assert!(updated.contains("default_provider = \"deepseek\""));
    assert!(updated.contains("default_model = \"deepseek-v4-flash\""));
    assert_eq!(
        parse_model_providers_toml(&updated)
            .unwrap()
            .and_then(|addon| addon.default_selection),
        Some(selection)
    );

    persist_addon_default_selection(codex_home.path(), None).unwrap();

    let cleared = std::fs::read_to_string(&addon_path).unwrap();
    assert!(!cleared.contains("default_provider"));
    assert!(!cleared.contains("default_model"));
    assert!(cleared.contains("# provider comment"));
    assert_eq!(
        std::fs::read_to_string(config_path).unwrap(),
        official_config
    );
}
