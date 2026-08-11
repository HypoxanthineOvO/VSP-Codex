//! Informational, warning, update, and policy notice history cells.

use super::*;

#[cfg_attr(debug_assertions, allow(dead_code))]
#[derive(Debug)]
pub(crate) struct UpdateAvailableHistoryCell {
    latest_version: String,
    update_action: Option<UpdateAction>,
}

#[cfg_attr(debug_assertions, allow(dead_code))]
impl UpdateAvailableHistoryCell {
    pub(crate) fn new(latest_version: String, update_action: Option<UpdateAction>) -> Self {
        Self {
            latest_version,
            update_action,
        }
    }
}

#[cfg_attr(debug_assertions, allow(dead_code))]
pub(crate) fn update_history_action_for_vsp_notice(
    vsp_official_services_disabled: bool,
    vsp_manifest_action: Option<UpdateAction>,
    default_action: Option<UpdateAction>,
) -> Option<UpdateAction> {
    if vsp_official_services_disabled {
        vsp_manifest_action
    } else {
        default_action
    }
}

impl HistoryCell for UpdateAvailableHistoryCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        use ratatui_macros::line;
        let update_instruction = if let Some(update_action) = self.update_action.as_ref() {
            line!["Run ", update_action.command_str().cyan(), " to update."]
        } else if crate::vsp_provider::official_services_disabled() {
            line!["No compatible VSP update action is available for this target."]
        } else {
            line![
                "See ",
                "https://github.com/openai/codex".cyan().underlined(),
                " for installation options."
            ]
        };

        let release_notes_url = self
            .update_action
            .as_ref()
            .and_then(UpdateAction::release_notes_url)
            .map(str::to_string)
            .or_else(|| {
                (!crate::vsp_provider::official_services_disabled())
                    .then(|| "https://github.com/openai/codex/releases/latest".to_string())
            });
        let mut lines = vec![
            line![
                padded_emoji("✨").bold().cyan(),
                "Update available!".bold().cyan(),
                " ",
                format!("{CODEX_CLI_VERSION} -> {}", self.latest_version).bold(),
            ],
            update_instruction,
            "".into(),
        ];
        if let Some(release_notes_url) = release_notes_url {
            lines.push("See full release notes:".into());
            lines.push(release_notes_url.cyan().underlined().into());
        } else {
            lines.push(
                "VSP release notes are unavailable without a compatible manifest action.".into(),
            );
        }

        let inner_width = lines
            .iter()
            .map(Line::width)
            .max()
            .unwrap_or_default()
            .min(usize::from(width.saturating_sub(4)))
            .max(1);
        let lines = adaptive_wrap_lines(lines, RtOptions::new(inner_width));
        with_border_with_inner_width(lines, inner_width)
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        let update_instruction = if let Some(update_action) = self.update_action.as_ref() {
            format!("Run {} to update.", update_action.command_str())
        } else if crate::vsp_provider::official_services_disabled() {
            "No compatible VSP update action is available for this target.".to_string()
        } else {
            "See https://github.com/openai/codex for installation options.".to_string()
        };
        let release_notes_url = self
            .update_action
            .as_ref()
            .and_then(UpdateAction::release_notes_url);
        let mut lines = vec![
            Line::from("Update available!"),
            Line::from(format!("{CODEX_CLI_VERSION} -> {}", self.latest_version)),
            Line::from(update_instruction),
            Line::from(""),
        ];
        if let Some(release_notes_url) = release_notes_url {
            lines.push(Line::from("See full release notes:"));
            lines.push(Line::from(release_notes_url.to_string()));
        } else if crate::vsp_provider::official_services_disabled() {
            lines.push(Line::from(
                "VSP release notes are unavailable without a compatible manifest action.",
            ));
        } else {
            lines.push(Line::from("See full release notes:"));
            lines.push(Line::from(
                "https://github.com/openai/codex/releases/latest".to_string(),
            ));
        }
        lines
    }
}

#[cfg_attr(debug_assertions, allow(dead_code))]
#[derive(Debug)]
pub(crate) struct UpstreamSyncAvailableHistoryCell {
    latest_upstream_tag: String,
    baseline_upstream_tag: String,
    release_url: String,
}

#[cfg_attr(debug_assertions, allow(dead_code))]
impl UpstreamSyncAvailableHistoryCell {
    pub(crate) fn new(
        latest_upstream_tag: String,
        baseline_upstream_tag: String,
        release_url: String,
    ) -> Self {
        Self {
            latest_upstream_tag,
            baseline_upstream_tag,
            release_url,
        }
    }
}

impl HistoryCell for UpstreamSyncAvailableHistoryCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        use ratatui_macros::line;

        let lines = vec![
            "Upstream sync available".bold().cyan().into(),
            line![
                "VSP baseline: ".dim(),
                self.baseline_upstream_tag.clone().bold()
            ],
            line![
                "Latest upstream: ".dim(),
                self.latest_upstream_tag.clone().bold()
            ],
            "".into(),
            "Review upstream and Cherry Pick compatible commits into VSP.".into(),
            "Do not install the official Codex binary over VSP."
                .red()
                .into(),
            "".into(),
            "Official release:".into(),
            self.release_url.clone().cyan().underlined().into(),
        ];
        let inner_width = lines
            .iter()
            .map(Line::width)
            .max()
            .unwrap_or_default()
            .min(usize::from(width.saturating_sub(4)))
            .max(1);
        let lines = adaptive_wrap_lines(lines, RtOptions::new(inner_width));
        with_border_with_inner_width(lines, inner_width)
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        vec![
            "Upstream sync available".into(),
            format!("VSP baseline: {}", self.baseline_upstream_tag).into(),
            format!("Latest upstream: {}", self.latest_upstream_tag).into(),
            "".into(),
            "Review upstream and Cherry Pick compatible commits into VSP.".into(),
            "Do not install the official Codex binary over VSP.".into(),
            "".into(),
            "Official release:".into(),
            self.release_url.clone().into(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c7_m7_vsp_update_notice_does_not_fallback_to_official_update_actions() {
        let selected =
            update_history_action_for_vsp_notice(true, None, Some(UpdateAction::StandaloneUnix));

        assert_eq!(selected, None);
    }

    #[test]
    fn c7_m7_vsp_update_notice_without_action_omits_official_links() {
        let cell = UpdateAvailableHistoryCell::new("9.9.9".to_string(), None);
        let text = cell
            .raw_lines()
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(
            !text.contains("github.com/openai/codex"),
            "VSP update notice without a manifest action must not show official update links: {text}"
        );
        assert!(
            text.contains("No compatible VSP update action"),
            "notice should explain the target compatibility failure: {text}"
        );
    }
}
#[allow(clippy::disallowed_methods)]
pub(crate) fn new_warning_event(message: String) -> PrefixedWrappedHistoryCell {
    PrefixedWrappedHistoryCell::new(message.yellow(), "⚠ ".yellow(), "  ")
}

#[derive(Debug)]
pub(crate) struct SafetyAccessBlockCell {
    body: &'static str,
    trusted_access_url: &'static str,
}

const SAFETY_ACCESS_BLOCK_TITLE: &str = "This content can't be shown";
const SAFETY_ACCESS_BLOCK_LEARN_MORE_URL: &str = "https://help.openai.com/en/articles/20001326";

pub(crate) fn new_safety_access_block_event() -> SafetyAccessBlockCell {
    SafetyAccessBlockCell {
        body: "We take extra caution with requests involving biological research and applications that could pose safety risks. Eligible researchers can apply for Trusted Access.",
        trusted_access_url: "https://www.openai.com/form/trusted-access-for-biology-research/",
    }
}

pub(crate) fn new_cyber_policy_error_event() -> SafetyAccessBlockCell {
    SafetyAccessBlockCell {
        body: "We take extra caution with cybersecurity requests. If you’re a security professional, you may be able to apply for Trusted Access.",
        trusted_access_url: "https://openai.com/form/enterprise-trusted-access-for-cyber/",
    }
}

impl HistoryCell for SafetyAccessBlockCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        visible_lines(self.display_hyperlink_lines(width))
    }

    fn display_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        let mut lines = vec![HyperlinkLine::new(
            vec!["ⓘ ".cyan(), SAFETY_ACCESS_BLOCK_TITLE.bold()].into(),
        )];
        let body = Line::from(vec!["  ".into(), self.body.dim()]);
        let wrap_width = width.saturating_sub(2).max(1) as usize;
        let wrapped = adaptive_wrap_line(
            &body,
            RtOptions::new(wrap_width).subsequent_indent("  ".into()),
        );
        let mut wrapped_body = Vec::new();
        push_owned_lines(&wrapped, &mut wrapped_body);
        lines.extend(plain_hyperlink_lines(wrapped_body));

        for (label, url) in [
            ("Trusted Access", self.trusted_access_url),
            ("Learn more", SAFETY_ACCESS_BLOCK_LEARN_MORE_URL),
        ] {
            let source = crate::terminal_hyperlinks::annotate_web_urls_in_line(
                vec![format!("  {label}: ").dim(), url.cyan().underlined()].into(),
            );
            let wrapped = crate::wrapping::word_wrap_line(
                &source.line,
                RtOptions::new(wrap_width).subsequent_indent("  ".into()),
            );
            let mut wrapped_links = Vec::new();
            push_owned_lines(&wrapped, &mut wrapped_links);
            lines.extend(crate::terminal_hyperlinks::remap_wrapped_line(
                &source,
                wrapped_links,
            ));
        }
        lines
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        let trusted_access_url = self.trusted_access_url;
        vec![
            Line::from(SAFETY_ACCESS_BLOCK_TITLE),
            Line::from(self.body),
            Line::from(format!("Trusted Access: {trusted_access_url}")),
            Line::from(format!("Learn more: {SAFETY_ACCESS_BLOCK_LEARN_MORE_URL}")),
        ]
    }

    fn transcript_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        self.display_hyperlink_lines(width)
    }
}

#[derive(Debug)]
pub(crate) struct DeprecationNoticeCell {
    summary: String,
    details: Option<String>,
}

pub(crate) fn new_deprecation_notice(
    summary: String,
    details: Option<String>,
) -> DeprecationNoticeCell {
    DeprecationNoticeCell { summary, details }
}

impl HistoryCell for DeprecationNoticeCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();
        lines.push(vec!["⚠ ".red().bold(), self.summary.clone().red()].into());

        let wrap_width = width.saturating_sub(4).max(1) as usize;

        if let Some(details) = &self.details {
            let detail_line = Line::from(details.clone().dim());
            let wrapped = adaptive_wrap_line(&detail_line, RtOptions::new(wrap_width));
            push_owned_lines(&wrapped, &mut lines);
        }

        lines
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        let mut lines = vec![Line::from(self.summary.clone())];
        if let Some(details) = &self.details {
            lines.extend(raw_lines_from_source(details));
        }
        lines
    }
}
pub(crate) fn new_info_event(message: String, hint: Option<String>) -> PlainHistoryCell {
    let mut line = vec!["• ".dim(), message.into()];
    if let Some(hint) = hint {
        line.push(" ".into());
        line.push(hint.dark_gray());
    }
    let lines: Vec<Line<'static>> = vec![line.into()];
    PlainHistoryCell { lines }
}

pub(crate) fn new_error_event(message: String) -> PlainHistoryCell {
    // Use a hair space (U+200A) to create a subtle, near-invisible separation
    // before the text. VS16 is intentionally omitted to keep spacing tighter
    // in terminals like Ghostty.
    let lines: Vec<Line<'static>> = vec![vec![format!("■ {message}").red()].into()];
    PlainHistoryCell { lines }
}
