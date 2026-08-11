//! Shared history-cell building blocks reused across transcript concerns.

use super::*;

#[derive(Debug)]
pub(crate) struct PlainHistoryCell {
    pub(super) lines: Vec<Line<'static>>,
}

impl PlainHistoryCell {
    pub(crate) fn new(lines: Vec<Line<'static>>) -> Self {
        Self { lines }
    }
}

impl HistoryCell for PlainHistoryCell {
    fn display_lines(&self, _width: u16) -> Vec<Line<'static>> {
        self.lines.clone()
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        plain_lines(self.lines.clone())
    }
}

#[derive(Debug)]
pub(crate) struct TranslatedHistoryCell {
    inner: Box<dyn HistoryCell>,
    translated: TranslatedText,
    display: TranslationDisplay,
    cwd: Option<PathBuf>,
}

impl TranslatedHistoryCell {
    pub(crate) fn new(
        inner: Box<dyn HistoryCell>,
        source: TranslationSource,
        translator: &RuntimeTranslator,
        cwd: Option<PathBuf>,
    ) -> Self {
        let display = source.display;
        Self {
            inner,
            translated: translator.translate(source),
            display,
            cwd,
        }
    }

    fn translated_lines(&self, width: u16) -> Vec<Line<'static>> {
        let text = self
            .translated
            .translated()
            .unwrap_or_else(|| self.translated.original().to_string());
        let Some(wrap_width) =
            crate::width::usable_content_width_u16(width, /*reserved_cols*/ 2)
        else {
            return prefix_lines(vec![Line::default()], "• ".dim(), "  ".into());
        };

        let mut lines = Vec::new();
        append_reasoning_markdown(
            text.trim(),
            Some(wrap_width),
            self.cwd.as_deref(),
            &mut lines,
        );
        let translated_style = Style::default()
            .fg(active_palette().subtle_border)
            .add_modifier(Modifier::DIM | Modifier::ITALIC)
            .remove_modifier(Modifier::BOLD);
        for line in &mut lines {
            for span in &mut line.spans {
                span.style = span.style.patch(translated_style);
            }
        }
        prefix_lines(lines, "• ".dim(), "  ".into())
    }

    #[cfg(test)]
    pub(crate) fn translated_text_for_test(&self) -> Option<String> {
        self.translated.translated()
    }
}

impl HistoryCell for TranslatedHistoryCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        match self.display {
            TranslationDisplay::Replace => {
                if self.inner.display_lines(width).is_empty() {
                    Vec::new()
                } else {
                    self.translated_lines(width)
                }
            }
        }
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        self.inner.raw_lines()
    }

    fn transcript_lines(&self, width: u16) -> Vec<Line<'static>> {
        match self.display {
            TranslationDisplay::Replace => {
                if self.inner.display_lines(width).is_empty() && self.inner.raw_lines().is_empty() {
                    Vec::new()
                } else {
                    self.translated_lines(width)
                }
            }
        }
    }

    fn transcript_animation_tick(&self) -> Option<u64> {
        self.inner.transcript_animation_tick()
    }
}

pub(crate) fn wrap_translated(
    cell: Box<dyn HistoryCell>,
    translator: &RuntimeTranslator,
) -> Box<dyn HistoryCell> {
    if let Some(reasoning) = cell.as_any().downcast_ref::<ReasoningSummaryCell>()
        && let Some(source) = reasoning.translation_source()
    {
        let cwd = Some(reasoning.cwd().to_path_buf());
        return Box::new(TranslatedHistoryCell::new(cell, source, translator, cwd));
    }
    cell
}

pub(crate) fn wrap_collab_translated(
    cell: PlainHistoryCell,
    _translator: &RuntimeTranslator,
) -> Box<dyn HistoryCell> {
    Box::new(cell)
}

#[derive(Debug)]
pub(crate) struct WebHyperlinkHistoryCell {
    lines: Vec<Line<'static>>,
}

impl WebHyperlinkHistoryCell {
    pub(crate) fn new(lines: Vec<Line<'static>>) -> Self {
        Self { lines }
    }
}

impl HistoryCell for WebHyperlinkHistoryCell {
    fn display_lines(&self, _width: u16) -> Vec<Line<'static>> {
        self.lines.clone()
    }

    fn display_hyperlink_lines(&self, _width: u16) -> Vec<HyperlinkLine> {
        crate::terminal_hyperlinks::annotate_web_urls(self.lines.clone())
    }

    fn transcript_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        self.display_hyperlink_lines(width)
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        plain_lines(self.lines.clone())
    }
}

#[derive(Debug)]
pub(crate) struct PrefixedWrappedHistoryCell {
    text: Text<'static>,
    initial_prefix: Line<'static>,
    subsequent_prefix: Line<'static>,
}

impl PrefixedWrappedHistoryCell {
    pub(crate) fn new(
        text: impl Into<Text<'static>>,
        initial_prefix: impl Into<Line<'static>>,
        subsequent_prefix: impl Into<Line<'static>>,
    ) -> Self {
        Self {
            text: text.into(),
            initial_prefix: initial_prefix.into(),
            subsequent_prefix: subsequent_prefix.into(),
        }
    }
}

impl HistoryCell for PrefixedWrappedHistoryCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        if width == 0 {
            return Vec::new();
        }
        let opts = RtOptions::new(width.max(1) as usize)
            .initial_indent(self.initial_prefix.clone())
            .subsequent_indent(self.subsequent_prefix.clone());
        adaptive_wrap_lines(&self.text, opts)
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        plain_lines(self.text.clone().lines)
    }
}
#[derive(Debug)]
pub(crate) struct CompositeHistoryCell {
    pub(super) parts: Vec<Box<dyn HistoryCell>>,
}

impl CompositeHistoryCell {
    pub(crate) fn new(parts: Vec<Box<dyn HistoryCell>>) -> Self {
        Self { parts }
    }
}

impl HistoryCell for CompositeHistoryCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        let mut out: Vec<Line<'static>> = Vec::new();
        let mut first = true;
        for part in &self.parts {
            let mut lines = part.display_lines(width);
            if !lines.is_empty() {
                if !first {
                    out.push(Line::from(""));
                }
                out.append(&mut lines);
                first = false;
            }
        }
        out
    }

    fn display_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        let mut out = Vec::new();
        let mut first = true;
        for part in &self.parts {
            let mut lines = part.display_hyperlink_lines(width);
            if !lines.is_empty() {
                if !first {
                    out.push(HyperlinkLine::from(""));
                }
                out.append(&mut lines);
                first = false;
            }
        }
        out
    }

    fn transcript_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        let mut out = Vec::new();
        let mut first = true;
        for part in &self.parts {
            let mut lines = part.transcript_hyperlink_lines(width);
            if !lines.is_empty() {
                if !first {
                    out.push(HyperlinkLine::from(""));
                }
                out.append(&mut lines);
                first = false;
            }
        }
        out
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        let mut out: Vec<Line<'static>> = Vec::new();
        let mut first = true;
        for part in &self.parts {
            let mut lines = part.raw_lines();
            if !lines.is_empty() {
                if !first {
                    out.push(Line::from(""));
                }
                out.append(&mut lines);
                first = false;
            }
        }
        out
    }
}
