use pretty_assertions::assert_eq;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;

use super::*;

#[test]
fn default_filter_disables_persistent_log_bridge_targets() {
    let filter = default_filter();
    assert_eq!(filter.would_enable("log", &tracing::Level::TRACE), false);
    assert_eq!(
        filter.would_enable("codex_otel.log_only", &tracing::Level::TRACE),
        false
    );
    assert_eq!(
        filter.would_enable("codex_otel.trace_safe", &tracing::Level::TRACE),
        false
    );
    assert_eq!(
        filter.would_enable("codex_state", &tracing::Level::TRACE),
        false
    );
    assert_eq!(
        filter.would_enable("codex_state", &tracing::Level::INFO),
        false
    );
    assert_eq!(
        filter.would_enable("codex_state", &tracing::Level::WARN),
        true
    );
    // transport TRACE captures full HTTP bodies; it must be suppressed at
    // TRACE/DEBUG/INFO but still surface genuine WARN/ERROR problems.
    assert_eq!(
        filter.would_enable("codex_http_client::transport", &tracing::Level::TRACE),
        false
    );
    assert_eq!(
        filter.would_enable("codex_http_client::transport", &tracing::Level::INFO),
        false
    );
    assert_eq!(
        filter.would_enable("codex_http_client::transport", &tracing::Level::WARN),
        true
    );
}

#[test]
fn feedback_log_body_is_capped() {
    let oversized = "x".repeat(MAX_FEEDBACK_LOG_BODY_BYTES * 2);
    let capped = truncate_feedback_log_body(oversized);
    assert!(capped.len() <= MAX_FEEDBACK_LOG_BODY_BYTES + "…[truncated]".len());
    assert!(capped.ends_with("…[truncated]"));

    let small = "short body".to_string();
    assert_eq!(truncate_feedback_log_body(small.clone()), small);
}

#[tokio::test]
async fn sqlite_sink_persists_only_actionable_logs_by_default() {
    let codex_home =
        std::env::temp_dir().join(format!("codex-state-log-db-filter-{}", Uuid::new_v4()));
    let runtime = StateRuntime::init(codex_home.clone(), "test-provider".to_string())
        .await
        .expect("initialize runtime");
    let layer = start(runtime.clone());

    let guard = tracing_subscriber::registry()
        .with(layer.clone().with_filter(default_filter()))
        .set_default();

    tracing::trace!(target: "opentelemetry_sdk", "dropped-trace");
    tracing::debug!(target: "opentelemetry_sdk", "dropped-debug");
    tracing::info!(target: "opentelemetry_sdk", "retained-info");
    tracing::trace!(target: "codex_state", "retained-trace");
    tracing::warn!(target: "codex_state", "retained-warning");
    tracing::trace!(
        target: "codex_api::responses_websocket_timing",
        payload = "complete timing payload",
        "dropped-websocket-timing"
    );

    layer.flush().await;
    drop(guard);

    let logs = runtime
        .query_logs(&crate::LogQuery::default())
        .await
        .expect("query logs after flush");
    assert_eq!(
        logs.iter()
            .map(|row| (
                row.level.as_str(),
                row.target.as_str(),
                row.message.as_deref()
            ))
            .collect::<Vec<_>>(),
        vec![("WARN", "codex_state", Some("retained-warning"))]
    );

    let _ = tokio::fs::remove_dir_all(codex_home).await;
}

#[tokio::test]
async fn sqlite_sink_drops_tracing_log_bridge_events_even_without_outer_filter() {
    let codex_home =
        std::env::temp_dir().join(format!("codex-state-log-db-filter-{}", Uuid::new_v4()));
    let runtime = StateRuntime::init(codex_home.clone(), "test-provider".to_string())
        .await
        .expect("initialize runtime");
    let layer = start(runtime.clone());

    let guard = tracing_subscriber::registry()
        .with(
            layer
                .clone()
                .with_filter(Targets::new().with_default(tracing::Level::TRACE)),
        )
        .set_default();

    tracing::trace!(target: "log", "dropped-bridge-trace");
    tracing::info!(target: "log", "dropped-bridge-info");
    tracing::info!(target: "codex_state", "retained-info");

    layer.flush().await;
    drop(guard);

    let logs = runtime
        .query_logs(&crate::LogQuery::default())
        .await
        .expect("query logs after flush");
    assert_eq!(
        logs.iter()
            .map(|row| (
                row.level.as_str(),
                row.target.as_str(),
                row.message.as_deref()
            ))
            .collect::<Vec<_>>(),
        vec![("INFO", "codex_state", Some("retained-info"))]
    );

    let _ = tokio::fs::remove_dir_all(codex_home).await;
}
