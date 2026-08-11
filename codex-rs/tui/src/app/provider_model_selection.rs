//! Atomic provider/model application for the provider-grouped picker.

use super::App;
use crate::app_server_session::AppServerSession;
use crate::config_update::build_model_selection_edits;
use crate::config_update::replace_config_value;
use crate::config_update::write_config_batch;
use crate::legacy_core::config::AddonModelSelection;
use codex_app_server_protocol::ModelProviderWriteParams;
use codex_app_server_protocol::ThreadSettingsUpdateParams;
use codex_protocol::openai_models::ReasoningEffort;

impl App {
    pub(super) async fn apply_provider_model_selection(
        &mut self,
        app_server: &mut AppServerSession,
        provider_id: String,
        model_id: String,
        effort: Option<ReasoningEffort>,
    ) {
        let Some(provider) = self.config.model_providers.get(&provider_id).cloned() else {
            self.chat_widget
                .add_error_message(format!("Unknown provider: {provider_id}"));
            return;
        };
        let selected_addon_provider = self
            .config
            .addon_model_providers
            .as_ref()
            .and_then(|addon| addon.providers.get(&provider_id));
        if let Some(addon_provider) = selected_addon_provider {
            if !addon_provider
                .models
                .iter()
                .any(|model| model.id == model_id)
            {
                self.chat_widget.add_error_message(format!(
                    "Unknown model {model_id} for provider {provider_id}"
                ));
                return;
            }
            let credential_available = addon_provider.credential_available.unwrap_or_else(|| {
                std::env::var(&addon_provider.env_key)
                    .ok()
                    .is_some_and(|value| !value.trim().is_empty())
            });
            if !credential_available {
                self.chat_widget.add_error_message(format!(
                    "Provider {provider_id} requires {}",
                    addon_provider.env_key
                ));
                return;
            }
        }
        let selected_provider_is_addon = selected_addon_provider.is_some();

        let previous_provider_id = self.config.model_provider_id.clone();
        let previous_model_id = self.chat_widget.current_model().to_string();
        let previous_effort = self.chat_widget.current_reasoning_effort();
        let previous_collaboration_mode = self.chat_widget.effective_collaboration_mode();
        let previous_addon_selection = self
            .config
            .addon_model_providers
            .as_ref()
            .and_then(|addon| addon.default_selection.clone());
        let mut selected_collaboration_mode = previous_collaboration_mode.clone();
        selected_collaboration_mode.settings.model = model_id.clone();
        selected_collaboration_mode.settings.reasoning_effort = effort.clone();

        if let Some(thread_id) = self.active_thread_id {
            let update = ThreadSettingsUpdateParams {
                thread_id: thread_id.to_string(),
                model: Some(model_id.clone()),
                model_provider: Some(provider_id.clone()),
                effort: effort.clone(),
                collaboration_mode: Some(selected_collaboration_mode),
                ..Default::default()
            };
            if let Err(error) = app_server.thread_settings_update_strict(update).await {
                self.chat_widget
                    .add_error_message(format!("Failed to switch provider/model: {error}"));
                return;
            }
        }

        if let Err(error) = app_server
            .model_provider_write(ModelProviderWriteParams {
                provider_id: provider_id.clone(),
                model_id: model_id.clone(),
            })
            .await
        {
            let rollback_error = if let Some(thread_id) = self.active_thread_id {
                let rollback = ThreadSettingsUpdateParams {
                    thread_id: thread_id.to_string(),
                    model: Some(previous_model_id),
                    model_provider: Some(previous_provider_id),
                    effort: previous_effort,
                    collaboration_mode: Some(previous_collaboration_mode),
                    ..Default::default()
                };
                app_server
                    .thread_settings_update_strict(rollback)
                    .await
                    .err()
                    .map(|rollback_error| rollback_error.to_string())
            } else {
                None
            };
            let rollback_detail = rollback_error
                .map(|rollback_error| format!(" Rollback also failed: {rollback_error}"))
                .unwrap_or_default();
            self.chat_widget.add_error_message(format!(
                "Failed to save provider/model selection: {error}.{rollback_detail}"
            ));
            return;
        }

        if !selected_provider_is_addon {
            let mut edits = build_model_selection_edits(&model_id, effort.as_ref());
            edits.push(replace_config_value(
                "model_provider",
                serde_json::json!(provider_id),
            ));
            if let Err(error) = write_config_batch(app_server.request_handle(), edits).await {
                let file_rollback_error = match previous_addon_selection.clone() {
                    Some(selection) => app_server
                        .model_provider_write(ModelProviderWriteParams {
                            provider_id: selection.provider_id,
                            model_id: selection.model_id,
                        })
                        .await
                        .err()
                        .map(|error| error.to_string()),
                    None => None,
                };
                let thread_rollback_error = if let Some(thread_id) = self.active_thread_id {
                    let rollback = ThreadSettingsUpdateParams {
                        thread_id: thread_id.to_string(),
                        model: Some(previous_model_id),
                        model_provider: Some(previous_provider_id),
                        effort: previous_effort,
                        collaboration_mode: Some(previous_collaboration_mode),
                        ..Default::default()
                    };
                    app_server
                        .thread_settings_update_strict(rollback)
                        .await
                        .err()
                        .map(|rollback_error| rollback_error.to_string())
                } else {
                    None
                };
                let rollback_errors = [file_rollback_error, thread_rollback_error]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join("; ");
                let rollback_detail = if rollback_errors.is_empty() {
                    String::new()
                } else {
                    format!(" Rollback also failed: {rollback_errors}")
                };
                self.chat_widget.add_error_message(format!(
                    "Failed to save provider/model selection: {error}.{rollback_detail}"
                ));
                return;
            }
        }

        self.config.model_provider_id.clone_from(&provider_id);
        self.config.model_provider = provider;
        self.config.model = Some(model_id.clone());
        self.config.model_reasoning_effort = effort.clone();
        if let Some(addon) = self.config.addon_model_providers.as_mut() {
            addon.default_selection = selected_provider_is_addon.then_some(AddonModelSelection {
                provider_id: provider_id.clone(),
                model_id: model_id.clone(),
            });
        }
        self.chat_widget.set_model_provider(&provider_id);
        self.chat_widget.set_model(&model_id);
        self.chat_widget.set_reasoning_effort(effort);
        self.chat_widget.add_info_message(
            format!("Model changed to {provider_id}/{model_id}"),
            /*hint*/ None,
        );
    }
}

#[cfg(test)]
#[path = "provider_model_selection_tests.rs"]
mod tests;
