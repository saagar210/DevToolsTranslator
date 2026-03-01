use serde_json::Value;

#[derive(Debug, Clone)]
pub(crate) struct LifecycleRecord {
    pub lifecycle_id: String,
    pub name: String,
    pub frame_id: Option<String>,
    pub loader_id: Option<String>,
    pub value_json: String,
}

pub(crate) fn normalize_page_lifecycle(
    event_id: &str,
    cdp_method: &str,
    payload: &Value,
) -> Option<LifecycleRecord> {
    let params = payload.get("params").cloned().unwrap_or(Value::Null);

    let (name, frame_id, loader_id) = match cdp_method {
        "Page.lifecycleEvent" => {
            let name =
                params.get("name").and_then(Value::as_str).unwrap_or("lifecycleEvent").to_string();
            let frame_id = params.get("frameId").and_then(Value::as_str).map(ToOwned::to_owned);
            let loader_id = params.get("loaderId").and_then(Value::as_str).map(ToOwned::to_owned);
            (name, frame_id, loader_id)
        }
        "Page.loadEventFired" => ("loadEventFired".to_string(), None, None),
        "Page.domContentEventFired" => ("domContentEventFired".to_string(), None, None),
        _ => return None,
    };

    let value_json = serde_json::to_string(&params).ok()?;

    Some(LifecycleRecord {
        lifecycle_id: event_id.to_string(),
        name,
        frame_id,
        loader_id,
        value_json,
    })
}

#[cfg(test)]
mod tests {
    use super::normalize_page_lifecycle;
    use serde_json::json;

    #[test]
    fn lifecycle_event_maps_expected_name_and_ids() {
        let payload = json!({
            "params": {
                "name": "networkIdle",
                "frameId": "frame_1",
                "loaderId": "loader_1"
            }
        });

        let row = normalize_page_lifecycle("evt_life_1", "Page.lifecycleEvent", &payload)
            .expect("lifecycle row");
        assert_eq!(row.name, "networkIdle");
        assert_eq!(row.frame_id.as_deref(), Some("frame_1"));
        assert_eq!(row.loader_id.as_deref(), Some("loader_1"));
    }
}
