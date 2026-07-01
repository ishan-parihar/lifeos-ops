use std::collections::HashMap;
use std::path::Path;
use std::fs;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::notion::types::*;
use crate::transform::{blocks_to_markdown, extract_properties_yaml};
use crate::vault::{self, vault_path, IndexEntry};
use crate::sync::merge::store_base_snapshot;

pub struct PullReport {
    pub pages_processed: usize,
    pub files_created: usize,
    pub files_updated: usize,
    pub errors: Vec<String>,
}

fn extract_page_title(properties: &HashMap<String, PropertyValue>) -> Option<String> {
    for value in properties.values() {
        if let PropertyValue::Title { title, .. } = value {
            let parts: Vec<&str> = title
                .iter()
                .filter_map(|t| t.plain_text.as_deref())
                .collect();
            if parts.is_empty() {
                return None;
            }
            return Some(parts.join(""));
        }
    }
    None
}

fn extract_page_date(properties: &HashMap<String, PropertyValue>) -> Option<chrono::NaiveDate> {
    for value in properties.values() {
        if let PropertyValue::Date { date, .. } = value {
            if let Some(d) = date {
                if let Ok(parsed) =
                    chrono::NaiveDate::parse_from_str(&d.start, "%Y-%m-%d")
                {
                    return Some(parsed);
                }
                if let Ok(parsed) =
                    chrono::NaiveDate::parse_from_str(&d.start, "%Y-%m-%dT%H:%M:%S%.f%:z")
                {
                    return Some(parsed);
                }
                if let Ok(parsed) =
                    chrono::NaiveDate::parse_from_str(&d.start, "%Y-%m-%dT%H:%M:%S%.fZ")
                {
                    return Some(parsed);
                }
            }
        }
    }
    None
}

/// Pull pages from Notion, optionally filtering by `since` timestamp.
/// When `since` is `Some(iso_timestamp)`, only pages modified after that
/// time are fetched from Notion, making repeated pulls much cheaper.
/// Supports both reservoir and satellite keys via `resolve_db`.
pub async fn pull_database_since(
    notion: &NotionClient,
    config: &LifeOSConfig,
    db_key: &str,
    vault_dir: &Path,
    index: &mut HashMap<String, IndexEntry>,
    since: Option<&str>,
) -> Result<PullReport, String> {
    // Use resolve_db to support both reservoir and satellite keys
    let (ds_id, db_name, properties) = match crate::config::resolve_db(config, db_key) {
        Some(crate::config::ResolvedDb::Reservoir(_key, db)) => {
            (db.ds_id().to_string(), db.name.clone(), db.properties.clone())
        }
        Some(crate::config::ResolvedDb::Satellite(_, _, sat)) => {
            (sat.ds_id().to_string(), sat.name.clone(), sat.properties.clone())
        }
        None => return Err(format!("Database key '{}' not found in config", db_key)),
    };

    tracing::info!("Pulling database: {} ({})", db_name, db_key);

    if !vault_dir.exists() {
        fs::create_dir_all(vault_dir)
            .map_err(|e| format!("Create vault dir: {e}"))?;
    }

    let pages = notion
        .query_data_source_all_since(&ds_id, since)
        .await?;
    tracing::info!(
        "  Found {} pages in data source{}",
        pages.len(),
        since.map(|s| format!(" (since {s})")).unwrap_or_default()
    );

    let mut report = PullReport {
        pages_processed: 0,
        files_created: 0,
        files_updated: 0,
        errors: Vec::new(),
    };

    let mut title_cache: HashMap<String, String> = index
        .iter()
        .map(|(id, entry)| (id.clone(), entry.title.clone()))
        .collect();

    for page in &pages {
        if let Some(t) = extract_page_title(&page.properties) {
            title_cache.entry(page.id.clone()).or_insert_with(|| t);
        }
    }

    for page in &pages {
        report.pages_processed += 1;

        let title = match extract_page_title(&page.properties) {
            Some(t) => t,
            None => {
                let fallback = format!("untitled-{}", &page.id[..8]);
                tracing::warn!("  Page {} has no title, using {}", page.id, fallback);
                fallback
            }
        };

        title_cache.insert(page.id.clone(), title.clone());

        let date = extract_page_date(&page.properties);

        let file_path = vault_path(vault_dir, db_key, &page.id, date);

        if let Some(entry) = index.get(&page.id) {
            if entry.last_edited_time == page.last_edited_time {
                tracing::info!("  [skip] {} — no changes", title);
                continue;
            }
        }

        let blocks = notion.get_page_blocks(&page.id).await;
        let blocks = match blocks {
            Ok(b) => b,
            Err(e) => {
                let msg = format!("  [err] {} failed to get blocks: {e}", title);
                tracing::error!("{msg}");
                report.errors.push(msg);
                continue;
            }
        };

        let frontmatter_yaml = match extract_properties_yaml(page, &properties, &title_cache) {
            Ok(y) => serde_yaml::to_string(&y).unwrap_or_default(),
            Err(e) => {
                tracing::warn!("  [warn] {} frontmatter: {e}", title);
                String::new()
            }
        };
        let markdown_body = blocks_to_markdown(&blocks);

        let file_content = format!("---\n{}---\n\n{}", frontmatter_yaml, markdown_body);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Create dir {}: {e}", parent.display()))?;
        }

        let existed = file_path.exists();
        if existed {
            if let Ok(old_content) = fs::read_to_string(&file_path) {
                let _ = store_base_snapshot(vault_dir, &page.id, &old_content);
            }
        }
        fs::write(&file_path, &file_content)
            .map_err(|e| format!("Write {}: {e}", file_path.display()))?;

        let rel_path = file_path
            .strip_prefix(vault_dir)
            .unwrap_or(&file_path)
            .to_string_lossy()
            .to_string();

        index.insert(
            page.id.clone(),
            IndexEntry {
                page_id: page.id.clone(),
                last_edited_time: page.last_edited_time.clone(),
                file_path: rel_path,
                db_key: db_key.to_string(),
                title,
            },
        );

        if existed {
            report.files_updated += 1;
        } else {
            report.files_created += 1;
        }
    }

    vault::write_index(vault_dir, index)
        .map_err(|e| format!("Write index: {e}"))?;

    tracing::info!(
        "  Done: {} processed, {} created, {} updated, {} errors",
        report.pages_processed,
        report.files_created,
        report.files_updated,
        report.errors.len()
    );

    Ok(report)
}
