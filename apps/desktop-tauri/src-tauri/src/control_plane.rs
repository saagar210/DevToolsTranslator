use dtt_core::{
    CmdListTabsPayload, CmdSetUiCapturePayload, CmdStartCapturePayload, CmdStopCapturePayload,
    EvtErrorPayload, EvtHelloPayload, EvtSessionEndedPayload, EvtSessionStartedPayload,
    EvtTabsListPayload, JsonEnvelope, RedactionLevel, ENVELOPE_VERSION,
};
use serde_json::Value;

pub const CMD_LIST_TABS: &str = "cmd.list_tabs";
pub const CMD_START_CAPTURE: &str = "cmd.start_capture";
pub const CMD_STOP_CAPTURE: &str = "cmd.stop_capture";
pub const CMD_SET_UI_CAPTURE: &str = "cmd.set_ui_capture";

pub const EVT_HELLO: &str = "evt.hello";
pub const EVT_TABS_LIST: &str = "evt.tabs_list";
pub const EVT_SESSION_STARTED: &str = "evt.session_started";
pub const EVT_RAW_EVENT: &str = "evt.raw_event";
pub const EVT_SESSION_ENDED: &str = "evt.session_ended";
pub const EVT_ERROR: &str = "evt.error";

pub fn build_list_tabs_command(ts_ms: i64, request_id: String, token: String) -> JsonEnvelope {
    JsonEnvelope {
        v: ENVELOPE_VERSION,
        envelope_type: CMD_LIST_TABS.to_string(),
        ts_ms,
        token: Some(token),
        request_id: Some(request_id),
        correlation_id: None,
        session_id: None,
        event_seq: None,
        privacy_mode: None,
        payload: serde_json::to_value(CmdListTabsPayload {}).expect("serialize list tabs payload"),
    }
}

pub fn build_start_capture_command(
    ts_ms: i64,
    request_id: String,
    token: String,
    tab_id: i64,
    privacy_mode: RedactionLevel,
    session_id: String,
) -> JsonEnvelope {
    JsonEnvelope {
        v: ENVELOPE_VERSION,
        envelope_type: CMD_START_CAPTURE.to_string(),
        ts_ms,
        token: Some(token),
        request_id: Some(request_id),
        correlation_id: None,
        session_id: Some(session_id.clone()),
        event_seq: None,
        privacy_mode: Some(privacy_mode),
        payload: serde_json::to_value(CmdStartCapturePayload {
            tab_id,
            privacy_mode,
            session_id,
            enable_security_domain: false,
        })
        .expect("serialize start capture payload"),
    }
}

pub fn build_stop_capture_command(
    ts_ms: i64,
    request_id: String,
    token: String,
    session_id: String,
) -> JsonEnvelope {
    JsonEnvelope {
        v: ENVELOPE_VERSION,
        envelope_type: CMD_STOP_CAPTURE.to_string(),
        ts_ms,
        token: Some(token),
        request_id: Some(request_id),
        correlation_id: None,
        session_id: Some(session_id.clone()),
        event_seq: None,
        privacy_mode: None,
        payload: serde_json::to_value(CmdStopCapturePayload { session_id })
            .expect("serialize stop capture payload"),
    }
}

pub fn build_set_ui_capture_command(
    ts_ms: i64,
    request_id: String,
    token: String,
    enabled: bool,
) -> JsonEnvelope {
    JsonEnvelope {
        v: ENVELOPE_VERSION,
        envelope_type: CMD_SET_UI_CAPTURE.to_string(),
        ts_ms,
        token: Some(token),
        request_id: Some(request_id),
        correlation_id: None,
        session_id: None,
        event_seq: None,
        privacy_mode: None,
        payload: serde_json::to_value(CmdSetUiCapturePayload { enabled })
            .expect("serialize set ui capture payload"),
    }
}

pub fn parse_tabs_payload(
    envelope: &JsonEnvelope,
) -> Result<EvtTabsListPayload, serde_json::Error> {
    serde_json::from_value(envelope.payload.clone())
}

pub fn parse_session_started_payload(
    envelope: &JsonEnvelope,
) -> Result<EvtSessionStartedPayload, serde_json::Error> {
    serde_json::from_value(envelope.payload.clone())
}

pub fn parse_session_ended_payload(
    envelope: &JsonEnvelope,
) -> Result<EvtSessionEndedPayload, serde_json::Error> {
    serde_json::from_value(envelope.payload.clone())
}

pub fn parse_hello_payload(envelope: &JsonEnvelope) -> Result<EvtHelloPayload, serde_json::Error> {
    serde_json::from_value(envelope.payload.clone())
}

pub fn parse_error_payload(envelope: &JsonEnvelope) -> Result<EvtErrorPayload, serde_json::Error> {
    serde_json::from_value(envelope.payload.clone())
}

pub fn event_matches_pending(envelope: &JsonEnvelope, expected_type: &str) -> bool {
    envelope.envelope_type == EVT_ERROR || envelope.envelope_type == expected_type
}

pub fn correlation_id_of(envelope: &JsonEnvelope) -> Option<&str> {
    envelope.correlation_id.as_deref()
}

pub fn to_json_text(envelope: &JsonEnvelope) -> Result<String, serde_json::Error> {
    serde_json::to_string(envelope)
}

pub fn from_json_text(text: &str) -> Result<JsonEnvelope, serde_json::Error> {
    serde_json::from_str(text)
}

pub fn payload_get_string(payload: &Value, key: &str) -> Option<String> {
    payload.get(key).and_then(Value::as_str).map(ToOwned::to_owned)
}
