//! Runtime client for the local Hypo-Translate service.

mod stream_reducer;

use crate::app_event::AppEvent;
use crate::app_event_sender::AppEventSender;
use crate::tui::FrameRequester;
use reqwest::StatusCode;
use reqwest::header::ACCEPT;
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;
use stream_reducer::SseDecoder;
use stream_reducer::SseParseResult;
#[cfg(test)]
use stream_reducer::parse_sse_stream_result;
#[cfg(test)]
use stream_reducer::parse_sse_visible_text;
use tracing::debug;

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8998";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(Clone, Copy, Debug)]
pub(crate) enum TranslationScope {
    Reasoning,
    Subagent,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum TranslationDisplay {
    Replace,
}

#[derive(Clone, Debug)]
pub(crate) struct TranslationSource {
    pub(crate) text: String,
    pub(crate) scope: TranslationScope,
    pub(crate) display: TranslationDisplay,
}

#[derive(Clone, Debug)]
pub(crate) struct TranslatedText {
    original: String,
    translated: Arc<RwLock<Option<String>>>,
}

impl TranslatedText {
    fn new(original: String) -> Self {
        Self {
            original,
            translated: Arc::new(RwLock::new(None)),
        }
    }

    pub(crate) fn original(&self) -> &str {
        &self.original
    }

    pub(crate) fn translated(&self) -> Option<String> {
        self.translated.read().ok().and_then(|guard| guard.clone())
    }

    fn set_translated(&self, translated: String) -> bool {
        if translated.trim().is_empty() || translated.trim() == self.original.trim() {
            return false;
        }
        if let Ok(mut guard) = self.translated.write() {
            *guard = Some(translated);
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RuntimeTranslator {
    client: reqwest::Client,
    base_url: Arc<str>,
    frame_requester: FrameRequester,
    app_event_tx: Option<AppEventSender>,
}

impl RuntimeTranslator {
    pub(crate) fn new(frame_requester: FrameRequester, app_event_tx: AppEventSender) -> Self {
        let base_url = std::env::var("HYPO_TRANSLATE_BASE_URL")
            .ok()
            .filter(|url| !url.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        Self {
            client: reqwest::Client::new(),
            base_url: Arc::from(base_url.trim_end_matches('/')),
            frame_requester,
            app_event_tx: Some(app_event_tx),
        }
    }

    pub(crate) fn translate(&self, source: TranslationSource) -> TranslatedText {
        let translated = TranslatedText::new(source.text.clone());
        if source.text.trim().is_empty() {
            return translated;
        }

        let task_result = translated.clone();
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let frame_requester = self.frame_requester.clone();
        let app_event_tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let result = translate_with_fallback(client, &base_url, &source.text).await;
            match result {
                Some(text) => {
                    let updated = task_result.set_translated(text);
                    if updated {
                        frame_requester.schedule_frame();
                        if let Some(app_event_tx) = app_event_tx {
                            app_event_tx.send(AppEvent::TranslatedHistoryCellUpdated);
                        }
                    }
                }
                None => {
                    debug!(scope = ?source.scope, "hypo translate unavailable; using original text");
                }
            }
        });

        translated
    }

    #[cfg(test)]
    pub(crate) fn test_with_base_url(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: Arc::from(base_url.trim_end_matches('/')),
            frame_requester: FrameRequester::test_dummy(),
            app_event_tx: None,
        }
    }
}

#[derive(Serialize)]
struct TranslateRequest<'a> {
    text: &'a str,
    source_lang: &'static str,
    target_lang: &'static str,
}

fn request_body(text: &str) -> TranslateRequest<'_> {
    TranslateRequest {
        text,
        source_lang: "auto",
        target_lang: "zh-CN",
    }
}

async fn translate_with_fallback(
    client: reqwest::Client,
    base_url: &str,
    text: &str,
) -> Option<String> {
    if let Some(streamed) = translate_stream(&client, base_url, text).await
        && !streamed.trim().is_empty()
    {
        return Some(streamed);
    }
    translate_json(&client, base_url, text).await
}

async fn translate_stream(client: &reqwest::Client, base_url: &str, text: &str) -> Option<String> {
    let response = tokio::time::timeout(
        REQUEST_TIMEOUT,
        client
            .post(format!("{base_url}/translate/stream"))
            .header(ACCEPT, "text/event-stream")
            .json(&request_body(text))
            .send(),
    )
    .await
    .ok()?
    .ok()?;
    if response.status() != StatusCode::OK {
        return None;
    }
    let mut response = response;
    let mut decoder = SseDecoder::default();
    loop {
        let chunk = tokio::time::timeout(REQUEST_TIMEOUT, response.chunk())
            .await
            .ok()?
            .ok()?;
        let Some(chunk) = chunk else {
            break;
        };
        match decoder.push(&chunk) {
            SseParseResult::Complete(text) => return Some(text),
            SseParseResult::Error => return None,
            SseParseResult::Pending => {}
        }
    }
    match decoder.finish() {
        SseParseResult::Complete(text) => Some(text),
        SseParseResult::Error | SseParseResult::Pending => None,
    }
}

async fn translate_json(client: &reqwest::Client, base_url: &str, text: &str) -> Option<String> {
    let response = tokio::time::timeout(
        REQUEST_TIMEOUT,
        client
            .post(format!("{base_url}/translate"))
            .header(CONTENT_TYPE, "application/json")
            .json(&request_body(text))
            .send(),
    )
    .await
    .ok()?
    .ok()?;
    if response.status() != StatusCode::OK {
        return None;
    }
    let value = tokio::time::timeout(REQUEST_TIMEOUT, response.json::<TranslateResponse>())
        .await
        .ok()?
        .ok()?;
    value.into_text().filter(|text| !text.trim().is_empty())
}

#[derive(Deserialize)]
struct TranslateResponse {
    text: Option<String>,
    translation: Option<String>,
    translated_text: Option<String>,
    output: Option<String>,
}

impl TranslateResponse {
    fn into_text(self) -> Option<String> {
        self.translation
            .or(self.translated_text)
            .or(self.text)
            .or(self.output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;
    use wiremock::matchers::method;
    use wiremock::matchers::path;

    #[test]
    fn parses_json_data_line_without_event_using_type_field() {
        let body = r#"data: {"type":"commit","text":"译文"}"#;

        assert_eq!(parse_sse_visible_text(body), Some("译文".to_string()));
    }

    #[test]
    fn parses_message_event_json_data_using_type_field() {
        let body = concat!(
            "event: message\n",
            "data: {\"type\":\"commit\",\"text\":\"译文\"}\n\n",
        );

        assert_eq!(parse_sse_visible_text(body), Some("译文".to_string()));
    }

    #[test]
    fn parser_returns_complete_commit_before_unfinished_long_connection_fragment() {
        let body = concat!(
            "event: commit\n",
            "data: {\"text\":\"译文\"}\n",
            "\n",
            "event: delta\n",
            "data: should not wait for this unfinished fragment",
        );

        assert_eq!(parse_sse_visible_text(body), Some("译文".to_string()));
    }

    #[test]
    fn stream_parser_keeps_delta_pending_until_commit() {
        let body = concat!(
            "event: delta\n",
            "data: {\"text\":\"译\"}\n",
            "\n",
            "event: delta\n",
            "data: {\"text\":\"文\"}\n",
            "\n",
        );

        assert_eq!(parse_sse_stream_result(body), SseParseResult::Pending);
        assert_eq!(parse_sse_visible_text(body), Some("译文".to_string()));
    }

    #[test]
    fn source_hidden_and_stats_are_not_visible_text() {
        let body = concat!(
            "event: source_hidden\n",
            "data: {\"text\":\"secret source\"}\n",
            "\n",
            "event: commit\n",
            "data: {\"text\":\"译文\"}\n",
            "\n",
            "event: stats\n",
            "data: {\"text\":\"token counts\"}\n",
            "\n",
        );

        assert_eq!(parse_sse_visible_text(body), Some("译文".to_string()));
    }

    #[tokio::test]
    async fn stream_error_falls_back_to_translate_json() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/translate/stream"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "text/event-stream")
                    .set_body_string("event: error\ndata: {\"message\":\"boom\"}\n\n"),
            )
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/translate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "translation": "JSON 译文"
            })))
            .expect(1)
            .mount(&server)
            .await;

        let translated =
            translate_with_fallback(reqwest::Client::new(), &server.uri(), "original").await;

        assert_eq!(translated, Some("JSON 译文".to_string()));
    }

    #[tokio::test]
    async fn non_success_stream_response_falls_back_to_translate_json() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/translate/stream"))
            .respond_with(ResponseTemplate::new(503))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/translate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "text": "回退译文"
            })))
            .expect(1)
            .mount(&server)
            .await;

        let translated =
            translate_with_fallback(reqwest::Client::new(), &server.uri(), "original").await;

        assert_eq!(translated, Some("回退译文".to_string()));
    }

    #[tokio::test]
    async fn translate_failure_returns_none_for_passthrough_callers() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/translate/stream"))
            .respond_with(ResponseTemplate::new(503))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/translate"))
            .respond_with(ResponseTemplate::new(503))
            .expect(1)
            .mount(&server)
            .await;

        let translated =
            translate_with_fallback(reqwest::Client::new(), &server.uri(), "original").await;

        assert_eq!(translated, None);
    }
}

#[cfg(test)]
#[path = "hypo_translate_stream_tests.rs"]
mod stream_tests;
