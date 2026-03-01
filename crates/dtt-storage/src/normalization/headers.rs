use crate::{blake3_hash_hex, canonical_json_bytes, Result};
use dtt_core::{HeaderMap, HeaderValue};
use serde_json::{Map, Value};

pub(crate) const REDACTED_VALUE: &str = "[redacted]";

pub(crate) fn sanitize_and_lowercase_headers(raw: Option<&Value>) -> HeaderMap {
    let mut headers = HeaderMap::new();

    let Some(Value::Object(values)) = raw else {
        return headers;
    };

    for (key, value) in values {
        let lowered = key.to_ascii_lowercase();
        if is_sensitive_header_key(&lowered) {
            headers.insert(lowered, HeaderValue::Single(REDACTED_VALUE.to_string()));
            continue;
        }

        if let Some(parsed) = parse_header_value(value) {
            headers.insert(lowered, parsed);
        }
    }

    headers
}

pub(crate) fn merge_headers(existing: &mut HeaderMap, incoming: HeaderMap) {
    for (key, value) in incoming {
        existing.insert(key, value);
    }
}

pub(crate) fn headers_to_json(headers: &HeaderMap) -> Value {
    let mut map = Map::new();
    for (key, value) in headers {
        let value_json = match value {
            HeaderValue::Single(single) => Value::String(single.clone()),
            HeaderValue::Multi(values) => {
                Value::Array(values.iter().cloned().map(Value::String).collect::<Vec<Value>>())
            }
        };
        map.insert(key.clone(), value_json);
    }

    Value::Object(map)
}

pub(crate) fn headers_hash(headers: &HeaderMap) -> Result<String> {
    let headers_json = headers_to_json(headers);
    let canonical = canonical_json_bytes(&headers_json)?;
    Ok(blake3_hash_hex(&canonical))
}

pub(crate) fn header_first_value(headers: &HeaderMap, name: &str) -> Option<String> {
    let header = headers.get(&name.to_ascii_lowercase())?;
    match header {
        HeaderValue::Single(value) => Some(value.clone()),
        HeaderValue::Multi(values) => values.first().cloned(),
    }
}

fn is_sensitive_header_key(header: &str) -> bool {
    matches!(
        header,
        "authorization"
            | "cookie"
            | "set-cookie"
            | "proxy-authorization"
            | "x-api-key"
            | "api-key"
            | "token"
    )
}

fn parse_header_value(value: &Value) -> Option<HeaderValue> {
    match value {
        Value::String(single) => Some(HeaderValue::Single(single.clone())),
        Value::Number(number) => Some(HeaderValue::Single(number.to_string())),
        Value::Bool(boolean) => Some(HeaderValue::Single(boolean.to_string())),
        Value::Array(values) => {
            let parsed: Vec<String> = values.iter().filter_map(value_as_header_string).collect();
            if parsed.is_empty() {
                None
            } else {
                Some(HeaderValue::Multi(parsed))
            }
        }
        _ => None,
    }
}

fn value_as_header_string(value: &Value) -> Option<String> {
    match value {
        Value::String(single) => Some(single.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(boolean) => Some(boolean.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{headers_hash, sanitize_and_lowercase_headers, REDACTED_VALUE};
    use dtt_core::HeaderValue;
    use serde_json::json;

    #[test]
    fn sanitize_headers_lowercases_and_redacts_sensitive_keys() {
        let headers = sanitize_and_lowercase_headers(Some(&json!({
            "Authorization": "Bearer SECRET",
            "Cookie": "session=abc",
            "X-Test": ["A", 5],
            "ETag": "xyz"
        })));

        assert_eq!(
            headers.get("authorization"),
            Some(&HeaderValue::Single(REDACTED_VALUE.to_string()))
        );
        assert_eq!(headers.get("cookie"), Some(&HeaderValue::Single(REDACTED_VALUE.to_string())));
        assert_eq!(
            headers.get("x-test"),
            Some(&HeaderValue::Multi(vec!["A".to_string(), "5".to_string()]))
        );
        assert_eq!(headers.get("etag"), Some(&HeaderValue::Single("xyz".to_string())));
    }

    #[test]
    fn headers_hash_is_stable_for_equivalent_maps() {
        let left = sanitize_and_lowercase_headers(Some(&json!({
            "B": "2",
            "A": "1"
        })));
        let right = sanitize_and_lowercase_headers(Some(&json!({
            "A": "1",
            "B": "2"
        })));

        let left_hash = headers_hash(&left).expect("hash left");
        let right_hash = headers_hash(&right).expect("hash right");

        assert_eq!(left_hash, right_hash);
    }
}
