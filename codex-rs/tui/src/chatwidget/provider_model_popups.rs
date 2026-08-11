//! Provider-grouped model selection for standalone addon catalogs.

use super::*;
use crate::bottom_pane::SelectionTab;
use codex_protocol::openai_models::ReasoningEffortPreset;

impl ChatWidget {
    pub(crate) fn open_provider_grouped_model_popup(&mut self) {
        let Some(addon) = self.config.addon_model_providers.as_ref() else {
            return;
        };
        let current_provider = self.config.model_provider_id.clone();
        let mut tabs = Vec::new();

        let base_provider_id = self.config.base_model_provider_id.clone();
        let base_models = self
            .config
            .model_catalog
            .clone()
            .map(|catalog| {
                catalog
                    .models
                    .into_iter()
                    .map(ModelPreset::from)
                    .collect::<Vec<_>>()
            })
            .or_else(|| {
                (base_provider_id == current_provider)
                    .then(|| self.model_catalog.try_list_models().ok())
                    .flatten()
            })
            .or_else(|| {
                codex_models_manager::bundled_models_response()
                    .ok()
                    .map(|catalog| catalog.models.into_iter().map(ModelPreset::from).collect())
            });
        if let Some(models) = base_models {
            tabs.push(self.provider_model_tab(
                &base_provider_id,
                models,
                /*missing_env_key*/ None,
            ));
        }

        for (provider_id, provider) in &addon.providers {
            let models = addon
                .model_catalog(provider_id)
                .map(|catalog| catalog.models.into_iter().map(ModelPreset::from).collect())
                .unwrap_or_default();
            let credential_available = provider.credential_available.unwrap_or_else(|| {
                std::env::var(&provider.env_key)
                    .ok()
                    .is_some_and(|value| !value.trim().is_empty())
            });
            let missing_env_key = (!credential_available).then(|| provider.env_key.clone());
            tabs.push(self.provider_model_tab(provider_id, models, missing_env_key));
        }

        tabs.sort_by(|left, right| left.label.cmp(&right.label));
        self.bottom_pane.show_selection_view(SelectionViewParams {
            title: Some("Select Provider and Model".to_string()),
            subtitle: Some("Switch provider tabs, then choose a model.".to_string()),
            initial_tab_id: Some(current_provider),
            tabs,
            is_searchable: true,
            search_placeholder: Some("Search models".to_string()),
            footer_hint: Some(self.bottom_pane.standard_popup_hint_line()),
            ..Default::default()
        });
    }

    fn provider_model_tab(
        &self,
        provider_id: &str,
        models: Vec<ModelPreset>,
        missing_env_key: Option<String>,
    ) -> SelectionTab {
        let provider_name = self
            .config
            .model_providers
            .get(provider_id)
            .map(|provider| provider.name.trim())
            .filter(|name| !name.is_empty())
            .unwrap_or(provider_id)
            .to_string();
        let mut header = ColumnRenderable::new();
        header.push(Line::from(provider_name.clone().bold()));
        header.push(Line::from(format!("Provider: {provider_id}").dim()));
        let items = if let Some(env_key) = missing_env_key {
            vec![SelectionItem {
                name: "Credential required".to_string(),
                description: Some(format!("Set {env_key} to use this provider.")),
                is_disabled: true,
                ..Default::default()
            }]
        } else {
            models
                .into_iter()
                .filter(|model| model.show_in_picker)
                .map(|model| self.provider_model_item(provider_id, model))
                .collect()
        };
        SelectionTab {
            id: provider_id.to_string(),
            label: provider_name,
            header: Box::new(header),
            items,
        }
    }

    fn provider_model_item(&self, provider_id: &str, model: ModelPreset) -> SelectionItem {
        let has_multiple_efforts = model.supported_reasoning_efforts.len() > 1;
        let provider_id = provider_id.to_string();
        let provider_id_for_action = provider_id.clone();
        let model_for_action = model.clone();
        let actions: Vec<SelectionAction> = vec![Box::new(move |tx| {
            if has_multiple_efforts {
                tx.send(AppEvent::OpenProviderReasoningPopup {
                    provider_id: provider_id_for_action.clone(),
                    model: model_for_action.clone(),
                });
            } else {
                tx.send(AppEvent::ApplyProviderModelSelection {
                    provider_id: provider_id_for_action.clone(),
                    model_id: model_for_action.model.clone(),
                    effort: Some(model_for_action.default_reasoning_effort.clone()),
                });
            }
        })];
        SelectionItem {
            name: model.display_name.clone(),
            description: Some(if model.description.is_empty() {
                model.model.clone()
            } else {
                format!("{} | {}", model.model, model.description)
            }),
            is_current: provider_id == self.config.model_provider_id
                && model.model == self.current_model(),
            is_default: model.is_default,
            actions,
            dismiss_on_select: !has_multiple_efforts,
            dismiss_parent_on_child_accept: has_multiple_efforts,
            search_value: Some(model.model),
            ..Default::default()
        }
    }

    pub(crate) fn open_provider_reasoning_popup(
        &mut self,
        provider_id: String,
        model: ModelPreset,
    ) {
        let choices = if model.supported_reasoning_efforts.is_empty() {
            vec![ReasoningEffortPreset {
                effort: model.default_reasoning_effort.clone(),
                description: String::new(),
            }]
        } else {
            model.supported_reasoning_efforts.clone()
        };
        let items = choices
            .into_iter()
            .map(|choice| {
                let provider_id = provider_id.clone();
                let model_id = model.model.clone();
                let effort = choice.effort.clone();
                let actions: Vec<SelectionAction> = vec![Box::new(move |tx| {
                    tx.send(AppEvent::ApplyProviderModelSelection {
                        provider_id: provider_id.clone(),
                        model_id: model_id.clone(),
                        effort: Some(effort.clone()),
                    });
                })];
                SelectionItem {
                    name: choice.effort.to_string(),
                    description: (!choice.description.is_empty()).then_some(choice.description),
                    is_default: choice.effort == model.default_reasoning_effort,
                    actions,
                    dismiss_on_select: true,
                    ..Default::default()
                }
            })
            .collect();
        self.bottom_pane.show_selection_view(SelectionViewParams {
            title: Some(format!("Reasoning for {}", model.display_name)),
            subtitle: Some(format!("Provider: {provider_id}")),
            items,
            footer_hint: Some(self.bottom_pane.standard_popup_hint_line()),
            ..Default::default()
        });
    }
}

#[cfg(test)]
#[path = "provider_model_popups_tests.rs"]
mod tests;
