use super::AgentMarkdownCell;
use super::HistoryCell;
use super::ReasoningSummaryCell;
use super::wrap_translated;
use crate::hypo_translate::RuntimeTranslator;
use codex_config::types::TuiUiTheme;
use ratatui::style::Modifier;
use ratatui::style::Style;
use std::path::Path;
use std::time::Duration;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path;

struct UiThemeGuard(TuiUiTheme);

impl UiThemeGuard {
    fn new() -> Self {
        Self(crate::style::active_ui_theme())
    }
}

impl Drop for UiThemeGuard {
    fn drop(&mut self) {
        crate::style::set_ui_theme(self.0);
    }
}

fn find_span_style(cell: &dyn HistoryCell, content: &str) -> Option<Style> {
    cell.display_lines(/*width*/ 120)
        .into_iter()
        .find_map(|line| {
            line.spans
                .into_iter()
                .find(|span| span.content == content)
                .map(|span| span.style.patch(line.style))
        })
}

fn span_style(cell: &dyn HistoryCell, content: &str) -> Style {
    find_span_style(cell, content).unwrap_or_else(|| panic!("missing span {content:?}"))
}

async fn wait_for_span_style(cell: &dyn HistoryCell, content: &str) -> Style {
    for _ in 0..50 {
        if let Some(style) = find_span_style(cell, content) {
            return style;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("timed out waiting for span {content:?}");
}

#[test]
#[serial_test::serial]
fn reasoning_strong_is_not_a_link_but_real_links_and_assistant_styles_are_unchanged() {
    let _theme_guard = UiThemeGuard::new();
    for theme in [TuiUiTheme::Teal, TuiUiTheme::Violet, TuiUiTheme::Amber] {
        crate::style::set_ui_theme(theme);
        let source = "**Strong** and [docs](https://example.com/docs)";
        let reasoning = ReasoningSummaryCell::new(
            "Plan".to_string(),
            source.to_string(),
            Path::new("/workspace"),
            false,
        );
        let assistant = AgentMarkdownCell::new(source.to_string(), Path::new("/workspace"));

        let reasoning_strong = span_style(&reasoning, "Strong");
        assert!(reasoning_strong.add_modifier.contains(Modifier::BOLD));
        assert!(!reasoning_strong.add_modifier.contains(Modifier::UNDERLINED));
        assert!(
            span_style(&reasoning, "https://example.com/docs")
                .add_modifier
                .contains(Modifier::UNDERLINED)
        );
        assert!(
            span_style(&assistant, "Strong")
                .add_modifier
                .contains(Modifier::UNDERLINED)
        );
    }
}

#[tokio::test]
#[serial_test::serial]
async fn translated_reasoning_is_subtle_without_bold_or_strong_underline() {
    let _theme_guard = UiThemeGuard::new();
    crate::style::set_ui_theme(TuiUiTheme::Teal);
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/translate/stream"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(concat!(
                    "event: commit\n",
                    "data: {\"text\":\"Translated **strong** and [docs](https://example.com/docs)\"}\n\n",
                )),
        )
        .expect(1)
        .mount(&server)
        .await;
    let translator = RuntimeTranslator::test_with_base_url(server.uri());
    let cell = wrap_translated(
        Box::new(ReasoningSummaryCell::new(
            "Plan".to_string(),
            "Translate this reasoning.".to_string(),
            Path::new("/workspace"),
            false,
        )),
        &translator,
    );

    let ordinary = wait_for_span_style(&*cell, "Translated ").await;
    assert!(ordinary.add_modifier.contains(Modifier::DIM));
    assert!(ordinary.add_modifier.contains(Modifier::ITALIC));
    let strong = span_style(&*cell, "strong");
    assert!(!strong.add_modifier.contains(Modifier::BOLD));
    assert!(!strong.add_modifier.contains(Modifier::UNDERLINED));
    assert!(
        span_style(&*cell, "https://example.com/docs")
            .add_modifier
            .contains(Modifier::UNDERLINED)
    );

    for width in [20, 40, 80] {
        let lines = cell.display_lines(width);
        let visible = lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .map(|span| span.content.as_ref())
            .collect::<String>();
        assert!(
            !visible.contains("**"),
            "bare emphasis marker at width {width}"
        );

        let strong = lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .find(|span| span.content == "strong")
            .unwrap_or_else(|| panic!("missing strong span at width {width}"));
        assert!(!strong.style.add_modifier.contains(Modifier::BOLD));
        assert!(!strong.style.add_modifier.contains(Modifier::UNDERLINED));

        let link = lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .find(|span| span.content == "https://example.com/docs")
            .unwrap_or_else(|| panic!("missing link span at width {width}"));
        assert!(link.style.add_modifier.contains(Modifier::UNDERLINED));
    }

    insta::assert_debug_snapshot!("translated_reasoning_styles", cell.display_lines(80));
}
