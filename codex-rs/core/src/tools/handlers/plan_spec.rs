use codex_tools::JsonSchema;
use codex_tools::ResponsesApiTool;
use codex_tools::ToolSpec;
use std::collections::BTreeMap;

pub fn create_update_plan_tool() -> ToolSpec {
    let status_schema = JsonSchema::string_enum(
        vec![
            serde_json::json!("pending"),
            serde_json::json!("in_progress"),
            serde_json::json!("completed"),
        ],
        Some("One of: pending, in_progress, completed".to_string()),
    );
    let plan_item_properties = BTreeMap::from([
        ("step".to_string(), JsonSchema::string(/*description*/ None)),
        ("status".to_string(), status_schema.clone()),
    ]);
    let plan_sub_item_properties = BTreeMap::from([
        (
            "id".to_string(),
            JsonSchema::string(Some("Optional stable subplan item id.".to_string())),
        ),
        (
            "title".to_string(),
            JsonSchema::string(/*description*/ None),
        ),
        (
            "summary".to_string(),
            JsonSchema::string(Some(
                "Optional short detail for this subplan item.".to_string(),
            )),
        ),
        ("status".to_string(), status_schema.clone()),
    ]);
    let subplan_properties = BTreeMap::from([
        (
            "title".to_string(),
            JsonSchema::string(Some(
                "Optional title for this one-level subplan.".to_string(),
            )),
        ),
        (
            "summary".to_string(),
            JsonSchema::string(Some("Optional short detail for this subplan.".to_string())),
        ),
        (
            "items".to_string(),
            JsonSchema::array(
                JsonSchema::object(
                    plan_sub_item_properties,
                    Some(vec!["title".to_string(), "status".to_string()]),
                    Some(false.into()),
                ),
                Some("One-level subplan items.".to_string()),
            ),
        ),
    ]);
    let plan_node_properties = BTreeMap::from([
        (
            "id".to_string(),
            JsonSchema::string(Some("Optional stable root plan item id.".to_string())),
        ),
        (
            "title".to_string(),
            JsonSchema::string(/*description*/ None),
        ),
        (
            "summary".to_string(),
            JsonSchema::string(Some(
                "Optional short detail for this root item.".to_string(),
            )),
        ),
        ("status".to_string(), status_schema),
        (
            "subplan".to_string(),
            JsonSchema::object(
                subplan_properties,
                Some(vec!["items".to_string()]),
                Some(false.into()),
            ),
        ),
    ]);
    let plan_state_properties = BTreeMap::from([
        (
            "id".to_string(),
            JsonSchema::string(Some("Optional stable plan id.".to_string())),
        ),
        (
            "title".to_string(),
            JsonSchema::string(Some("Optional plan title.".to_string())),
        ),
        (
            "summary".to_string(),
            JsonSchema::string(Some("Optional plan summary.".to_string())),
        ),
        (
            "items".to_string(),
            JsonSchema::array(
                JsonSchema::object(
                    plan_node_properties,
                    Some(vec!["title".to_string(), "status".to_string()]),
                    Some(false.into()),
                ),
                Some("Root plan items. Each item may include one subplan.".to_string()),
            ),
        ),
    ]);

    let properties = BTreeMap::from([
        (
            "explanation".to_string(),
            JsonSchema::string(Some(
                "Optional explanation for this plan update.".to_string(),
            )),
        ),
        (
            "plan".to_string(),
            JsonSchema::array(
                JsonSchema::object(
                    plan_item_properties,
                    Some(vec!["step".to_string(), "status".to_string()]),
                    Some(false.into()),
                ),
                Some("The list of steps".to_string()),
            ),
        ),
        (
            "state".to_string(),
            JsonSchema::object(
                plan_state_properties,
                Some(vec!["items".to_string()]),
                Some(false.into()),
            ),
        ),
    ]);

    ToolSpec::Function(ResponsesApiTool {
        name: "update_plan".to_string(),
        description: r#"Updates the task plan.
Provide an optional explanation and either the legacy flat `plan` list or structured `state`.
Structured state has root items and may include one `subplan` per root item.
At most one root or subplan item should be in_progress at a time.
"#
        .to_string(),
        strict: false,
        defer_loading: None,
        parameters: JsonSchema::object(
            properties,
            Some(vec!["plan".to_string()]),
            Some(false.into()),
        ),
        output_schema: None,
    })
}
