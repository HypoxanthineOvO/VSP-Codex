use crate::color::blend;
use crate::color::is_light;
use crate::terminal_palette::StdoutColorLevel;
use crate::terminal_palette::best_color;
use crate::terminal_palette::default_bg;
use crate::terminal_palette::default_fg;
use crate::terminal_palette::rgb_color;
use crate::terminal_palette::stdout_color_level;
use codex_config::types::TuiUiTheme;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::style::Stylize;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering;

const LIGHT_BG_ACCENT_RGB: (u8, u8, u8) = (0, 95, 135);
const BLOCKQUOTE_TINT_RGB: (u8, u8, u8) = (72, 187, 120);
static ACTIVE_UI_THEME: AtomicU8 = AtomicU8::new(0);
// Decorative table rules should remain visible without competing with cell content.
const TABLE_SEPARATOR_FG_ALPHA: f32 = 0.20;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiPalette {
    pub(crate) primary: Color,
    pub(crate) secondary: Color,
    pub(crate) accent: Color,
    pub(crate) emphasis: Color,
    pub(crate) strong: Color,
    pub(crate) structure: Color,
    pub(crate) subtle_border: Color,
    pub(crate) user: Color,
    pub(crate) assistant: Color,
    pub(crate) reasoning: Color,
    pub(crate) tool: Color,
    pub(crate) ask: Color,
    pub(crate) heading_surface_rgb: (u8, u8, u8),
}

const fn ui_theme_id(theme: TuiUiTheme) -> u8 {
    match theme {
        TuiUiTheme::Teal => 0,
        TuiUiTheme::Violet => 1,
        TuiUiTheme::Amber => 2,
    }
}

const fn ui_theme_from_id(id: u8) -> TuiUiTheme {
    match id {
        1 => TuiUiTheme::Violet,
        2 => TuiUiTheme::Amber,
        _ => TuiUiTheme::Teal,
    }
}

pub(crate) fn set_ui_theme(theme: TuiUiTheme) {
    ACTIVE_UI_THEME.store(ui_theme_id(theme), Ordering::Relaxed);
}

pub(crate) fn active_ui_theme() -> TuiUiTheme {
    ui_theme_from_id(ACTIVE_UI_THEME.load(Ordering::Relaxed))
}

pub(crate) fn active_palette() -> UiPalette {
    palette_for(active_ui_theme())
}

pub(crate) fn palette_for(theme: TuiUiTheme) -> UiPalette {
    match theme {
        TuiUiTheme::Teal => UiPalette {
            primary: Color::Cyan,
            secondary: Color::Green,
            accent: Color::LightCyan,
            emphasis: Color::Green,
            strong: Color::Magenta,
            structure: Color::LightBlue,
            subtle_border: Color::DarkGray,
            user: Color::Cyan,
            assistant: Color::Green,
            reasoning: Color::Magenta,
            tool: Color::LightBlue,
            ask: Color::Cyan,
            heading_surface_rgb: (35, 126, 138),
        },
        TuiUiTheme::Violet => UiPalette {
            primary: Color::Magenta,
            secondary: Color::LightBlue,
            accent: Color::LightMagenta,
            emphasis: Color::LightBlue,
            strong: Color::LightMagenta,
            structure: Color::Magenta,
            subtle_border: Color::DarkGray,
            user: Color::LightMagenta,
            assistant: Color::LightBlue,
            reasoning: Color::Magenta,
            tool: Color::Cyan,
            ask: Color::LightMagenta,
            heading_surface_rgb: (126, 72, 191),
        },
        TuiUiTheme::Amber => UiPalette {
            primary: Color::Yellow,
            secondary: Color::Green,
            accent: Color::LightYellow,
            emphasis: Color::Green,
            strong: Color::Yellow,
            structure: Color::LightYellow,
            subtle_border: Color::DarkGray,
            user: Color::Yellow,
            assistant: Color::Green,
            reasoning: Color::Magenta,
            tool: Color::LightYellow,
            ask: Color::Yellow,
            heading_surface_rgb: (176, 116, 24),
        },
    }
}

pub fn user_message_style() -> Style {
    user_message_style_for(default_bg())
}

pub fn proposed_plan_style() -> Style {
    proposed_plan_style_for(default_bg())
}

/// Returns a low-contrast rule style for separators within markdown tables.
pub(crate) fn table_separator_style() -> Style {
    table_separator_style_for(default_fg(), default_bg(), stdout_color_level())
}

/// Returns the shared accent style for active or selected TUI controls.
pub(crate) fn accent_style() -> Style {
    accent_style_for(default_bg())
}

/// Returns a subtle green surface for quoted Markdown without assuming a light or dark terminal.
pub(crate) fn blockquote_style() -> Style {
    blockquote_style_for(default_bg())
}

pub(crate) fn heading1_style(palette: UiPalette) -> Style {
    let background = match default_bg() {
        Some(terminal_bg) => {
            let alpha = if is_light(terminal_bg) { 0.08 } else { 0.18 };
            best_color(blend(palette.heading_surface_rgb, terminal_bg, alpha))
        }
        None => Color::DarkGray,
    };
    Style::default().fg(palette.accent).bg(background).bold()
}

/// Returns the style for a user-authored message using the provided terminal background.
pub fn user_message_style_for(terminal_bg: Option<(u8, u8, u8)>) -> Style {
    match terminal_bg {
        Some(bg) => Style::default().bg(user_message_bg(bg)),
        None => Style::default(),
    }
}

pub fn proposed_plan_style_for(terminal_bg: Option<(u8, u8, u8)>) -> Style {
    match terminal_bg {
        Some(bg) => Style::default().bg(proposed_plan_bg(bg)),
        None => Style::default(),
    }
}

/// Returns the shared accent style for the provided terminal background.
pub(crate) fn accent_style_for(terminal_bg: Option<(u8, u8, u8)>) -> Style {
    if terminal_bg.is_some_and(is_light) {
        Style::default().fg(best_color(LIGHT_BG_ACCENT_RGB)).bold()
    } else {
        Style::default().fg(active_palette().primary).bold()
    }
}

fn blockquote_style_for(terminal_bg: Option<(u8, u8, u8)>) -> Style {
    let Some(terminal_bg) = terminal_bg else {
        return Style::default().bg(Color::DarkGray);
    };
    let alpha = if is_light(terminal_bg) { 0.10 } else { 0.16 };
    Style::default().bg(best_color(blend(BLOCKQUOTE_TINT_RGB, terminal_bg, alpha)))
}

fn table_separator_style_for(
    terminal_fg: Option<(u8, u8, u8)>,
    terminal_bg: Option<(u8, u8, u8)>,
    color_level: StdoutColorLevel,
) -> Style {
    let (Some(fg), Some(bg)) = (terminal_fg, terminal_bg) else {
        return Style::default().dim();
    };
    let separator_rgb = blend(fg, bg, TABLE_SEPARATOR_FG_ALPHA);
    match color_level {
        StdoutColorLevel::TrueColor => Style::default().fg(rgb_color(separator_rgb)),
        StdoutColorLevel::Ansi256 => Style::default().fg(best_color(separator_rgb)),
        StdoutColorLevel::Ansi16 | StdoutColorLevel::Unknown => Style::default().dim(),
    }
}

#[allow(clippy::disallowed_methods)]
pub fn user_message_bg(terminal_bg: (u8, u8, u8)) -> Color {
    let (top, alpha) = if is_light(terminal_bg) {
        ((0, 0, 0), 0.04)
    } else {
        ((255, 255, 255), 0.12)
    };
    best_color(blend(top, terminal_bg, alpha))
}

#[allow(clippy::disallowed_methods)]
pub fn proposed_plan_bg(terminal_bg: (u8, u8, u8)) -> Color {
    user_message_bg(terminal_bg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use ratatui::style::Modifier;

    #[test]
    fn accent_style_uses_darker_cyan_on_light_backgrounds() {
        let style = accent_style_for(Some((255, 255, 255)));

        assert_eq!(style.fg, Some(best_color(LIGHT_BG_ACCENT_RGB)));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn accent_style_uses_cyan_on_dark_or_unknown_backgrounds() {
        let expected = Style::default().fg(Color::Cyan).bold();

        assert_eq!(accent_style_for(Some((0, 0, 0))), expected);
        assert_eq!(accent_style_for(/*terminal_bg*/ None), expected);
    }

    #[test]
    fn blockquote_style_blends_a_subtle_green_surface_for_light_and_dark_backgrounds() {
        let dark = blockquote_style_for(Some((0, 0, 0)));
        let light = blockquote_style_for(Some((255, 255, 255)));

        assert_eq!(dark.bg, Some(best_color((11, 29, 19))));
        assert_eq!(light.bg, Some(best_color((236, 248, 241))));
        assert_eq!(
            blockquote_style_for(None),
            Style::default().bg(Color::DarkGray)
        );
    }

    #[test]
    fn table_separator_blends_toward_dark_background() {
        let style = table_separator_style_for(
            Some((255, 255, 255)),
            Some((0, 0, 0)),
            StdoutColorLevel::TrueColor,
        );

        assert_eq!(style.fg, Some(rgb_color((51, 51, 51))));
    }

    #[test]
    fn table_separator_blends_toward_light_background() {
        let style = table_separator_style_for(
            Some((0, 0, 0)),
            Some((255, 255, 255)),
            StdoutColorLevel::TrueColor,
        );

        assert_eq!(style.fg, Some(rgb_color((204, 204, 204))));
    }

    #[test]
    fn table_separator_dims_when_palette_aware_color_is_unavailable() {
        let expected = Style::default().dim();

        assert_eq!(
            table_separator_style_for(
                Some((255, 255, 255)),
                Some((0, 0, 0)),
                StdoutColorLevel::Ansi16,
            ),
            expected
        );
        assert_eq!(
            table_separator_style_for(
                /*terminal_fg*/ None,
                Some((0, 0, 0)),
                StdoutColorLevel::TrueColor,
            ),
            expected
        );
    }
}
