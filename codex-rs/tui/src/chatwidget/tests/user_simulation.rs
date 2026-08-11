use super::*;
use codex_model_provider_info::ModelProviderInfo;
use pretty_assertions::assert_eq;
use std::time::Duration;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn type_text(chat: &mut ChatWidget, text: &str) {
    let flush_delay = crate::bottom_pane::ChatComposer::recommended_paste_flush_delay()
        + Duration::from_millis(1);

    for ch in text.chars() {
        chat.handle_key_event(key(KeyCode::Char(ch)));
        std::thread::sleep(flush_delay);
        chat.handle_paste_burst_tick(FrameRequester::test_dummy());
    }
}

#[tokio::test]
async fn user_typing_slash_prefix_filters_command_popup() {
    let (mut chat, _rx, _op_rx) = make_chatwidget_manual(Some("gpt-5.2")).await;

    type_text(&mut chat, "/mo");
    chat.pre_draw_tick();

    let popup = render_bottom_popup(&chat, /*width*/ 80);
    assert!(
        popup.contains("/model"),
        "expected slash command popup to include /model after typing /mo:\n{popup}"
    );
    assert_chatwidget_snapshot!("user_simulation_slash_model_filter", popup);
}

#[tokio::test]
async fn user_selecting_model_slash_command_opens_model_picker() {
    let (mut chat, _rx, _op_rx) = make_chatwidget_manual(Some("gpt-5.2")).await;
    chat.thread_id = Some(ThreadId::new());

    type_text(&mut chat, "/mo");
    chat.handle_key_event(key(KeyCode::Enter));

    let popup = render_bottom_popup(&chat, /*width*/ 80);
    assert!(
        popup.contains("Select Model"),
        "expected selecting /model from slash popup to open model picker:\n{popup}"
    );
    assert_chatwidget_snapshot!("user_simulation_model_picker_from_slash", popup);
}

#[tokio::test]
async fn user_typing_provider_slash_command_opens_provider_picker() {
    let (mut chat, _rx, _op_rx) = make_chatwidget_manual(Some("gpt-5.2")).await;
    chat.thread_id = Some(ThreadId::new());
    chat.config.model_providers.insert(
        crate::vsp_provider::VSP_PROVIDER_ID.to_string(),
        ModelProviderInfo {
            name: crate::vsp_provider::VSP_PROVIDER_NAME.to_string(),
            base_url: Some("https://provider.example/v1".to_string()),
            experimental_bearer_token: Some("sk-secret".to_string()),
            ..Default::default()
        },
    );
    chat.config.model_providers.insert(
        "openai".to_string(),
        ModelProviderInfo {
            name: "OpenAI".to_string(),
            base_url: Some("https://api.openai.com/v1".to_string()),
            requires_openai_auth: true,
            ..Default::default()
        },
    );

    type_text(&mut chat, "/pro");
    chat.handle_key_event(key(KeyCode::Enter));

    let popup = render_bottom_popup(&chat, /*width*/ 80);
    assert!(
        popup.contains("Select Provider"),
        "expected selecting /provider from slash popup to open provider picker:\n{popup}"
    );
    assert!(
        !popup.contains("sk-secret"),
        "provider popup must not render API keys:\n{popup}"
    );
    assert!(
        !popup.contains("OpenAI"),
        "provider popup must hide official OpenAI auth providers:\n{popup}"
    );
    assert_chatwidget_snapshot!("user_simulation_provider_picker_from_slash", popup);
}

#[tokio::test]
async fn user_typing_prompt_and_pressing_enter_submits_user_turn() {
    let (mut chat, _rx, mut op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
    chat.thread_id = Some(ThreadId::new());

    type_text(&mut chat, "Summarize repo status");
    chat.handle_key_event(key(KeyCode::Enter));

    match next_submit_op(&mut op_rx) {
        Op::UserTurn { items, .. } => assert_eq!(
            items,
            vec![UserInput::Text {
                text: "Summarize repo status".to_string(),
                text_elements: Vec::new(),
            }]
        ),
        other => panic!("expected prompt submission to produce Op::UserTurn, got {other:?}"),
    }
}
