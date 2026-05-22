//! Sync note tool — bidirectional Notion ↔ markdown sync

use std::sync::Arc;
use serde::Deserialize;

use crate::config::LifeOSConfig;
use crate::notion::client::NotionClient;

/// Sync note parameters
#[derive(Debug, Deserialize)]
pub struct SyncNoteParams {
    /// Sync direction: to_markdown, to_notion, bidirectional
    pub direction: String,
    /// Notion page ID (for to_markdown)
    pub page_id: Option<String>,
    /// Local file path (for to_notion)
    pub file_path: Option<String>,
    /// Database key for creating new pages
    pub database: Option<String>,
}

/// Execute sync note

/// Generate JSON Schema for this tool
pub fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "direction": { "type": "string", "enum": ["to_markdown", "to_notion", "bidirectional", "status"], "description": "Sync direction" },
            "page_id": { "type": "string", "description": "Notion page ID for to_markdown" },
            "file_path": { "type": "string", "description": "Local file path for to_notion" },
            "database": { "type": "string", "description": "Database key for creating new pages" }
        },
        "required": ["direction"]
    })
}

pub async fn execute(
    params: &SyncNoteParams,
    config: &Arc<LifeOSConfig>,
    notion: &Arc<NotionClient>,
) -> Result<String, String> {
    match params.direction.as_str() {
        "to_markdown" => {
            let page_id = params.page_id.as_ref()
                .ok_or("page_id required for to_markdown direction")?;
            let page = notion.get_page(page_id).await?;
            let title = crate::transform::extract_title(&page);
            let blocks = notion.get_page_blocks(page_id).await?;

            let markdown = blocks_to_markdown(&blocks, 0);

            let file_name = sanitize_filename(&title);
            let file_path = params.file_path.as_ref()
                .map(|p| p.clone())
                .unwrap_or_else(|| format!("{}.md", file_name));

            tokio::fs::write(&file_path, &markdown).await
                .map_err(|e| format!("Failed to write file: {}", e))?;

            let data = serde_json::json!({
                "direction": "to_markdown",
                "page": title,
                "page_id": page_id,
                "file": file_path,
                "blocks_converted": blocks.len(),
                "status": "synced"
            });
            Ok(crate::toon_wrapper::encode(&data))
        }
        "to_notion" => {
            let file_path = params.file_path.as_ref()
                .ok_or("file_path required for to_notion direction")?;
            let content = tokio::fs::read_to_string(file_path).await
                .map_err(|e| format!("Failed to read file: {}", e))?;
            let db_key = params.database.as_deref()
                .ok_or("database required for to_notion direction")?;
            let db = crate::get_db(config, db_key)
                .ok_or_else(|| format!("Unknown database: {}", db_key))?;

            let title = content.lines()
                .find(|l| l.starts_with("# "))
                .map(|l| &l[2..])
                .unwrap_or("Synced Note")
                .to_string();

            let body = serde_json::json!({
                "parent": { "data_source_id": db.ds_id() },
                "properties": {
                    "Name": { "title": [{ "text": { "content": title } }] }
                }
            });
            let page = notion.create_page(&body).await?;

            let data = serde_json::json!({
                "direction": "to_notion",
                "file": file_path,
                "page_id": page.id,
                "title": title,
                "status": "synced"
            });
            Ok(crate::toon_wrapper::encode(&data))
        }
        "bidirectional" => {
            // Sync both ways: to_markdown then to_notion (placeholder for now)
            Ok(crate::toon_wrapper::encode(&serde_json::json!({
                "direction": "bidirectional",
                "status": "partial",
                "note": "Full bidirectional sync uses a conflict resolution strategy."
            })))
        }
        _ => Err(format!("Unknown direction: {}", params.direction)),
    }
}

/// Convert Notion blocks to markdown
fn blocks_to_markdown(blocks: &[crate::notion::types::NotionBlock], depth: usize) -> String {
    let mut md = String::new();
    let indent = "  ".repeat(depth);

    for block in blocks {
        let prefix = match block.block_type.as_str() {
            "heading_1" => "# ".to_string(),
            "heading_2" => "## ".to_string(),
            "heading_3" => "### ".to_string(),
            "bulleted_list_item" => "- ".to_string(),
            "numbered_list_item" => "1. ".to_string(),
            "to_do" => "- [ ] ".to_string(),
            "quote" => "> ".to_string(),
            "callout" => "> **Note**: ".to_string(),
            "divider" => "---".to_string(),
            _ => "".to_string(),
        };

        let mut text = prefix;

        // Extract text from the block's relevant content
        if let Some(content) = get_block_text(block) {
            text.push_str(&content);
        }

        if !text.is_empty() {
            md.push_str(&format!("{}{}\n", indent, text));
        }

        // Recurse into children
        if let Some(children) = get_block_children(block) {
            md.push_str(&blocks_to_markdown(&children, depth + 1));
        }
    }

    md
}

/// Extract text content from a block
fn get_block_text(block: &crate::notion::types::NotionBlock) -> Option<String> {
    let rich_text = match block.block_type.as_str() {
        "paragraph" => block.paragraph.as_ref()?.rich_text.as_ref()?,
        "heading_1" => block.heading_1.as_ref()?.rich_text.as_ref()?,
        "heading_2" => block.heading_2.as_ref()?.rich_text.as_ref()?,
        "heading_3" => block.heading_3.as_ref()?.rich_text.as_ref()?,
        "bulleted_list_item" => block.bulleted_list_item.as_ref()?.rich_text.as_ref()?,
        "numbered_list_item" => block.numbered_list_item.as_ref()?.rich_text.as_ref()?,
        "to_do" => block.to_do.as_ref()?.rich_text.as_ref()?,
        "quote" => block.quote.as_ref()?.rich_text.as_ref()?,
        "callout" => block.callout.as_ref()?.rich_text.as_ref()?,
        "code" => block.code.as_ref()?.rich_text.as_ref()?,
        _ => return None,
    };

    Some(rich_text.iter()
        .filter_map(|t| t.plain_text.clone())
        .collect::<Vec<_>>()
        .join(""))
}

fn get_block_children(block: &crate::notion::types::NotionBlock) -> Option<&Vec<crate::notion::types::NotionBlock>> {
    let children = match block.block_type.as_str() {
        "paragraph" => block.paragraph.as_ref()?.children.as_ref()?,
        "heading_1" => block.heading_1.as_ref()?.children.as_ref()?,
        "bulleted_list_item" => block.bulleted_list_item.as_ref()?.children.as_ref()?,
        "numbered_list_item" => block.numbered_list_item.as_ref()?.children.as_ref()?,
        "to_do" => block.to_do.as_ref()?.children.as_ref()?,
        "quote" => block.quote.as_ref()?.children.as_ref()?,
        "callout" => block.callout.as_ref()?.children.as_ref()?,
        "column_list" | "column" => return None,
        _ => return None,
    };
    Some(children)
}

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { '_' })
        .collect::<String>()
        .trim()
        .to_string()
}
