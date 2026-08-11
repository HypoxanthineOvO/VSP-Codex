use codex_protocol::protocol::PLUGINS_INSTRUCTIONS_CLOSE_TAG;
use codex_protocol::protocol::PLUGINS_INSTRUCTIONS_OPEN_TAG;

use super::ContextualUserFragment;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AvailablePluginsInstructions;

impl ContextualUserFragment for AvailablePluginsInstructions {
    fn role(&self) -> &'static str {
        "developer"
    }

    fn markers(&self) -> (&'static str, &'static str) {
        Self::type_markers()
    }

    fn type_markers() -> (&'static str, &'static str) {
        (
            PLUGINS_INSTRUCTIONS_OPEN_TAG,
            PLUGINS_INSTRUCTIONS_CLOSE_TAG,
        )
    }

    fn body(&self) -> String {
        r#"
## Plugins
A plugin bundles skills, MCP servers, and apps. Plugins are not invoked directly; use their available capabilities.
- Plugin Skills use the `plugin_name:skill_name` namespace. MCP tools keep their normal names; use tool provenance to identify the plugin.
- When the user explicitly names a plugin, prefer its relevant available capabilities for that turn.
- If the requested plugin has no callable capability in the current session, say so briefly and continue with the best available fallback.
"#
        .to_string()
    }
}
