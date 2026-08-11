//! Builds the `/ui-theme` picker dialog for TUI message and Markdown colors.

use codex_config::types::TuiUiTheme;

use crate::app_event::AppEvent;
use crate::bottom_pane::SelectionItem;
use crate::bottom_pane::SelectionViewParams;
use crate::bottom_pane::popup_consts::standard_popup_hint_line;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct UiThemeEntry {
    theme: TuiUiTheme,
    name: &'static str,
    description: &'static str,
}

const UI_THEME_ENTRIES: [UiThemeEntry; 3] = [
    UiThemeEntry {
        theme: TuiUiTheme::Teal,
        name: "Teal",
        description: "cyan + green + magenta accents",
    },
    UiThemeEntry {
        theme: TuiUiTheme::Violet,
        name: "Violet",
        description: "violet + blue + cyan accents",
    },
    UiThemeEntry {
        theme: TuiUiTheme::Amber,
        name: "Amber",
        description: "amber + green + magenta accents",
    },
];

pub(crate) fn build_ui_theme_picker_params(current: TuiUiTheme) -> SelectionViewParams {
    let items = UI_THEME_ENTRIES
        .iter()
        .map(|entry| {
            let theme = entry.theme;
            SelectionItem {
                name: entry.name.to_string(),
                description: Some(entry.description.to_string()),
                is_current: theme == current,
                dismiss_on_select: true,
                search_value: Some(entry.name.to_string()),
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::UiThemeSelected { theme });
                })],
                ..Default::default()
            }
        })
        .collect();

    SelectionViewParams {
        title: Some("Select UI Theme".to_string()),
        subtitle: Some("Applies to Markdown roles, message accents, and the composer.".to_string()),
        footer_hint: Some(standard_popup_hint_line()),
        items,
        is_searchable: true,
        search_placeholder: Some("Type to filter UI themes...".to_string()),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_theme_picker_exposes_three_persistable_theme_groups() {
        let params = build_ui_theme_picker_params(TuiUiTheme::Violet);

        assert_eq!(params.items.len(), 3);
        assert_eq!(
            params.items.iter().filter(|item| item.is_current).count(),
            1
        );
        assert!(
            params.items.iter().all(|item| item.actions.len() == 1),
            "each UI theme entry must persist through an action"
        );
    }
}
