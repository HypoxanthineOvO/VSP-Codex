use anyhow::Result;
use codex_model_provider_info::ModelProviderInfo;
use codex_protocol::openai_models::ModelsResponse;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use core_test_support::responses;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use pretty_assertions::assert_eq;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn manual_compact_uses_selected_provider_catalog_and_endpoint() -> Result<()> {
    let server = responses::start_mock_server().await;
    let response_mock = responses::mount_sse_once(
        &server,
        responses::sse(vec![
            responses::ev_assistant_message("compact-message", "summary"),
            responses::ev_completed("compact-response"),
        ]),
    )
    .await;
    let base_url = format!("{}/v1", server.uri());
    let provider = ModelProviderInfo {
        name: "Alpha".to_string(),
        base_url: Some(base_url),
        env_key: Some("PATH".to_string()),
        ..Default::default()
    };
    let mut model_info = codex_models_manager::model_info::model_info_from_slug("shared-model");
    model_info.display_name = "Shared Model".to_string();
    model_info.context_window = Some(100_000);
    model_info.max_context_window = Some(100_000);
    model_info.used_fallback_model_metadata = false;

    let codex = test_codex()
        .with_config(move |config| {
            config.model_provider_id = "alpha".to_string();
            config.model_provider = provider;
            config.model = Some("shared-model".to_string());
            config.model_catalog = Some(ModelsResponse {
                models: vec![model_info],
            });
        })
        .build(&server)
        .await?
        .codex;

    codex.submit(Op::Compact).await?;
    wait_for_event(&codex, |event| matches!(event, EventMsg::TurnComplete(_))).await;

    let request = response_mock.single_request();
    assert_eq!(request.path(), "/v1/responses");
    assert_eq!(request.body_json()["model"], "shared-model");
    Ok(())
}
