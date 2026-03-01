use crate::blake3_hash_hex;
use serde_json::Value;

#[derive(Debug, Clone)]
pub(crate) struct ConsoleEntryRecord {
    pub console_id: String,
    pub level: Option<String>,
    pub source: Option<String>,
    pub message_redacted: String,
    pub message_hash: String,
    pub message_len: i64,
}

pub(crate) fn normalize_runtime_console(
    event_id: &str,
    payload: &Value,
) -> Option<ConsoleEntryRecord> {
    let params = payload.get("params")?.as_object()?;
    let level = params.get("type").and_then(Value::as_str).map(ToOwned::to_owned);

    let message = params
        .get("args")
        .and_then(Value::as_array)
        .map(|args| {
            args.iter()
                .filter_map(|arg| {
                    arg.get("value").or_else(|| arg.get("description")).and_then(Value::as_str)
                })
                .collect::<Vec<&str>>()
                .join(" ")
        })
        .unwrap_or_default();

    Some(build_console_entry(event_id, level, Some("runtime.console".to_string()), message))
}

pub(crate) fn normalize_log_entry(event_id: &str, payload: &Value) -> Option<ConsoleEntryRecord> {
    let entry = payload.get("params")?.get("entry")?.as_object()?;
    let level = entry.get("level").and_then(Value::as_str).map(ToOwned::to_owned);
    let source = entry.get("source").and_then(Value::as_str).map(ToOwned::to_owned);
    let message = entry.get("text").and_then(Value::as_str).unwrap_or_default().to_string();

    Some(build_console_entry(event_id, level, source, message))
}

fn build_console_entry(
    event_id: &str,
    level: Option<String>,
    source: Option<String>,
    original_message: String,
) -> ConsoleEntryRecord {
    let redacted = redact_console_message(&original_message);

    ConsoleEntryRecord {
        console_id: event_id.to_string(),
        level,
        source,
        message_hash: blake3_hash_hex(original_message.as_bytes()),
        message_len: i64::try_from(original_message.len()).unwrap_or(0),
        message_redacted: redacted,
    }
}

fn redact_console_message(input: &str) -> String {
    let lowered = input.to_ascii_lowercase();
    if lowered.contains("authorization")
        || lowered.contains("cookie")
        || lowered.contains("x-api-key")
        || lowered.contains("api-key")
        || lowered.contains("token")
    {
        "[redacted console message]".to_string()
    } else {
        input.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_runtime_console;
    use serde_json::json;

    #[test]
    fn console_normalization_redacts_sensitive_message() {
        let payload = json!({
            "params": {
                "type": "error",
                "args": [
                    {"value": "Authorization: Bearer secret-token"}
                ]
            }
        });

        let normalized =
            normalize_runtime_console("evt_console_1", &payload).expect("console entry");
        assert_eq!(normalized.message_redacted, "[redacted console message]");
        assert!(normalized.message_len > 0);
        assert!(!normalized.message_hash.is_empty());
    }
}
