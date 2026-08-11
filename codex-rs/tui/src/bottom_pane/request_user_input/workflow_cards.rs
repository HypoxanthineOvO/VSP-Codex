#![allow(dead_code)]

use codex_app_server_protocol::ToolRequestUserInputOption;
use codex_app_server_protocol::ToolRequestUserInputParams;
use codex_app_server_protocol::ToolRequestUserInputQuestion;
use codex_app_server_protocol::ToolRequestUserInputQuestionKind;

pub(crate) fn plan_confirmation_zh_cn(
    summary: impl Into<String>,
    description: impl Into<String>,
) -> ToolRequestUserInputParams {
    let mut params = workflow_card(
        summary,
        description,
        "workflow_plan_confirmation",
        "计划确认",
        "请确认是否批准当前 Workflow 计划。",
        Some("批准后继续执行；也可以请求修改或取消本次流程。"),
        vec![
            option(
                "approve_plan",
                "批准计划/确认计划",
                "确认计划满足预期，继续进入下一步。",
            ),
            option("request_revision", "请求修改", "返回修改计划后再继续。"),
            option("cancel", "取消", "停止本次 Workflow 决策。"),
        ],
    );
    if let Some(question) = params.questions.first_mut() {
        question.workflow_decision_type = Some("plan_confirmation".to_string());
    }
    params
}

pub(crate) fn authorization_intent_zh_cn(
    summary: impl Into<String>,
    description: impl Into<String>,
) -> ToolRequestUserInputParams {
    let mut params = workflow_card(
        summary,
        description,
        "workflow_authorization_intent",
        "授权意图确认",
        "请确认是否表达执行该 Workflow 操作的授权意图。",
        Some(
            "这不是权限授予；真实权限仍由 request_permissions 或审批机制处理（approval overlay）。",
        ),
        vec![
            option(
                "confirm_intent",
                "确认意图",
                "确认继续推进，但不授予任何真实权限。",
            ),
            option(
                "return_for_revision",
                "返回修改",
                "调整操作范围或方案后再确认。",
            ),
            option("cancel", "取消", "停止本次 Workflow 决策。"),
        ],
    );
    if let Some(question) = params.questions.first_mut() {
        question.workflow_decision_type = Some("authorization_intent".to_string());
        question.intent_only = Some(true);
        question.grants_approval = Some(false);
    }
    params
}

/// Synthetic turn/thread id used for the client-only VSP release update card so its
/// answer can be intercepted locally instead of being forwarded to the app-server.
pub(crate) const VSP_RELEASE_UPDATE_TURN_ID: &str = "vsp_release_update";
/// Question id shared by [`release_update_confirmation_zh_cn`] and the local decision handler.
pub(crate) const VSP_RELEASE_UPDATE_QUESTION_ID: &str = "workflow_release_update_confirmation";

/// Build the in-chat update confirmation card for VSP (non-official) mode.
///
/// Reuses [`release_update_confirmation_zh_cn`] but rebinds the thread/turn ids to a sentinel so
/// the answer is handled locally (trigger the upgrade on exit / dismiss the version / view notes)
/// rather than routed to the app-server, which has no matching pending request.
pub(crate) fn vsp_release_update_card(
    current_version: &str,
    latest_version: &str,
    update_command: &str,
) -> ToolRequestUserInputParams {
    let summary = format!("发现新版本：{current_version} → {latest_version}");
    let description = format!("运行 `{update_command}` 即可更新到 {latest_version}。");
    let mut params = release_update_confirmation_zh_cn(summary, description);
    params.thread_id = VSP_RELEASE_UPDATE_TURN_ID.to_string();
    params.turn_id = VSP_RELEASE_UPDATE_TURN_ID.to_string();
    params.item_id = VSP_RELEASE_UPDATE_QUESTION_ID.to_string();
    params
}

pub(crate) fn release_update_confirmation_zh_cn(
    summary: impl Into<String>,
    description: impl Into<String>,
) -> ToolRequestUserInputParams {
    let mut params = workflow_card(
        summary,
        description,
        "workflow_release_update_confirmation",
        "更新确认",
        "请选择本次发布或更新的处理方式。",
        Some("该卡片只确认更新选择，不替代真实权限审批。"),
        vec![
            option("update_now", "立即更新", ""),
            option("skip_once", "本次跳过/跳过本次", ""),
            option("skip_this_version", "跳过此版本", ""),
            option("view_notes", "查看说明", ""),
            option("cancel", "取消", ""),
        ],
    );
    if let Some(question) = params.questions.first_mut() {
        question.workflow_decision_type = Some("release_update_confirmation".to_string());
    }
    params
}

fn workflow_card(
    summary: impl Into<String>,
    description: impl Into<String>,
    question_id: &str,
    header: &str,
    question: &str,
    question_description: Option<&str>,
    options: Vec<ToolRequestUserInputOption>,
) -> ToolRequestUserInputParams {
    ToolRequestUserInputParams {
        thread_id: "workflow".to_string(),
        turn_id: "workflow".to_string(),
        item_id: question_id.to_string(),
        summary: Some(summary.into()),
        description: Some(description.into()),
        questions: vec![ToolRequestUserInputQuestion {
            id: question_id.to_string(),
            kind: Some(ToolRequestUserInputQuestionKind::SingleChoice),
            header: header.to_string(),
            question: question.to_string(),
            description: question_description.map(str::to_string),
            is_other: false,
            is_secret: false,
            workflow_decision_type: None,
            intent_only: None,
            grants_approval: None,
            options: Some(options),
        }],
        auto_resolution_ms: None,
    }
}

fn option(id: &str, label: &str, description: &str) -> ToolRequestUserInputOption {
    ToolRequestUserInputOption {
        id: Some(id.to_string()),
        label: label.to_string(),
        description: description.to_string(),
    }
}
