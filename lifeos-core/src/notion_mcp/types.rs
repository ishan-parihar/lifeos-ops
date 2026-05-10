//! Notion API types

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A Notion page property
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    Title { title: Vec<RichTextItem> },
    RichText { rich_text: Vec<RichTextItem> },
    Select { select: Option<SelectOption> },
    Status { status: Option<SelectOption> },
    MultiSelect { multi_select: Vec<SelectOption> },
    Date { date: Option<DateValue> },
    Number { number: Option<f64> },
    Checkbox { checkbox: bool },
    Formula { formula: FormulaValue },
    Relation { relation: Vec<RelationItem> },
    Url { url: Option<String> },
    Email { email: Option<String> },
    PhoneNumber { phone_number: Option<String> },
    Files { files: Vec<FileItem> },
    CreatedTime { created_time: String },
    CreatedBy { created_by: UserItem },
    LastEditedTime { last_edited_time: String },
    LastEditedBy { last_edited_by: UserItem },
    UniqueId { unique_id: Option<UniqueIdValue> },
    Rollup { rollup: RollupValue },
}

/// Notion page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionPage {
    pub id: String,
    pub url: Option<String>,
    pub properties: HashMap<String, PropertyValue>,
    pub created_time: String,
    pub last_edited_time: String,
    pub parent: Option<ParentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentInfo {
    #[serde(rename = "type")]
    pub parent_type: Option<String>,
    pub database_id: Option<String>,
    pub page_id: Option<String>,
}

/// Query response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    pub object: String,
    pub results: Vec<NotionPage>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

/// Data source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionDataSource {
    pub object: String,
    pub id: String,
    pub title: Option<Vec<RichTextItem>>,
    pub properties: HashMap<String, PropertySchema>,
}

/// Database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionDatabase {
    pub id: String,
    pub properties: Option<HashMap<String, PropertySchema>>,
    pub data_sources: Option<Vec<DataSourceRef>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceRef {
    pub id: String,
    #[serde(rename = "type")]
    pub ds_type: Option<String>,
}

/// Property schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySchema {
    pub id: String,
    #[serde(rename = "type")]
    pub prop_type: String,
    pub name: Option<String>,
    pub select: Option<SelectConfig>,
    pub status: Option<StatusConfig>,
    pub multi_select: Option<SelectConfig>,
    pub relation: Option<RelationConfig>,
    pub formula: Option<FormulaConfig>,
    pub rollup: Option<RollupConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectConfig {
    pub options: Option<Vec<SelectOption>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusConfig {
    pub options: Option<Vec<SelectOption>>,
    pub groups: Option<Vec<StatusGroup>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusGroup {
    pub id: String,
    pub name: String,
    pub color: String,
    pub option_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub id: Option<String>,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationConfig {
    pub database_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaConfig {
    pub expression: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollupConfig {
    pub function: Option<String>,
    pub relation_property_name: Option<String>,
    pub rollup_property_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichTextItem {
    #[serde(rename = "type")]
    pub rt_type: String,
    pub text: TextContent,
    pub annotations: Option<Annotations>,
    pub plain_text: Option<String>,
    pub href: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    pub content: String,
    pub link: Option<LinkInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkInfo {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotations {
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub strikethrough: Option<bool>,
    pub underline: Option<bool>,
    pub code: Option<bool>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateValue {
    pub start: String,
    pub end: Option<String>,
    pub time_zone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaValue {
    #[serde(rename = "type")]
    pub formula_type: String,
    pub string: Option<String>,
    pub number: Option<f64>,
    pub boolean: Option<bool>,
    pub date: Option<DateValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationItem {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileItem {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub file_type: Option<String>,
    pub external: Option<ExternalFile>,
    pub file: Option<FileUrl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalFile {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUrl {
    pub url: String,
    pub expiry_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserItem {
    pub id: String,
    pub object: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniqueIdValue {
    pub prefix: Option<String>,
    pub number: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollupValue {
    #[serde(rename = "type")]
    pub rollup_type: String,
    pub number: Option<f64>,
    pub string: Option<String>,
    pub date: Option<DateValue>,
    pub array: Option<Vec<PropertyValue>>,
}

/// Notion block types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionBlock {
    pub object: String,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub block_type: String,
    pub paragraph: Option<BlockContent>,
    pub heading_1: Option<BlockContent>,
    pub heading_2: Option<BlockContent>,
    pub heading_3: Option<BlockContent>,
    pub bulleted_list_item: Option<BlockContent>,
    pub numbered_list_item: Option<BlockContent>,
    pub to_do: Option<ToDoContent>,
    pub code: Option<CodeContent>,
    pub quote: Option<BlockContent>,
    pub callout: Option<CalloutContent>,
    pub divider: Option<Value>,
    pub child_page: Option<ChildPageContent>,
    pub image: Option<ImageContent>,
    pub video: Option<VideoContent>,
    pub bookmark: Option<BookmarkContent>,
    pub table: Option<TableContent>,
    pub table_row: Option<TableRowContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockContent {
    pub rich_text: Option<Vec<RichTextItem>>,
    pub color: Option<String>,
    pub children: Option<Vec<NotionBlock>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToDoContent {
    pub rich_text: Option<Vec<RichTextItem>>,
    pub checked: Option<bool>,
    pub children: Option<Vec<NotionBlock>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeContent {
    pub rich_text: Option<Vec<RichTextItem>>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalloutContent {
    pub rich_text: Option<Vec<RichTextItem>>,
    pub icon: Option<Value>,
    pub children: Option<Vec<NotionBlock>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildPageContent {
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageContent {
    pub image: Option<FileUrl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoContent {
    pub video: Option<FileUrl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkContent {
    pub url: String,
    pub caption: Option<Vec<RichTextItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableContent {
    pub table_width: Option<i64>,
    pub has_column_header: Option<bool>,
    pub has_row_header: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRowContent {
    pub cells: Option<Vec<Vec<RichTextItem>>>,
}

/// Block list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockListResponse {
    pub object: String,
    pub results: Vec<NotionBlock>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}
