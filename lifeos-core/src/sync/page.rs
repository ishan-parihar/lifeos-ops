use std::collections::HashMap;
use std::path::Path;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;
use crate::notion::types::*;
use crate::transform::{blocks_to_markdown, extract_properties_yaml, extract_title, yaml_to_properties};
use crate::vault::{vault_path, IndexEntry, read_index, write_index};
use super::merge::*;

pub async fn cmd_page_new(
    notion: &NotionClient,
    config: &LifeOSConfig,
    vault_dir: &Path,
    db_key: &str,
    title: &str,
) -> Result<(), String> {
    let db = config.databases.get(db_key)
        .ok_or_else(|| format!("Unknown database key: {db_key}"))?;

    let mut properties = serde_json::Map::new();

    let title_key = db.properties.iter()
        .find(|(_, v)| **v == "title")
        .or_else(|| db.properties.iter().find(|(k, _)| k == &"Name" || k == &"name" || k == &"Title"))
        .map(|(k, _)| k.clone())
        .ok_or_else(|| "Could not determine title property".to_string())?;

    let title_notion_name = db.properties.get(&title_key)
        .ok_or_else(|| "Title key not in properties mapping".to_string())?;

    properties.insert(
        title_notion_name.clone(),
        serde_json::json!({
            "title": [{"type": "text", "text": {"content": title}}]
        }),
    );

    let body = serde_json::json!({
        "parent": { "data_source_id": db.ds_id() },
        "properties": properties,
    });

    tracing::info!("Creating page '{title}' in {db_key}...");
    let page = notion.create_page(&body).await?;
    tracing::info!("Created page {} (id: {})", title, page.id);

    let mut index = read_index(vault_dir)?;
    let blocks = notion.get_page_blocks(&page.id).await.unwrap_or_default();
    let title_cache: HashMap<String, String> = index.iter()
        .map(|(id, e)| (id.clone(), e.title.clone()))
        .collect();

    let frontmatter_yaml = extract_properties_yaml(&page, &db.properties, &title_cache)
        .map(|y| serde_yaml::to_string(&y).unwrap_or_default())
        .unwrap_or_default();
    let markdown_body = blocks_to_markdown(&blocks);

    let title_plain = extract_title(&page);
    let date = extract_page_date_opt(&page.properties);
    let file_path = vault_path(vault_dir, db_key, &page.id, date);

    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Create dir: {e}"))?;
    }

    let content = format!("---\n{}---\n\n{}", frontmatter_yaml, markdown_body);
    std::fs::write(&file_path, &content)
        .map_err(|e| format!("Write vault file: {e}"))?;

    let rel_path = file_path.strip_prefix(vault_dir).unwrap_or(&file_path)
        .to_string_lossy().to_string();

    index.insert(page.id.clone(), IndexEntry {
        page_id: page.id.clone(),
        last_edited_time: page.last_edited_time.clone(),
        file_path: rel_path,
        db_key: db_key.to_string(),
        title: title_plain,
    });

    write_index(vault_dir, &index)?;
    tracing::info!("Written to vault: {}", file_path.display());
    Ok(())
}

pub async fn cmd_page_edit(
    notion: &NotionClient,
    config: &LifeOSConfig,
    vault_dir: &Path,
    page_id: &str,
) -> Result<(), String> {
    let index = read_index(vault_dir)?;
    let entry = index.get(page_id)
        .ok_or_else(|| format!("Page {page_id} not found in vault index"))?;

    let vault_path = vault_dir.join(&entry.file_path);
    if !vault_path.exists() {
        return Err(format!("Vault file not found: {}", vault_path.display()));
    }

    let local_before = std::fs::read_to_string(&vault_path)
        .map_err(|e| format!("Read vault file: {e}"))?;

    let base_content = match read_base_snapshot(vault_dir, page_id) {
        Ok(c) => c,
        Err(_) => local_before.clone(),
    };

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    tracing::info!("Opening {} with {} ...", vault_path.display(), editor);

    let status = std::process::Command::new(&editor)
        .arg(&vault_path)
        .status()
        .map_err(|e| format!("Failed to launch editor {}: {e}", editor))?;

    if !status.success() {
        return Err(format!("Editor exited with non-zero status"));
    }

    let local_after = std::fs::read_to_string(&vault_path)
        .map_err(|e| format!("Read modified vault file: {e}"))?;

    if local_after == local_before {
        tracing::info!("No changes made. Nothing to do.");
        return Ok(());
    }

    tracing::info!("Fetching current Notion version...");
    let remote_page = notion.get_page(page_id).await?;
    let remote_blocks = notion.get_page_blocks(page_id).await.unwrap_or_default();
    let title_cache = index.iter()
        .map(|(id, e)| (id.clone(), e.title.clone()))
        .collect::<HashMap<_, _>>();
    let remote_fm = extract_properties_yaml(&remote_page, &config.databases[&entry.db_key].properties, &title_cache)
        .map(|y| serde_yaml::to_string(&y).unwrap_or_default())
        .unwrap_or_default();
    let remote_body = blocks_to_markdown(&remote_blocks);
    let remote_content = format!("---\n{}---\n\n{}", remote_fm, remote_body);

    let (base_fm, base_body) = parse_vault_file(&base_content)?;
    let (local_fm, local_body) = parse_vault_file(&local_after)?;
    let (remote_fm, remote_body) = parse_vault_file(&remote_content)?;

    if remote_fm == base_fm && remote_body == base_body {
        tracing::info!("No Notion changes detected. Updating local edits.");
        let rel_path = vault_path.strip_prefix(vault_dir).unwrap_or(&vault_path)
            .to_string_lossy().to_string();

        let mut index = read_index(vault_dir)?;
        index.insert(page_id.to_string(), IndexEntry {
            page_id: page_id.to_string(),
            last_edited_time: remote_page.last_edited_time.clone(),
            file_path: rel_path,
            db_key: entry.db_key.clone(),
            title: extract_title(&remote_page),
        });
        write_index(vault_dir, &index)?;
        tracing::info!("Local edits saved. Run `lifeos-sync push` to push to Notion.");
        return Ok(());
    }

    tracing::info!("Notion has changed since last pull. Performing 3-way merge...");

    let base_vf = VaultFile { frontmatter: base_fm, body: base_body };
    let local_vf = VaultFile { frontmatter: local_fm, body: local_body };
    let remote_vf = VaultFile { frontmatter: remote_fm, body: remote_body };

    let merge_result = three_way_merge(&base_vf, &local_vf, &remote_vf)?;

    if merge_result.has_conflicts {
        tracing::warn!("Merge conflicts detected:");
        println!("{}", format_conflicts(&merge_result.conflicts));

        let merged_content = make_vault_content(&merge_result.merged_frontmatter, &merge_result.merged_body);
        std::fs::write(&vault_path, &merged_content)
            .map_err(|e| format!("Write merged file: {e}"))?;
        tracing::info!("Merged file written with conflict markers to {}", vault_path.display());
        tracing::info!("Resolve conflicts manually, then run `lifeos-sync page merge {}` to complete.", page_id);
    } else {
        tracing::info!("Merge successful (no conflicts).");
        let merged_content = make_vault_content(&merge_result.merged_frontmatter, &merge_result.merged_body);
        std::fs::write(&vault_path, &merged_content)
            .map_err(|e| format!("Write merged file: {e}"))?;
        tracing::info!("Updated vault file: {}", vault_path.display());

        let _ = push_page_changes(notion, config, vault_dir, page_id, &entry.db_key, &merge_result).await;
    }

    Ok(())
}

pub async fn cmd_page_diff(
    notion: &NotionClient,
    config: &LifeOSConfig,
    vault_dir: &Path,
    page_id: &str,
) -> Result<(), String> {
    let index = read_index(vault_dir)?;
    let entry = index.get(page_id)
        .ok_or_else(|| format!("Page {page_id} not found in vault index"))?;

    let vault_path = vault_dir.join(&entry.file_path);
    if !vault_path.exists() {
        return Err(format!("Vault file not found: {}", vault_path.display()));
    }

    let local = std::fs::read_to_string(&vault_path)
        .map_err(|e| format!("Read vault file: {e}"))?;
    let base = read_base_snapshot(vault_dir, page_id).unwrap_or_else(|_| local.clone());

    let remote_page = notion.get_page(page_id).await?;
    let remote_blocks = notion.get_page_blocks(page_id).await.unwrap_or_default();
    let title_cache = index.iter()
        .map(|(id, e)| (id.clone(), e.title.clone()))
        .collect::<HashMap<_, _>>();
    let remote_fm = extract_properties_yaml(&remote_page, &config.databases[&entry.db_key].properties, &title_cache)
        .map(|y| serde_yaml::to_string(&y).unwrap_or_default())
        .unwrap_or_default();
    let remote_body = blocks_to_markdown(&remote_blocks);
    let remote = format!("---\n{}---\n\n{}", remote_fm, remote_body);

    let (base_fm, base_body) = parse_vault_file(&base)?;
    let (_, local_body) = parse_vault_file(&local)?;
    let (_, remote_body) = parse_vault_file(&remote)?;

    println!("─── Frontmatter Diff (vault vs Notion) ───");
    let fm_remote_raw = read_frontmatter_raw(&remote);
    let fm_hunks = diff_frontmatter(&base_fm, &parse_frontmatter_into_mapping(&fm_remote_raw));
    for hunk in &fm_hunks {
        if hunk.kind != DiffKind::Unchanged {
            println!("  {} {}", match hunk.kind {
                DiffKind::Added => "+",
                DiffKind::Removed => "-",
                DiffKind::Changed => "~",
                _ => "?",
            }, hunk.location);
            if let Some(ref l) = hunk.local {
                println!("    local:  {l}");
            }
            if let Some(ref r) = hunk.remote {
                println!("    remote: {r}");
            }
        }
    }
    if fm_hunks.iter().all(|h| h.kind == DiffKind::Unchanged) {
        println!("  (no differences)");
    }

    println!("\n─── Body Diff (vault vs Notion) ───");
    let body_hunks = diff_body(&base_body, &remote_body);
    let body_hunks_local = diff_body(&base_body, &local_body);
    let all_body_changes: Vec<DiffHunk> = body_hunks.into_iter()
        .chain(body_hunks_local)
        .filter(|h| h.kind != DiffKind::Unchanged)
        .collect();

    if all_body_changes.is_empty() {
        println!("  (no differences)");
    } else {
        for hunk in &all_body_changes {
            let prefix = match hunk.kind {
                DiffKind::Added => "+ ",
                DiffKind::Removed => "- ",
                DiffKind::Changed => "~ ",
                _ => "  ",
            };
            println!("  {}{}", prefix, hunk.location);
        }
    }

    Ok(())
}

pub async fn cmd_page_merge(
    notion: &NotionClient,
    config: &LifeOSConfig,
    vault_dir: &Path,
    page_id: &str,
) -> Result<(), String> {
    let index = read_index(vault_dir)?;
    let entry = index.get(page_id)
        .ok_or_else(|| format!("Page {page_id} not found in vault index"))?;

    let vault_path = vault_dir.join(&entry.file_path);
    if !vault_path.exists() {
        return Err(format!("Vault file not found: {}", vault_path.display()));
    }

    let local_content = std::fs::read_to_string(&vault_path)
        .map_err(|e| format!("Read vault file: {e}"))?;

    if local_content.contains("<<<<<<<") || local_content.contains("=======") || local_content.contains(">>>>>>>") {
        tracing::warn!("File still contains conflict markers. Please resolve them manually first.");
        return Err("Unresolved conflict markers remain. Edit the file to resolve conflicts, then re-run merge.".to_string());
    }

    let remote_page = notion.get_page(page_id).await?;
    let remote_blocks = notion.get_page_blocks(page_id).await.unwrap_or_default();
    let title_cache = index.iter()
        .map(|(id, e)| (id.clone(), e.title.clone()))
        .collect::<HashMap<_, _>>();
    let remote_fm = extract_properties_yaml(&remote_page, &config.databases[&entry.db_key].properties, &title_cache)
        .map(|y| serde_yaml::to_string(&y).unwrap_or_default())
        .unwrap_or_default();
    let remote_body = blocks_to_markdown(&remote_blocks);
    let remote_content = format!("---\n{}---\n\n{}", remote_fm, remote_body);

    let (remote_fm, remote_body) = parse_vault_file(&remote_content)?;
    let (local_fm, local_body) = parse_vault_file(&local_content)?;

    let base_content = read_base_snapshot(vault_dir, page_id).unwrap_or_else(|_| local_content.clone());
    let (base_fm, base_body) = parse_vault_file(&base_content)?;

    let base_vf = VaultFile { frontmatter: base_fm, body: base_body };
    let local_vf = VaultFile { frontmatter: local_fm, body: local_body };
    let remote_vf = VaultFile { frontmatter: remote_fm, body: remote_body };

    let merge_result = three_way_merge(&base_vf, &local_vf, &remote_vf)?;

    if merge_result.has_conflicts {
        tracing::warn!("Still {} conflict(s). Resolve them and re-run merge.", merge_result.conflicts.len());
        println!("{}", format_conflicts(&merge_result.conflicts));
        return Err("Conflicts remain after merge".to_string());
    }

    let merged_content = make_vault_content(&merge_result.merged_frontmatter, &merge_result.merged_body);
    std::fs::write(&vault_path, &merged_content)
        .map_err(|e| format!("Write merged file: {e}"))?;

    let _ = push_page_changes(notion, config, vault_dir, page_id, &entry.db_key, &merge_result).await;

    tracing::info!("Merge complete and pushed to Notion.");
    Ok(())
}

async fn push_page_changes(
    notion: &NotionClient,
    config: &LifeOSConfig,
    vault_dir: &Path,
    page_id: &str,
    db_key: &str,
    merge_result: &MergeResult,
) -> Result<(), String> {
    let db = config.databases.get(db_key)
        .ok_or_else(|| format!("Unknown db: {db_key}"))?;

    let merged_yaml = serde_yaml::Value::Mapping(merge_result.merged_frontmatter.clone());
    let properties = yaml_to_properties(&merged_yaml, &db.properties);

    if !properties.is_empty() {
        let prop_body = serde_json::json!({ "properties": properties });
        notion.update_page_full(page_id, &prop_body).await?;
        tracing::info!("Updated {} properties on page {page_id}", properties.len());
    }

    let parsed_blocks = crate::transform::markdown_to_blocks(&merge_result.merged_body);
    let existing_blocks = notion.get_page_blocks(page_id).await?;
    for block in &existing_blocks {
        if let Some(ref bid) = block.id {
            let _ = notion.delete_block(bid).await;
        }
    }
    notion.append_blocks(page_id, parsed_blocks).await?;
    tracing::info!("Replaced body blocks on page {page_id}");

    let merged_file = make_vault_content(&merge_result.merged_frontmatter, &merge_result.merged_body);
    store_base_snapshot(vault_dir, page_id, &merged_file)?;

    let mut index = read_index(vault_dir)?;
    let fresh = notion.get_page(page_id).await?;
    let rel_path = vault_dir.join(db_key).join(format!("{page_id}.md"))
        .strip_prefix(vault_dir).unwrap().to_string_lossy().to_string();

    index.insert(page_id.to_string(), IndexEntry {
        page_id: page_id.to_string(),
        last_edited_time: fresh.last_edited_time.clone(),
        file_path: rel_path,
        db_key: db_key.to_string(),
        title: extract_title(&fresh),
    });
    write_index(vault_dir, &index)?;

    Ok(())
}

fn extract_page_date_opt(properties: &HashMap<String, PropertyValue>) -> Option<chrono::NaiveDate> {
    for value in properties.values() {
        if let PropertyValue::Date { date, .. } = value {
            if let Some(d) = date {
                if let Ok(parsed) = chrono::NaiveDate::parse_from_str(&d.start, "%Y-%m-%d") {
                    return Some(parsed);
                }
            }
        }
    }
    None
}

fn read_frontmatter_raw(content: &str) -> String {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---\n") {
        return String::new();
    }
    let after = &trimmed[4..];
    if let Some(end) = after.find("\n---\n") {
        after[..end].to_string()
    } else {
        String::new()
    }
}

fn parse_frontmatter_into_mapping(raw: &str) -> serde_yaml::Mapping {
    if raw.is_empty() {
        return serde_yaml::Mapping::new();
    }
    let v: serde_yaml::Value = serde_yaml::from_str(raw).unwrap_or(serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));
    match v {
        serde_yaml::Value::Mapping(m) => m,
        _ => serde_yaml::Mapping::new(),
    }
}
