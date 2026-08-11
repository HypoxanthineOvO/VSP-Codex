use super::*;
use crate::app::tests::make_test_app;
use crate::app::tests::next_thread_settings_updated;
use crate::app::tests::start_config_write_test_app_server;
use crate::app_server_session::TurnPermissionsOverride;
use crate::legacy_core::config::AddonModel;
use crate::legacy_core::config::AddonModelProviders;
use crate::legacy_core::config::AddonModelSelection;
use crate::legacy_core::config::AddonProvider;
use crate::test_support::PathBufExt;
use codex_app_server_client::AppServerEvent;
use codex_app_server_protocol::ServerNotification;
use codex_app_server_protocol::UserInput;
use codex_model_provider_info::ModelProviderInfo;
use color_eyre::eyre::Result;
use core_test_support::responses;
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;
use tempfile::tempdir;
use tokio::time;
use wiremock::MockServer;

#[test]
fn addon_provider_model_selection_updates_thread_and_rolls_back_on_persist_failure() -> Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_stack_size(8 * 1024 * 1024)
        .enable_all()
        .build()?;
    runtime.block_on(Box::pin(async {
        let mut app = make_test_app().await;
        let codex_home = tempdir()?;
        let server = MockServer::start().await;
        let response_mock = responses::mount_sse_once(
            &server,
            responses::sse(vec![
                responses::ev_response_created("response-1"),
                responses::ev_assistant_message("message-1", "done"),
                responses::ev_completed("response-1"),
            ]),
        )
        .await;
        let base_url = format!("{}/v1", server.uri());
        let addon_path = codex_home.path().join("model-providers.toml");
        std::fs::write(
            &addon_path,
            format!(
                r#"
version = 1

[providers.alpha]
name = "Alpha"
base_url = "{base_url}"
env_key = "PATH"
wire_api = "responses"

[[providers.alpha.models]]
id = "shared-model"
display_name = "Alpha Shared"
context_window = 100000
supported_reasoning_efforts = ["low", "high"]
default_reasoning_effort = "high"

[[providers.alpha.models]]
id = "other-model"
display_name = "Alpha Other"
context_window = 120000
supported_reasoning_efforts = ["low"]
default_reasoning_effort = "low"
"#
            ),
        )?;
        let provider = AddonProvider {
            name: "Alpha".to_string(),
            base_url,
            env_key: "PATH".to_string(),
            credential_available: None,
            models: vec![
                AddonModel {
                    id: "shared-model".to_string(),
                    display_name: "Alpha Shared".to_string(),
                    context_window: 100_000,
                    effective_context_window_percent: 95,
                    supported_reasoning_efforts: vec![ReasoningEffort::Low, ReasoningEffort::High],
                    default_reasoning_effort: Some(ReasoningEffort::High),
                },
                AddonModel {
                    id: "other-model".to_string(),
                    display_name: "Alpha Other".to_string(),
                    context_window: 120_000,
                    effective_context_window_percent: 95,
                    supported_reasoning_efforts: vec![ReasoningEffort::Low],
                    default_reasoning_effort: Some(ReasoningEffort::Low),
                },
            ],
        };
        app.config.codex_home = codex_home.path().to_path_buf().abs();
        app.config.addon_model_providers = Some(AddonModelProviders {
            version: 1,
            default_selection: None,
            providers: BTreeMap::from([("alpha".to_string(), provider.clone())]),
        });
        app.config.model_providers.insert(
            "alpha".to_string(),
            ModelProviderInfo {
                name: provider.name,
                base_url: Some(provider.base_url),
                env_key: Some(provider.env_key),
                ..Default::default()
            },
        );

        let mut app_server = start_config_write_test_app_server(&app).await?;
        let started = app_server.start_thread(&app.config).await?;
        let thread_id = started.session.thread_id;
        app.enqueue_primary_thread_session(started.session, started.turns)
            .await?;

        Box::pin(app.apply_provider_model_selection(
            &mut app_server,
            "alpha".to_string(),
            "shared-model".to_string(),
            Some(ReasoningEffort::High),
        ))
        .await;

        let notification = next_thread_settings_updated(&mut app_server, thread_id).await;
        assert_eq!(
            (
                notification.thread_settings.model_provider,
                notification.thread_settings.model,
                notification.thread_settings.effort,
            ),
            (
                "alpha".to_string(),
                "shared-model".to_string(),
                Some(ReasoningEffort::High),
            )
        );
        assert_eq!(
            app.config
                .addon_model_providers
                .as_ref()
                .and_then(|addon| addon.default_selection.clone()),
            Some(AddonModelSelection {
                provider_id: "alpha".to_string(),
                model_id: "shared-model".to_string(),
            })
        );
        let persisted = std::fs::read_to_string(&addon_path)?;
        assert!(persisted.contains("default_provider = \"alpha\""));
        assert!(persisted.contains("default_model = \"shared-model\""));

        app_server
            .turn_start(
                thread_id,
                vec![UserInput::Text {
                    text: "hello".to_string(),
                    text_elements: Vec::new(),
                }],
                app.config.cwd.to_path_buf(),
                app.config.permissions.approval_policy.value().into(),
                app.config.approvals_reviewer,
                TurnPermissionsOverride::Preserve,
                app.config.permissions.user_visible_workspace_roots(),
                "shared-model".to_string(),
                Some(ReasoningEffort::High),
                app.config.model_reasoning_summary,
                /*service_tier*/ None,
                /*collaboration_mode*/ None,
                app.config.personality,
                /*output_schema*/ None,
            )
            .await?;
        time::timeout(std::time::Duration::from_secs(/*secs*/ 10), async {
            loop {
                let event = app_server
                    .next_event()
                    .await
                    .expect("app-server event stream should remain open");
                if matches!(
                    event,
                    AppServerEvent::ServerNotification(ServerNotification::TurnCompleted(_))
                ) {
                    break;
                }
            }
        })
        .await
        .expect("turn should complete");
        let request = response_mock.single_request();
        assert_eq!(request.path(), "/v1/responses");
        assert_eq!(request.body_json()["model"], "shared-model");

        let resumed = app_server
            .resume_thread(app.config.clone(), thread_id)
            .await?;
        assert_eq!(
            (
                resumed.session.model_provider_id,
                resumed.session.model,
                resumed.session.reasoning_effort,
            ),
            (
                "alpha".to_string(),
                "shared-model".to_string(),
                Some(ReasoningEffort::High),
            )
        );
        let forked = app_server
            .fork_thread(app.config.clone(), thread_id)
            .await?;
        assert_eq!(
            (
                forked.session.model_provider_id,
                forked.session.model,
                forked.session.reasoning_effort,
            ),
            (
                "alpha".to_string(),
                "shared-model".to_string(),
                Some(ReasoningEffort::High),
            )
        );

        std::fs::remove_file(&addon_path)?;
        Box::pin(app.apply_provider_model_selection(
            &mut app_server,
            "alpha".to_string(),
            "other-model".to_string(),
            Some(ReasoningEffort::Low),
        ))
        .await;

        let attempted = next_thread_settings_updated(&mut app_server, thread_id).await;
        assert_eq!(attempted.thread_settings.model, "other-model");
        let rolled_back = next_thread_settings_updated(&mut app_server, thread_id).await;
        assert_eq!(
            (
                rolled_back.thread_settings.model_provider,
                rolled_back.thread_settings.model,
                rolled_back.thread_settings.effort,
            ),
            (
                "alpha".to_string(),
                "shared-model".to_string(),
                Some(ReasoningEffort::High),
            )
        );
        assert_eq!(app.config.model.as_deref(), Some("shared-model"));
        Ok(())
    }))
}
