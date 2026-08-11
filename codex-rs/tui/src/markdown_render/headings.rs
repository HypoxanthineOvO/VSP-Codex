use pulldown_cmark::HeadingLevel;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use unicode_width::UnicodeWidthStr;

use crate::terminal_hyperlinks::HyperlinkLine;

const H1_HORIZONTAL_PADDING: usize = 2;
const H2_GAP: usize = 1;
const H2_MIN_RULE_WIDTH: usize = 3;

pub(super) struct ActiveHeading {
    level: HeadingLevel,
    output_start: usize,
    initial_prefix: Vec<Span<'static>>,
    subsequent_prefix: Vec<Span<'static>>,
}

impl ActiveHeading {
    pub(super) fn new(
        level: HeadingLevel,
        output_start: usize,
        initial_prefix: Vec<Span<'static>>,
        subsequent_prefix: Vec<Span<'static>>,
    ) -> Self {
        Self {
            level,
            output_start,
            initial_prefix,
            subsequent_prefix,
        }
    }

    fn prefix_for_line(&self, index: usize) -> &[Span<'static>] {
        if index == 0 {
            &self.initial_prefix
        } else {
            &self.subsequent_prefix
        }
    }
}

pub(super) struct HeadingStyles {
    pub(super) h1: Style,
    pub(super) h2: Style,
    pub(super) h2_rule: Style,
    pub(super) blockquote: Option<Style>,
}

pub(super) fn decorate_heading(
    output: &mut Vec<HyperlinkLine>,
    heading: ActiveHeading,
    wrap_width: Option<usize>,
    styles: HeadingStyles,
) {
    let end = output.len();
    if heading.output_start >= end {
        return;
    }

    let mut content_lines = output.drain(heading.output_start..end).collect::<Vec<_>>();
    while content_lines.len() > 1 {
        let index = content_lines.len() - 1;
        if line_has_heading_content(&content_lines[index], heading.prefix_for_line(index)) {
            break;
        }
        content_lines.pop();
    }
    match heading.level {
        HeadingLevel::H1 => decorate_h1(
            &mut content_lines,
            &heading,
            wrap_width,
            styles.h1,
            styles.blockquote,
        ),
        HeadingLevel::H2 => decorate_h2(
            &mut content_lines,
            &heading,
            wrap_width,
            styles.h2,
            styles.h2_rule,
            styles.blockquote,
        ),
        HeadingLevel::H3 | HeadingLevel::H4 | HeadingLevel::H5 | HeadingLevel::H6 => {}
    }
    output.extend(content_lines);
}

fn decorate_h1(
    lines: &mut Vec<HyperlinkLine>,
    heading: &ActiveHeading,
    wrap_width: Option<usize>,
    style: Style,
    blockquote_style: Option<Style>,
) {
    let widest_content = lines
        .iter()
        .enumerate()
        .map(|(index, line)| content_width(line, heading.prefix_for_line(index)))
        .max()
        .unwrap_or_default();
    let band_width = available_width(
        wrap_width,
        heading.initial_prefix.width(),
        widest_content + H1_HORIZONTAL_PADDING * 2,
    );

    for (index, line) in lines.iter_mut().enumerate() {
        center_line(
            line,
            heading.prefix_for_line(index),
            band_width,
            style,
            Some(style),
        );
    }

    let top = surface_line(
        &heading.subsequent_prefix,
        band_width,
        style,
        blockquote_style,
    );
    let bottom = surface_line(
        &heading.subsequent_prefix,
        band_width,
        style,
        blockquote_style,
    );
    lines.insert(/*index*/ 0, top);
    lines.push(bottom);
}

fn decorate_h2(
    lines: &mut [HyperlinkLine],
    heading: &ActiveHeading,
    wrap_width: Option<usize>,
    title_style: Style,
    rule_style: Style,
    blockquote_style: Option<Style>,
) {
    let single_line = lines.len() == 1;
    for (index, line) in lines.iter_mut().enumerate() {
        let prefix = heading.prefix_for_line(index);
        let title_width = content_width(line, prefix);
        let available = available_width(
            wrap_width,
            prefix.width(),
            title_width + (H2_MIN_RULE_WIDTH + H2_GAP) * 2,
        );
        let remaining = available.saturating_sub(title_width + H2_GAP * 2);
        let rule_width = remaining / 2;
        if single_line && rule_width >= H2_MIN_RULE_WIDTH {
            frame_h2_line(
                line,
                prefix,
                rule_width,
                title_style,
                rule_style,
                blockquote_style,
            );
        } else {
            center_line(line, prefix, available, title_style, blockquote_style);
        }
    }
}

fn frame_h2_line(
    line: &mut HyperlinkLine,
    expected_prefix: &[Span<'static>],
    rule_width: usize,
    title_style: Style,
    rule_style: Style,
    blockquote_style: Option<Style>,
) {
    let (prefix, mut content) = take_prefix_and_content(line, expected_prefix.len());
    patch_spans(&mut content, title_style);
    let surrounding_style = blockquote_style.unwrap_or_default();
    let rule_style = rule_style.patch(surrounding_style);
    let leading_width = rule_width + H2_GAP;

    let mut spans = prefix;
    spans.push(Span::styled("─".repeat(rule_width), rule_style));
    spans.push(Span::styled(" ".repeat(H2_GAP), surrounding_style));
    spans.append(&mut content);
    spans.push(Span::styled(" ".repeat(H2_GAP), surrounding_style));
    spans.push(Span::styled("─".repeat(rule_width), rule_style));
    shift_hyperlinks(line, expected_prefix.width(), leading_width);
    line.line = Line::from(spans);
}

fn center_line(
    line: &mut HyperlinkLine,
    expected_prefix: &[Span<'static>],
    available: usize,
    content_style: Style,
    surrounding_style: Option<Style>,
) {
    let (prefix, mut content) = take_prefix_and_content(line, expected_prefix.len());
    patch_spans(&mut content, content_style);
    let content_width = content.width();
    let remaining = available.saturating_sub(content_width);
    let left_padding = remaining / 2;
    let right_padding = remaining - left_padding;
    let surrounding_style = surrounding_style.unwrap_or(content_style);

    let mut spans = prefix;
    spans.push(Span::styled(" ".repeat(left_padding), surrounding_style));
    spans.append(&mut content);
    spans.push(Span::styled(" ".repeat(right_padding), surrounding_style));
    shift_hyperlinks(line, expected_prefix.width(), left_padding);
    line.line = Line::from(spans);
}

fn surface_line(
    prefix: &[Span<'static>],
    width: usize,
    surface_style: Style,
    blockquote_style: Option<Style>,
) -> HyperlinkLine {
    let mut prefix = prefix.to_vec();
    if let Some(blockquote_style) = blockquote_style {
        patch_spans(&mut prefix, blockquote_style);
    }
    prefix.push(Span::styled(" ".repeat(width), surface_style));
    HyperlinkLine::new(Line::from(prefix))
}

fn take_prefix_and_content(
    line: &mut HyperlinkLine,
    expected_prefix_len: usize,
) -> (Vec<Span<'static>>, Vec<Span<'static>>) {
    let mut spans = std::mem::take(&mut line.line.spans);
    let mut content = spans.split_off(expected_prefix_len.min(spans.len()));
    while content.last().is_some_and(|span| {
        !span.content.is_empty() && span.content.chars().all(char::is_whitespace)
    }) {
        content.pop();
    }
    (spans, content)
}

fn patch_spans(spans: &mut [Span<'static>], style: Style) {
    for span in spans {
        span.style = span.style.patch(style);
    }
}

fn shift_hyperlinks(line: &mut HyperlinkLine, prefix_width: usize, shift: usize) {
    for hyperlink in &mut line.hyperlinks {
        if hyperlink.columns.start >= prefix_width {
            hyperlink.columns = hyperlink.columns.start + shift..hyperlink.columns.end + shift;
        }
    }
}

fn content_width(line: &HyperlinkLine, prefix: &[Span<'static>]) -> usize {
    line.line
        .spans
        .iter()
        .skip(prefix.len())
        .map(|span| span.content.as_ref())
        .collect::<String>()
        .trim_end_matches(char::is_whitespace)
        .width()
}

fn line_has_heading_content(line: &HyperlinkLine, prefix: &[Span<'static>]) -> bool {
    line.line.spans.iter().skip(prefix.len()).any(|span| {
        span.content
            .chars()
            .any(|character| !character.is_whitespace())
    })
}

fn available_width(
    wrap_width: Option<usize>,
    prefix_width: usize,
    intrinsic_width: usize,
) -> usize {
    wrap_width
        .map(|width| width.saturating_sub(prefix_width))
        .unwrap_or(intrinsic_width)
        .max(1)
}

trait SpanSliceWidth {
    fn width(&self) -> usize;
}

impl SpanSliceWidth for [Span<'static>] {
    fn width(&self) -> usize {
        self.iter().map(|span| span.content.width()).sum()
    }
}

#[cfg(test)]
#[path = "headings_tests.rs"]
mod tests;
