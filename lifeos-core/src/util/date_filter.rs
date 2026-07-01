//! Shared date filter builder for Notion API queries.

/// Build a Notion API date filter for the given range.
///
/// When `date_prop` is provided, wraps the filter with `"property": date_prop`.
/// When `date_prop` is None, returns a bare `{ "date": {...} }` filter.
pub fn build_date_filter(range: &str, date_prop: Option<&str>) -> Option<serde_json::Value> {
    let now = chrono::Utc::now();
    let inner = match range {
        "today" => Some(serde_json::json!({ "equals": now.format("%Y-%m-%d").to_string() })),
        "this_week" => {
            let start = (now - chrono::Duration::days(7)).format("%Y-%m-%d").to_string();
            Some(serde_json::json!({ "on_or_after": start }))
        }
        "this_month" => {
            let start = (now - chrono::Duration::days(30)).format("%Y-%m-%d").to_string();
            Some(serde_json::json!({ "on_or_after": start }))
        }
        "this_quarter" => {
            let start = (now - chrono::Duration::days(90)).format("%Y-%m-%d").to_string();
            Some(serde_json::json!({ "on_or_after": start }))
        }
        _ => None,
    }?;
    Some(match date_prop {
        Some(prop) => serde_json::json!({ "property": prop, "date": inner }),
        None => serde_json::json!({ "date": inner }),
    })
}
