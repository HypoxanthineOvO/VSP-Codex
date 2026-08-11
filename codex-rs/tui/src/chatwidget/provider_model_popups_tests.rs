use crate::app_event::AppEvent;
use crate::chatwidget::tests::helpers::make_chatwidget_manual_with_sender;
use crate::chatwidget::tests::helpers::render_bottom_popup;
use crate::legacy_core::config::AddonModel;
use crate::legacy_core::config::AddonModelProviders;
use crate::legacy_core::config::AddonProvider;
use codex_model_provider_info::ModelProviderInfo;
use codex_protocol::ThreadId;
use codex_protocol::openai_models::ReasoningEffort;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;

fn addon_provider(name: &str, env_key: &str, context_window: i64) -> AddonProvider {
    AddonProvider {
        name: name.to_string(),
        base_url: format!("https://{}.example/v1", name.to_lowercase()),
        env_key: env_key.to_string(),
        credential_available: None,
        models: vec![AddonModel {
            id: "shared-model".to_string(),
            display_name: format!("{name} Shared"),
            context_window,
            effective_context_window_percent: 95,
            supported_reasoning_efforts: vec![ReasoningEffort::Low, ReasoningEffort::High],
            default_reasoning_effort: Some(ReasoningEffort::High),
        }],
    }
}

#[tokio::test]
async fn provider_grouped_model_popup_snapshot() {
    let (mut chat, _tx, mut rx, _op_rx) = make_chatwidget_manual_with_sender().await;
    chat.thread_id = Some(ThreadId::new());
    let providers = BTreeMap::from([
        (
            "alpha".to_string(),
            addon_provider("Alpha", "PATH", 100_000),
        ),
        ("beta".to_string(), addon_provider("Beta", "PATH", 200_000)),
    ]);
    chat.config.addon_model_providers = Some(AddonModelProviders {
        version: 1,
        default_selection: None,
        providers: providers.clone(),
    });
    for (provider_id, provider) in providers {
        chat.config.model_providers.insert(
            provider_id,
            ModelProviderInfo {
                name: provider.name,
                base_url: Some(provider.base_url),
                env_key: Some(provider.env_key),
                ..Default::default()
            },
        );
    }
    chat.config.model_provider_id = "alpha".to_string();
    chat.config.model = Some("shared-model".to_string());
    chat.set_model_provider("alpha");
    chat.set_model("shared-model");

    chat.open_model_popup();

    insta::assert_snapshot!(
        "provider_grouped_model_popup",
        render_bottom_popup(&chat, /*width*/ 80)
    );
    insta::assert_snapshot!(
        "provider_grouped_model_popup_narrow",
        render_bottom_popup(&chat, /*width*/ 44)
    );
    chat.handle_key_event(KeyEvent::from(KeyCode::Enter));
    match rx.try_recv() {
        Ok(AppEvent::OpenProviderReasoningPopup { provider_id, model }) => {
            assert_eq!(
                (provider_id, model.model),
                ("alpha".to_string(), "shared-model".to_string())
            );
        }
        other => panic!("expected provider-aware reasoning popup event, got {other:?}"),
    }
}

#[tokio::test]
async fn provider_grouped_model_popup_single_provider_snapshot() {
    let (mut chat, _tx, _rx, _op_rx) = make_chatwidget_manual_with_sender().await;
    chat.thread_id = Some(ThreadId::new());
    let provider = addon_provider("Solo", "PATH", 100_000);
    chat.config.addon_model_providers = Some(AddonModelProviders {
        version: 1,
        default_selection: None,
        providers: BTreeMap::from([("solo".to_string(), provider.clone())]),
    });
    chat.config.model_providers.insert(
        "solo".to_string(),
        ModelProviderInfo {
            name: provider.name,
            base_url: Some(provider.base_url),
            env_key: Some(provider.env_key),
            ..Default::default()
        },
    );
    chat.config.model_provider_id = "solo".to_string();
    chat.config.model = Some("shared-model".to_string());
    chat.set_model_provider("solo");
    chat.set_model("shared-model");

    chat.open_model_popup();

    insta::assert_snapshot!(
        "provider_grouped_model_popup_single_provider",
        render_bottom_popup(&chat, /*width*/ 80)
    );
}

#[tokio::test]
async fn provider_grouped_model_popup_missing_credentials_snapshot() {
    let (mut chat, _tx, _rx, _op_rx) = make_chatwidget_manual_with_sender().await;
    chat.thread_id = Some(ThreadId::new());
    let provider = addon_provider("Missing", "C24_MISSING_API_KEY", 100_000);
    chat.config.addon_model_providers = Some(AddonModelProviders {
        version: 1,
        default_selection: None,
        providers: BTreeMap::from([("missing".to_string(), provider.clone())]),
    });
    chat.config.model_providers.insert(
        "missing".to_string(),
        ModelProviderInfo {
            name: provider.name,
            base_url: Some(provider.base_url),
            env_key: Some(provider.env_key),
            ..Default::default()
        },
    );
    chat.config.model_provider_id = "missing".to_string();
    chat.set_model_provider("missing");

    chat.open_model_popup();

    insta::assert_snapshot!(
        "provider_grouped_model_popup_missing_credentials",
        render_bottom_popup(&chat, /*width*/ 80)
    );
}
