#![allow(dead_code)]

use crate::notion::types::*;
use std::collections::HashMap;

fn rich_text_to_plain(rich_text: &[RichTextItem]) -> String {
    rich_text.iter().filter_map(|rt| rt.plain_text.as_deref()).collect()
}

fn date_to_yaml(date: Option<&DateValue>) -> serde_yaml::Value {
    let d = match date { Some(d) => d, None => return serde_yaml::Value::Null };
    let mut map = serde_yaml::Mapping::new();
    map.insert(serde_yaml::Value::String("start".to_string()), serde_yaml::Value::String(d.start.clone()));
    if let Some(end) = &d.end {
        map.insert(serde_yaml::Value::String("end".to_string()), serde_yaml::Value::String(end.clone()));
    }
    if let Some(tz) = &d.time_zone {
        map.insert(serde_yaml::Value::String("time_zone".to_string()), serde_yaml::Value::String(tz.clone()));
    }
    serde_yaml::Value::Mapping(map)
}

fn number_to_yaml(n: Option<f64>) -> serde_yaml::Value {
    let n = match n { Some(n) => n, None => return serde_yaml::Value::Null };
    if n.fract() == 0.0 && n.is_finite() && n >= i64::MIN as f64 && n <= i64::MAX as f64 {
        serde_yaml::Value::Number(serde_yaml::Number::from(n as i64))
    } else if n.is_finite() {
        match serde_yaml::to_value(&n) { Ok(v) => v, Err(_) => serde_yaml::Value::Null }
    } else {
        serde_yaml::Value::Null
    }
}

fn formula_to_yaml(formula: &FormulaValue) -> serde_yaml::Value {
    match formula.formula_type.as_str() {
        "string" => formula.string.as_ref().map_or(serde_yaml::Value::Null, |s| serde_yaml::Value::String(s.clone())),
        "number" => number_to_yaml(formula.number),
        "boolean" => formula.boolean.map_or(serde_yaml::Value::Null, serde_yaml::Value::Bool),
        "date" => date_to_yaml(formula.date.as_ref()),
        _ => serde_yaml::Value::Null,
    }
}

fn rollup_to_yaml(rollup: &RollupValue) -> serde_yaml::Value {
    match rollup.rollup_type.as_str() {
        "number" => number_to_yaml(rollup.number),
        "string" => rollup.string.as_ref().map_or(serde_yaml::Value::Null, |s| serde_yaml::Value::String(s.clone())),
        "date" => date_to_yaml(rollup.date.as_ref()),
        "array" => {
            let arr = match rollup.array.as_ref() { Some(a) => a, None => return serde_yaml::Value::Null };
            let items: Vec<serde_yaml::Value> = arr.iter().map(property_to_yaml).collect();
            serde_yaml::Value::Sequence(items)
        }
        _ => serde_yaml::Value::Null,
    }
}

fn files_to_yaml(files: &[FileItem]) -> serde_yaml::Value {
    let items: Vec<serde_yaml::Value> = files.iter().map(|f| {
        let mut map = serde_yaml::Mapping::new();
        map.insert(serde_yaml::Value::String("name".to_string()), serde_yaml::Value::String(f.name.as_deref().unwrap_or("").to_string()));
        let url = f.external.as_ref().map(|e| e.url.clone()).or_else(|| f.file.as_ref().map(|fu| fu.url.clone())).unwrap_or_default();
        map.insert(serde_yaml::Value::String("url".to_string()), serde_yaml::Value::String(url));
        serde_yaml::Value::Mapping(map)
    }).collect();
    serde_yaml::Value::Sequence(items)
}

fn user_to_yaml(user: &UserItem) -> serde_yaml::Value {
    let mut map = serde_yaml::Mapping::new();
    map.insert(serde_yaml::Value::String("id".to_string()), serde_yaml::Value::String(user.id.clone()));
    map.insert(serde_yaml::Value::String("name".to_string()), serde_yaml::Value::String(user.name.as_deref().unwrap_or("").to_string()));
    serde_yaml::Value::Mapping(map)
}

fn unique_id_to_yaml(uid: Option<&UniqueIdValue>) -> serde_yaml::Value {
    let uid = match uid { Some(u) => u, None => return serde_yaml::Value::Null };
    let prefix = uid.prefix.as_deref().unwrap_or("");
    if prefix.is_empty() { serde_yaml::Value::Number(serde_yaml::Number::from(uid.number)) }
    else { serde_yaml::Value::String(format!("{}-{}", prefix, uid.number)) }
}

fn property_to_yaml(prop: &PropertyValue) -> serde_yaml::Value {
    match prop {
        PropertyValue::Title { title, .. } => serde_yaml::Value::String(rich_text_to_plain(title)),
        PropertyValue::RichText { rich_text, .. } => serde_yaml::Value::String(rich_text_to_plain(rich_text)),
        PropertyValue::Select { select, .. } => select.as_ref().map_or(serde_yaml::Value::Null, |opt| serde_yaml::Value::String(opt.name.clone())),
        PropertyValue::Status { status, .. } => status.as_ref().map_or(serde_yaml::Value::Null, |opt| serde_yaml::Value::String(opt.name.clone())),
        PropertyValue::MultiSelect { multi_select, .. } => {
            let names: Vec<serde_yaml::Value> = multi_select.iter().map(|opt| serde_yaml::Value::String(opt.name.clone())).collect();
            serde_yaml::Value::Sequence(names)
        }
        PropertyValue::Date { date, .. } => date_to_yaml(date.as_ref()),
        PropertyValue::Number { number, .. } => number_to_yaml(*number),
        PropertyValue::Checkbox { checkbox, .. } => serde_yaml::Value::Bool(*checkbox),
        PropertyValue::Formula { formula, .. } => formula_to_yaml(formula),
        PropertyValue::Relation { relation, .. } => {
            let ids: Vec<serde_yaml::Value> = relation.iter().map(|r| serde_yaml::Value::String(r.id.clone())).collect();
            serde_yaml::Value::Sequence(ids)
        }
        PropertyValue::Url { url, .. } => url.as_ref().map_or(serde_yaml::Value::Null, |u| serde_yaml::Value::String(u.clone())),
        PropertyValue::Email { email, .. } => email.as_ref().map_or(serde_yaml::Value::Null, |e| serde_yaml::Value::String(e.clone())),
        PropertyValue::PhoneNumber { phone_number, .. } => phone_number.as_ref().map_or(serde_yaml::Value::Null, |p| serde_yaml::Value::String(p.clone())),
        PropertyValue::Files { files, .. } => files_to_yaml(files),
        PropertyValue::CreatedTime { created_time, .. } => serde_yaml::Value::String(created_time.clone()),
        PropertyValue::CreatedBy { created_by, .. } => user_to_yaml(created_by),
        PropertyValue::LastEditedTime { last_edited_time, .. } => serde_yaml::Value::String(last_edited_time.clone()),
        PropertyValue::LastEditedBy { last_edited_by, .. } => user_to_yaml(last_edited_by),
        PropertyValue::UniqueId { unique_id, .. } => unique_id_to_yaml(unique_id.as_ref()),
        PropertyValue::Rollup { rollup, .. } => rollup_to_yaml(rollup),
        PropertyValue::People { people, .. } => {
            let items: Vec<serde_yaml::Value> = people.iter().map(user_to_yaml).collect();
            serde_yaml::Value::Sequence(items)
        }
        PropertyValue::Button { .. } => serde_yaml::Value::Null,
    }
}

pub fn extract_properties_yaml(page: &NotionPage, property_mapping: &HashMap<String, String>, title_cache: &HashMap<String, String>) -> Result<serde_yaml::Value, String> {
    let mut yaml_map = serde_yaml::Mapping::new();
    for (db_key, notion_name) in property_mapping {
        if db_key.ends_with("_json") { continue; }
        let prop = match page.properties.get(notion_name) { Some(p) => p, None => continue };
        yaml_map.insert(serde_yaml::Value::String(db_key.clone()), property_to_yaml_with_cache(prop, title_cache));
    }
    Ok(serde_yaml::Value::Mapping(yaml_map))
}

fn property_to_yaml_with_cache(prop: &PropertyValue, title_cache: &HashMap<String, String>) -> serde_yaml::Value {
    match prop {
        PropertyValue::Relation { relation, .. } => {
            let titles: Vec<serde_yaml::Value> = relation.iter().map(|r| {
                let title = title_cache.get(&r.id);
                serde_yaml::Value::String(format!("[[{}]]", title.as_deref().unwrap_or(&r.id)))
            }).collect();
            serde_yaml::Value::Sequence(titles)
        }
        _ => property_to_yaml(prop),
    }
}

fn yaml_value_to_notion_json(value: &serde_yaml::Value) -> serde_json::Value {
    match value {
        serde_yaml::Value::String(s) => serde_json::json!({ "rich_text": [{ "type": "text", "text": { "content": s } }] }),
        serde_yaml::Value::Number(n) => serde_json::json!({ "number": n.as_f64().unwrap_or(0.0) }),
        serde_yaml::Value::Bool(b) => serde_json::json!({ "checkbox": b }),
        serde_yaml::Value::Null => serde_json::Value::Null,
        serde_yaml::Value::Sequence(seq) => yaml_seq_to_notion_json(seq),
        serde_yaml::Value::Mapping(m) => yaml_map_to_notion_json(m),
        serde_yaml::Value::Tagged(t) => yaml_value_to_notion_json(&t.value),
    }
}

fn yaml_seq_to_notion_json(seq: &[serde_yaml::Value]) -> serde_json::Value {
    if seq.is_empty() { return serde_json::json!({ "multi_select": [] }); }
    match &seq[0] {
        serde_yaml::Value::String(_) => {
            let names: Vec<serde_json::Value> = seq.iter().map(|v| serde_json::json!({ "name": v.as_str().unwrap_or("") })).collect();
            serde_json::json!({ "multi_select": names })
        }
        serde_yaml::Value::Mapping(m) => {
            let id_key = serde_yaml::Value::String("id".to_string());
            if m.contains_key(&id_key) {
                let people: Vec<serde_json::Value> = seq.iter().filter_map(|v| {
                    let map = v.as_mapping()?;
                    let id = map.get(&id_key).and_then(|v| v.as_str()).unwrap_or("");
                    let name = map.get(&serde_yaml::Value::String("name".to_string())).and_then(|v| v.as_str()).unwrap_or("");
                    Some(serde_json::json!({ "object": "user", "id": id, "name": name }))
                }).collect();
                serde_json::json!({ "people": people })
            } else {
                serde_json::Value::Null
            }
        }
        _ => serde_json::Value::Null,
    }
}

fn yaml_map_to_notion_json(m: &serde_yaml::Mapping) -> serde_json::Value {
    let start_key = serde_yaml::Value::String("start".to_string());
    let id_key = serde_yaml::Value::String("id".to_string());
    if m.contains_key(&start_key) {
        let mut date_obj = serde_json::Map::new();
        if let Some(start) = m.get(&start_key).and_then(|v| v.as_str()) { date_obj.insert("start".to_string(), serde_json::Value::String(start.to_string())); }
        if let Some(end) = m.get(&serde_yaml::Value::String("end".to_string())).and_then(|v| v.as_str()) { date_obj.insert("end".to_string(), serde_json::Value::String(end.to_string())); }
        if let Some(tz) = m.get(&serde_yaml::Value::String("time_zone".to_string())).and_then(|v| v.as_str()) { date_obj.insert("time_zone".to_string(), serde_json::Value::String(tz.to_string())); }
        serde_json::json!({ "date": date_obj })
    } else if m.contains_key(&id_key) {
        let id = m.get(&id_key).and_then(|v| v.as_str()).unwrap_or("");
        let name = m.get(&serde_yaml::Value::String("name".to_string())).and_then(|v| v.as_str()).unwrap_or("");
        serde_json::json!({ "people": [{ "object": "user", "id": id, "name": name }] })
    } else {
        serde_json::Value::Null
    }
}

pub fn yaml_to_properties(yaml: &serde_yaml::Value, property_mapping: &HashMap<String, String>) -> HashMap<String, serde_json::Value> {
    let mut result = HashMap::new();
    let yaml_map = match yaml.as_mapping() { Some(m) => m, None => return result };
    for (yaml_key, yaml_value) in yaml_map {
        let db_key = match yaml_key.as_str() { Some(k) => k, None => continue };
        let notion_name = match property_mapping.get(db_key) { Some(n) => n, None => continue };
        // Title must use "title" type, not "rich_text"
        let json_value = if db_key == "title" {
            let text = match yaml_value {
                serde_yaml::Value::String(s) => s.as_str(),
                _ => continue,
            };
            serde_json::json!({ "title": [{ "type": "text", "text": { "content": text } }] })
        } else {
            yaml_value_to_notion_json(yaml_value)
        };
        if json_value.is_null() { continue; }
        result.insert(notion_name.clone(), json_value);
    }
    result
}

pub fn extract_title(page: &NotionPage) -> String {
    for prop in page.properties.values() {
        if let PropertyValue::Title { title, .. } = prop {
            return rich_text_to_plain(title);
        }
    }
    String::new()
}

pub fn extract_relation_ids(page: &NotionPage, prop_name: &str) -> Vec<String> {
    match page.properties.get(prop_name) {
        Some(PropertyValue::Relation { relation, .. }) => relation.iter().map(|r| r.id.clone()).collect(),
        _ => Vec::new(),
    }
}

pub fn extract_relation_count(page: &NotionPage, prop_name: &str) -> usize {
    extract_relation_ids(page, prop_name).len()
}

pub fn extract_string(page: &NotionPage, prop_name: &str) -> String {
    match page.properties.get(prop_name) {
        Some(PropertyValue::Title { title, .. }) => rich_text_to_plain(title),
        Some(PropertyValue::RichText { rich_text, .. }) => rich_text_to_plain(rich_text),
        Some(PropertyValue::Select { select, .. }) => select.as_ref().map_or(String::new(), |o| o.name.clone()),
        Some(PropertyValue::Status { status, .. }) => status.as_ref().map_or(String::new(), |o| o.name.clone()),
        Some(PropertyValue::Url { url, .. }) => url.clone().unwrap_or_default(),
        Some(PropertyValue::Email { email, .. }) => email.clone().unwrap_or_default(),
        Some(PropertyValue::PhoneNumber { phone_number, .. }) => phone_number.clone().unwrap_or_default(),
        _ => String::new(),
    }
}

pub fn extract_number(page: &NotionPage, prop_name: &str) -> Option<f64> {
    match page.properties.get(prop_name) {
        Some(PropertyValue::Number { number, .. }) => *number,
        Some(PropertyValue::Formula { formula, .. }) => formula.number,
        _ => None,
    }
}

pub fn extract_date(page: &NotionPage, prop_name: &str) -> String {
    match page.properties.get(prop_name) {
        Some(PropertyValue::Date { date, .. }) => date.as_ref().map(|d| d.start.clone()).unwrap_or_default(),
        _ => String::new(),
    }
}

pub fn extract_boolean(page: &NotionPage, prop_name: &str) -> Option<bool> {
    match page.properties.get(prop_name) {
        Some(PropertyValue::Checkbox { checkbox, .. }) => Some(*checkbox),
        _ => None,
    }
}