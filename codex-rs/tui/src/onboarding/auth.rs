//! Authentication step UI and state transitions used by onboarding.
//!
//! This module owns the auth-step state machine (ChatGPT login/device-code/API
//! key), renders the corresponding UI, and handles auth-scoped keyboard input.
//! It intentionally does not decide onboarding flow completion; the enclosing
//! onboarding screen coordinates step progression.

#![allow(clippy::unwrap_used)]

use codex_app_server_client::AppServerRequestHandle;
use codex_app_server_protocol::AccountLoginCompletedNotification;
use codex_app_server_protocol::AccountUpdatedNotification;
use codex_app_server_protocol::AuthMode as ApiAuthMode;
use codex_app_server_protocol::CancelLoginAccountParams;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::LoginAccountParams;
use codex_app_server_protocol::LoginAccountResponse;
use codex_login::read_openai_api_key_from_env;
use codex_protocol::auth::AuthMode;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::BorderType;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::widgets::WidgetRef;
use ratatui::widgets::Wrap;

use codex_protocol::config_types::ForcedLoginMethod;
use std::cell::Cell;
use std::sync::Arc;
use std::sync::RwLock;
use uuid::Uuid;

use crate::LoginStatus;
use crate::key_hint::KeyBinding;
use crate::key_hint::KeyBindingListExt;
use crate::motion::MotionMode;
use crate::motion::shimmer_text;
use crate::onboarding::keys;
use crate::onboarding::onboarding_screen::KeyboardHandler;
use crate::onboarding::onboarding_screen::StepStateProvider;
use crate::tui::FrameRequester;
use crate::vsp_provider;

/// Marks buffer cells that have cyan+underlined style as an OSC 8 hyperlink.
///
/// Terminal emulators recognise the OSC 8 escape sequence and treat the entire
/// marked region as a single clickable link, regardless of row wrapping.  This
/// is necessary because ratatui's cell-based rendering emits `MoveTo` at every
/// row boundary, which breaks normal terminal URL detection for long URLs that
/// wrap across multiple rows.
pub(crate) fn mark_url_hyperlink(buf: &mut Buffer, area: Rect, url: &str) {
    crate::terminal_hyperlinks::mark_url_hyperlink(buf, area, url);
}

/// Marks any underlined buffer cells as an OSC 8 hyperlink.
pub(crate) fn mark_underlined_hyperlink(buf: &mut Buffer, area: Rect, url: &str) {
    crate::terminal_hyperlinks::mark_underlined_hyperlink(buf, area, url);
}

use super::onboarding_screen::StepState;

mod headless_chatgpt_login;

#[derive(Clone)]
#[allow(dead_code)]
pub(crate) enum SignInState {
    PickMode,
    ChatGptContinueInBrowser(ContinueInBrowserState),
    #[allow(dead_code)]
    ChatGptDeviceCode(ContinueWithDeviceCodeState),
    ChatGptSuccessMessage,
    ChatGptSuccess,
    ApiKeyEntry(ApiKeyInputState),
    ApiKeyConfigured,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum SignInOption {
    ChatGpt,
    DeviceCode,
    ApiKey,
}

fn onboarding_request_id() -> codex_app_server_protocol::RequestId {
    codex_app_server_protocol::RequestId::String(Uuid::new_v4().to_string())
}

pub(super) async fn cancel_login_attempt(
    request_handle: &AppServerRequestHandle,
    login_id: String,
) {
    let _ = request_handle
        .request_typed::<codex_app_server_protocol::CancelLoginAccountResponse>(
            ClientRequest::CancelLoginAccount {
                request_id: onboarding_request_id(),
                params: CancelLoginAccountParams { login_id },
            },
        )
        .await;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProviderInputField {
    BaseUrl,
    ApiKey,
}

#[derive(Clone)]
pub(crate) struct ApiKeyInputState {
    base_url: String,
    api_key: String,
    focused_field: ProviderInputField,
    prepopulated_api_key_from_env: bool,
}

impl ApiKeyInputState {
    pub(crate) fn new(
        base_url: String,
        api_key: String,
        prepopulated_api_key_from_env: bool,
    ) -> Self {
        Self {
            base_url,
            api_key,
            focused_field: ProviderInputField::BaseUrl,
            prepopulated_api_key_from_env,
        }
    }

    fn has_text(&self) -> bool {
        !self.base_url.is_empty() || !self.api_key.is_empty()
    }

    fn focused_value_mut(&mut self) -> &mut String {
        match self.focused_field {
            ProviderInputField::BaseUrl => &mut self.base_url,
            ProviderInputField::ApiKey => &mut self.api_key,
        }
    }

    fn toggle_focus(&mut self) {
        self.focused_field = match self.focused_field {
            ProviderInputField::BaseUrl => ProviderInputField::ApiKey,
            ProviderInputField::ApiKey => ProviderInputField::BaseUrl,
        };
    }
}

#[derive(Clone)]
/// Used to manage the lifecycle of SpawnedLogin and ensure it gets cleaned up.
pub(crate) struct ContinueInBrowserState {
    login_id: String,
    auth_url: String,
}

#[derive(Clone)]
#[allow(dead_code)]
pub(crate) struct ContinueWithDeviceCodeState {
    request_id: String,
    login_id: Option<String>,
    verification_url: Option<String>,
    user_code: Option<String>,
}

#[allow(dead_code)]
impl ContinueWithDeviceCodeState {
    pub(crate) fn pending(request_id: String) -> Self {
        Self {
            request_id,
            login_id: None,
            verification_url: None,
            user_code: None,
        }
    }

    pub(crate) fn ready(
        request_id: String,
        login_id: String,
        verification_url: String,
        user_code: String,
    ) -> Self {
        Self {
            request_id,
            login_id: Some(login_id),
            verification_url: Some(verification_url),
            user_code: Some(user_code),
        }
    }

    pub(crate) fn login_id(&self) -> Option<&str> {
        self.login_id.as_deref()
    }

    pub(crate) fn is_showing_copyable_auth(&self) -> bool {
        self.verification_url
            .as_deref()
            .is_some_and(|url| !url.is_empty())
            && self
                .user_code
                .as_deref()
                .is_some_and(|user_code| !user_code.is_empty())
    }
}

impl KeyboardHandler for AuthModeWidget {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if self.handle_api_key_entry_key_event(&key_event) {
            return;
        }

        if keys::MOVE_UP.is_pressed(key_event) {
            self.move_highlight(/*delta*/ -1);
            return;
        }
        if keys::MOVE_DOWN.is_pressed(key_event) {
            self.move_highlight(/*delta*/ 1);
            return;
        }
        if keys::SELECT_FIRST.is_pressed(key_event) {
            self.select_option_by_index(/*index*/ 0);
            return;
        }
        if keys::SELECT_SECOND.is_pressed(key_event) {
            self.select_option_by_index(/*index*/ 1);
            return;
        }
        if keys::SELECT_THIRD.is_pressed(key_event) {
            self.select_option_by_index(/*index*/ 2);
            return;
        }
        if keys::CONFIRM.is_pressed(key_event) {
            let sign_in_state = { (*self.sign_in_state.read().unwrap()).clone() };
            match sign_in_state {
                SignInState::PickMode => {
                    self.handle_sign_in_option(self.highlighted_mode);
                }
                SignInState::ChatGptSuccessMessage => {
                    *self.sign_in_state.write().unwrap() = SignInState::ChatGptSuccess;
                }
                _ => {}
            }
            return;
        }
        if keys::CANCEL.is_pressed(key_event) {
            tracing::info!("Cancel onboarding auth step");
            self.cancel_active_attempt();
        }
    }

    fn handle_paste(&mut self, pasted: String) {
        let _ = self.handle_api_key_entry_paste(pasted);
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub(crate) struct AuthModeWidget {
    pub request_frame: FrameRequester,
    pub highlighted_mode: SignInOption,
    pub error: Arc<RwLock<Option<String>>>,
    pub sign_in_state: Arc<RwLock<SignInState>>,
    pub login_status: LoginStatus,
    pub app_server_request_handle: AppServerRequestHandle,
    pub forced_login_method: Option<ForcedLoginMethod>,
    pub animations_enabled: bool,
    pub animations_suppressed: Cell<bool>,
}

impl AuthModeWidget {
    pub(crate) fn set_animations_suppressed(&self, suppressed: bool) {
        self.animations_suppressed.set(suppressed);
    }

    pub(crate) fn should_suppress_animations(&self) -> bool {
        matches!(
            &*self.sign_in_state.read().unwrap(),
            SignInState::ChatGptContinueInBrowser(_) | SignInState::ChatGptDeviceCode(_)
        )
    }

    pub(crate) fn cancel_active_attempt(&self) {
        let mut sign_in_state = self.sign_in_state.write().unwrap();
        match &*sign_in_state {
            SignInState::ChatGptContinueInBrowser(state) => {
                let request_handle = self.app_server_request_handle.clone();
                let login_id = state.login_id.clone();
                tokio::spawn(async move {
                    cancel_login_attempt(&request_handle, login_id).await;
                });
            }
            SignInState::ChatGptDeviceCode(state) => {
                if let Some(login_id) = state.login_id().map(str::to_owned) {
                    let request_handle = self.app_server_request_handle.clone();
                    tokio::spawn(async move {
                        cancel_login_attempt(&request_handle, login_id).await;
                    });
                }
            }
            _ => return,
        }
        *sign_in_state = SignInState::PickMode;
        drop(sign_in_state);
        self.set_error(/*message*/ None);
        self.request_frame.schedule_frame();
    }

    fn set_error(&self, message: Option<String>) {
        *self.error.write().unwrap() = message;
    }

    fn error_message(&self) -> Option<String> {
        self.error.read().unwrap().clone()
    }

    /// Returns whether the auth flow is currently in API-key entry mode.
    pub(crate) fn is_api_key_entry_active(&self) -> bool {
        self.sign_in_state
            .read()
            .is_ok_and(|guard| matches!(&*guard, SignInState::ApiKeyEntry(_)))
    }

    /// Returns whether the provider setup form currently contains any text.
    pub(crate) fn api_key_entry_has_text(&self) -> bool {
        self.sign_in_state.read().is_ok_and(
            |guard| matches!(&*guard, SignInState::ApiKeyEntry(state) if state.has_text()),
        )
    }

    fn confirm_binding(&self) -> KeyBinding {
        keys::CONFIRM[0]
    }

    fn cancel_binding(&self) -> KeyBinding {
        keys::CANCEL[0]
    }

    fn displayed_sign_in_options(&self) -> Vec<SignInOption> {
        vec![SignInOption::ApiKey]
    }

    fn selectable_sign_in_options(&self) -> Vec<SignInOption> {
        vec![SignInOption::ApiKey]
    }

    fn move_highlight(&mut self, delta: isize) {
        let options = self.selectable_sign_in_options();
        if options.is_empty() {
            return;
        }

        let current_index = options
            .iter()
            .position(|option| *option == self.highlighted_mode)
            .unwrap_or(0);
        let next_index =
            (current_index as isize + delta).rem_euclid(options.len() as isize) as usize;
        self.highlighted_mode = options[next_index];
    }

    fn select_option_by_index(&mut self, index: usize) {
        let options = self.displayed_sign_in_options();
        if let Some(option) = options.get(index).copied() {
            self.handle_sign_in_option(option);
        }
    }

    fn handle_sign_in_option(&mut self, option: SignInOption) {
        match option {
            SignInOption::ChatGpt | SignInOption::DeviceCode => self.start_api_key_entry(),
            SignInOption::ApiKey => self.start_api_key_entry(),
        }
    }

    fn render_pick_mode(&self, area: Rect, buf: &mut Buffer) {
        let mut lines: Vec<Line> = vec![
            Line::from(vec!["  ".into(), "配置 VSP Provider".into()]),
            Line::from(vec![
                "  ".into(),
                "填写 OpenAI Responses API 兼容的 Base URL 和 API Key".into(),
            ]),
            "".into(),
        ];

        let create_mode_item = |idx: usize,
                                selected_mode: SignInOption,
                                text: &str,
                                description: &str|
         -> Vec<Line<'static>> {
            let is_selected = self.highlighted_mode == selected_mode;
            let caret = if is_selected { ">" } else { " " };

            let line1 = if is_selected {
                Line::from(vec![
                    format!("{caret} {index}. ", index = idx + 1).cyan().dim(),
                    text.to_string().cyan(),
                ])
            } else {
                format!("  {index}. {text}", index = idx + 1).into()
            };

            let line2 = if is_selected {
                Line::from(format!("     {description}"))
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::DIM)
            } else {
                Line::from(format!("     {description}"))
                    .style(Style::default().add_modifier(Modifier::DIM))
            };

            vec![line1, line2]
        };

        for (idx, option) in self.displayed_sign_in_options().into_iter().enumerate() {
            match option {
                SignInOption::ChatGpt => {
                    lines.extend(create_mode_item(
                        idx,
                        option,
                        "配置 VSP Provider",
                        "保存到 ~/.vsp-codex/config.toml",
                    ));
                }
                SignInOption::DeviceCode => {
                    lines.extend(create_mode_item(
                        idx,
                        option,
                        "继续配置",
                        "使用 Base URL / API Key",
                    ));
                }
                SignInOption::ApiKey => {
                    lines.extend(create_mode_item(
                        idx,
                        option,
                        "配置 API Key",
                        "OpenAI Responses API 兼容 Provider",
                    ));
                }
            }
            lines.push("".into());
        }

        lines.push(Line::from(vec![
            "  Press ".dim(),
            self.confirm_binding().into(),
            " to continue".dim(),
        ]));
        if let Some(err) = self.error_message() {
            lines.push("".into());
            lines.push(err.red().into());
        }

        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }

    fn render_continue_in_browser(&self, area: Rect, buf: &mut Buffer) {
        let mut spans = vec!["  ".into()];
        if self.animations_enabled && !self.animations_suppressed.get() {
            // Schedule a follow-up frame to keep the shimmer animation going.
            self.request_frame
                .schedule_frame_in(std::time::Duration::from_millis(100));
            spans.extend(shimmer_text(
                "Finish signing in via your browser",
                MotionMode::Animated,
            ));
        } else {
            spans.push("Finish signing in via your browser".into());
        }
        let mut lines = vec![spans.into(), "".into()];

        let sign_in_state = self.sign_in_state.read().unwrap();
        let auth_url = if let SignInState::ChatGptContinueInBrowser(state) = &*sign_in_state
            && !state.auth_url.is_empty()
        {
            lines.push("  If the link doesn't open automatically, open the following link to authenticate:".into());
            lines.push("".into());
            lines.push(Line::from(vec![
                "  ".into(),
                state.auth_url.as_str().cyan().underlined(),
            ]));
            lines.push("".into());
            lines.push(Line::from(vec![
                "  On a remote or headless machine? Press ".into(),
                self.cancel_binding().into(),
                " and choose ".into(),
                "Sign in with Device Code".cyan(),
                ".".into(),
            ]));
            lines.push("".into());
            Some(state.auth_url.clone())
        } else {
            None
        };

        lines.push(Line::from(vec![
            "  Press ".dim(),
            self.cancel_binding().into(),
            " to cancel".dim(),
        ]));
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .render(area, buf);

        // Wrap cyan+underlined URL cells with OSC 8 so the terminal treats
        // the entire region as a single clickable hyperlink.
        if let Some(url) = &auth_url {
            mark_url_hyperlink(buf, area, url);
        }
    }

    fn render_chatgpt_success_message(&self, area: Rect, buf: &mut Buffer) {
        let lines = vec![
            "✓ Signed in with your ChatGPT account"
                .fg(Color::Green)
                .into(),
            "".into(),
            "  Before you start:".into(),
            "".into(),
            "  Decide how much autonomy you want to grant Codex".into(),
            Line::from(vec![
                "  For more details see the ".into(),
                crate::terminal_hyperlinks::osc8_hyperlink(
                    "https://developers.openai.com/codex/security",
                    "Codex docs",
                )
                .underlined(),
            ])
            .dim(),
            "".into(),
            "  Codex can make mistakes".into(),
            "  Review the code it writes and commands it runs"
                .dim()
                .into(),
            "".into(),
            "  Powered by your ChatGPT account".into(),
            Line::from(vec![
                "  Uses your plan's rate limits and ".into(),
                crate::terminal_hyperlinks::osc8_hyperlink(
                    "https://chatgpt.com/#settings",
                    "training data preferences",
                )
                .underlined(),
            ])
            .dim(),
            "".into(),
            Line::from(vec![
                "  Press ".fg(Color::Cyan),
                self.confirm_binding().into(),
                " to continue".fg(Color::Cyan),
            ]),
        ];

        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }

    fn render_chatgpt_success(&self, area: Rect, buf: &mut Buffer) {
        let lines = vec![
            "✓ Signed in with your ChatGPT account"
                .fg(Color::Green)
                .into(),
        ];

        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }

    fn render_api_key_configured(&self, area: Rect, buf: &mut Buffer) {
        let lines = vec![
            "✓ VSP Provider 已配置".fg(Color::Green).into(),
            "".into(),
            "  后续请求将使用配置的 Base URL 和 API Key。".into(),
        ];

        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }

    fn render_api_key_entry(&self, area: Rect, buf: &mut Buffer, state: &ApiKeyInputState) {
        let [intro_area, base_url_area, api_key_area, footer_area] = Layout::vertical([
            Constraint::Min(4),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(2),
        ])
        .areas(area);

        let mut intro_lines: Vec<Line> = vec![
            Line::from(vec!["> ".into(), "配置 VSP Provider".bold()]),
            "".into(),
            "  填写 OpenAI Responses API 兼容的 Base URL 和 API Key。".into(),
            "  配置将保存到 ~/.vsp-codex/config.toml。".dim().into(),
            "".into(),
        ];
        if state.prepopulated_api_key_from_env {
            intro_lines.push("  检测到 OPENAI_API_KEY，已预填 API Key。".into());
            intro_lines.push("  如需使用其他 Key，可以直接覆盖。".dim().into());
            intro_lines.push("".into());
        }
        Paragraph::new(intro_lines)
            .wrap(Wrap { trim: false })
            .render(intro_area, buf);

        Self::render_provider_input(
            base_url_area,
            buf,
            "Base URL",
            state.base_url.as_str(),
            "https://.../v1",
            state.focused_field == ProviderInputField::BaseUrl,
            /*masked*/ false,
        );
        Self::render_provider_input(
            api_key_area,
            buf,
            "API Key",
            state.api_key.as_str(),
            "Paste or type API Key",
            state.focused_field == ProviderInputField::ApiKey,
            /*masked*/ true,
        );

        let mut footer_lines: Vec<Line> = vec![Line::from(vec![
            "  ".dim(),
            self.confirm_binding().into(),
            " 保存".dim(),
            "   Tab 切换字段".dim(),
            "   ".dim(),
            self.cancel_binding().into(),
            " 返回".dim(),
        ])];
        if let Some(error) = self.error_message() {
            footer_lines.push("".into());
            footer_lines.push(error.red().into());
        }
        Paragraph::new(footer_lines)
            .wrap(Wrap { trim: false })
            .render(footer_area, buf);
    }

    fn render_provider_input(
        area: Rect,
        buf: &mut Buffer,
        title: &'static str,
        value: &str,
        placeholder: &'static str,
        focused: bool,
        masked: bool,
    ) {
        let content_line: Line = if value.is_empty() {
            vec![placeholder.dim()].into()
        } else if masked {
            Line::from(vsp_provider::masked_secret(value))
        } else {
            Line::from(value.to_string())
        };
        Paragraph::new(content_line)
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(if focused {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default()
                    }),
            )
            .render(area, buf);
    }

    fn handle_api_key_entry_key_event(&mut self, key_event: &KeyEvent) -> bool {
        let mut should_save: Option<(String, String)> = None;
        let mut should_request_frame = false;

        {
            let mut guard = self.sign_in_state.write().unwrap();
            if let SignInState::ApiKeyEntry(state) = &mut *guard {
                if keys::CANCEL.is_pressed(*key_event) {
                    self.set_error(Some("请完成 Provider 配置，或按 q 退出。".to_string()));
                    should_request_frame = true;
                } else if keys::CONFIRM.is_pressed(*key_event) {
                    match vsp_provider::normalize_base_url(&state.base_url) {
                        Ok(base_url) => {
                            let api_key = state.api_key.trim().to_string();
                            if api_key.is_empty() {
                                self.set_error(Some("API Key 不能为空".to_string()));
                                should_request_frame = true;
                            } else {
                                should_save = Some((base_url, api_key));
                            }
                        }
                        Err(message) => {
                            self.set_error(Some(message));
                            should_request_frame = true;
                        }
                    }
                } else {
                    match key_event.code {
                        KeyCode::Tab | KeyCode::Up | KeyCode::Down => {
                            state.toggle_focus();
                            self.set_error(/*message*/ None);
                            should_request_frame = true;
                        }
                        KeyCode::Backspace => {
                            if state.focused_field == ProviderInputField::ApiKey
                                && state.prepopulated_api_key_from_env
                            {
                                state.api_key.clear();
                                state.prepopulated_api_key_from_env = false;
                            } else {
                                state.focused_value_mut().pop();
                            }
                            self.set_error(/*message*/ None);
                            should_request_frame = true;
                        }
                        KeyCode::Char(c)
                            if key_event.kind == KeyEventKind::Press
                                && !key_event.modifiers.contains(KeyModifiers::SUPER)
                                && !key_event.modifiers.contains(KeyModifiers::CONTROL)
                                && !key_event.modifiers.contains(KeyModifiers::ALT) =>
                        {
                            if state.focused_field == ProviderInputField::ApiKey
                                && state.prepopulated_api_key_from_env
                            {
                                state.api_key.clear();
                                state.prepopulated_api_key_from_env = false;
                            }
                            state.focused_value_mut().push(c);
                            self.set_error(/*message*/ None);
                            should_request_frame = true;
                        }
                        _ => {}
                    }
                }
                // handled; let guard drop before potential save
            } else {
                return false;
            }
        }

        if let Some((base_url, api_key)) = should_save {
            self.save_api_key(base_url, api_key);
        } else if should_request_frame {
            self.request_frame.schedule_frame();
        }
        true
    }

    fn handle_api_key_entry_paste(&mut self, pasted: String) -> bool {
        let trimmed = pasted.trim();
        if trimmed.is_empty() {
            return false;
        }

        let mut guard = self.sign_in_state.write().unwrap();
        if let SignInState::ApiKeyEntry(state) = &mut *guard {
            if state.focused_field == ProviderInputField::ApiKey
                && state.prepopulated_api_key_from_env
            {
                state.api_key = trimmed.to_string();
                state.prepopulated_api_key_from_env = false;
            } else {
                state.focused_value_mut().push_str(trimmed);
            }
            self.set_error(/*message*/ None);
        } else {
            return false;
        }

        drop(guard);
        self.request_frame.schedule_frame();
        true
    }

    fn start_api_key_entry(&mut self) {
        self.set_error(/*message*/ None);
        let prefill_from_env = read_openai_api_key_from_env();
        let mut guard = self.sign_in_state.write().unwrap();
        match &mut *guard {
            SignInState::ApiKeyEntry(state) => {
                if state.api_key.is_empty() {
                    if let Some(prefill) = prefill_from_env {
                        state.api_key = prefill;
                        state.prepopulated_api_key_from_env = true;
                    } else {
                        state.prepopulated_api_key_from_env = false;
                    }
                }
            }
            _ => {
                *guard = SignInState::ApiKeyEntry(ApiKeyInputState::new(
                    vsp_provider::default_provider_base_url(),
                    prefill_from_env.clone().unwrap_or_default(),
                    prefill_from_env.is_some(),
                ));
            }
        }
        drop(guard);
        self.request_frame.schedule_frame();
    }

    fn save_api_key(&mut self, base_url: String, api_key: String) {
        self.set_error(/*message*/ None);
        let request_handle = self.app_server_request_handle.clone();
        let sign_in_state = self.sign_in_state.clone();
        let error = self.error.clone();
        let request_frame = self.request_frame.clone();
        tokio::spawn(async move {
            match crate::config_update::write_config_batch(
                request_handle,
                vsp_provider::build_vsp_provider_setup_edits(&base_url, &api_key),
            )
            .await
            {
                Ok(_) => {
                    *error.write().unwrap() = None;
                    *sign_in_state.write().unwrap() = SignInState::ApiKeyConfigured;
                }
                Err(err) => {
                    *error.write().unwrap() = Some(format!("Provider 保存失败: {err}"));
                    *sign_in_state.write().unwrap() =
                        SignInState::ApiKeyEntry(ApiKeyInputState::new(
                            base_url, api_key, /*prepopulated_api_key_from_env*/ false,
                        ));
                }
            }
            request_frame.schedule_frame();
        });
        self.request_frame.schedule_frame();
    }

    #[allow(dead_code)]
    fn handle_existing_chatgpt_login(&mut self) -> bool {
        if matches!(
            self.login_status,
            LoginStatus::AuthMode(auth_mode) if auth_mode.has_chatgpt_account()
        ) {
            *self.sign_in_state.write().unwrap() = SignInState::ChatGptSuccess;
            self.request_frame.schedule_frame();
            true
        } else {
            false
        }
    }

    /// Kicks off the ChatGPT auth flow and keeps the UI state consistent with the attempt.
    #[allow(dead_code)]
    fn start_chatgpt_login(&mut self) {
        // If we're already authenticated with ChatGPT, don't start a new login –
        // just proceed to the success message flow.
        if self.handle_existing_chatgpt_login() {
            return;
        }

        self.set_error(/*message*/ None);
        let request_handle = self.app_server_request_handle.clone();
        let sign_in_state = self.sign_in_state.clone();
        let error = self.error.clone();
        let request_frame = self.request_frame.clone();
        tokio::spawn(async move {
            match request_handle
                .request_typed::<LoginAccountResponse>(ClientRequest::LoginAccount {
                    request_id: onboarding_request_id(),
                    params: LoginAccountParams::Chatgpt {
                        app_brand: None,
                        codex_streamlined_login: false,
                        use_hosted_login_success_page: false,
                    },
                })
                .await
            {
                Ok(LoginAccountResponse::Chatgpt { login_id, auth_url }) => {
                    maybe_open_auth_url_in_browser(&request_handle, &auth_url);
                    *error.write().unwrap() = None;
                    *sign_in_state.write().unwrap() =
                        SignInState::ChatGptContinueInBrowser(ContinueInBrowserState {
                            login_id,
                            auth_url,
                        });
                }
                Ok(other) => {
                    *sign_in_state.write().unwrap() = SignInState::PickMode;
                    *error.write().unwrap() = Some(format!(
                        "Unexpected account/login/start response: {other:?}"
                    ));
                }
                Err(err) => {
                    *sign_in_state.write().unwrap() = SignInState::PickMode;
                    *error.write().unwrap() = Some(err.to_string());
                }
            }
            request_frame.schedule_frame();
        });
    }

    #[allow(dead_code)]
    fn start_device_code_login(&mut self) {
        if self.handle_existing_chatgpt_login() {
            return;
        }

        self.set_error(/*message*/ None);
        headless_chatgpt_login::start_headless_chatgpt_login(self);
    }

    pub(crate) fn on_account_login_completed(
        &mut self,
        notification: AccountLoginCompletedNotification,
    ) {
        let Some(login_id) = notification.login_id else {
            return;
        };
        let guard = self.sign_in_state.read().unwrap();
        let is_matching_login = matches!(
            &*guard,
            SignInState::ChatGptContinueInBrowser(state) if state.login_id == login_id
        ) || matches!(
            &*guard,
            SignInState::ChatGptDeviceCode(state) if state.login_id() == Some(login_id.as_str())
        );
        drop(guard);
        if !is_matching_login {
            return;
        }

        if notification.success {
            self.set_error(/*message*/ None);
            *self.sign_in_state.write().unwrap() = SignInState::ChatGptSuccessMessage;
        } else {
            self.set_error(notification.error);
            *self.sign_in_state.write().unwrap() = SignInState::PickMode;
        }
        self.request_frame.schedule_frame();
    }

    pub(crate) fn on_account_updated(&mut self, notification: AccountUpdatedNotification) {
        self.login_status = notification
            .auth_mode
            .map(|auth_mode| {
                LoginStatus::AuthMode(match auth_mode {
                    ApiAuthMode::ApiKey => AuthMode::ApiKey,
                    ApiAuthMode::Chatgpt => AuthMode::Chatgpt,
                    ApiAuthMode::ChatgptAuthTokens => AuthMode::ChatgptAuthTokens,
                    ApiAuthMode::Headers => AuthMode::Headers,
                    ApiAuthMode::AgentIdentity => AuthMode::AgentIdentity,
                    ApiAuthMode::PersonalAccessToken => AuthMode::PersonalAccessToken,
                    ApiAuthMode::BedrockApiKey => AuthMode::BedrockApiKey,
                })
            })
            .unwrap_or(LoginStatus::NotAuthenticated);
    }
}

impl StepStateProvider for AuthModeWidget {
    fn get_step_state(&self) -> StepState {
        let sign_in_state = self.sign_in_state.read().unwrap();
        match &*sign_in_state {
            SignInState::PickMode
            | SignInState::ApiKeyEntry(_)
            | SignInState::ChatGptContinueInBrowser(_)
            | SignInState::ChatGptDeviceCode(_)
            | SignInState::ChatGptSuccessMessage => StepState::InProgress,
            SignInState::ChatGptSuccess | SignInState::ApiKeyConfigured => StepState::Complete,
        }
    }
}

impl WidgetRef for AuthModeWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let sign_in_state = self.sign_in_state.read().unwrap();
        match &*sign_in_state {
            SignInState::PickMode => {
                self.render_pick_mode(area, buf);
            }
            SignInState::ChatGptContinueInBrowser(_) => {
                self.render_continue_in_browser(area, buf);
            }
            SignInState::ChatGptDeviceCode(state) => {
                headless_chatgpt_login::render_device_code_login(self, area, buf, state);
            }
            SignInState::ChatGptSuccessMessage => {
                self.render_chatgpt_success_message(area, buf);
            }
            SignInState::ChatGptSuccess => {
                self.render_chatgpt_success(area, buf);
            }
            SignInState::ApiKeyEntry(state) => {
                self.render_api_key_entry(area, buf, state);
            }
            SignInState::ApiKeyConfigured => {
                self.render_api_key_configured(area, buf);
            }
        }
    }
}

#[allow(dead_code)]
pub(super) fn maybe_open_auth_url_in_browser(request_handle: &AppServerRequestHandle, url: &str) {
    if !matches!(request_handle, AppServerRequestHandle::InProcess(_)) {
        return;
    }

    if let Err(err) = webbrowser::open(url) {
        tracing::warn!("failed to open browser for login URL: {err}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legacy_core::config::ConfigBuilder;
    use codex_app_server_client::AppServerRequestHandle;
    use codex_app_server_client::DEFAULT_IN_PROCESS_CHANNEL_CAPACITY;
    use codex_app_server_client::InProcessAppServerClient;
    use codex_app_server_client::InProcessClientStartArgs;
    use codex_arg0::Arg0DispatchPaths;
    use codex_cloud_config::cloud_config_bundle_loader_for_storage;
    use codex_config::types::AuthCredentialsStoreMode;
    use codex_login::AuthKeyringBackendKind;

    use pretty_assertions::assert_eq;
    use std::sync::Arc;
    use tempfile::TempDir;

    async fn widget_forced_chatgpt() -> (AuthModeWidget, TempDir) {
        let codex_home = TempDir::new().unwrap();
        let codex_home_path = codex_home.path().to_path_buf();
        let config = ConfigBuilder::default()
            .codex_home(codex_home_path.clone())
            .build()
            .await
            .unwrap();
        let client = InProcessAppServerClient::start(InProcessClientStartArgs {
            arg0_paths: Arg0DispatchPaths::default(),
            config: Arc::new(config),
            cli_overrides: Vec::new(),
            loader_overrides: Default::default(),
            strict_config: false,
            cloud_config_bundle: cloud_config_bundle_loader_for_storage(
                codex_home_path.clone(),
                /*enable_codex_api_key_env*/ false,
                AuthCredentialsStoreMode::File,
                AuthKeyringBackendKind::default(),
                "https://chatgpt.com/backend-api/".to_string(),
                /*auth_route_config*/ None,
            )
            .await,
            feedback: codex_feedback::CodexFeedback::new(),
            log_db: None,
            state_db: None,
            environment_manager: Arc::new(
                codex_app_server_client::EnvironmentManager::default_for_tests(),
            ),
            config_warnings: Vec::new(),
            session_source: serde_json::from_value(serde_json::json!("cli"))
                .expect("cli session source should deserialize"),
            enable_codex_api_key_env: false,
            client_name: "test".to_string(),
            client_version: "test".to_string(),
            experimental_api: true,
            native_ask_ui: false,
            mcp_server_openai_form_elicitation: false,
            opt_out_notification_methods: Vec::new(),
            channel_capacity: DEFAULT_IN_PROCESS_CHANNEL_CAPACITY,
        })
        .await
        .unwrap();
        let widget = AuthModeWidget {
            request_frame: FrameRequester::test_dummy(),
            highlighted_mode: SignInOption::ChatGpt,
            error: Arc::new(RwLock::new(None)),
            sign_in_state: Arc::new(RwLock::new(SignInState::PickMode)),
            login_status: LoginStatus::NotAuthenticated,
            app_server_request_handle: AppServerRequestHandle::InProcess(client.request_handle()),
            forced_login_method: Some(ForcedLoginMethod::Chatgpt),
            animations_enabled: true,
            animations_suppressed: std::cell::Cell::new(false),
        };
        (widget, codex_home)
    }

    #[tokio::test]
    async fn provider_flow_is_available_when_chatgpt_is_forced() {
        let (mut widget, _tmp) = widget_forced_chatgpt().await;

        widget.start_api_key_entry();

        assert_eq!(widget.error_message(), None);
        assert!(matches!(
            &*widget.sign_in_state.read().unwrap(),
            SignInState::ApiKeyEntry(_)
        ));
    }

    #[tokio::test]
    async fn saving_api_key_is_blocked_when_chatgpt_forced() {
        let (mut widget, _tmp) = widget_forced_chatgpt().await;

        widget.save_api_key(
            vsp_provider::default_provider_base_url(),
            "sk-test".to_string(),
        );

        assert_eq!(widget.error_message().as_deref(), None);
    }

    #[tokio::test]
    async fn existing_non_oauth_chatgpt_login_counts_as_signed_in() {
        for auth_mode in [AuthMode::ChatgptAuthTokens, AuthMode::PersonalAccessToken] {
            let (mut widget, _tmp) = widget_forced_chatgpt().await;
            widget.login_status = LoginStatus::AuthMode(auth_mode);

            let handled = widget.handle_existing_chatgpt_login();

            assert_eq!(handled, true);
            assert!(matches!(
                &*widget.sign_in_state.read().unwrap(),
                SignInState::ChatGptSuccess
            ));
        }
    }

    #[tokio::test]
    async fn cancel_active_attempt_resets_browser_login_state() {
        let (widget, _tmp) = widget_forced_chatgpt().await;
        *widget.error.write().unwrap() = Some("still logging in".to_string());
        *widget.sign_in_state.write().unwrap() =
            SignInState::ChatGptContinueInBrowser(ContinueInBrowserState {
                login_id: "login-1".to_string(),
                auth_url: "https://auth.example.com".to_string(),
            });

        widget.cancel_active_attempt();

        assert_eq!(widget.error_message(), None);
        assert!(matches!(
            &*widget.sign_in_state.read().unwrap(),
            SignInState::PickMode
        ));
    }

    #[tokio::test]
    async fn cancel_active_attempt_notifies_device_code_login() {
        let (widget, _tmp) = widget_forced_chatgpt().await;
        *widget.error.write().unwrap() = Some("still logging in".to_string());
        *widget.sign_in_state.write().unwrap() =
            SignInState::ChatGptDeviceCode(ContinueWithDeviceCodeState::ready(
                "request-1".to_string(),
                "login-1".to_string(),
                "https://chatgpt.com/device".to_string(),
                "ABCD-EFGH".to_string(),
            ));

        widget.cancel_active_attempt();

        assert_eq!(widget.error_message(), None);
        assert!(matches!(
            &*widget.sign_in_state.read().unwrap(),
            SignInState::PickMode
        ));
    }

    /// Collects all buffer cell symbols that contain the OSC 8 open sequence
    /// for the given URL.  Returns the concatenated "inner" characters.
    fn collect_osc8_chars(buf: &Buffer, area: Rect, url: &str) -> String {
        let open = format!("\x1B]8;;{url}\x07");
        let close = "\x1B]8;;\x07";
        let mut chars = String::new();
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let sym = buf[(x, y)].symbol();
                if let Some(rest) = sym.strip_prefix(open.as_str())
                    && let Some(ch) = rest.strip_suffix(close)
                {
                    chars.push_str(ch);
                }
            }
        }
        chars
    }

    #[test]
    fn continue_in_browser_renders_osc8_hyperlink() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let (widget, _tmp) = runtime.block_on(widget_forced_chatgpt());
        let url = "https://auth.example.com/login?state=abc123";
        *widget.sign_in_state.write().unwrap() =
            SignInState::ChatGptContinueInBrowser(ContinueInBrowserState {
                login_id: "login-1".to_string(),
                auth_url: url.to_string(),
            });

        // Render into a narrow buffer so the URL wraps across multiple rows.
        let area = Rect::new(0, 0, 30, 20);
        let mut buf = Buffer::empty(area);
        widget.render_continue_in_browser(area, &mut buf);

        // Every character of the URL should be present as an OSC 8 cell.
        let found = collect_osc8_chars(&buf, area, url);
        assert_eq!(found, url, "OSC 8 hyperlink should cover the full URL");
    }

    #[test]
    fn auth_widget_suppresses_animations_when_device_code_is_visible() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let (widget, _tmp) = runtime.block_on(widget_forced_chatgpt());
        *widget.sign_in_state.write().unwrap() =
            SignInState::ChatGptDeviceCode(ContinueWithDeviceCodeState::ready(
                "request-1".to_string(),
                "login-1".to_string(),
                "https://chatgpt.com/device".to_string(),
                "ABCD-EFGH".to_string(),
            ));

        assert_eq!(widget.should_suppress_animations(), true);
    }

    #[test]
    fn auth_widget_suppresses_animations_while_requesting_device_code() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let (widget, _tmp) = runtime.block_on(widget_forced_chatgpt());
        *widget.sign_in_state.write().unwrap() = SignInState::ChatGptDeviceCode(
            ContinueWithDeviceCodeState::pending("request-1".to_string()),
        );

        assert_eq!(widget.should_suppress_animations(), true);
    }

    #[tokio::test]
    async fn device_code_login_completion_advances_to_success_message() {
        let (mut widget, _tmp) = widget_forced_chatgpt().await;
        *widget.sign_in_state.write().unwrap() =
            SignInState::ChatGptDeviceCode(ContinueWithDeviceCodeState::ready(
                "request-1".to_string(),
                "login-1".to_string(),
                "https://chatgpt.com/device".to_string(),
                "ABCD-EFGH".to_string(),
            ));

        widget.on_account_login_completed(AccountLoginCompletedNotification {
            login_id: Some("login-1".to_string()),
            success: true,
            error: None,
        });

        assert!(matches!(
            &*widget.sign_in_state.read().unwrap(),
            SignInState::ChatGptSuccessMessage
        ));
    }

    #[test]
    fn mark_url_hyperlink_wraps_cyan_underlined_cells() {
        let url = "https://example.com";
        let area = Rect::new(0, 0, 20, 1);
        let mut buf = Buffer::empty(area);

        // Manually write some cyan+underlined characters to simulate a rendered URL.
        for (i, ch) in "example".chars().enumerate() {
            let cell = &mut buf[(i as u16, 0)];
            cell.set_symbol(&ch.to_string());
            cell.fg = Color::Cyan;
            cell.modifier = Modifier::UNDERLINED;
        }
        // Leave a plain cell that should NOT be marked.
        buf[(7, 0)].set_symbol("X");

        mark_url_hyperlink(&mut buf, area, url);

        // Each cyan+underlined cell should now carry the OSC 8 wrapper.
        let found = collect_osc8_chars(&buf, area, url);
        assert_eq!(found, "example");

        // The plain "X" cell should be untouched.
        assert_eq!(buf[(7, 0)].symbol(), "X");
    }

    #[test]
    fn mark_url_hyperlink_sanitizes_control_chars() {
        let area = Rect::new(0, 0, 10, 1);
        let mut buf = Buffer::empty(area);

        // One cyan+underlined cell to mark.
        let cell = &mut buf[(0, 0)];
        cell.set_symbol("a");
        cell.fg = Color::Cyan;
        cell.modifier = Modifier::UNDERLINED;

        // URL contains ESC and BEL that could break the OSC 8 sequence.
        let malicious_url = "https://evil.com/\x1B]8;;\x07injected";
        mark_url_hyperlink(&mut buf, area, malicious_url);

        let sym = buf[(0, 0)].symbol().to_string();
        // The sanitized URL retains `]` (printable) but strips ESC and BEL.
        let sanitized = "https://evil.com/]8;;injected";
        assert!(
            sym.contains(sanitized),
            "symbol should contain sanitized URL, got: {sym:?}"
        );
        // The injected close-sequence must not survive: \x1B and \x07 are gone.
        assert!(
            !sym.contains("\x1B]8;;\x07injected"),
            "symbol must not contain raw control chars from URL"
        );
    }
}
