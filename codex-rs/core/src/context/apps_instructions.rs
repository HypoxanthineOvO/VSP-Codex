use codex_protocol::protocol::APPS_INSTRUCTIONS_CLOSE_TAG;
use codex_protocol::protocol::APPS_INSTRUCTIONS_OPEN_TAG;

use super::ContextualUserFragment;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AppsInstructions;

impl ContextualUserFragment for AppsInstructions {
    fn role(&self) -> &'static str {
        "developer"
    }

    fn markers(&self) -> (&'static str, &'static str) {
        Self::type_markers()
    }

    fn type_markers() -> (&'static str, &'static str) {
        (APPS_INSTRUCTIONS_OPEN_TAG, APPS_INSTRUCTIONS_CLOSE_TAG)
    }

    fn body(&self) -> String {
        "\n## Apps (Connectors)\nApps expose MCP tools from the `codex_apps` server. Use an app when the user names it through an app link or when an available app materially fits the request.\nUse already-visible app tools directly. Otherwise use `tool_search` when available. For app tool discovery, do not also call `list_mcp_resources` or `list_mcp_resource_templates`.\n"
            .to_string()
    }
}
