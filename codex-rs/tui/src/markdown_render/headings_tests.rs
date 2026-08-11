use pretty_assertions::assert_eq;
use pulldown_cmark::HeadingLevel;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;

use super::*;

fn text(line: &HyperlinkLine) -> String {
    line.line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

fn styles() -> HeadingStyles {
    HeadingStyles {
        h1: Style::new().fg(Color::LightCyan).bg(Color::DarkGray).bold(),
        h2: Style::new().fg(Color::LightCyan).bold(),
        h2_rule: Style::new().fg(Color::Cyan).dim(),
        blockquote: None,
    }
}

#[test]
fn h1_is_a_centered_three_row_surface() {
    let mut output = vec![HyperlinkLine::new(Line::from(Span::raw("Title")))];
    let heading = ActiveHeading::new(
        HeadingLevel::H1,
        /*output_start*/ 0,
        Vec::new(),
        Vec::new(),
    );

    decorate_heading(&mut output, heading, Some(15), styles());

    assert_eq!(
        output.iter().map(text).collect::<Vec<_>>(),
        vec!["               ", "     Title     ", "               "]
    );
    assert!(output.iter().all(|line| line.width() == 15));
}

#[test]
fn h2_uses_equal_rules_and_drops_both_when_narrow() {
    let decorate = |width| {
        let mut output = vec![HyperlinkLine::new(Line::from(Span::raw("Title")))];
        let heading = ActiveHeading::new(
            HeadingLevel::H2,
            /*output_start*/ 0,
            Vec::new(),
            Vec::new(),
        );
        decorate_heading(&mut output, heading, Some(width), styles());
        text(&output[0])
    };

    assert_eq!(decorate(15), "──── Title ────");
    assert_eq!(decorate(10), "  Title   ");
}

#[test]
fn blockquote_prefix_stays_outside_the_heading_surface() {
    let quote = Style::new().bg(Color::Green);
    let prefix = vec![Span::styled("│ ", Style::new().fg(Color::Green))];
    let mut output = vec![HyperlinkLine::new(Line::from_iter([
        Span::styled("│ ", Style::new().fg(Color::Green).bg(Color::Green)),
        Span::styled("Title", Style::new().bg(Color::Green)),
    ]))];
    let heading = ActiveHeading::new(
        HeadingLevel::H1,
        /*output_start*/ 0,
        prefix.clone(),
        prefix,
    );
    let mut heading_styles = styles();
    heading_styles.blockquote = Some(quote);

    decorate_heading(&mut output, heading, Some(15), heading_styles);

    assert_eq!(
        output.iter().map(text).collect::<Vec<_>>(),
        vec!["│              ", "│     Title    ", "│              "]
    );
    assert!(output.iter().all(|line| line.width() == 15));
    assert_eq!(output[1].line.spans[0].style.bg, Some(Color::Green));
    assert_eq!(output[1].line.spans[1].style.bg, Some(Color::DarkGray));
}
