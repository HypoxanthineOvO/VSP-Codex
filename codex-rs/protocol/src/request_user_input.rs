use std::collections::HashMap;

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct RequestUserInputQuestionOption {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub enum RequestUserInputQuestionKind {
    SingleChoice,
    MultiChoice,
    #[serde(alias = "shortAnswer", alias = "short_answer", alias = "free_text")]
    FreeText,
    #[serde(alias = "orderOnly", alias = "order_only")]
    Ranking,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, JsonSchema, TS)]
pub struct RequestUserInputQuestion {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<RequestUserInputQuestionKind>,
    pub header: String,
    pub question: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(
        rename = "isOther",
        default,
        skip_serializing_if = "std::ops::Not::not"
    )]
    #[schemars(rename = "isOther")]
    #[ts(rename = "isOther")]
    pub is_other: bool,
    #[serde(
        rename = "isSecret",
        default,
        skip_serializing_if = "std::ops::Not::not"
    )]
    #[schemars(rename = "isSecret")]
    #[ts(rename = "isSecret")]
    pub is_secret: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<RequestUserInputQuestionOption>>,
    #[serde(
        rename = "workflowDecisionType",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(rename = "workflowDecisionType")]
    #[ts(rename = "workflowDecisionType")]
    pub workflow_decision_type: Option<String>,
    #[serde(
        rename = "intentOnly",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(rename = "intentOnly")]
    #[ts(rename = "intentOnly")]
    pub intent_only: Option<bool>,
    #[serde(
        rename = "grantsApproval",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(rename = "grantsApproval")]
    #[ts(rename = "grantsApproval")]
    pub grants_approval: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, JsonSchema, TS)]
pub struct RequestUserInputArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub questions: Vec<RequestUserInputQuestion>,
    #[serde(rename = "autoResolutionMs", skip_serializing_if = "Option::is_none")]
    #[schemars(rename = "autoResolutionMs")]
    pub auto_resolution_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, JsonSchema, TS)]
pub struct RequestUserInputAnswer {
    #[serde(
        rename = "selectedOptionIds",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(rename = "selectedOptionIds")]
    #[ts(rename = "selectedOptionIds")]
    pub selected_option_ids: Option<Vec<String>>,
    #[serde(
        rename = "orderedOptionIds",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(rename = "orderedOptionIds")]
    #[ts(rename = "orderedOptionIds")]
    pub ordered_option_ids: Option<Vec<String>>,
    #[serde(rename = "freeText", default, skip_serializing_if = "Option::is_none")]
    #[schemars(rename = "freeText")]
    #[ts(rename = "freeText")]
    pub free_text: Option<String>,
    #[serde(
        rename = "legacyAnswers",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(rename = "legacyAnswers")]
    #[ts(rename = "legacyAnswers")]
    pub legacy_answers: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub answers: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, JsonSchema, TS)]
pub struct RequestUserInputResponse {
    pub answers: HashMap<String, RequestUserInputAnswer>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, JsonSchema, TS)]
pub struct RequestUserInputEvent {
    /// Responses API call id for the associated tool call, if available.
    pub call_id: String,
    /// Turn ID that this request belongs to.
    /// Uses `#[serde(default)]` for backwards compatibility.
    #[serde(default)]
    pub turn_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub questions: Vec<RequestUserInputQuestion>,
    #[serde(rename = "autoResolutionMs", skip_serializing_if = "Option::is_none")]
    #[schemars(rename = "autoResolutionMs")]
    pub auto_resolution_ms: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn request_user_input_args_round_trip_extended_ask_contract() {
        let payload = json!({
            "summary": "Choose release settings",
            "description": "These answers determine the deployment plan.",
            "questions": [
                {
                    "id": "deploy_target",
                    "kind": "singleChoice",
                    "header": "Target",
                    "question": "Where should this deploy?",
                    "description": "Pick exactly one environment.",
                    "options": [
                        {
                            "id": "staging",
                            "label": "Staging",
                            "description": "Internal validation"
                        },
                        {
                            "id": "prod",
                            "label": "Production",
                            "description": "Customer traffic"
                        }
                    ]
                },
                {
                    "id": "approvers",
                    "kind": "multiChoice",
                    "header": "Approvers",
                    "question": "Who must approve?",
                    "description": "Select every required reviewer.",
                    "options": [
                        {
                            "id": "qa",
                            "label": "QA",
                            "description": "Quality gate"
                        },
                        {
                            "id": "ops",
                            "label": "Ops",
                            "description": "Runtime gate"
                        }
                    ]
                },
                {
                    "id": "rollback_order",
                    "kind": "ranking",
                    "header": "Rollback",
                    "question": "Rank rollback checks.",
                    "description": "First item runs first.",
                    "options": [
                        {
                            "id": "db",
                            "label": "Database",
                            "description": "Verify migrations"
                        },
                        {
                            "id": "api",
                            "label": "API",
                            "description": "Verify health"
                        }
                    ]
                },
                {
                    "id": "secret_note",
                    "kind": "freeText",
                    "header": "Secret",
                    "question": "Paste the temporary token.",
                    "description": "This must be masked in native clients.",
                    "isSecret": true
                }
            ]
        });

        let args: RequestUserInputArgs =
            serde_json::from_value(payload.clone()).expect("extended Ask args deserialize");

        assert_eq!(
            serde_json::to_value(args).expect("extended Ask args serialize"),
            payload
        );
    }

    #[test]
    fn request_user_input_response_round_trip_structured_answers() {
        let payload = json!({
            "answers": {
                "deploy_target": {
                    "selectedOptionIds": ["staging"],
                    "legacyAnswers": ["Staging"]
                },
                "approvers": {
                    "selectedOptionIds": ["qa", "ops"]
                },
                "rollback_order": {
                    "orderedOptionIds": ["db", "api"]
                },
                "secret_note": {
                    "freeText": "token-123"
                }
            }
        });

        let response: RequestUserInputResponse =
            serde_json::from_value(payload.clone()).expect("structured Ask response deserialize");

        assert_eq!(
            serde_json::to_value(response).expect("structured Ask response serialize"),
            payload
        );
    }

    #[test]
    fn request_user_input_legacy_payloads_still_deserialize() {
        let args: RequestUserInputArgs = serde_json::from_value(json!({
            "questions": [{
                "id": "target",
                "header": "Target",
                "question": "Where should this deploy?",
                "options": [{
                    "label": "Staging",
                    "description": "Internal validation"
                }]
            }]
        }))
        .expect("legacy Ask args deserialize");

        assert_eq!(
            args.questions,
            vec![RequestUserInputQuestion {
                id: "target".to_string(),
                kind: None,
                header: "Target".to_string(),
                question: "Where should this deploy?".to_string(),
                description: None,
                is_other: false,
                is_secret: false,
                options: Some(vec![RequestUserInputQuestionOption {
                    id: None,
                    label: "Staging".to_string(),
                    description: "Internal validation".to_string(),
                }]),
                workflow_decision_type: None,
                intent_only: None,
                grants_approval: None,
            }]
        );

        let response: RequestUserInputResponse = serde_json::from_value(json!({
            "answers": {
                "target": {
                    "answers": ["Staging"]
                }
            }
        }))
        .expect("legacy Ask response deserialize");

        assert_eq!(
            response
                .answers
                .get("target")
                .expect("target answer is present")
                .answers,
            vec!["Staging".to_string()]
        );
    }
}
