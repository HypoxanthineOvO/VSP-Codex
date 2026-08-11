const NATIVE_ASK_UI_COMPAT_CLIENT_NAMES: &[&str] = &[
    "VS Code",
    "Visual Studio Code",
    "codex_vscode",
    "openai.chatgpt",
    "codex_desktop",
    "codex_chatgpt_desktop",
    "codex_chatgpt_desktop_remote",
    "codex_chatgpt_android_remote",
    "codex_chatgpt_ios_remote",
];

pub(crate) fn effective_native_ask_ui(client_name: &str, declared_native_ask_ui: bool) -> bool {
    declared_native_ask_ui
        || NATIVE_ASK_UI_COMPAT_CLIENT_NAMES
            .iter()
            .any(|known| known.eq_ignore_ascii_case(client_name.trim()))
}

#[cfg(test)]
mod tests {
    use super::effective_native_ask_ui;

    #[test]
    fn official_popup_capable_clients_get_native_ask_ui_compatibility() {
        for client_name in [
            "VS Code",
            "Visual Studio Code",
            "codex_vscode",
            "openai.chatgpt",
            "codex_desktop",
            "codex_chatgpt_desktop",
            "codex_chatgpt_desktop_remote",
            "codex_chatgpt_android_remote",
            "codex_chatgpt_ios_remote",
        ] {
            assert!(
                effective_native_ask_ui(client_name, false),
                "{client_name} should be treated as native Ask UI capable"
            );
        }
    }

    #[test]
    fn unknown_clients_still_need_explicit_native_ask_ui_capability() {
        assert!(!effective_native_ask_ui("codex-app-server-tests", false));
        assert!(effective_native_ask_ui("codex-app-server-tests", true));
    }
}
