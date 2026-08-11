use pretty_assertions::assert_eq;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::text::Text;
use std::path::Path;
use unicode_width::UnicodeWidthStr;

use crate::markdown_render::COLON_LOCATION_SUFFIX_RE;
use crate::markdown_render::HASH_LOCATION_SUFFIX_RE;
use crate::markdown_render::render_markdown_lines_with_width_and_cwd;
use crate::markdown_render::render_markdown_text;
use crate::markdown_render::render_markdown_text_with_width;
use crate::markdown_render::render_markdown_text_with_width_and_cwd;
use insta::assert_debug_snapshot;
use insta::assert_snapshot;

struct UiThemeGuard(codex_config::types::TuiUiTheme);

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

fn render_markdown_text_for_cwd(input: &str, cwd: &Path) -> Text<'static> {
    render_markdown_text_with_width_and_cwd(input, /*width*/ None, Some(cwd))
}

fn plain_lines(text: &Text<'_>) -> Vec<String> {
    text.lines
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.clone())
                .collect::<String>()
        })
        .collect()
}

fn line_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

#[test]
fn prose_preserves_spaces_between_ascii_words_and_cjk() {
    let input =
        "我们不应直接照搬 Remodeled Trace，而应先结合硬件文档、论文、现有 failure 和 simulator 需求";

    let rendered = render_markdown_text_with_width(input, /*width*/ Some(200));

    assert_eq!(plain_lines(&rendered), vec![input.to_string()]);
}

fn with_blockquote_background(mut text: Text<'static>) -> Text<'static> {
    let quote_background = Style::default().bg(Color::DarkGray);
    for line in &mut text.lines {
        let Some(rail_index) = line
            .spans
            .iter()
            .position(|span| span.content.contains('│'))
        else {
            continue;
        };
        for span in &mut line.spans[rail_index..] {
            span.style = span.style.patch(quote_background);
        }
    }
    text
}

#[test]
fn bare_url_with_tilde_keeps_complete_hyperlink() {
    let destination =
        "https://www.cs.tufts.edu/~nr/cs257/archive/olin-shivers/dissertation.pdf";
    let lines = render_markdown_lines_with_width_and_cwd(
        destination,
        /*width*/ Some(80),
        /*cwd*/ None,
    );
    let rendered = lines
        .iter()
        .map(|line| {
            let text = line
                .line
                .spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>();
            let hyperlinks = line
                .hyperlinks
                .iter()
                .map(|link| (link.columns.clone(), link.destination.as_str()))
                .collect::<Vec<_>>();
            (text, hyperlinks)
        })
        .collect::<Vec<_>>();

    assert_debug_snapshot!(rendered);
}

#[test]
fn table_url_with_tilde_keeps_complete_hyperlink() {
    let destination =
        "https://www.cs.tufts.edu/~nr/cs257/archive/olin-shivers/dissertation.pdf";
    let markdown = format!("| URL |\n| --- |\n| {destination} |\n");
    let lines = render_markdown_lines_with_width_and_cwd(
        &markdown,
        /*width*/ Some(32),
        /*cwd*/ None,
    );
    let destinations = lines
        .iter()
        .flat_map(|line| line.hyperlinks.iter())
        .map(|link| link.destination.as_str())
        .collect::<Vec<_>>();

    assert!(!destinations.is_empty());
    assert_eq!(destinations, vec![destination; destinations.len()]);
}

#[test]
fn merged_text_events_preserve_entity_decoding() {
    let source = "https://example.com/a&amp;b~c";
    let destination = "https://example.com/a&b~c";
    let lines = render_markdown_lines_with_width_and_cwd(
        source,
        /*width*/ Some(80),
        /*cwd*/ None,
    );
    let rendered = lines
        .iter()
        .map(|line| {
            let text = line
                .line
                .spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>();
            let hyperlinks = line
                .hyperlinks
                .iter()
                .map(|link| (link.columns.clone(), link.destination.as_str()))
                .collect::<Vec<_>>();
            (text, hyperlinks)
        })
        .collect::<Vec<_>>();

    assert_eq!(
        rendered,
        vec![(
            destination.to_string(),
            vec![(0..destination.len(), destination)],
        )],
    );
}

#[test]
fn empty() {
    assert_eq!(render_markdown_text(""), Text::default());
}

#[test]
fn paragraph_single() {
    assert_eq!(
        render_markdown_text("Hello, world!"),
        Text::from("Hello, world!")
    );
}

#[test]
fn paragraph_soft_break() {
    assert_eq!(
        render_markdown_text("Hello\nWorld"),
        Text::from_iter(["Hello", "World"])
    );
}

#[test]
fn paragraph_multiple() {
    assert_eq!(
        render_markdown_text("Paragraph 1\n\nParagraph 2"),
        Text::from_iter(["Paragraph 1", "", "Paragraph 2"])
    );
}

#[test]
#[serial_test::serial]
fn headings() {
    let _theme_guard = UiThemeGuard::new();
    crate::style::set_ui_theme(codex_config::types::TuiUiTheme::Teal);
    let md = "# Heading 1\n## Heading 2\n### Heading 3\n#### Heading 4\n##### Heading 5\n###### Heading 6\n";
    let text = render_markdown_text(md);
    let palette = crate::style::active_palette();
    let h1 = crate::style::heading1_style(palette);
    let h2 = Style::new().fg(palette.strong).bold();
    let h2_rule = Style::new().fg(palette.structure).dim();
    let expected = Text::from_iter([
        Line::from(Span::styled("             ", h1)),
        Line::from_iter([
            Span::styled("  ", h1),
            Span::styled("Heading 1", h1),
            Span::styled("  ", h1),
        ]),
        Line::from(Span::styled("             ", h1)),
        Line::default(),
        Line::from_iter([
            Span::styled("───", h2_rule),
            Span::raw(" "),
            Span::styled("Heading 2", h2),
            Span::raw(" "),
            Span::styled("───", h2_rule),
        ]),
        Line::default(),
        Line::from("Heading 3".green().bold()),
        Line::default(),
        Line::from("Heading 4".magenta().italic()),
        Line::default(),
        Line::from("Heading 5".green().italic()),
        Line::default(),
        Line::from("Heading 6".green().italic()),
    ]);
    assert_eq!(text, expected);
}

#[test]
#[serial_test::serial]
fn heading_visual_contract_snapshot() {
    let _theme_guard = UiThemeGuard::new();
    crate::style::set_ui_theme(codex_config::types::TuiUiTheme::Teal);
    let rendered = render_markdown_text_with_width(
        "# Primary heading\n\n## Secondary heading\n\n### Tertiary heading\n\n---\n",
        Some(40),
    );

    assert_debug_snapshot!(rendered);
}

#[test]
#[serial_test::serial]
fn heading_hierarchy_and_rule_are_width_aware_across_themes() {
    use codex_config::types::TuiUiTheme;

    let _theme_guard = UiThemeGuard::new();
    let markdown = "# H1\n## H2\n### H3\n#### H4\n\n---\n";
    let line_text = |line: &Line<'_>| {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>()
    };
    for theme in [TuiUiTheme::Teal, TuiUiTheme::Violet, TuiUiTheme::Amber] {
        crate::style::set_ui_theme(theme);
        for width in [40, 80, 120] {
            let text = render_markdown_text_with_width(markdown, Some(width));
            let visible = text
                .lines
                .iter()
                .filter(|line| line.width() > 0)
                .collect::<Vec<_>>();
            assert_eq!(visible.len(), 7);
            assert!(line_text(visible[0]).trim().is_empty());
            assert_eq!(line_text(visible[1]).trim(), "H1");
            assert!(line_text(visible[2]).trim().is_empty());
            assert_eq!(line_text(visible[3]).trim_matches(['─', ' ']), "H2");
            assert_eq!(line_text(visible[4]), "H3");
            assert_eq!(line_text(visible[5]), "H4");
            assert!(visible.iter().all(|line| !line_text(line).contains('#')),);
            assert!(visible[..3].iter().all(|line| line.width() == width));
            assert!(visible[..3].iter().all(|line| {
                line.spans.iter().all(|span| {
                    span.style.bg.is_some() && span.style.add_modifier.contains(Modifier::BOLD)
                })
            }));

            let h2 = &visible[3].spans[2].style;
            assert_eq!(h2.fg, Some(crate::style::active_palette().strong));
            assert!(h2.add_modifier.contains(Modifier::BOLD));
            assert!(!h2.add_modifier.contains(Modifier::UNDERLINED));
            let left_rule = &visible[3].spans[0];
            let right_rule = &visible[3].spans[4];
            assert_eq!(left_rule.content.width(), right_rule.content.width());
            assert_eq!(left_rule.style, right_rule.style);
            let h3 = &visible[4].spans[0].style;
            assert_eq!(h3.fg, Some(crate::style::active_palette().emphasis));
            assert!(h3.add_modifier.contains(Modifier::BOLD));
            let h4 = &visible[5].spans[0].style;
            assert_eq!(h4.fg, Some(Color::Magenta));
            assert!(h4.add_modifier.contains(Modifier::ITALIC));
            assert!(!h4.add_modifier.contains(Modifier::BOLD));

            let rule = visible[6];
            assert_eq!(rule.width(), width);
            assert!(line_text(rule).chars().all(|ch| ch == '─'));
        }
    }
}

#[test]
fn heading_layout_wraps_h1_and_drops_both_h2_rules_when_narrow() {
    let rendered = render_markdown_text_with_width(
        "# A long heading\n\n## Secondary\n",
        /*width*/ Some(12),
    );
    let separator = rendered
        .lines
        .iter()
        .position(|line| line.width() == 0)
        .expect("heading levels should remain separated");
    let h1 = &rendered.lines[..separator];
    let h2 = rendered
        .lines
        .iter()
        .skip(separator + 1)
        .find(|line| line.width() > 0)
        .expect("second-level heading");

    assert!(h1.len() > 3, "wrapped H1 should grow within its surface");
    assert!(h1.iter().all(|line| line.width() == 12));
    assert!(
        h1.iter()
            .all(|line| line.spans.iter().all(|span| span.style.bg.is_some()))
    );
    assert_eq!(line_text(h2).trim(), "Secondary");
    assert!(!line_text(h2).contains('─'));
    assert_eq!(h2.width(), 12);
}

#[test]
fn centered_heading_links_keep_their_visible_columns() {
    for markdown in [
        "# [Title](https://example.com)\n",
        "## [Title](https://example.com)\n",
    ] {
        let rendered = render_markdown_lines_with_width_and_cwd(
            markdown,
            /*width*/ Some(20),
            /*cwd*/ None,
        );
        let line = rendered
            .iter()
            .find(|line| line.line.spans.iter().any(|span| span.content == "Title"))
            .expect("heading title line");

        let title_start = line
            .line
            .spans
            .iter()
            .take_while(|span| span.content != "Title")
            .map(|span| span.content.width())
            .sum::<usize>();
        assert!(line.hyperlinks.iter().any(|link| {
            link.columns == (title_start..title_start + "Title".width())
                && link.destination == "https://example.com"
        }));
    }
}

#[test]
fn heading_long_tokens_wrap_without_losing_hyperlinks() {
    let destination = "https://example.com/a-very-long-heading-token";
    let markdown = format!("# [{destination}]({destination})\n");
    let rendered = render_markdown_lines_with_width_and_cwd(
        &markdown,
        /*width*/ Some(20),
        /*cwd*/ None,
    );

    assert!(rendered.iter().all(|line| line.width() <= 20));
    let linked_fragments = rendered
        .iter()
        .flat_map(|line| &line.hyperlinks)
        .collect::<Vec<_>>();
    assert!(linked_fragments.len() > 1, "URL label should wrap");
    assert!(
        linked_fragments
            .iter()
            .all(|link| link.destination == destination)
    );

    let long_word = "supercalifragilisticexpialidocioussupercalifragilistic";
    let rendered = render_markdown_text_with_width(&format!("# {long_word}\n"), Some(20));
    assert!(rendered.lines.iter().all(|line| line.width() <= 20));
    assert_eq!(
        plain_lines(&rendered)
            .join("")
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>(),
        long_word
    );
}

#[test]
fn headings_keep_quote_and_list_container_prefixes_within_width() {
    for markdown in [
        "> # Quoted heading with enough words to wrap\n",
        "- # Listed heading with enough words to wrap\n",
    ] {
        let rendered = render_markdown_text_with_width(markdown, Some(24));
        let plain = plain_lines(&rendered).join("\n");

        assert!(rendered.lines.iter().all(|line| line.width() <= 24));
        assert_eq!(plain.matches("heading").count(), 1);
        assert!(!plain.contains('#'));
    }
}

#[test]
fn thematic_rule_respects_blockquote_and_list_container_widths() {
    let width = 24;
    for markdown in ["> ---\n", "- item\n\n  ---\n"] {
        let text = render_markdown_text_with_width(markdown, Some(width));
        let rule = text
            .lines
            .iter()
            .find(|line| {
                line.spans
                    .iter()
                    .any(|span| span.content.contains('─'))
            })
            .expect("expected thematic rule");
        assert_eq!(rule.width(), width);
    }
}

#[test]
fn blockquote_single() {
    let text = render_markdown_text("> Blockquote");
    let expected = with_blockquote_background(Text::from_iter([
        Line::from("│ ".green()),
        Line::from_iter(["│ ".green(), "Blockquote".into()]),
        Line::from("│ ".green()),
    ]));
    assert_eq!(text, expected);
}

#[test]
fn blockquote_soft_break() {
    // Soft break via lazy continuation should render as a new line in blockquotes.
    let text = render_markdown_text("> This is a blockquote\nwith a soft break\n");
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect();
    assert_eq!(
        lines,
        vec![
            "│ ".to_string(),
            "│ This is a blockquote".to_string(),
            "│ with a soft break".to_string(),
            "│ ".to_string(),
        ]
    );
}

#[test]
fn blockquote_multiple_with_break() {
    let text = render_markdown_text("> Blockquote 1\n\n> Blockquote 2\n");
    let expected = with_blockquote_background(Text::from_iter([
        Line::from("│ ".green()),
        Line::from_iter(["│ ".green(), "Blockquote 1".into()]),
        Line::from("│ ".green()),
        Line::default(),
        Line::from("│ ".green()),
        Line::from_iter(["│ ".green(), "Blockquote 2".into()]),
        Line::from("│ ".green()),
    ]));
    assert_eq!(text, expected);
}

#[test]
fn blockquote_three_paragraphs_short_lines() {
    let md = "> one\n>\n> two\n>\n> three\n";
    let text = render_markdown_text(md);
    let expected = with_blockquote_background(Text::from_iter([
        Line::from("│ ".green()),
        Line::from_iter(["│ ".green(), "one".into()]),
        Line::from_iter(["│ ".green()]),
        Line::from_iter(["│ ".green(), "two".into()]),
        Line::from_iter(["│ ".green()]),
        Line::from_iter(["│ ".green(), "three".into()]),
        Line::from("│ ".green()),
    ]));
    assert_eq!(text, expected);
}

#[test]
fn blockquote_nested_two_levels() {
    let md = "> Level 1\n>> Level 2\n";
    let text = render_markdown_text(md);
    let expected = with_blockquote_background(Text::from_iter([
        Line::from("│ ".green()),
        Line::from_iter(["│ ".green(), "Level 1".into()]),
        Line::from_iter(["│ ".green()]),
        Line::from_iter(["│ ".green(), "│ ".green()]),
        Line::from_iter(["│ ".green(), "│ ".green(), "Level 2".into()]),
        Line::from_iter(["│ ".green(), "│ ".green()]),
        Line::from("│ ".green()),
    ]));
    assert_eq!(text, expected);
}

#[test]
fn blockquote_with_list_items() {
    let md = "> - item 1\n> - item 2\n";
    let text = render_markdown_text(md);
    let expected = with_blockquote_background(Text::from_iter([
        Line::from("│ ".green()),
        Line::from_iter(vec![
            "│ ".green(),
            "• ".light_blue(),
            Span::from("item 1"),
        ]),
        Line::from_iter(vec![
            "│ ".green(),
            "• ".light_blue(),
            Span::from("item 2"),
        ]),
        Line::from("│ ".green()),
    ]));
    assert_eq!(text, expected);
}

#[test]
fn blockquote_with_ordered_list() {
    let md = "> 1. first\n> 2. second\n";
    let text = render_markdown_text(md);
    let expected = with_blockquote_background(Text::from_iter([
        Line::from("│ ".green()),
        Line::from_iter(vec![
            "│ ".green(),
            "1. ".light_blue(),
            Span::from("first"),
        ]),
        Line::from_iter(vec![
            "│ ".green(),
            "2. ".light_blue(),
            Span::from("second"),
        ]),
        Line::from("│ ".green()),
    ]));
    assert_eq!(text, expected);
}

#[test]
fn blockquote_list_then_nested_blockquote() {
    let md = "> - parent\n>   > child\n";
    let text = render_markdown_text(md);
    let expected = with_blockquote_background(Text::from_iter([
        Line::from("│ ".green()),
        Line::from_iter(vec![
            "│ ".green(),
            "• ".light_blue(),
            Span::from("parent"),
        ]),
        Line::from_iter([
            "│ ".green(),
            "  ".into(),
            "│ ".green(),
        ]),
        Line::from_iter([
            "│ ".green(),
            "  ".into(),
            "│ ".green(),
            "child".into(),
        ]),
        Line::from_iter(["│ ".green(), "  ".into(), "│ ".green()]),
        Line::from("│ ".green()),
    ]));
    assert_eq!(text, expected);
}

#[test]
fn list_item_with_inline_blockquote_on_same_line() {
    let md = "1. > quoted\n";
    let text = render_markdown_text(md);
    assert_eq!(plain_lines(&text), ["1. │ ", "   │ quoted", "   │ "]);
}

#[test]
fn blockquote_surrounded_by_blank_lines() {
    let md = "foo\n\n> bar\n\nbaz\n";
    let text = render_markdown_text(md);
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect();
    assert_eq!(
        lines,
        vec![
            "foo".to_string(),
            "".to_string(),
            "│ ".to_string(),
            "│ bar".to_string(),
            "│ ".to_string(),
            "".to_string(),
            "baz".to_string(),
        ]
    );
}

#[test]
fn blockquote_in_ordered_list_on_next_line() {
    // Blockquote begins on a new line within an ordered list item; it should
    // render inline on the same marker line.
    let md = "1.\n   > quoted\n";
    let text = render_markdown_text(md);
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect();
    assert_eq!(lines, ["1. │ ", "   │ quoted", "   │ "]);
}

#[test]
fn blockquote_in_unordered_list_on_next_line() {
    // Blockquote begins on a new line within an unordered list item; it should
    // render inline on the same marker line.
    let md = "-\n  > quoted\n";
    let text = render_markdown_text(md);
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect();
    assert_eq!(lines, ["• │ ", "  │ quoted", "  │ "]);
}

#[test]
fn blockquote_two_paragraphs_inside_ordered_list_has_blank_line() {
    // Two blockquote paragraphs inside a list item should be separated by a blank line.
    let md = "1.\n   > para 1\n   >\n   > para 2\n";
    let text = render_markdown_text(md);
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect();
    assert_eq!(
        lines,
        vec![
            "1. │ ".to_string(),
            "   │ para 1".to_string(),
            "   │ ".to_string(),
            "   │ para 2".to_string(),
            "   │ ".to_string(),
        ],
        "expected blockquote content to stay aligned after list marker"
    );
}

#[test]
fn blockquote_inside_nested_list() {
    let md = "1. A\n    - B\n      > inner\n";
    let text = render_markdown_text(md);
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect();
    assert_eq!(
        lines,
        ["1. A", "    ◦ B", "      │ ", "      │ inner", "      │ "]
    );
}

#[test]
fn list_item_text_then_blockquote() {
    let md = "1. before\n   > quoted\n";
    let text = render_markdown_text(md);
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect();
    assert_eq!(lines, ["1. before", "   │ ", "   │ quoted", "   │ "]);
}

#[test]
fn list_item_blockquote_then_text() {
    let md = "1.\n   > quoted\n   after\n";
    let text = render_markdown_text(md);
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect();
    assert_eq!(lines, ["1. │ ", "   │ quoted", "   │ after", "   │ "]);
}

#[test]
fn list_item_text_blockquote_text() {
    let md = "1. before\n   > quoted\n   after\n";
    let text = render_markdown_text(md);
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect();
    assert_eq!(
        lines,
        ["1. before", "   │ ", "   │ quoted", "   │ after", "   │ "]
    );
}

#[test]
fn blockquote_with_heading_and_paragraph() {
    let md = "> # Heading\n> paragraph text\n";
    let text = render_markdown_text(md);
    // Validate on content shape; styling is handled elsewhere
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect();
    assert_eq!(
        lines,
        vec![
            "│ ".to_string(),
            "│            ".to_string(),
            "│   Heading  ".to_string(),
            "│            ".to_string(),
            "│ ".to_string(),
            "│ paragraph text".to_string(),
            "│ ".to_string(),
        ]
    );
}

#[test]
#[serial_test::serial]
fn blockquote_heading_inherits_heading_style() {
    let _theme_guard = UiThemeGuard::new();
    crate::style::set_ui_theme(codex_config::types::TuiUiTheme::Teal);
    let text = render_markdown_text("> # test header\n> in blockquote\n");
    let h1 = crate::style::heading1_style(crate::style::active_palette());
    assert_eq!(
        text.lines,
        with_blockquote_background(Text::from_iter([
            Line::from("│ ".green()),
            Line::from_iter(["│ ".green(), Span::styled("               ", h1)]),
            Line::from_iter([
                "│ ".green(),
                Span::styled("  ", h1),
                Span::styled("test header", h1),
                Span::styled("  ", h1),
            ]),
            Line::from_iter(["│ ".green(), Span::styled("               ", h1)]),
            Line::from_iter(["│ ".green()]),
            Line::from_iter(["│ ".green(), "in blockquote".into()]),
            Line::from("│ ".green()),
        ]))
        .lines
    );
}

#[test]
fn blockquote_with_code_block() {
    let md = "> ```\n> code\n> ```\n";
    let text = render_markdown_text(md);
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect();
    assert_eq!(lines, ["│ ", "│ ", "│ code", "│ "]);
}

#[test]
fn blockquote_with_multiline_code_block() {
    let md = "> ```\n> first\n> second\n> ```\n";
    let text = render_markdown_text(md);
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect();
    assert_eq!(lines, ["│ ", "│ ", "│ first", "│ second", "│ "]);
}

#[test]
fn nested_blockquote_with_inline_and_fenced_code() {
    /*
    let md = \"> Nested quote with code:\n\
    > > Inner quote and `inline code`\n\
    > >\n\
    > > ```\n\
    > > # fenced code inside a quote\n\
    > > echo \"hello from a quote\"\n\
    > > ```\n";
    */
    let md = r#"> Nested quote with code:
> > Inner quote and `inline code`
> >
> > ```
> > # fenced code inside a quote
> > echo "hello from a quote"
> > ```
"#;
    let text = render_markdown_text(md);
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect();
    assert_eq!(
        lines,
        vec![
            "│ ".to_string(),
            "│ Nested quote with code:".to_string(),
            "│ ".to_string(),
            "│ │ ".to_string(),
            "│ │ Inner quote and inline code".to_string(),
            "│ │ ".to_string(),
            "│ │ # fenced code inside a quote".to_string(),
            "│ │ echo \"hello from a quote\"".to_string(),
            "│ │ ".to_string(),
            "│ ".to_string(),
        ]
    );
}

#[test]
fn list_unordered_single() {
    let text = render_markdown_text("- List item 1\n");
    let expected = Text::from_iter([Line::from_iter([
        "• ".light_blue(),
        "List item 1".into(),
    ])]);
    assert_eq!(text, expected);
}

#[test]
fn list_unordered_multiple() {
    let text = render_markdown_text("- List item 1\n- List item 2\n");
    let expected = Text::from_iter([
        Line::from_iter(["• ".light_blue(), "List item 1".into()]),
        Line::from_iter(["• ".light_blue(), "List item 2".into()]),
    ]);
    assert_eq!(text, expected);
}

#[test]
fn list_ordered() {
    let text = render_markdown_text("1. List item 1\n2. List item 2\n");
    let expected = Text::from_iter([
        Line::from_iter(["1. ".light_blue(), "List item 1".into()]),
        Line::from_iter(["2. ".light_blue(), "List item 2".into()]),
    ]);
    assert_eq!(text, expected);
}

#[test]
fn list_nested() {
    let text = render_markdown_text("- List item 1\n  - Nested list item 1\n");
    let expected = Text::from_iter([
        Line::from_iter(["• ".light_blue(), "List item 1".into()]),
        Line::from_iter(["    ◦ ".light_blue(), "Nested list item 1".into()]),
    ]);
    assert_eq!(text, expected);
}

#[test]
fn list_unordered_uses_distinct_markers_for_first_three_depths() {
    let md = "- First\n  - Second\n    - Third\n";
    let text = render_markdown_text(md);
    let expected = Text::from_iter([
        Line::from_iter(["• ".light_blue(), "First".into()]),
        Line::from_iter(["    ◦ ".light_blue(), "Second".into()]),
        Line::from_iter(["        ▪ ".light_blue(), "Third".into()]),
    ]);
    assert_eq!(text, expected);
}

#[test]
fn list_ordered_custom_start() {
    let text = render_markdown_text("3. First\n4. Second\n");
    let expected = Text::from_iter([
        Line::from_iter(["3. ".light_blue(), "First".into()]),
        Line::from_iter(["4. ".light_blue(), "Second".into()]),
    ]);
    assert_eq!(text, expected);
}

#[test]
fn nested_unordered_in_ordered() {
    let md = "1. Outer\n    - Inner A\n    - Inner B\n2. Next\n";
    let text = render_markdown_text(md);
    let expected = Text::from_iter([
        Line::from_iter(["1. ".light_blue(), "Outer".into()]),
        Line::from_iter(["    ◦ ".light_blue(), "Inner A".into()]),
        Line::from_iter(["    ◦ ".light_blue(), "Inner B".into()]),
        Line::default(),
        Line::from_iter(["2. ".light_blue(), "Next".into()]),
    ]);
    assert_eq!(text, expected);
}

#[test]
fn nested_ordered_in_unordered() {
    let md = "- Outer\n    1. One\n    2. Two\n- Last\n";
    let text = render_markdown_text(md);
    let expected = Text::from_iter([
        Line::from_iter(["• ".light_blue(), "Outer".into()]),
        Line::from_iter(["    1. ".light_blue(), "One".into()]),
        Line::from_iter(["    2. ".light_blue(), "Two".into()]),
        Line::default(),
        Line::from_iter(["• ".light_blue(), "Last".into()]),
    ]);
    assert_eq!(text, expected);
}

#[test]
fn loose_list_item_multiple_paragraphs() {
    let md = "1. First paragraph\n\n   Second paragraph of same item\n\n2. Next item\n";
    let text = render_markdown_text(md);
    let expected = Text::from_iter([
        Line::from_iter(["1. ".light_blue(), "First paragraph".into()]),
        Line::default(),
        Line::from_iter(["   ", "Second paragraph of same item"]),
        Line::default(),
        Line::from_iter(["2. ".light_blue(), "Next item".into()]),
    ]);
    assert_eq!(text, expected);
}

#[test]
fn tight_item_with_soft_break() {
    let md = "- item line1\n  item line2\n";
    let text = render_markdown_text(md);
    let expected = Text::from_iter([
        Line::from_iter(["• ".light_blue(), "item line1".into()]),
        Line::from_iter(["  ", "item line2"]),
    ]);
    assert_eq!(text, expected);
}

#[test]
fn deeply_nested_mixed_three_levels() {
    let md = "1. A\n    - B\n        1. C\n2. D\n";
    let text = render_markdown_text(md);
    let expected = Text::from_iter([
        Line::from_iter(["1. ".light_blue(), "A".into()]),
        Line::from_iter(["    ◦ ".light_blue(), "B".into()]),
        Line::from_iter(["        1. ".light_blue(), "C".into()]),
        Line::default(),
        Line::from_iter(["2. ".light_blue(), "D".into()]),
    ]);
    assert_eq!(text, expected);
}

#[test]
fn loose_items_due_to_blank_line_between_items() {
    let md = "1. First\n\n2. Second\n";
    let text = render_markdown_text(md);
    let expected = Text::from_iter([
        Line::from_iter(["1. ".light_blue(), "First".into()]),
        Line::from_iter(["2. ".light_blue(), "Second".into()]),
    ]);
    assert_eq!(text, expected);
}

#[test]
fn mixed_tight_then_loose_in_one_list() {
    let md = "1. Tight\n\n2.\n   Loose\n";
    let text = render_markdown_text(md);
    let expected = Text::from_iter([
        Line::from_iter(["1. ".light_blue(), "Tight".into()]),
        Line::from_iter(["2. ".light_blue(), "Loose".into()]),
    ]);
    assert_eq!(text, expected);
}

#[test]
fn ordered_item_with_indented_continuation_is_tight() {
    let md = "1. Foo\n   Bar\n";
    let text = render_markdown_text(md);
    let expected = Text::from_iter([
        Line::from_iter(["1. ".light_blue(), "Foo".into()]),
        Line::from_iter(["   ", "Bar"]),
    ]);
    assert_eq!(text, expected);
}

#[test]
fn inline_code() {
    let text = render_markdown_text("Example of `Inline code`");
    let expected = Line::from_iter(["Example of ".into(), "Inline code".cyan()]).into();
    assert_eq!(text, expected);
}

#[test]
fn strong() {
    assert_eq!(
        render_markdown_text("**Strong**"),
        Text::from(Line::from("Strong".magenta().bold().underlined()))
    );
}

#[test]
fn emphasis() {
    assert_eq!(
        render_markdown_text("*Emphasis*"),
        Text::from(Line::from("Emphasis".green().italic()))
    );
}

#[test]
fn strikethrough() {
    assert_eq!(
        render_markdown_text("~~Strikethrough~~"),
        Text::from(Line::from("Strikethrough".crossed_out()))
    );
}

#[test]
fn strong_emphasis() {
    let text = render_markdown_text("**Strong *emphasis***");
    let expected = Text::from(Line::from_iter([
        "Strong ".magenta().bold().underlined(),
        "emphasis".green().bold().italic().underlined(),
    ]));
    assert_eq!(text, expected);
}

#[test]
fn link() {
    let text = render_markdown_text("[Link](https://example.com)");
    let expected = Text::from(Line::from_iter([
        "Link".into(),
        " (".into(),
        "https://example.com".cyan().underlined(),
        ")".into(),
    ]));
    assert_eq!(text, expected);
}

#[test]
fn load_location_suffix_regexes() {
    let _colon = &*COLON_LOCATION_SUFFIX_RE;
    let _hash = &*HASH_LOCATION_SUFFIX_RE;
}

#[test]
fn file_link_hides_destination() {
    let text = render_markdown_text_for_cwd(
        "[codex-rs/tui/src/markdown_render.rs](/Users/example/code/codex/codex-rs/tui/src/markdown_render.rs)",
        Path::new("/Users/example/code/codex"),
    );
    let expected =
        Text::from(Line::from_iter(["codex-rs/tui/src/markdown_render.rs".cyan()]));
    assert_eq!(text, expected);
}

#[test]
fn file_link_hyperlink_covers_only_path_text_after_prefix() {
    let cwd = Path::new("/Users/example/code/codex");
    let lines = render_markdown_lines_with_width_and_cwd(
        "see [markdown_render.rs](/Users/example/code/codex/codex-rs/tui/src/markdown_render.rs:74) for details",
        /*width*/ Some(120),
        Some(cwd),
    );
    let rendered = lines
        .iter()
        .map(|line| {
            line.line
                .spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        rendered,
        vec!["see codex-rs/tui/src/markdown_render.rs:74 for details"]
    );

    let links = lines
        .iter()
        .flat_map(|line| line.hyperlinks.iter())
        .collect::<Vec<_>>();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].columns, 4..42);
    assert_eq!(
        links[0].destination,
        "file:///Users/example/code/codex/codex-rs/tui/src/markdown_render.rs#L74"
    );
}

#[test]
fn file_link_hyperlink_excludes_chinese_prefix() {
    let cwd = Path::new("/home/heyx/HUINA/JINGMAO");
    let lines = render_markdown_lines_with_width_and_cwd(
        "输出文件： [report](/home/heyx/HUINA/JINGMAO/build/outputs/run_report.md)",
        /*width*/ Some(120),
        Some(cwd),
    );
    let rendered = lines
        .iter()
        .map(|line| {
            line.line
                .spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<_>>();
    assert_eq!(rendered, vec!["输出文件： build/outputs/run_report.md"]);

    let links = lines
        .iter()
        .flat_map(|line| line.hyperlinks.iter())
        .collect::<Vec<_>>();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].columns, "输出文件： ".width().."输出文件： build/outputs/run_report.md".width());
    assert_eq!(
        links[0].destination,
        "file:///home/heyx/HUINA/JINGMAO/build/outputs/run_report.md"
    );
}

#[test]
fn file_link_decodes_percent_encoded_bare_path_destination() {
    let text = render_markdown_text_for_cwd(
        "[report](/Users/example/code/codex/Example%20Folder/R%C3%A9sum%C3%A9/report.md)",
        Path::new("/Users/example/code/codex"),
    );
    let expected = Text::from(Line::from_iter([
        "Example Folder/Résumé/report.md".cyan(),
    ]));
    assert_eq!(text, expected);
}

#[test]
fn file_link_appends_line_number_when_label_lacks_it() {
    let text = render_markdown_text_for_cwd(
        "[markdown_render.rs](/Users/example/code/codex/codex-rs/tui/src/markdown_render.rs:74)",
        Path::new("/Users/example/code/codex"),
    );
    let expected = Text::from(Line::from_iter([
        "codex-rs/tui/src/markdown_render.rs:74".cyan(),
    ]));
    assert_eq!(text, expected);
}

#[test]
fn file_link_keeps_absolute_paths_outside_cwd() {
    let text = render_markdown_text_for_cwd(
        "[README.md:74](/Users/example/code/codex/README.md:74)",
        Path::new("/Users/example/code/codex/codex-rs/tui"),
    );
    let expected = Text::from(Line::from_iter(["/Users/example/code/codex/README.md:74".cyan()]));
    assert_eq!(text, expected);
}

#[test]
fn file_link_appends_hash_anchor_when_label_lacks_it() {
    let text = render_markdown_text_for_cwd(
        "[markdown_render.rs](file:///Users/example/code/codex/codex-rs/tui/src/markdown_render.rs#L74C3)",
        Path::new("/Users/example/code/codex"),
    );
    let expected =
        Text::from(Line::from_iter([
            "codex-rs/tui/src/markdown_render.rs:74:3".cyan(),
        ]));
    assert_eq!(text, expected);
}

#[test]
fn file_link_uses_target_path_for_hash_anchor() {
    let text = render_markdown_text_for_cwd(
        "[markdown_render.rs#L74C3](file:///Users/example/code/codex/codex-rs/tui/src/markdown_render.rs#L74C3)",
        Path::new("/Users/example/code/codex"),
    );
    let expected =
        Text::from(Line::from_iter([
            "codex-rs/tui/src/markdown_render.rs:74:3".cyan(),
        ]));
    assert_eq!(text, expected);
}

#[test]
fn file_link_appends_range_when_label_lacks_it() {
    let text = render_markdown_text_for_cwd(
        "[markdown_render.rs](/Users/example/code/codex/codex-rs/tui/src/markdown_render.rs:74:3-76:9)",
        Path::new("/Users/example/code/codex"),
    );
    let expected =
        Text::from(Line::from_iter([
            "codex-rs/tui/src/markdown_render.rs:74:3-76:9".cyan(),
        ]));
    assert_eq!(text, expected);
}

#[test]
fn file_link_uses_target_path_for_range() {
    let text = render_markdown_text_for_cwd(
        "[markdown_render.rs:74:3-76:9](/Users/example/code/codex/codex-rs/tui/src/markdown_render.rs:74:3-76:9)",
        Path::new("/Users/example/code/codex"),
    );
    let expected =
        Text::from(Line::from_iter([
            "codex-rs/tui/src/markdown_render.rs:74:3-76:9".cyan(),
        ]));
    assert_eq!(text, expected);
}

#[test]
fn file_link_appends_hash_range_when_label_lacks_it() {
    let text = render_markdown_text_for_cwd(
        "[markdown_render.rs](file:///Users/example/code/codex/codex-rs/tui/src/markdown_render.rs#L74C3-L76C9)",
        Path::new("/Users/example/code/codex"),
    );
    let expected =
        Text::from(Line::from_iter([
            "codex-rs/tui/src/markdown_render.rs:74:3-76:9".cyan(),
        ]));
    assert_eq!(text, expected);
}

#[test]
fn multiline_file_link_label_after_styled_prefix_does_not_panic() {
    let text = render_markdown_text_for_cwd(
        "**bold** plain [foo\nbar](file:///Users/example/code/codex/codex-rs/tui/src/markdown_render.rs#L74C3)",
        Path::new("/Users/example/code/codex"),
    );
    let expected = Text::from(Line::from_iter([
        "bold".magenta().bold().underlined(),
        " plain ".into(),
        "codex-rs/tui/src/markdown_render.rs:74:3".cyan(),
    ]));
    assert_eq!(text, expected);
}

#[test]
fn file_link_uses_target_path_for_hash_range() {
    let text = render_markdown_text_for_cwd(
        "[markdown_render.rs#L74C3-L76C9](file:///Users/example/code/codex/codex-rs/tui/src/markdown_render.rs#L74C3-L76C9)",
        Path::new("/Users/example/code/codex"),
    );
    let expected =
        Text::from(Line::from_iter([
            "codex-rs/tui/src/markdown_render.rs:74:3-76:9".cyan(),
        ]));
    assert_eq!(text, expected);
}

#[test]
fn url_link_shows_destination() {
    let text = render_markdown_text("[docs](https://example.com/docs)");
    let expected = Text::from(Line::from_iter([
        "docs".into(),
        " (".into(),
        "https://example.com/docs".cyan().underlined(),
        ")".into(),
    ]));
    assert_eq!(text, expected);
}

#[test]
fn markdown_render_file_link_snapshot() {
    let text = render_markdown_text_for_cwd(
        "See [markdown_render.rs:74](/Users/example/code/codex/codex-rs/tui/src/markdown_render.rs:74).",
        Path::new("/Users/example/code/codex"),
    );
    let rendered = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert_snapshot!(rendered);
}

#[test]
fn unordered_list_local_file_link_stays_inline_with_following_text() {
    let text = render_markdown_text_with_width_and_cwd(
        "- [binary](/Users/example/code/codex/codex-rs/README.md:93): core is the agent/business logic, tui is the terminal UI, exec is the headless automation surface, and cli is the top-level multitool binary.",
        Some(72),
        Some(Path::new("/Users/example/code/codex")),
    );
    let rendered = text
        .lines
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        rendered,
        vec![
            "• codex-rs/README.md:93: core is the agent/business logic, tui is the",
            "  terminal UI, exec is the headless automation surface, and cli is the",
            "  top-level multitool binary.",
        ]
    );
}

#[test]
fn unordered_list_local_file_link_soft_break_before_colon_stays_inline() {
    let text = render_markdown_text_with_width_and_cwd(
        "- [binary](/Users/example/code/codex/codex-rs/README.md:93)\n  : core is the agent/business logic.",
        Some(72),
        Some(Path::new("/Users/example/code/codex")),
    );
    let rendered = text
        .lines
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        rendered,
        vec!["• codex-rs/README.md:93: core is the agent/business logic.",]
    );
}

#[test]
fn consecutive_unordered_list_local_file_links_do_not_detach_paths() {
    let text = render_markdown_text_with_width_and_cwd(
        "- [binary](/Users/example/code/codex/codex-rs/README.md:93)\n  : cli is the top-level multitool binary.\n- [expectations](/Users/example/code/codex/codex-rs/core/README.md:1)\n  : codex-core owns the real runtime behavior.",
        Some(72),
        Some(Path::new("/Users/example/code/codex")),
    );
    let rendered = text
        .lines
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        rendered,
        vec![
            "• codex-rs/README.md:93: cli is the top-level multitool binary.",
            "• codex-rs/core/README.md:1: codex-core owns the real runtime behavior.",
        ]
    );
}

#[test]
fn code_block_known_lang_has_syntax_colors() {
    let text = render_markdown_text("```rust\nfn main() {}\n```\n");
    let content: Vec<String> = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect();
    // Content should be preserved; ignore trailing empty line from highlighting.
    let content: Vec<&str> = content
        .iter()
        .map(std::string::String::as_str)
        .filter(|s| !s.is_empty())
        .collect();
    assert_eq!(content, vec!["fn main() {}"]);

    // At least one span should have non-default style (syntax highlighting).
    let has_colored_span = text
        .lines
        .iter()
        .flat_map(|l| l.spans.iter())
        .any(|sp| sp.style.fg.is_some());
    assert!(has_colored_span, "expected syntax-highlighted spans with color");
}

#[test]
fn code_block_unknown_lang_plain() {
    let text = render_markdown_text("```xyzlang\nhello world\n```\n");
    let content: Vec<String> = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect();
    let content: Vec<&str> = content
        .iter()
        .map(std::string::String::as_str)
        .filter(|s| !s.is_empty())
        .collect();
    assert_eq!(content, vec!["hello world"]);

    // No syntax coloring for unknown language — all spans have default style.
    let has_colored_span = text
        .lines
        .iter()
        .flat_map(|l| l.spans.iter())
        .any(|sp| sp.style.fg.is_some());
    assert!(!has_colored_span, "expected no syntax coloring for unknown lang");
}

#[test]
fn code_block_no_lang_plain() {
    let text = render_markdown_text("```\nno lang specified\n```\n");
    let content: Vec<String> = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect();
    let content: Vec<&str> = content
        .iter()
        .map(std::string::String::as_str)
        .filter(|s| !s.is_empty())
        .collect();
    assert_eq!(content, vec!["no lang specified"]);
}

#[test]
fn code_block_multiple_lines_root() {
    let md = "```\nfirst\nsecond\n```\n";
    let text = render_markdown_text(md);
    let expected = Text::from_iter([
        Line::from_iter(["", "first"]),
        Line::from_iter(["", "second"]),
    ]);
    assert_eq!(text, expected);
}

#[test]
fn code_block_indented() {
    let md = "    function greet() {\n      console.log(\"Hi\");\n    }\n";
    let text = render_markdown_text(md);
    let expected = Text::from_iter([
        Line::from_iter(["    ", "function greet() {"]),
        Line::from_iter(["    ", "  console.log(\"Hi\");"]),
        Line::from_iter(["    ", "}"]),
    ]);
    assert_eq!(text, expected);
}

#[test]
fn horizontal_rule_renders_em_dashes() {
    let md = "Before\n\n---\n\nAfter\n";
    let text = render_markdown_text(md);
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect();
    assert_eq!(lines, vec!["Before", "", "───", "", "After"]);
}

#[test]
fn code_block_with_inner_triple_backticks_outer_four() {
    let md = r#"````text
Here is a code block that shows another fenced block:

```md
# Inside fence
- bullet
- `inline code`
```
````
"#;
    let text = render_markdown_text(md);
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect();
    // Filter empty trailing lines for stability; the code block may or may
    // not emit a trailing blank depending on the highlighting path.
    let trimmed: Vec<&str> = {
        let mut v: Vec<&str> = lines.iter().map(std::string::String::as_str).collect();
        while v.last() == Some(&"") {
            v.pop();
        }
        v
    };
    assert_eq!(
        trimmed,
        vec![
            "Here is a code block that shows another fenced block:",
            "",
            "```md",
            "# Inside fence",
            "- bullet",
            "- `inline code`",
            "```",
        ]
    );
}

#[test]
fn code_block_inside_unordered_list_item_is_indented() {
    let md = "- Item\n\n  ```\n  code line\n  ```\n";
    let text = render_markdown_text(md);
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect();
    assert_eq!(lines, vec!["• Item", "", "  code line"]);
}

#[test]
fn code_block_multiple_lines_inside_unordered_list() {
    let md = "- Item\n\n  ```\n  first\n  second\n  ```\n";
    let text = render_markdown_text(md);
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect();
    assert_eq!(lines, vec!["• Item", "", "  first", "  second"]);
}

#[test]
fn code_block_inside_unordered_list_item_multiple_lines() {
    let md = "- Item\n\n  ```\n  first\n  second\n  ```\n";
    let text = render_markdown_text(md);
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect();
    assert_eq!(lines, vec!["• Item", "", "  first", "  second"]);
}

#[test]
fn list_item_after_code_block_keeps_blank_separator() {
    let md = "1. First:\n\n   ```rust\n   fn first() {}\n   ```\n\n2. Second:\n";
    let text = render_markdown_text(md);
    let lines = plain_lines(&text);
    assert_eq!(
        lines,
        vec!["1. First:", "", "   fn first() {}", "", "2. Second:"]
    );
    assert_snapshot!(
        "list_item_after_code_block_keeps_blank_separator",
        lines.join("\n")
    );
}

#[test]
fn outer_list_item_after_nested_code_block_keeps_blank_separator() {
    let md = "1. First:\n   - Nested:\n\n     ```rust\n     fn first() {}\n     ```\n\n2. Second:\n";
    let text = render_markdown_text(md);
    let lines = plain_lines(&text);
    assert_eq!(
        lines,
        vec![
            "1. First:",
            "    ◦ Nested:",
            "",
            "      fn first() {}",
            "",
            "2. Second:",
        ]
    );
}

#[test]
fn list_item_after_simple_item_stays_compact() {
    let md = "1. First\n\n2. Second\n";
    let text = render_markdown_text(md);
    assert_eq!(plain_lines(&text), vec!["1. First", "2. Second"]);
}

#[test]
fn multiline_finding_items_are_separated_snapshot() {
    let md = r#"**Findings**

1. **Correctness issue: server tool-search completions are always rejected.**

   In `next_prompt_suggestion.rs`, the output is ignored, suppressing suggestions after completed searches.

   Minimal correction: count matching outputs and suppress only missing ones.

2. **High-confidence simplification: remove the unused error channel.**

   The implementation resolves failures to `None`, so its contract can be narrower.

3. **High-confidence churn reduction: consolidate table-driven filter tests.**
"#;
    let text = render_markdown_text(md);
    assert_snapshot!(plain_lines(&text).join("\n"));
}

#[test]
fn wrapped_list_item_is_separated_from_next_sibling() {
    let md = "1. This item wraps onto another visible rendered line\n2. Next item\n";
    let text = render_markdown_text_with_width(md, Some(/*width*/ 24));
    assert_eq!(
        plain_lines(&text),
        vec![
            "1. This item wraps onto",
            "   another visible",
            "   rendered line",
            "",
            "2. Next item",
        ]
    );
}

#[test]
fn mixed_url_markdown_wraps_prose_without_splitting_words_snapshot() {
    let md = "This paragraph keeps **strikethrough** intact near a [link](https://example.com/path) while enough surrounding prose forces wrapping.";
    let text = render_markdown_text_with_width(md, Some(/*width*/ 48));
    assert_snapshot!(plain_lines(&text).join("\n"));
}

#[test]
fn markdown_render_complex_snapshot() {
    let md = r#"# H1: Markdown Streaming Test
Intro paragraph with bold **text**, italic *text*, and inline code `x=1`.
Combined bold-italic ***both*** and escaped asterisks \*literal\*.
Auto-link: <https://example.com> and reference link [ref][r1].
Link with title: [hover me](https://example.com "Example") and mailto <mailto:test@example.com>.
Image: ![alt text](https://example.com/img.png "Title")
> Blockquote level 1
>> Blockquote level 2 with `inline code`
- Unordered list item 1
  - Nested bullet with italics _inner_
- Unordered list item 2 with ~~strikethrough~~
1. Ordered item one
2. Ordered item two with sublist:
   1) Alt-numbered subitem
- [ ] Task: unchecked
- [x] Task: checked with link [home](https://example.org)
---
Table below (alignment test):
| Left | Center | Right |
|:-----|:------:|------:|
| a    |   b    |     c |
Inline HTML: <sup>sup</sup> and <sub>sub</sub>.
HTML block:
<div style="border:1px solid #ccc;padding:2px">inline block</div>
Escapes: \_underscores\_, backslash \\, ticks ``code with `backtick` inside``.
Emoji shortcodes: :sparkles: :tada: (if supported).
Hard break test (line ends with two spaces)  
Next line should be close to previous.
Footnote reference here[^1] and another[^longnote].
Horizontal rule with asterisks:
***
Fenced code block (JSON):
```json
{ "a": 1, "b": [true, false] }
```
Fenced code with tildes and triple backticks inside:
~~~markdown
To close ``` you need tildes.
~~~
Indented code block:
    for i in range(3): print(i)
Definition-like list:
Term
: Definition with `code`.
Character entities: &amp; &lt; &gt; &quot; &#39;
[^1]: This is the first footnote.
[^longnote]: A longer footnote with a link to [Rust](https://www.rust-lang.org/).
Escaped pipe in text: a \| b \| c.
URL with parentheses: [link](https://example.com/path_(with)_parens).
[r1]: https://example.com/ref "Reference link title"
"#;

    let text = render_markdown_text(md);
    let heading_background = crate::style::heading1_style(crate::style::active_palette()).bg;
    // Convert to plain text lines for snapshot (ignore styles). Quote rails and H1 surfaces keep
    // their semantic content without serializing layout padding as trailing whitespace.
    let rendered = text
        .lines
        .iter()
        .map(|l| {
            let rendered = l
                .spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>();
            let is_heading_surface = !l.spans.is_empty()
                && l.spans.iter().all(|span| {
                    span.style.bg == heading_background
                        && span.style.add_modifier.contains(Modifier::BOLD)
                });
            if rendered.trim_end().ends_with('│') || is_heading_surface {
                rendered.trim_end().to_string()
            } else {
                rendered
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert_snapshot!(rendered);
}

#[test]
fn ordered_item_with_code_block_and_nested_bullet() {
    let md = "1. **item 1**\n\n2. **item 2**\n   ```\n   code\n   ```\n   - `PROCESS_START` (a `OnceLock<Instant>`) keeps the start time for the entire process.\n";
    let text = render_markdown_text(md);
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.clone())
                .collect::<String>()
        })
        .collect();
    assert_eq!(
        lines,
        vec![
            "1. item 1".to_string(),
            "2. item 2".to_string(),
            String::new(),
            "   code".to_string(),
            "    ◦ PROCESS_START (a OnceLock<Instant>) keeps the start time for the entire process.".to_string(),
        ]
    );
}

#[test]
fn nested_five_levels_mixed_lists() {
    let md = "1. First\n   - Second level\n     1. Third level (ordered)\n        - Fourth level (bullet)\n          - Fifth level to test indent consistency\n";
    let text = render_markdown_text(md);
    let expected = Text::from_iter([
        Line::from_iter(["1. ".light_blue(), "First".into()]),
        Line::from_iter(["    ◦ ".light_blue(), "Second level".into()]),
        Line::from_iter(["        1. ".light_blue(), "Third level (ordered)".into()]),
        Line::from_iter(["            • ".light_blue(), "Fourth level (bullet)".into()]),
        Line::from_iter([
            "                ◦ ".light_blue(),
            "Fifth level to test indent consistency".into(),
        ]),
    ]);
    assert_eq!(text, expected);
}

#[test]
fn html_inline_is_verbatim() {
    let md = "Hello <span>world</span>!";
    let text = render_markdown_text(md);
    let expected: Text = Line::from_iter(["Hello ", "<span>", "world", "</span>", "!"]).into();
    assert_eq!(text, expected);
}

#[test]
fn html_block_is_verbatim_multiline() {
    let md = "<div>\n  <span>hi</span>\n</div>\n";
    let text = render_markdown_text(md);
    let expected = Text::from_iter([
        Line::from_iter(["<div>"]),
        Line::from_iter(["  <span>hi</span>"]),
        Line::from_iter(["</div>"]),
    ]);
    assert_eq!(text, expected);
}

#[test]
fn html_in_tight_ordered_item_soft_breaks_with_space() {
    let md = "1. Foo\n   <i>Bar</i>\n";
    let text = render_markdown_text(md);
    let expected = Text::from_iter([
        Line::from_iter(["1. ".light_blue(), "Foo".into()]),
        Line::from_iter(["   ", "<i>", "Bar", "</i>"]),
    ]);
    assert_eq!(text, expected);
}

#[test]
fn html_continuation_paragraph_in_unordered_item_indented() {
    let md = "- Item\n\n  <em>continued</em>\n";
    let text = render_markdown_text(md);
    let expected = Text::from_iter([
        Line::from_iter(["• ".light_blue(), "Item".into()]),
        Line::default(),
        Line::from_iter(["  ", "<em>", "continued", "</em>"]),
    ]);
    assert_eq!(text, expected);
}

#[test]
fn unordered_item_continuation_paragraph_is_indented() {
    let md = "- Intro\n\n  Continuation paragraph line 1\n  Continuation paragraph line 2\n";
    let text = render_markdown_text(md);
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.clone())
                .collect::<String>()
        })
        .collect();
    assert_eq!(
        lines,
        vec![
            "• Intro".to_string(),
            String::new(),
            "  Continuation paragraph line 1".to_string(),
            "  Continuation paragraph line 2".to_string(),
        ]
    );
}

#[test]
fn ordered_item_continuation_paragraph_is_indented() {
    let md = "1. Intro\n\n   More details about intro\n";
    let text = render_markdown_text(md);
    let expected = Text::from_iter([
        Line::from_iter(["1. ".light_blue(), "Intro".into()]),
        Line::default(),
        Line::from_iter(["   ", "More details about intro"]),
    ]);
    assert_eq!(text, expected);
}

#[test]
fn nested_item_continuation_paragraph_is_indented() {
    let md = "1. A\n    - B\n\n      Continuation for B\n2. C\n";
    let text = render_markdown_text(md);
    let expected = Text::from_iter([
        Line::from_iter(["1. ".light_blue(), "A".into()]),
        Line::from_iter(["    ◦ ".light_blue(), "B".into()]),
        Line::default(),
        Line::from_iter(["      ", "Continuation for B"]),
        Line::default(),
        Line::from_iter(["2. ".light_blue(), "C".into()]),
    ]);
    assert_eq!(text, expected);
}

#[test]
fn code_block_preserves_trailing_blank_lines() {
    // A fenced code block with an intentional trailing blank line must keep it.
    let md = "```rust\nfn main() {}\n\n```\n";
    let text = render_markdown_text(md);
    let content: Vec<String> = text
        .lines
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.clone())
                .collect::<String>()
        })
        .collect();
    // Should have: "fn main() {}" then "" (the blank line).
    // Filter only to content lines (skip leading/trailing empty from rendering).
    assert!(
        content.iter().any(|c| c == "fn main() {}"),
        "expected code line, got {content:?}"
    );
    // The trailing blank line inside the fence should be preserved.
    let code_start = content.iter().position(|c| c == "fn main() {}").unwrap();
    assert!(
        content.len() > code_start + 1,
        "expected a line after 'fn main() {{}}' but content ends: {content:?}"
    );
    assert_eq!(
        content[code_start + 1], "",
        "trailing blank line inside code fence was lost: {content:?}"
    );
}

#[test]
fn table_renders_app_style_rows_with_themed_bold_header() {
    let md = "| A | B |\n|---|---|\n| 1 | 2 |\n";
    let text = render_markdown_text(md);
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|line| line.spans.iter().map(|span| span.content.clone()).collect())
        .collect();

    assert_eq!(
        lines,
        vec![
            " A      B".to_string(),
            "━━━━━  ━━━━━".to_string(),
            " 1      2".to_string(),
        ]
    );
    assert!(
        text.lines[0]
            .style
            .add_modifier
            .contains(Modifier::BOLD)
    );
    assert!(
        text.lines[0].style.fg.is_some(),
        "expected the syntax theme to provide a table header accent"
    );
    assert!(
        text.lines[1].spans[0]
            .style
            .add_modifier
            .contains(Modifier::DIM)
    );
    assert!(
        !text.lines[2]
            .style
            .add_modifier
            .contains(Modifier::BOLD)
    );
}

#[test]
fn table_alignment_respects_markers() {
    let md = "| Left | Center | Right |\n|:-----|:------:|------:|\n| a | b | c |\n";
    let text = render_markdown_text(md);
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|line| line.spans.iter().map(|span| span.content.clone()).collect())
        .collect();

    assert_eq!(lines[0], " Left    Center    Right");
    assert_eq!(lines[2], " a         b           c");
}

#[test]
fn table_separates_logical_rows_after_wrapped_content() {
    let md = "| Key | Description |\n| --- | --- |\n| -v | Enable very verbose logging output for debugging |\n| -q | Quiet output |\n";
    let text = crate::markdown_render::render_markdown_text_with_width(md, Some(30));
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|line| line.spans.iter().map(|span| span.content.clone()).collect())
        .collect();

    assert!(
        lines
            .iter()
            .any(|line| line.contains("Enable very verbose"))
            && lines.iter().any(|line| line.contains("logging output")),
        "expected wrapped row content: {lines:?}"
    );
    let separator_indices: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(idx, line)| {
            ((line.contains('━') || line.contains('─'))
                && line.chars().all(|ch| matches!(ch, '━' | '─' | ' ')))
            .then_some(idx)
        })
        .collect();
    let wrapped_row_end = lines
        .iter()
        .position(|line| line.contains("logging output"))
        .expect("expected final wrapped line");
    assert_eq!(separator_indices.len(), 2);
    assert!(separator_indices[1] > wrapped_row_end);
    assert!(
        !lines
            .last()
            .is_some_and(|line| line.contains('━') || line.contains('─'))
    );
}

#[test]
fn table_wraps_file_paths_before_collapsing_narrative_columns_snapshot() {
    let md = r#"| Unit | Files | Adds | Removes | What It Adds |
|---|---:|---:|---:|---|
| Suggestion engine and unit coverage | [next_prompt_suggestion.rs](/Users/example/code/codex/codex-rs/core/src/next_prompt_suggestion.rs:1), [next_prompt_suggestion_tests.rs](/Users/example/code/codex/codex-rs/core/src/next_prompt_suggestion_tests.rs:1) | 704 | 0 | Sampling workflow, stable-history checks, tool-flow suppression, fast reasoning profile, filtering rules, cancellation and timeout. |
| Model instruction fragment and contextual isolation | [next_prompt_suggestion.rs](/Users/example/code/codex/codex-rs/core/src/context/next_prompt_suggestion.rs:1), [contextual_user_message_tests.rs](/Users/example/code/codex/codex-rs/core/src/context/contextual_user_message_tests.rs:1) | 54 | 0 | Synthetic suggestion prompt and an isolation test for ordinary user text. |
"#;
    let text = render_markdown_text_with_width_and_cwd(
        md,
        Some(/*width*/ 120),
        Some(Path::new("/Users/example/code/codex")),
    );

    assert_snapshot!(plain_lines(&text).join("\n"));
}

#[test]
fn table_renders_stacked_key_value_records_when_path_column_becomes_too_narrow_snapshot() {
    let md = r#"| Session | Why useful | Detected table blocks |
| --- | --- | --- |
| [2026-05-25 current gallery](/Users/felipe.coury/.codex/sessions/2026/05/25/rollout-2026-05-25T18-13-09-019e60fc-0518-7c21-9596-980fe97225ba.jsonl) | The large gallery from this thread: emojis, links, emphasis, code, alignment, paragraphs, and a 30+ row table | 7 |
| [2026-05-14 renderer testing](/Users/felipe.coury/.codex/sessions/2026/05/14/rollout-2026-05-14T12-57-18-019e2734-e500-7011-8278-975c94d06000.jsonl) | Explicit "markdown tables for testing" session with several successive assistant samples | 16 |
| [2026-05-14 five-table test](/Users/felipe.coury/.codex/sessions/2026/05/14/rollout-2026-05-14T12-27-57-019e271a-064c-78c3-a5cd-a6f20a0c1ad5.jsonl) | Explicit request for five tables containing emojis, code, italics, and varied cell content | 10 |
"#;
    let text = render_markdown_text_with_width(md, Some(/*width*/ 42));

    assert_snapshot!(plain_lines(&text).join("\n"));
}

#[test]
fn table_renders_records_when_multiple_prose_columns_are_starved_snapshot() {
    let md = r#"| Issue | Activity | Complexity | Why start |
| --- | ---: | ---: | --- |
| [#24485: newline shortcut fails in PyCharm terminal on Windows](https://github.com/openai/codex/issues/24485) | `+1` 0, substantive comments 0 | Low | New, deterministic regression range; localized composer/keymap path. |
| [#23926: Vim composer `e` stalls at word end](https://github.com/openai/codex/issues/23926) | `+1` 0, comments 0 | Low | Standing best quick win; deterministic motion bug. |
| [#23651: Zellij scrollback misses Codex transcript over SSH](https://github.com/openai/codex/issues/23651) | `+1` 3, human comments 2 | Medium | Clear regression and strong scrollback evidence. |
| [#23740: raw ANSI/control sequences in Windows Terminal](https://github.com/openai/codex/issues/23740) | `+1` 7, human comments 7 | Medium | Highest activity; established Windows rendering regression family. |
| [#24527: typing lag increases with session length](https://github.com/openai/codex/issues/24527) | `+1` 0, substantive comments 0 | Medium | New TUI-visible performance report; needs profiling before implementation. |
"#;
    let text = render_markdown_text_with_width(md, Some(/*width*/ 76));

    assert_snapshot!(plain_lines(&text).join("\n"));
}

#[test]
fn table_keeps_grid_when_only_one_compact_record_fragments_snapshot() {
    let md = r#"| Key | Date | State |
| --- | --- | --- |
| short | 2025-01-01 | Ready |
| verylongidentifier | 2025-02-02 | Ready |
| final | 2025-03-03 | Done |
"#;
    let text = render_markdown_text_with_width(md, Some(/*width*/ 40));

    assert_snapshot!(plain_lines(&text).join("\n"));
}

#[test]
fn table_renders_key_value_records_when_compact_fragmentation_is_systemic_snapshot() {
    let md = r#"| Key | Notes |
| --- | --- |
| firstlongid | A readable explanatory sentence for this row. |
| secondlongid | Another readable explanatory sentence for this row. |
| short | A final readable explanatory sentence for this row. |
"#;
    let text = render_markdown_text_with_width(md, Some(/*width*/ 17));

    assert_snapshot!(plain_lines(&text).join("\n"));
}

#[test]
fn table_inside_blockquote_has_quote_prefix() {
    let md = "> | A | B |\n> |---|---|\n> | 1 | 2 |\n";
    let text = render_markdown_text(md);
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|line| line.spans.iter().map(|span| span.content.clone()).collect())
        .collect();

    assert!(lines.iter().all(|line| line.starts_with("│ ")));
    assert!(lines.iter().any(|line| line.contains("━━━━━  ━━━━━")));
}

#[test]
fn escaped_pipes_render_in_table_cells() {
    let md = "| Col |\n| --- |\n| a \\| b |\n";
    let text = render_markdown_text(md);
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|line| line.spans.iter().map(|span| span.content.clone()).collect())
        .collect();

    assert!(lines.iter().any(|line| line.contains("a | b")));
}

#[test]
fn table_falls_back_to_key_value_records_if_grid_cannot_fit() {
    let md = "| c1 | c2 | c3 | c4 | c5 | c6 | c7 | c8 | c9 | c10 |\n|---|---|---|---|---|---|---|---|---|---|\n| 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |\n";
    let text = crate::markdown_render::render_markdown_text_with_width(md, Some(/*width*/ 20));
    let lines: Vec<String> = text
        .lines
        .iter()
        .map(|line| line.spans.iter().map(|span| span.content.clone()).collect())
        .collect();

    assert!(lines.first().is_some_and(|line| line.contains("c1")));
    assert!(lines.iter().any(|line| line.contains("c10") && line.contains("10")));
    assert!(
        !lines
            .iter()
            .any(|line| line.starts_with('|') || line.contains('━') || line.contains('─'))
    );
}

#[test]
fn table_key_value_fallback_preserves_rich_values_and_themed_labels() {
    let md = "| Key | Content | Extra | More |\n|---|---|---|---|\n| item | [link](https://example.com) | **bold** | `code` |\n";
    let text = crate::markdown_render::render_markdown_text_with_width(md, Some(/*width*/ 16));
    let lines = plain_lines(&text);

    assert!(lines.iter().any(|line| line.contains("Key")));
    assert!(lines.iter().any(|line| line.contains("item")));
    assert!(lines.iter().any(|line| line.contains("link")));
    assert!(lines.iter().any(|line| line.contains("bold")));
    assert!(lines.iter().any(|line| line.contains("code")));
    assert!(
        text.lines[0]
            .spans
            .iter()
            .any(|span| span.content.contains("Key")
                && span.style.add_modifier.contains(Modifier::BOLD)
                && span.style.fg.is_some())
    );
    assert!(text.lines.iter().any(|line| {
        line.spans
            .iter()
            .any(|span| span.style.add_modifier.contains(Modifier::UNDERLINED))
    }));
}
