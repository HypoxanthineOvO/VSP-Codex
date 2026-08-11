//! Hydrates provider-scoped catalogs returned by the owning app-server.

use crate::legacy_core::config::AddonModel;
use crate::legacy_core::config::AddonModelProviders;
use crate::legacy_core::config::AddonModelSelection;
use crate::legacy_core::config::AddonProvider;
use crate::legacy_core::config::Config;
use codex_app_server_protocol::ModelProviderListResponse;
use codex_model_provider_info::ModelProviderInfo;
use std::collections::BTreeMap;

pub(super) fn apply_server_provider_catalog(
    config: &mut Config,
    response: &ModelProviderListResponse,
    default_model: &str,
) {
    let mut addon_providers = BTreeMap::new();
    for provider in &response.data {
        config.model_providers.insert(
            provider.id.clone(),
            ModelProviderInfo {
                name: provider.name.clone(),
                base_url: provider.base_url.clone(),
                env_key: provider.credential_env_key.clone(),
                ..Default::default()
            },
        );
        let hydrated_provider = addon_provider_from_api(provider);
        if provider.is_addon {
            addon_providers.insert(provider.id.clone(), hydrated_provider);
        } else if provider.id == response.base_provider_id {
            config.model_catalog = AddonModelProviders {
                version: 1,
                default_selection: None,
                providers: BTreeMap::from([(provider.id.clone(), hydrated_provider)]),
            }
            .model_catalog(&provider.id);
        }
    }

    config
        .base_model_provider_id
        .clone_from(&response.base_provider_id);
    config
        .model_provider_id
        .clone_from(&response.current_provider_id);
    config.model = Some(default_model.to_string());
    if let Some(provider) = config
        .model_providers
        .get(&response.current_provider_id)
        .cloned()
    {
        config.model_provider = provider;
    }
    config.addon_model_providers = (!addon_providers.is_empty()).then(|| AddonModelProviders {
        version: 1,
        default_selection: addon_providers
            .contains_key(&response.current_provider_id)
            .then(|| AddonModelSelection {
                provider_id: response.current_provider_id.clone(),
                model_id: default_model.to_string(),
            }),
        providers: addon_providers,
    });
}

fn addon_provider_from_api(provider: &codex_app_server_protocol::ModelProvider) -> AddonProvider {
    AddonProvider {
        name: provider.name.clone(),
        base_url: provider.base_url.clone().unwrap_or_default(),
        env_key: provider.credential_env_key.clone().unwrap_or_default(),
        credential_available: Some(provider.credential_available),
        models: provider
            .models
            .iter()
            .map(|model| AddonModel {
                id: model.id.clone(),
                display_name: model.display_name.clone(),
                context_window: model.context_window,
                effective_context_window_percent: model.effective_context_window_percent,
                supported_reasoning_efforts: model.supported_reasoning_efforts.clone(),
                default_reasoning_effort: model.default_reasoning_effort.clone(),
            })
            .collect(),
    }
}

#[cfg(test)]
#[path = "provider_catalog_tests.rs"]
mod tests;
