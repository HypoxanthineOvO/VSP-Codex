use super::catalog_processor::CatalogRequestProcessor;
use super::*;

impl CatalogRequestProcessor {
    pub(crate) async fn model_provider_list(
        &self,
        params: ModelProviderListParams,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        self.model_provider_list_response(params)
            .map(|response| Some(response.into()))
    }

    pub(crate) async fn model_provider_write(
        &self,
        params: ModelProviderWriteParams,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        self.model_provider_write_response(params)
            .await
            .map(|response| Some(response.into()))
    }

    fn model_provider_list_response(
        &self,
        params: ModelProviderListParams,
    ) -> Result<ModelProviderListResponse, JSONRPCErrorError> {
        let ModelProviderListParams { cursor, limit } = params;
        let mut providers = Vec::new();
        let base_provider_id = self.config.base_model_provider_id.clone();
        if let Some(provider) = self.config.model_providers.get(&base_provider_id) {
            let catalog = self
                .config
                .model_catalog
                .clone()
                .or_else(|| codex_models_manager::bundled_models_response().ok())
                .unwrap_or_default();
            providers.push(ModelProvider {
                id: base_provider_id.clone(),
                name: provider.name.clone(),
                base_url: provider.base_url.clone(),
                credential_env_key: None,
                credential_available: true,
                is_addon: false,
                models: provider_models_from_catalog(catalog),
            });
        }
        if let Some(addon) = self.config.addon_model_providers.as_ref() {
            providers.extend(addon.providers.iter().map(|(provider_id, provider)| {
                ModelProvider {
                    id: provider_id.clone(),
                    name: provider.name.clone(),
                    base_url: Some(provider.base_url.clone()),
                    credential_env_key: Some(provider.env_key.clone()),
                    credential_available: std::env::var(&provider.env_key)
                        .ok()
                        .is_some_and(|value| !value.trim().is_empty()),
                    is_addon: true,
                    models: provider
                        .models
                        .iter()
                        .map(|model| ModelProviderModel {
                            id: model.id.clone(),
                            display_name: model.display_name.clone(),
                            context_window: model.context_window,
                            effective_context_window_percent: model
                                .effective_context_window_percent,
                            supported_reasoning_efforts: model.supported_reasoning_efforts.clone(),
                            default_reasoning_effort: model.default_reasoning_effort.clone(),
                        })
                        .collect(),
                }
            }));
        }
        providers.sort_by(|left, right| left.id.cmp(&right.id));
        let total = providers.len();
        let start = cursor
            .map(|cursor| {
                cursor
                    .parse::<usize>()
                    .map_err(|_| invalid_request(format!("invalid cursor: {cursor}")))
            })
            .transpose()?
            .unwrap_or(0);
        if start > total {
            return Err(invalid_request(format!(
                "cursor {start} exceeds total model providers {total}"
            )));
        }
        let effective_limit = limit.unwrap_or(total as u32).max(1) as usize;
        let end = start.saturating_add(effective_limit).min(total);
        let data = providers[start..end].to_vec();
        let next_cursor = (end < total).then(|| end.to_string());
        Ok(ModelProviderListResponse {
            data,
            next_cursor,
            base_provider_id,
            current_provider_id: self.config.model_provider_id.clone(),
        })
    }

    async fn model_provider_write_response(
        &self,
        params: ModelProviderWriteParams,
    ) -> Result<ModelProviderWriteResponse, JSONRPCErrorError> {
        let ModelProviderWriteParams {
            provider_id,
            model_id,
        } = params;
        let selection = if let Some(provider) = self
            .config
            .addon_model_providers
            .as_ref()
            .and_then(|addon| addon.providers.get(&provider_id))
        {
            if !provider.models.iter().any(|model| model.id == model_id) {
                return Err(invalid_request(format!(
                    "unknown model {model_id} for provider {provider_id}"
                )));
            }
            Some(codex_core::config::AddonModelSelection {
                provider_id,
                model_id,
            })
        } else if provider_id == self.config.base_model_provider_id {
            None
        } else {
            return Err(invalid_request(format!(
                "unknown model provider: {provider_id}"
            )));
        };
        let codex_home = self.config.codex_home.to_path_buf();
        tokio::task::spawn_blocking(move || {
            codex_core::config::persist_addon_default_selection(&codex_home, selection.as_ref())
        })
        .await
        .map_err(|error| internal_error(format!("provider write task failed: {error}")))?
        .map_err(|error| {
            internal_error(format!("failed to persist provider selection: {error}"))
        })?;
        Ok(ModelProviderWriteResponse {})
    }
}

fn provider_models_from_catalog(
    catalog: codex_protocol::openai_models::ModelsResponse,
) -> Vec<ModelProviderModel> {
    catalog
        .models
        .into_iter()
        .map(|model| {
            let context_window = model.resolved_context_window().unwrap_or_default();
            ModelProviderModel {
                id: model.slug,
                display_name: model.display_name,
                context_window,
                effective_context_window_percent: model.effective_context_window_percent,
                supported_reasoning_efforts: model
                    .supported_reasoning_levels
                    .into_iter()
                    .map(|effort| effort.effort)
                    .collect(),
                default_reasoning_effort: model.default_reasoning_level,
            }
        })
        .collect()
}
