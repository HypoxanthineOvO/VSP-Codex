//! Provider selection popup for `ChatWidget`.

use super::*;

impl ChatWidget {
    pub(crate) fn open_provider_popup(&mut self) {
        if !self.is_session_configured() {
            self.add_info_message(
                "Provider selection is disabled until startup completes.".to_string(),
                /*hint*/ None,
            );
            return;
        }

        let current_provider = self.config.model_provider_id.clone();
        let mut providers: Vec<_> = self.config.model_providers.iter().collect();
        providers.sort_by(|(left_id, left), (right_id, right)| {
            crate::vsp_provider::provider_display_name(left_id, left)
                .cmp(&crate::vsp_provider::provider_display_name(right_id, right))
        });

        let items = providers
            .into_iter()
            .filter(|(provider_id, provider)| {
                crate::vsp_provider::is_provider_allowed(provider_id, provider)
            })
            .map(|(provider_id, provider)| {
                let provider_id_for_action = provider_id.clone();
                let actions: Vec<SelectionAction> = vec![Box::new(move |tx| {
                    tx.send(AppEvent::ApplyProviderSelection {
                        provider_id: provider_id_for_action.clone(),
                    });
                })];
                SelectionItem {
                    name: crate::vsp_provider::provider_display_name(provider_id, provider),
                    description: Some(crate::vsp_provider::provider_description(provider)),
                    is_current: provider_id == &current_provider,
                    actions,
                    dismiss_on_select: true,
                    ..Default::default()
                }
            })
            .collect();

        let mut header = ColumnRenderable::new();
        header.push(Line::from("Select Provider".bold()));
        header.push(Line::from(
            "Provider controls Base URL and credentials. Use /model for model.".dim(),
        ));

        self.bottom_pane.show_selection_view(SelectionViewParams {
            footer_hint: Some(standard_popup_hint_line()),
            items,
            header: Box::new(header),
            ..Default::default()
        });
    }
}
