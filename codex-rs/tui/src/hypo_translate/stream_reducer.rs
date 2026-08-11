use serde_json::Map;
use serde_json::Value;

#[derive(Debug, Eq, PartialEq)]
pub(super) enum SseParseResult {
    Pending,
    Error,
    Complete(String),
}

#[derive(Default)]
pub(super) struct SseDecoder {
    buffer: Vec<u8>,
    reducer: StreamReducer,
}

impl SseDecoder {
    pub(super) fn push(&mut self, chunk: &[u8]) -> SseParseResult {
        if !self.reducer.terminal() {
            self.buffer.extend_from_slice(chunk);
            self.decode_frames();
        }
        self.reducer.result()
    }

    pub(super) fn finish(&mut self) -> SseParseResult {
        if !self.reducer.terminal() {
            self.decode_frames();
            if !self.reducer.terminal() && !self.buffer.is_empty() {
                let frame = std::mem::take(&mut self.buffer);
                self.decode_frame(&frame);
            }
            self.reducer.finished |= !self.reducer.failed;
        }
        self.reducer.result()
    }

    fn decode_frames(&mut self) {
        while !self.reducer.terminal() {
            let Some((end, delimiter_width)) = frame_end(&self.buffer) else {
                break;
            };
            let frame = self.buffer[..end].to_vec();
            self.buffer.drain(..end + delimiter_width);
            self.decode_frame(&frame);
        }
    }

    fn decode_frame(&mut self, frame: &[u8]) {
        let Ok(frame) = std::str::from_utf8(frame) else {
            self.reducer.failed = true;
            return;
        };
        let mut event = "";
        let mut data = Vec::new();
        for line in frame.strip_prefix('\u{feff}').unwrap_or(frame).lines() {
            if line.starts_with(':') {
                continue;
            }
            let (field, value) = line.split_once(':').unwrap_or((line, ""));
            let value = value.strip_prefix(' ').unwrap_or(value);
            if field == "event" {
                event = value;
            } else if field == "data" {
                data.push(value);
            }
        }
        if !event.is_empty() || !data.is_empty() {
            self.reducer.apply(event, &data.join("\n"));
        }
    }
}

fn frame_end(buffer: &[u8]) -> Option<(usize, usize)> {
    let lf = buffer
        .windows(2)
        .position(|bytes| bytes == b"\n\n")
        .map(|end| (end, 2));
    let crlf = buffer
        .windows(4)
        .position(|bytes| bytes == b"\r\n\r\n")
        .map(|end| (end, 4));
    [lf, crlf].into_iter().flatten().min_by_key(|(end, _)| *end)
}

#[cfg(test)]
pub(super) fn parse_sse_stream_result(body: &str) -> SseParseResult {
    let mut decoder = SseDecoder::default();
    decoder.push(body.as_bytes())
}

#[cfg(test)]
pub(super) fn parse_sse_visible_text(body: &str) -> Option<String> {
    let mut decoder = SseDecoder::default();
    let result = match decoder.push(body.as_bytes()) {
        SseParseResult::Pending => decoder.finish(),
        result => result,
    };
    match result {
        SseParseResult::Complete(text) if !text.trim().is_empty() => Some(text),
        SseParseResult::Pending | SseParseResult::Complete(_) | SseParseResult::Error => None,
    }
}

#[derive(Default)]
struct StreamReducer {
    segments: Vec<Segment>,
    finished: bool,
    failed: bool,
    mode: StreamMode,
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
enum StreamMode {
    #[default]
    Unknown,
    Legacy,
    Strict,
}

struct Segment {
    id: Option<String>,
    revision: Option<String>,
    text: String,
    committed: bool,
}

impl StreamReducer {
    fn apply(&mut self, event_fallback: &str, data: &str) {
        let value = serde_json::from_str::<Value>(data).ok();
        let object = value.as_ref().and_then(Value::as_object);
        let event = object
            .and_then(|map| map.get("type"))
            .and_then(Value::as_str)
            .filter(|event| !event.is_empty())
            .unwrap_or(event_fallback);
        match event {
            "delta" | "revise" | "commit" | "passthrough" => {
                let text = visible_text(value.as_ref(), data);
                let strict = object.is_some_and(|map| {
                    ["segment_id", "revision_id", "replaces_revision_id"]
                        .iter()
                        .any(|key| map.contains_key(*key))
                });
                let event_mode = if strict {
                    StreamMode::Strict
                } else {
                    StreamMode::Legacy
                };
                if self.mode == StreamMode::Unknown {
                    self.mode = event_mode;
                }
                if self.mode != event_mode {
                    self.failed = true;
                    return;
                }
                let applied = if strict {
                    object
                        .and_then(|map| strict_fields(event, map))
                        .is_some_and(|(id, revision, replaces)| {
                            self.apply_strict(event, id, revision, replaces, text)
                        })
                } else {
                    self.apply_legacy(event, text)
                };
                if applied
                    && self.mode == StreamMode::Legacy
                    && matches!(event, "commit" | "passthrough")
                {
                    self.finished = true;
                }
                self.failed |= !applied;
            }
            "stats" => self.finished = true,
            "error" => self.failed = true,
            "start" | "source_hidden" | "" => {}
            _ => {}
        }
    }

    fn apply_strict(
        &mut self,
        event: &str,
        id: String,
        revision: Option<String>,
        replaces: Option<String>,
        text: String,
    ) -> bool {
        let Some(index) = self
            .segments
            .iter()
            .position(|segment| segment.id.as_deref() == Some(id.as_str()))
        else {
            if event == "passthrough" {
                self.segments.push(Segment {
                    id: Some(id),
                    revision,
                    text,
                    committed: true,
                });
                return true;
            }
            if event == "revise" || replaces.is_some() || revision.is_none() {
                return false;
            }
            self.segments.push(Segment {
                id: Some(id),
                revision,
                text,
                committed: event == "commit",
            });
            return true;
        };
        let segment = &mut self.segments[index];
        if segment.committed {
            return match event {
                "passthrough" if segment.text == text => true,
                "commit" if segment.text == text && segment.revision == revision => true,
                _ => false,
            };
        }
        let current = segment.revision.as_deref();
        match event {
            "delta" => {
                let Some(revision) = revision.as_deref() else {
                    return false;
                };
                if replaces.is_some() || current != Some(revision) {
                    return false;
                }
                segment.text.push_str(&text);
            }
            "revise" => {
                let Some(revision) = revision.as_deref() else {
                    return false;
                };
                if replaces.as_deref() != current || replaces.as_deref() == Some(revision) {
                    return false;
                }
                segment.revision = Some(revision.to_string());
                segment.text = text;
            }
            "commit" => {
                let Some(revision) = revision.as_deref() else {
                    return false;
                };
                let valid_replace = replaces
                    .as_deref()
                    .is_some_and(|replaced| replaced != revision && current == Some(replaced));
                let keeps_current = replaces.is_none() && current == Some(revision);
                if !(valid_replace || keeps_current) {
                    return false;
                }
                segment.revision = Some(revision.to_string());
                segment.text = text;
                segment.committed = true;
            }
            "passthrough" => {
                segment.revision = revision.or_else(|| segment.revision.clone());
                segment.text = text;
                segment.committed = true;
            }
            _ => return false,
        }
        true
    }

    fn apply_legacy(&mut self, event: &str, text: String) -> bool {
        let index = match self
            .segments
            .iter()
            .position(|segment| segment.id.is_none())
        {
            Some(index) => index,
            None => {
                self.segments.push(Segment {
                    id: None,
                    revision: None,
                    text: String::new(),
                    committed: false,
                });
                self.segments.len() - 1
            }
        };
        let segment = &mut self.segments[index];
        match event {
            "delta" if !segment.committed => segment.text.push_str(&text),
            "commit" | "passthrough" => {
                segment.text = text;
                segment.committed = true;
            }
            "revise" if !segment.committed => segment.text = text,
            "delta" | "revise" => {}
            _ => return false,
        }
        true
    }

    fn terminal(&self) -> bool {
        self.finished || self.failed
    }

    fn result(&self) -> SseParseResult {
        if self.failed {
            SseParseResult::Error
        } else if self.finished {
            SseParseResult::Complete(self.segments.iter().map(|segment| &*segment.text).collect())
        } else {
            SseParseResult::Pending
        }
    }
}

fn strict_fields(
    event: &str,
    map: &Map<String, Value>,
) -> Option<(String, Option<String>, Option<String>)> {
    let required = |key| map.get(key)?.as_str().filter(|value| !value.is_empty());
    let revision = map
        .get("revision_id")
        .and_then(|value| value.as_str().filter(|value| !value.is_empty()))
        .map(str::to_string);
    let replaces = match map.get("replaces_revision_id") {
        None => None,
        Some(value) => Some(value.as_str().filter(|value| !value.is_empty())?),
    };
    if event != "passthrough" && revision.is_none() {
        return None;
    }
    Some((
        required("segment_id")?.to_string(),
        revision,
        replaces.map(str::to_string),
    ))
}

fn visible_text(value: Option<&Value>, data: &str) -> String {
    match value {
        Some(Value::String(text)) => text.clone(),
        Some(Value::Object(map)) => ["text", "delta", "commit", "passthrough", "revise"]
            .into_iter()
            .find_map(|key| map.get(key).and_then(Value::as_str))
            .unwrap_or_default()
            .to_string(),
        Some(_) => String::new(),
        None => data.to_string(),
    }
}
