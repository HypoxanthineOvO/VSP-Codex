use super::SseDecoder;
use super::SseParseResult;
use super::parse_sse_stream_result;
use super::parse_sse_visible_text;
use pretty_assertions::assert_eq;

#[test]
fn commit_replaces_deltas_without_creating_bare_emphasis_markers() {
    let body = concat!(
        "data: {\"type\":\"delta\",\"segment_id\":\"seg-1\",\"revision_id\":\"rev-1\",\"text\":\"**规划**\"}\n\n",
        "data: {\"type\":\"commit\",\"segment_id\":\"seg-1\",\"revision_id\":\"rev-1\",\"text\":\"**规划**\"}\n\n",
        "data: {\"type\":\"commit\",\"segment_id\":\"seg-2\",\"revision_id\":\"rev-1\",\"text\":\"保留相同 segment 文本\"}\n\n",
        "data: {\"type\":\"commit\",\"segment_id\":\"seg-3\",\"revision_id\":\"rev-1\",\"text\":\"保留相同 segment 文本\"}\n\n",
        "data: {\"type\":\"stats\"}\n\n",
    );

    assert_eq!(
        parse_sse_visible_text(body),
        Some("**规划**保留相同 segment 文本保留相同 segment 文本".to_string())
    );
}

#[test]
fn revise_commit_and_passthrough_are_authoritative_per_segment() {
    let body = concat!(
        "data: {\"type\":\"delta\",\"segment_id\":\"seg-1\",\"revision_id\":\"rev-1\",\"text\":\"old draft\"}\n\n",
        "data: {\"type\":\"revise\",\"segment_id\":\"seg-1\",\"revision_id\":\"rev-2\",\"replaces_revision_id\":\"rev-1\",\"text\":\"revised draft\"}\n\n",
        "data: {\"type\":\"commit\",\"segment_id\":\"seg-1\",\"revision_id\":\"rev-2\",\"text\":\"translated final\"}\n\n",
        "data: {\"type\":\"passthrough\",\"segment_id\":\"seg-2\",\"text\":\"original `x = 1`\"}\n\n",
        "data: {\"type\":\"stats\"}\n\n",
    );

    assert_eq!(
        parse_sse_visible_text(body),
        Some("translated finaloriginal `x = 1`".to_string())
    );
}

#[test]
fn unknown_replacement_revision_fails_closed() {
    let body = concat!(
        "data: {\"type\":\"delta\",\"segment_id\":\"seg-1\",\"revision_id\":\"rev-1\",\"text\":\"draft\"}\n\n",
        "data: {\"type\":\"revise\",\"segment_id\":\"seg-1\",\"revision_id\":\"rev-2\",\"replaces_revision_id\":\"missing\",\"text\":\"untrusted\"}\n\n",
        "data: {\"type\":\"stats\"}\n\n",
    );

    assert_eq!(parse_sse_stream_result(body), SseParseResult::Error);
    assert_eq!(parse_sse_visible_text(body), None);
}

#[test]
fn strict_stream_waits_for_stats_but_eof_keeps_complete_text() {
    let body = concat!(
        "data: {\"type\":\"delta\",\"segment_id\":\"seg-1\",\"revision_id\":\"rev-1\",\"text\":\"complete \"}\n\n",
        "data: {\"type\":\"delta\",\"segment_id\":\"seg-1\",\"revision_id\":\"rev-1\",\"text\":\"text\"}\n\n",
    );

    assert_eq!(parse_sse_stream_result(body), SseParseResult::Pending);
    assert_eq!(
        parse_sse_visible_text(body),
        Some("complete text".to_string())
    );
}

#[test]
fn decoder_preserves_utf8_across_single_byte_chunks() {
    let body = concat!(
        "data: {\"type\":\"delta\",\"segment_id\":\"seg-1\",\"revision_id\":\"rev-1\",\"text\":\"你\"}\r\n\r\n",
        "data: {\"type\":\"commit\",\"segment_id\":\"seg-1\",\"revision_id\":\"rev-1\",\"text\":\"你好，世界\"}\r\n\r\n",
        "data: {\"type\":\"stats\"}\r\n\r\n",
    );
    let mut decoder = SseDecoder::default();
    for byte in body.as_bytes() {
        let result = decoder.push(std::slice::from_ref(byte));
        if result != SseParseResult::Pending {
            assert_eq!(result, SseParseResult::Complete("你好，世界".to_string()));
            return;
        }
    }
    panic!("stats should complete the stream");
}

#[test]
fn legacy_commit_remains_an_end_of_stream_signal() {
    assert_eq!(
        parse_sse_stream_result("data: {\"type\":\"commit\",\"text\":\"legacy final\"}\n\n"),
        SseParseResult::Complete("legacy final".to_string())
    );
}
