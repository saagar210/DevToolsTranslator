use crate::normalization::headers::sanitize_and_lowercase_headers;
use dtt_core::{RedactionLevel, StreamSummaryV1};
use serde_json::{json, Value};
use url::Url;

#[derive(Debug, Clone)]
pub(crate) struct RequestRecord {
    pub net_request_id: String,
    pub event_seq: i64,
    pub ts_ms: i64,
    pub started_at_ms: i64,
    pub method: Option<String>,
    pub scheme: Option<String>,
    pub host: Option<String>,
    pub port: Option<i64>,
    pub path: Option<String>,
    pub query: Option<String>,
    pub request_headers: dtt_core::HeaderMap,
    pub timing_json: Value,
    pub redaction_level: RedactionLevel,
}

#[derive(Debug, Clone)]
pub(crate) struct ResponseRecord {
    pub net_request_id: String,
    pub ts_ms: i64,
    pub status_code: Option<i64>,
    pub protocol: Option<String>,
    pub mime_type: Option<String>,
    pub encoded_data_length: Option<i64>,
    pub response_headers: dtt_core::HeaderMap,
    pub headers_hash: String,
    pub stream_summary_json: Option<StreamSummaryV1>,
    pub redaction_level: RedactionLevel,
}

#[derive(Debug, Clone)]
pub(crate) struct CompletionRecord {
    pub net_request_id: String,
    pub ts_ms: i64,
    pub finished_at_ms: i64,
    pub duration_ms: i64,
    pub success: bool,
    pub error_text: Option<String>,
    pub canceled: bool,
    pub blocked_reason: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct UrlParts {
    scheme: Option<String>,
    host: Option<String>,
    port: Option<i64>,
    path: Option<String>,
    query: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) enum NetworkMutation {
    RequestWillBeSent(RequestRecord),
    RequestWillBeSentExtraInfo { request_id: String, headers: dtt_core::HeaderMap },
    ResponseReceived { response: ResponseRecord, response_timing_json: Option<Value> },
    ResponseReceivedExtraInfo { request_id: String, headers: dtt_core::HeaderMap },
    DataReceived { request_id: String, ts_ms: i64, bytes: u64 },
    LoadingFinished { request_id: String, ts_ms: i64, encoded_data_length: Option<u64> },
    LoadingFailed { completion: CompletionRecord },
    WebSocketActivity { request_id: String },
}

pub(crate) fn parse_network_mutation(
    cdp_method: &str,
    event_seq: i64,
    ts_ms: i64,
    raw_event: &Value,
    redaction_level: RedactionLevel,
) -> Option<NetworkMutation> {
    let params = raw_event.get("params")?;

    match cdp_method {
        "Network.requestWillBeSent" => {
            let request_id = params.get("requestId")?.as_str()?.to_string();
            let request = params.get("request").cloned().unwrap_or(Value::Null);
            let headers = sanitize_and_lowercase_headers(request.get("headers"));

            let url_parts =
                request.get("url").and_then(Value::as_str).map(parse_url_parts).unwrap_or_default();

            let request_time_s =
                params.get("timestamp").and_then(Value::as_f64).unwrap_or_default();

            Some(NetworkMutation::RequestWillBeSent(RequestRecord {
                net_request_id: request_id,
                event_seq,
                ts_ms,
                started_at_ms: ts_ms,
                method: request.get("method").and_then(Value::as_str).map(ToOwned::to_owned),
                scheme: url_parts.scheme,
                host: url_parts.host,
                port: url_parts.port,
                path: url_parts.path,
                query: url_parts.query,
                request_headers: headers,
                timing_json: default_timing_json(request_time_s),
                redaction_level,
            }))
        }
        "Network.requestWillBeSentExtraInfo" => {
            let request_id = params.get("requestId")?.as_str()?.to_string();
            let headers = sanitize_and_lowercase_headers(params.get("headers"));
            Some(NetworkMutation::RequestWillBeSentExtraInfo { request_id, headers })
        }
        "Network.responseReceived" => {
            let request_id = params.get("requestId")?.as_str()?.to_string();
            let response = params.get("response").cloned().unwrap_or(Value::Null);
            let headers = sanitize_and_lowercase_headers(response.get("headers"));

            Some(NetworkMutation::ResponseReceived {
                response: ResponseRecord {
                    net_request_id: request_id,
                    ts_ms,
                    status_code: response.get("status").and_then(Value::as_f64).map(|v| v as i64),
                    protocol: response
                        .get("protocol")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    mime_type: response
                        .get("mimeType")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    encoded_data_length: response
                        .get("encodedDataLength")
                        .and_then(Value::as_f64)
                        .map(|v| v as i64),
                    response_headers: headers,
                    headers_hash: String::new(),
                    stream_summary_json: None,
                    redaction_level,
                },
                response_timing_json: response.get("timing").map(response_timing_json),
            })
        }
        "Network.responseReceivedExtraInfo" => {
            let request_id = params.get("requestId")?.as_str()?.to_string();
            let headers = sanitize_and_lowercase_headers(params.get("headers"));
            Some(NetworkMutation::ResponseReceivedExtraInfo { request_id, headers })
        }
        "Network.dataReceived" => {
            let request_id = params.get("requestId")?.as_str()?.to_string();
            let bytes = params
                .get("dataLength")
                .or_else(|| params.get("encodedDataLength"))
                .and_then(Value::as_f64)
                .map(|v| v.max(0.0) as u64)
                .unwrap_or(0);

            Some(NetworkMutation::DataReceived { request_id, ts_ms, bytes })
        }
        "Network.loadingFinished" => {
            let request_id = params.get("requestId")?.as_str()?.to_string();
            let encoded_data_length =
                params.get("encodedDataLength").and_then(Value::as_f64).map(|v| v.max(0.0) as u64);

            Some(NetworkMutation::LoadingFinished { request_id, ts_ms, encoded_data_length })
        }
        "Network.loadingFailed" => {
            let request_id = params.get("requestId")?.as_str()?.to_string();

            Some(NetworkMutation::LoadingFailed {
                completion: CompletionRecord {
                    net_request_id: request_id,
                    ts_ms,
                    finished_at_ms: ts_ms,
                    duration_ms: 0,
                    success: false,
                    error_text: params
                        .get("errorText")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                    canceled: params.get("canceled").and_then(Value::as_bool).unwrap_or(false),
                    blocked_reason: params
                        .get("blockedReason")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                },
            })
        }
        method if method.starts_with("Network.webSocket") => {
            let request_id = params
                .get("requestId")
                .or_else(|| params.get("identifier"))
                .and_then(Value::as_str)?
                .to_string();

            Some(NetworkMutation::WebSocketActivity { request_id })
        }
        _ => None,
    }
}

pub(crate) fn response_timing_json(timing: &Value) -> Value {
    json!({
      "request_time_s": timing.get("requestTime").and_then(Value::as_f64).unwrap_or_default(),
      "dns_start_ms": number_or_null(timing.get("dnsStart")),
      "dns_end_ms": number_or_null(timing.get("dnsEnd")),
      "connect_start_ms": number_or_null(timing.get("connectStart")),
      "connect_end_ms": number_or_null(timing.get("connectEnd")),
      "ssl_start_ms": number_or_null(timing.get("sslStart")),
      "ssl_end_ms": number_or_null(timing.get("sslEnd")),
      "send_start_ms": number_or_null(timing.get("sendStart")),
      "send_end_ms": number_or_null(timing.get("sendEnd")),
      "receive_headers_end_ms": number_or_null(timing.get("receiveHeadersEnd")),
      "worker_start_ms": number_or_null(timing.get("workerStart")),
      "worker_ready_ms": number_or_null(timing.get("workerReady")),
      "worker_fetch_start_ms": number_or_null(timing.get("workerFetchStart")),
      "worker_respond_with_settled_ms": number_or_null(timing.get("workerRespondWithSettled"))
    })
}

pub(crate) fn default_timing_json(request_time_s: f64) -> Value {
    json!({
      "request_time_s": request_time_s,
      "dns_start_ms": null,
      "dns_end_ms": null,
      "connect_start_ms": null,
      "connect_end_ms": null,
      "ssl_start_ms": null,
      "ssl_end_ms": null,
      "send_start_ms": null,
      "send_end_ms": null,
      "receive_headers_end_ms": null,
      "worker_start_ms": null,
      "worker_ready_ms": null,
      "worker_fetch_start_ms": null,
      "worker_respond_with_settled_ms": null
    })
}

fn parse_url_parts(url: &str) -> UrlParts {
    if let Ok(parsed) = Url::parse(url) {
        return UrlParts {
            scheme: Some(parsed.scheme().to_string()),
            host: parsed.host_str().map(ToOwned::to_owned),
            port: parsed.port().map(i64::from),
            path: Some(parsed.path().to_string()),
            query: parsed.query().map(ToOwned::to_owned),
        };
    }

    UrlParts { path: Some(url.to_string()), ..UrlParts::default() }
}

fn number_or_null(value: Option<&Value>) -> Value {
    value.and_then(Value::as_f64).map(Value::from).unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::parse_network_mutation;
    use dtt_core::RedactionLevel;
    use serde_json::json;

    #[test]
    fn request_parsing_extracts_url_parts_and_headers() {
        let payload = json!({
            "params": {
                "requestId": "req_1",
                "timestamp": 12.5,
                "request": {
                    "url": "https://example.com:8443/path?q=1",
                    "method": "GET",
                    "headers": {
                        "Authorization": "secret",
                        "Accept": "application/json"
                    }
                }
            }
        });

        let mutation = parse_network_mutation(
            "Network.requestWillBeSent",
            1,
            1000,
            &payload,
            RedactionLevel::MetadataOnly,
        )
        .expect("request mutation");

        match mutation {
            super::NetworkMutation::RequestWillBeSent(row) => {
                assert_eq!(row.scheme.as_deref(), Some("https"));
                assert_eq!(row.host.as_deref(), Some("example.com"));
                assert_eq!(row.port, Some(8443));
                assert_eq!(row.path.as_deref(), Some("/path"));
                assert_eq!(row.query.as_deref(), Some("q=1"));
                assert!(row.request_headers.contains_key("authorization"));
            }
            _ => panic!("unexpected mutation kind"),
        }
    }
}
