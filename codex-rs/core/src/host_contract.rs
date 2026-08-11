use serde_json::Map;
use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;

pub const HOST_STATUS_RELATIVE_PATH: &str = ".pipeline/runtime/host-status-v1.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostContractStatus {
    Current,
    Invalidated,
    Legacy,
    Missing,
    Invalid(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostContractRead {
    pub status: HostContractStatus,
    pub projection: Option<Value>,
}

pub fn read_host_status(root: impl AsRef<Path>) -> HostContractRead {
    let root = root.as_ref();
    let path = root.join(HOST_STATUS_RELATIVE_PATH);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let status = if root.join(".pipeline/state.yaml").is_file() {
                HostContractStatus::Legacy
            } else {
                HostContractStatus::Missing
            };
            return HostContractRead {
                status,
                projection: None,
            };
        }
        Err(error) => return invalid(error.to_string()),
    };
    let projection: Value = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(error) => return invalid(error.to_string()),
    };
    match validate_projection(&projection) {
        Ok(HostContractStatus::Current) => HostContractRead {
            status: HostContractStatus::Current,
            projection: Some(projection),
        },
        Ok(HostContractStatus::Invalidated) => HostContractRead {
            status: HostContractStatus::Invalidated,
            projection: Some(projection),
        },
        Ok(_) => invalid("unsupported Host Contract status"),
        Err(error) => invalid(error),
    }
}

fn validate_projection(value: &Value) -> Result<HostContractStatus, String> {
    reject_sensitive_keys(value)?;
    let root = object_with_optional_keys(
        value,
        &[
            "schema_version",
            "contract_version",
            "projection_status",
            "generated_at",
            "generation",
            "workspace",
            "delivery",
            "continuation",
            "invalidation",
        ],
        &["work_items", "session"],
        "Host status projection",
    )?;
    require_literal(root, "schema_version", "1")?;
    require_literal(root, "contract_version", "1")?;
    require_text(root, "generated_at")?;
    require_non_negative_integer(root, "generation")?;
    match root.get("projection_status").and_then(Value::as_str) {
        Some("current") => {
            validate_workspace(required(root, "workspace")?)?;
            if let Some(delivery) = required(root, "delivery")?.as_object() {
                validate_delivery(delivery)?;
            } else if !required(root, "delivery")?.is_null() {
                return Err("Host status delivery must be an object or null".to_string());
            }
            if let Some(continuation) = required(root, "continuation")?.as_object() {
                validate_continuation(continuation)?;
            } else if !required(root, "continuation")?.is_null() {
                return Err("Host status continuation must be an object or null".to_string());
            }
            if let Some(work_items) = root.get("work_items")
                && !work_items.is_null()
            {
                validate_work_items(work_items)?;
            }
            if let Some(session) = root.get("session")
                && !session.is_null()
            {
                validate_session(session)?;
            }
            if !required(root, "invalidation")?.is_null() {
                return Err("Current Host status cannot contain invalidation data".to_string());
            }
            Ok(HostContractStatus::Current)
        }
        Some("invalidated") => {
            for field in ["workspace", "delivery", "continuation"] {
                if !required(root, field)?.is_null() {
                    return Err(format!("Invalidated Host status must clear {field}"));
                }
            }
            for field in ["work_items", "session"] {
                if root.get(field).is_some_and(|value| !value.is_null()) {
                    return Err(format!("Invalidated Host status must clear {field}"));
                }
            }
            validate_invalidation(required(root, "invalidation")?)?;
            Ok(HostContractStatus::Invalidated)
        }
        _ => Err("unsupported projection status".to_string()),
    }
}

fn validate_workspace(value: &Value) -> Result<(), String> {
    let object = exact_object(value, &["format_version", "status"], "workspace")?;
    require_text(object, "format_version")?;
    require_text(object, "status")?;
    Ok(())
}

fn validate_delivery(object: &Map<String, Value>) -> Result<(), String> {
    exact_keys(object, &["kind", "id", "status", "revision"], "delivery")?;
    match object.get("kind").and_then(Value::as_str) {
        Some("goal" | "plan" | "cycle") => {}
        _ => return Err("Host status delivery.kind is invalid".to_string()),
    }
    require_text(object, "id")?;
    require_text(object, "status")?;
    require_non_negative_integer(object, "revision")?;
    Ok(())
}

fn validate_work_items(value: &Value) -> Result<(), String> {
    let work_items = value
        .as_array()
        .ok_or_else(|| "Host status work_items must be an array".to_string())?;
    for work_item in work_items {
        let object = exact_object(
            work_item,
            &["kind", "id", "status", "placement_ids"],
            "work item",
        )?;
        match object.get("kind").and_then(Value::as_str) {
            Some("delivery" | "experiment") => {}
            _ => return Err("Host status work item kind is invalid".to_string()),
        }
        require_text(object, "id")?;
        match object.get("status").and_then(Value::as_str) {
            Some("active" | "expired" | "released") => {}
            _ => return Err("Host status work item status is invalid".to_string()),
        }
        let placements = required(object, "placement_ids")?
            .as_array()
            .ok_or_else(|| "Host status work item placement_ids must be an array".to_string())?;
        let mut unique = HashSet::new();
        for placement in placements {
            let placement = placement
                .as_str()
                .filter(|placement| !placement.trim().is_empty())
                .ok_or_else(|| {
                    "Host status work item placement_ids must contain non-empty text".to_string()
                })?;
            if !unique.insert(placement) {
                return Err(
                    "Host status work item placement_ids must contain unique values".to_string(),
                );
            }
        }
    }
    Ok(())
}

fn validate_session(value: &Value) -> Result<(), String> {
    let object = exact_object(
        value,
        &["host", "session_id", "status", "work_item_ref"],
        "session",
    )?;
    require_text(object, "host")?;
    require_text(object, "session_id")?;
    match object.get("status").and_then(Value::as_str) {
        Some("selected" | "selection_required" | "none") => {}
        _ => return Err("Host status session status is invalid".to_string()),
    }
    let work_item_ref = required(object, "work_item_ref")?;
    if !work_item_ref.is_null() {
        let reference = exact_object(work_item_ref, &["kind", "id"], "session work item ref")?;
        match reference.get("kind").and_then(Value::as_str) {
            Some("delivery" | "experiment") => {}
            _ => return Err("Host status session work item kind is invalid".to_string()),
        }
        require_text(reference, "id")?;
    }
    if object.get("status").and_then(Value::as_str) == Some("selected") && work_item_ref.is_null() {
        return Err("Selected Host status session requires a work item ref".to_string());
    }
    Ok(())
}

fn validate_continuation(object: &Map<String, Value>) -> Result<(), String> {
    exact_keys(
        object,
        &["available", "safe_resume_command"],
        "continuation",
    )?;
    if object.get("available").and_then(Value::as_bool).is_none() {
        return Err("Host status continuation.available must be boolean".to_string());
    }
    require_literal(object, "safe_resume_command", "/hw:resume")
}

fn validate_invalidation(value: &Value) -> Result<(), String> {
    let object = exact_object(value, &["invalidated_at", "reason"], "invalidation")?;
    require_text(object, "invalidated_at")?;
    require_text(object, "reason")?;
    Ok(())
}

fn reject_sensitive_keys(value: &Value) -> Result<(), String> {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let normalized = key.to_ascii_lowercase().replace('-', "_");
                if [
                    "api_key",
                    "authorization",
                    "credential",
                    "password",
                    "passwd",
                    "private_key",
                    "secret",
                    "session_token",
                    "token",
                ]
                .iter()
                .any(|needle| normalized.contains(needle))
                {
                    return Err(format!("Host status contains sensitive field {key}"));
                }
                reject_sensitive_keys(child)?;
            }
        }
        Value::Array(values) => {
            for child in values {
                reject_sensitive_keys(child)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn exact_object<'a>(
    value: &'a Value,
    keys: &[&str],
    label: &str,
) -> Result<&'a Map<String, Value>, String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("{label} must be an object"))?;
    exact_keys(object, keys, label)?;
    Ok(object)
}

fn object_with_optional_keys<'a>(
    value: &'a Value,
    required_keys: &[&str],
    optional_keys: &[&str],
    label: &str,
) -> Result<&'a Map<String, Value>, String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("{label} must be an object"))?;
    if required_keys.iter().any(|key| !object.contains_key(*key))
        || object.keys().any(|key| {
            !required_keys.contains(&key.as_str()) && !optional_keys.contains(&key.as_str())
        })
    {
        return Err(format!("{label} contains unknown or missing fields"));
    }
    Ok(object)
}

fn exact_keys(object: &Map<String, Value>, keys: &[&str], label: &str) -> Result<(), String> {
    if object.len() != keys.len() || object.keys().any(|key| !keys.contains(&key.as_str())) {
        return Err(format!("{label} contains unknown or missing fields"));
    }
    Ok(())
}

fn required<'a>(object: &'a Map<String, Value>, key: &str) -> Result<&'a Value, String> {
    object
        .get(key)
        .ok_or_else(|| format!("Host status is missing {key}"))
}

fn require_text(object: &Map<String, Value>, key: &str) -> Result<(), String> {
    match object.get(key).and_then(Value::as_str) {
        Some(value) if !value.trim().is_empty() => Ok(()),
        _ => Err(format!("Host status {key} must be non-empty text")),
    }
}

fn require_literal(object: &Map<String, Value>, key: &str, expected: &str) -> Result<(), String> {
    if object.get(key).and_then(Value::as_str) == Some(expected) {
        Ok(())
    } else {
        Err(format!("Host status {key} is unsupported"))
    }
}

fn require_non_negative_integer(object: &Map<String, Value>, key: &str) -> Result<(), String> {
    if object.get(key).and_then(Value::as_u64).is_some() {
        Ok(())
    } else {
        Err(format!("Host status {key} must be a non-negative integer"))
    }
}

fn invalid(message: impl Into<String>) -> HostContractRead {
    HostContractRead {
        status: HostContractStatus::Invalid(message.into()),
        projection: None,
    }
}
