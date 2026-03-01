use crate::normalization::headers::header_first_value;
use dtt_core::{
    HeaderMap, ReconstructionStatus, StreamReconstructionV1, StreamSummaryV1, StreamTransport,
};

#[derive(Debug, Clone)]
pub(crate) struct StreamAccumulator {
    transport: StreamTransport,
    content_type: Option<String>,
    chunk_count: u32,
    bytes_total: u64,
    first_byte_ms: Option<i64>,
    last_byte_ms: Option<i64>,
    parse_errors: u32,
    dropped_chunks: u32,
}

impl Default for StreamAccumulator {
    fn default() -> Self {
        Self {
            transport: StreamTransport::Unknown,
            content_type: None,
            chunk_count: 0,
            bytes_total: 0,
            first_byte_ms: None,
            last_byte_ms: None,
            parse_errors: 0,
            dropped_chunks: 0,
        }
    }
}

impl StreamAccumulator {
    pub(crate) fn observe_method(&mut self, cdp_method: &str) {
        if cdp_method.starts_with("Network.webSocket") {
            self.transport = StreamTransport::Websocket;
        }
    }

    pub(crate) fn observe_response_headers(&mut self, headers: &HeaderMap) {
        if let Some(content_type) = header_first_value(headers, "content-type") {
            self.content_type = Some(content_type.clone());
            if self.transport != StreamTransport::Websocket
                && content_type.to_ascii_lowercase().contains("text/event-stream")
            {
                self.transport = StreamTransport::Sse;
            }
        }

        if self.transport == StreamTransport::Unknown
            && header_first_value(headers, "transfer-encoding")
                .map(|value| value.to_ascii_lowercase().contains("chunked"))
                .unwrap_or(false)
        {
            self.transport = StreamTransport::ChunkedFetch;
        }
    }

    pub(crate) fn observe_data(&mut self, ts_ms: i64, bytes: u64) {
        self.chunk_count = self.chunk_count.saturating_add(1);
        self.bytes_total = self.bytes_total.saturating_add(bytes);

        if self.first_byte_ms.is_none() {
            self.first_byte_ms = Some(ts_ms);
        }
        self.last_byte_ms = Some(ts_ms);
    }

    pub(crate) fn observe_encoded_total(&mut self, encoded_total: Option<u64>) {
        if let Some(total) = encoded_total {
            self.bytes_total = self.bytes_total.max(total);
        }
    }

    pub(crate) fn snapshot(&self, completion_success: Option<bool>) -> StreamSummaryV1 {
        let status = match completion_success {
            Some(true) => {
                if self.parse_errors > 0 || self.dropped_chunks > 0 {
                    ReconstructionStatus::Partial
                } else {
                    ReconstructionStatus::Ok
                }
            }
            Some(false) => ReconstructionStatus::Failed,
            None => ReconstructionStatus::Partial,
        };

        let stream_duration_ms = match (self.first_byte_ms, self.last_byte_ms) {
            (Some(first), Some(last)) if last >= first => Some(last - first),
            _ => None,
        };

        StreamSummaryV1 {
            is_streaming: self.transport != StreamTransport::Unknown || self.chunk_count > 1,
            transport: self.transport,
            content_type: self.content_type.clone(),
            chunk_count: self.chunk_count,
            bytes_total: self.bytes_total,
            first_byte_ms: self.first_byte_ms,
            last_byte_ms: self.last_byte_ms,
            stream_duration_ms,
            reconstruction: StreamReconstructionV1 {
                status,
                parse_errors: self.parse_errors,
                dropped_chunks: self.dropped_chunks,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StreamAccumulator;
    use crate::normalization::headers::sanitize_and_lowercase_headers;
    use dtt_core::StreamTransport;
    use serde_json::json;

    #[test]
    fn stream_classifier_detects_sse_and_duration() {
        let headers = sanitize_and_lowercase_headers(Some(&json!({
            "Content-Type": "text/event-stream"
        })));

        let mut stream = StreamAccumulator::default();
        stream.observe_response_headers(&headers);
        stream.observe_data(1000, 10);
        stream.observe_data(1400, 20);

        let summary = stream.snapshot(Some(true));
        assert_eq!(summary.transport, StreamTransport::Sse);
        assert_eq!(summary.chunk_count, 2);
        assert_eq!(summary.bytes_total, 30);
        assert_eq!(summary.stream_duration_ms, Some(400));
        assert!(summary.is_streaming);
    }

    #[test]
    fn stream_classifier_detects_websocket_and_failure() {
        let mut stream = StreamAccumulator::default();
        stream.observe_method("Network.webSocketFrameReceived");
        stream.observe_data(2000, 5);

        let summary = stream.snapshot(Some(false));
        assert_eq!(summary.transport, StreamTransport::Websocket);
        assert_eq!(summary.reconstruction.status, dtt_core::ReconstructionStatus::Failed);
    }

    #[test]
    fn stream_classifier_detects_chunked_fetch() {
        let headers = sanitize_and_lowercase_headers(Some(&json!({
            "Transfer-Encoding": "chunked"
        })));

        let mut stream = StreamAccumulator::default();
        stream.observe_response_headers(&headers);
        let summary = stream.snapshot(None);
        assert_eq!(summary.transport, StreamTransport::ChunkedFetch);
    }
}
