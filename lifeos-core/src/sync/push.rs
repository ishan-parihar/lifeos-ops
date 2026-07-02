use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

use serde_json::Value;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;

use crate::transform::{markdown_to_blocks, yaml_to_properties};
use crate::vault::IndexEntry;

fn load_ignore_patterns(vault_dir: &Path) -> Vec<String> {
    let ignore_path = vault_dir.join(".lifeosignore");
    match fs::read_to_string(&ignore_path) {
        Ok(content) => content
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect(),
        Err(_) => Vec::new(),
    }
}

fn is_ignored(path: &Path, patterns: &[String]) -> bool {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let rel = path
        .components()
        .last()
        .and_then(|c| c.as_os_str().to_str())
        .unwrap_or("");
    patterns.iter().any(|p| {
        if p.contains('/') {
            path.to_string_lossy().contains(p.as_str())
        } else {
            simple_glob_match(file_name, p) || simple_glob_match(rel, p)
        }
    })
}

fn simple_glob_match(name: &str, pattern: &str) -> bool {
    if pattern == name {
        return true;
    }
    if pattern.starts_with('*') && name.ends_with(&pattern[1..]) {
        return true;
    }
    if pattern.ends_with('*') && name.starts_with(&pattern[..pattern.len() - 1]) {
        return true;
    }
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            return name.starts_with(parts[0]) && name.ends_with(parts[1]);
        }
    }
    false
}

pub struct PushReport {
    pub pages_created: usize,
    pub pages_updated: usize,
    pub errors: Vec<String>,
}

fn collect_md_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let mut dirs = vec![dir.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        if !dir.exists() {
            continue;
        }
        let entries =
            fs::read_dir(&dir).map_err(|e| format!("Read dir {}: {e}", dir.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("Entry: {e}"))?;
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
            } else if path.extension().map_or(false, |e| e == "md") {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn parse_frontmatter<'a>(content: &'a str) -> (Option<&'a str>, &'a str) {
    let trimmed = content.trim();
    if trimmed.starts_with("---") {
        if let Some(end) = trimmed[3..].find("\n---") {
            let frontmatter = &trimmed[3..3 + end];
            // Skip "\n---" (4 bytes) then optionally one "\n"
            let after_close = 3 + end + 4;
            let body = if after_close < trimmed.len() {
                let rest = &trimmed[after_close..];
                if rest.starts_with('\n') { &rest[1..] } else { rest }
            } else {
                ""
            };
            return (Some(frontmatter), body.trim());
        }
    }
    (None, trimmed)
}

fn find_index_by_file_path<'a>(
    index: &'a HashMap<String, IndexEntry>,
    file_path: &str,
) -> Option<&'a IndexEntry> {
    index.values().find(|e| e.file_path == file_path)
}

fn extract_title_from_frontmatter(yaml_str: &str) -> Option<String> {
    let parsed: serde_yaml::Value = serde_yaml::from_str(yaml_str).ok()?;
    parsed
        .get("title")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
}

fn extract_title_from_body(body: &str) -> Option<String> {
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            return Some(trimmed[2..].trim().to_string());
        }
    }
    None
}

fn is_newer_than_notion(file_path: &Path, last_edited_time: &str) -> Result<bool, String> {
    let file_mtime = fs::metadata(file_path)
        .and_then(|m| m.modified())
        .map_err(|e| format!("Read mtime {}: {e}", file_path.display()))?;

    let notion_dt = chrono::DateTime::parse_from_rfc3339(last_edited_time)
        .map_err(|e| format!("Parse Notion time '{last_edited_time}': {e}"))?;

    let notion_unix = notion_dt.timestamp();
    let file_unix = file_mtime
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("File time before epoch: {e}"))?
        .as_secs() as i64;

    Ok(file_unix > notion_unix)
}

async fn push_updated_page(
    notion: &NotionClient,
    page_id: &str,
    frontmatter: Option<&str>,
    body: &str,
    property_map: &HashMap<String, String>,
    dry_run: bool,
) -> Result<(), String> {
    if let Some(yaml_str) = frontmatter {
        let yaml_val: serde_yaml::Value = serde_yaml::from_str(yaml_str)
            .map_err(|e| format!("Parse YAML: {e}"))?;
        let properties = yaml_to_properties(&yaml_val, property_map, None);

        if !properties.is_empty() {
            if dry_run {
                tracing::info!("    Would update properties");
            } else {
                let props_value = serde_json::json!(properties);
                notion.update_page_properties(page_id, &props_value).await?;
            }
        }
    }

    if body.is_empty() {
        return Ok(());
    }

    if dry_run {
        tracing::info!("    Would update body content ({} chars)", body.len());
        return Ok(());
    }

    let existing_blocks = notion.get_page_blocks(page_id).await?;

    for block in &existing_blocks {
        if let Some(id) = &block.id {
            notion.delete_block(id).await?;
        }
    }

    let new_blocks = markdown_to_blocks(body);
    if !new_blocks.is_empty() {
        notion.append_blocks(page_id, new_blocks).await?;
    }

    Ok(())
}

async fn push_created_page(
    notion: &NotionClient,
    data_source_id: &str,
    title: &str,
    frontmatter: Option<&str>,
    body: &str,
    property_map: &HashMap<String, String>,
    dry_run: bool,
) -> Result<String, String> {
    let title_notion_name = property_map.get("title").map(|s| s.as_str()).unwrap_or("Name");

    let mut properties: HashMap<String, Value> = if let Some(yaml_str) = frontmatter {
        serde_yaml::from_str(yaml_str)
            .ok()
            .map(|v: serde_yaml::Value| yaml_to_properties(&v, property_map, None))
            .unwrap_or_default()
    } else {
        HashMap::new()
    };

    // Always overwrite the title with the correct Notion property name and title format.
    // yaml_to_properties handles the title key correctly now, but we re-insert here
    // to guarantee the `title` param (the file stem / CLI arg) takes precedence.
    let title_prop: Value =
        serde_json::json!({ "title": [{ "type": "text", "text": { "content": title } }] });
    properties.insert(title_notion_name.to_string(), title_prop);

    if dry_run {
        tracing::info!("  Would create page: {}", title);
        return Ok(String::new());
    }

    let create_body = serde_json::json!({
        "parent": {
            "type": "data_source_id",
            "data_source_id": data_source_id
        },
        "properties": properties,
    });

    let page = notion.create_page(&create_body).await?;

    if !body.is_empty() {
        let blocks = markdown_to_blocks(body);
        if !blocks.is_empty() {
            notion.append_blocks(&page.id, blocks).await?;
        }
    }

    Ok(page.id)
}

pub async fn push_database(
    notion: &NotionClient,
    config: &LifeOSConfig,
    db_key: &str,
    vault_dir: &Path,
    index: &HashMap<String, IndexEntry>,
    dry_run: bool,
) -> Result<PushReport, String> {
    // Use resolve_db to get database config
    let db = match crate::config::resolve_db(config, db_key) {
        Some(db) => db,
        None => return Err(format!("Database key '{}' not found in config", db_key)),
    };
    let ds_id = db.ds_id();
    let db_name = &db.name;
    let properties = &db.properties;

    tracing::info!(
        "Pushing database: {} ({}){}",
        db_name,
        db_key,
        if dry_run { " [DRY RUN]" } else { "" }
    );

    let db_dir = vault_dir.join(db_key);
    if !db_dir.exists() {
        tracing::warn!(
            "  Directory {} does not exist — nothing to push",
            db_dir.display()
        );
        return Ok(PushReport {
            pages_created: 0,
            pages_updated: 0,
            errors: Vec::new(),
        });
    }

    let ignore_patterns = load_ignore_patterns(vault_dir);

    let all_files = collect_md_files(&db_dir)?;
    let total = all_files.len();
    let files: Vec<PathBuf> = all_files
        .into_iter()
        .filter(|f| !is_ignored(f, &ignore_patterns))
        .collect();
    let ignored = total - files.len();
    tracing::info!("  Found {} .md files ({} ignored)", files.len(), ignored);

    let mut report = PushReport {
        pages_created: 0,
        pages_updated: 0,
        errors: Vec::new(),
    };

    for file_path in &files {
        let rel_path = file_path
            .strip_prefix(vault_dir)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                let msg = format!("Read {}: {e}", file_path.display());
                tracing::error!("  [err] {msg}");
                report.errors.push(msg);
                continue;
            }
        };

        let (frontmatter, body) = parse_frontmatter(&content);

        if let Some(entry) = find_index_by_file_path(index, &rel_path) {
            match is_newer_than_notion(file_path, &entry.last_edited_time) {
                Ok(true) => {
                    tracing::info!("  [update] {}", entry.title);
                    if let Err(e) = push_updated_page(
                        notion,
                        &entry.page_id,
                        frontmatter,
                        body,
                        &properties,
                        dry_run,
                    )
                    .await
                    {
                        let msg = format!("Update {}: {e}", entry.title);
                        tracing::error!("  [err] {msg}");
                        report.errors.push(msg);
                        continue;
                    }
                    report.pages_updated += 1;
                }
                Ok(false) => {
                    tracing::info!(
                        "  [skip] {} — Notion is newer or equal",
                        entry.title
                    );
                }
                Err(e) => {
                    tracing::warn!("  [warn] {} timestamp check: {e}", entry.title);
                    if let Err(e) = push_updated_page(
                        notion,
                        &entry.page_id,
                        frontmatter,
                        body,
                        &properties,
                        dry_run,
                    )
                    .await
                    {
                        let msg = format!("Fallback update {}: {e}", entry.title);
                        tracing::error!("  [err] {msg}");
                        report.errors.push(msg);
                        continue;
                    }
                    report.pages_updated += 1;
                }
            }
        } else {
            let title = frontmatter
                .and_then(extract_title_from_frontmatter)
                .or_else(|| extract_title_from_body(body))
                .unwrap_or_else(|| {
                    let stem = file_path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    stem
                });

            tracing::info!("  [create] {}", title);

            match push_created_page(
                notion,
                &ds_id,
                &title,
                frontmatter,
                body,
                &properties,
                dry_run,
            )
            .await
            {
                Ok(new_id) => {
                    if !dry_run && !new_id.is_empty() {
                        tracing::info!(
                            "    Created page {} (id: {})",
                            title,
                            &new_id[..8.min(new_id.len())]
                        );
                    }
                    report.pages_created += 1;
                }
                Err(e) => {
                    let msg = format!("Create {}: {e}", title);
                    tracing::error!("  [err] {msg}");
                    report.errors.push(msg);
                }
            }
        }
    }

    tracing::info!(
        "  Done: {} created, {} updated, {} errors",
        report.pages_created,
        report.pages_updated,
        report.errors.len()
    );

    Ok(report)
}
