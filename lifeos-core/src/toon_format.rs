//! TOON format — compact structured output for AI consumption
//!
//! Produces a YAML-like compact format optimized for token efficiency.

use serde_json::Value;

/// Encode a JSON value into TOON format (compact structured text)
pub fn encode(value: &Value) -> String {
    let mut out = String::new();
    encode_value(&mut out, value, 0);
    out
}

fn encode_value(out: &mut String, value: &Value, indent: usize) {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                match val {
                    Value::Array(arr) if !arr.is_empty() && arr.iter().all(|v| v.is_object()) => {
                        // Array of objects — compact table format
                        let fields = collect_fields(arr);
                        write_indent(out, indent);
                        out.push_str(&format!("{}[{}]{{{}}}:\n", key, arr.len(), fields.join(",")));
                        for item in arr {
                            write_indent(out, indent + 1);
                            out.push_str(&format_row(item, &fields));
                            out.push('\n');
                        }
                    }
                    Value::Object(_) => {
                        write_indent(out, indent);
                        out.push_str(key);
                        out.push_str(":\n");
                        encode_value(out, val, indent + 1);
                    }
                    _ => {
                        write_indent(out, indent);
                        out.push_str(key);
                        out.push_str(": ");
                        out.push_str(&format_scalar(val));
                        out.push('\n');
                    }
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                write_indent(out, indent);
                out.push_str("- ");
                out.push_str(&format_scalar(item));
                out.push('\n');
            }
        }
        _ => {
            write_indent(out, indent);
            out.push_str(&format_scalar(value));
            out.push('\n');
        }
    }
}

fn write_indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push_str("  ");
    }
}

fn format_scalar(val: &Value) -> String {
    match val {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => {
            if s.contains('\n') || s.contains(':') || s.contains(',') {
                format!("\"{}\"", s.replace('"', "\\\""))
            } else {
                s.clone()
            }
        }
        Value::Array(arr) => format!("[{}]", arr.iter().map(format_scalar).collect::<Vec<_>>().join(",")),
        Value::Object(_) => serde_json::to_string(val).unwrap_or_default(),
    }
}

fn collect_fields(arr: &[Value]) -> Vec<String> {
    if let Some(Value::Object(first)) = arr.first() {
        first.keys().cloned().collect()
    } else {
        vec![]
    }
}

fn format_row(item: &Value, fields: &[String]) -> String {
    if let Value::Object(map) = item {
        fields.iter()
            .map(|f| format_scalar(map.get(f).unwrap_or(&Value::Null)))
            .collect::<Vec<_>>()
            .join(",")
    } else {
        format_scalar(item)
    }
}
