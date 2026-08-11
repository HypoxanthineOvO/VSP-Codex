use std::time::Duration;

use anyhow::Result;
use app_test_support::TestAppServer;
use app_test_support::to_response;
use codex_app_server_protocol::JSONRPCResponse;
use codex_app_server_protocol::ModelProviderListParams;
use codex_app_server_protocol::ModelProviderListResponse;
use codex_app_server_protocol::ModelProviderWriteParams;
use codex_app_server_protocol::ModelProviderWriteResponse;
use codex_app_server_protocol::RequestId;
use pretty_assertions::assert_eq;
use tempfile::TempDir;
use tokio::time::timeout;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

fn write_addon_config(codex_home: &TempDir) -> Result<()> {
    std::fs::write(
        codex_home.path().join("model-providers.toml"),
        r#"
version = 1

[providers.alpha]
name = "Alpha"
base_url = "https://alpha.example/v1"
env_key = "PATH"
wire_api = "responses"

[[providers.alpha.models]]
id = "alpha-model"
display_name = "Alpha Model"
context_window = 120000
supported_reasoning_efforts = ["low", "high"]
default_reasoning_effort = "high"
"#,
    )?;
    Ok(())
}

#[tokio::test]
async fn model_provider_list_and_write_are_owned_by_app_server() -> Result<()> {
    let codex_home = TempDir::new()?;
    write_addon_config(&codex_home)?;
    let mut app_server = TestAppServer::builder()
        .with_codex_home(codex_home.path())
        .build()
        .await?;
    timeout(DEFAULT_TIMEOUT, app_server.initialize()).await??;

    let list_id = app_server
        .send_model_provider_list_request(ModelProviderListParams {
            cursor: None,
            limit: None,
        })
        .await?;
    let response: JSONRPCResponse = timeout(
        DEFAULT_TIMEOUT,
        app_server.read_stream_until_response_message(RequestId::Integer(list_id)),
    )
    .await??;
    let providers = to_response::<ModelProviderListResponse>(response)?;
    let alpha = providers
        .data
        .iter()
        .find(|provider| provider.id == "alpha")
        .expect("alpha provider should be listed");
    assert_eq!(alpha.name, "Alpha");
    assert_eq!(alpha.credential_env_key.as_deref(), Some("PATH"));
    assert!(alpha.credential_available);
    assert!(alpha.is_addon);
    assert_eq!(alpha.models[0].id, "alpha-model");
    assert!(
        providers
            .data
            .iter()
            .any(|provider| provider.id == providers.base_provider_id && !provider.is_addon)
    );

    let write_id = app_server
        .send_model_provider_write_request(ModelProviderWriteParams {
            provider_id: "alpha".to_string(),
            model_id: "alpha-model".to_string(),
        })
        .await?;
    let response: JSONRPCResponse = timeout(
        DEFAULT_TIMEOUT,
        app_server.read_stream_until_response_message(RequestId::Integer(write_id)),
    )
    .await??;
    let _: ModelProviderWriteResponse = to_response(response)?;
    let persisted = std::fs::read_to_string(codex_home.path().join("model-providers.toml"))?;
    assert!(persisted.contains("default_provider = \"alpha\""));
    assert!(persisted.contains("default_model = \"alpha-model\""));

    let clear_id = app_server
        .send_model_provider_write_request(ModelProviderWriteParams {
            provider_id: providers.base_provider_id,
            model_id: "gpt-5.4".to_string(),
        })
        .await?;
    let response: JSONRPCResponse = timeout(
        DEFAULT_TIMEOUT,
        app_server.read_stream_until_response_message(RequestId::Integer(clear_id)),
    )
    .await??;
    let _: ModelProviderWriteResponse = to_response(response)?;
    let persisted = std::fs::read_to_string(codex_home.path().join("model-providers.toml"))?;
    assert!(!persisted.contains("default_provider"));
    assert!(!persisted.contains("default_model"));

    Ok(())
}
