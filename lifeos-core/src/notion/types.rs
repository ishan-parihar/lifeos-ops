#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PropertyValue {
    #[serde(rename = "title")]
    Title { title: Vec<RichTextItem>, #[serde(skip_serializing_if = "Option::is_none")] id: Option<String> },
    #[serde(rename = "rich_text")]
    RichText { rich_text: Vec<RichTextItem>, #[serde(skip_serializing_if = "Option::is_none")] id: Option<String> },
    #[serde(rename = "select")]
    Select { select: Option<SelectOption>, #[serde(skip_serializing_if = "Option::is_none")] id: Option<String> },
    #[serde(rename = "status")]
    Status { status: Option<SelectOption>, #[serde(skip_serializing_if = "Option::is_none")] id: Option<String> },
    #[serde(rename = "multi_select")]
    MultiSelect { multi_select: Vec<SelectOption>, #[serde(skip_serializing_if = "Option::is_none")] id: Option<String> },
    #[serde(rename = "date")]
    Date { date: Option<DateValue>, #[serde(skip_serializing_if = "Option::is_none")] id: Option<String> },
    #[serde(rename = "number")]
    Number { number: Option<f64>, #[serde(skip_serializing_if = "Option::is_none")] id: Option<String> },
    #[serde(rename = "checkbox")]
    Checkbox { checkbox: bool, #[serde(skip_serializing_if = "Option::is_none")] id: Option<String> },
    #[serde(rename = "formula")]
    Formula { formula: FormulaValue, #[serde(skip_serializing_if = "Option::is_none")] id: Option<String> },
    #[serde(rename = "relation")]
    Relation { relation: Vec<RelationItem>, #[serde(skip_serializing_if = "Option::is_none")] id: Option<String> },
    #[serde(rename = "url")]
    Url { url: Option<String>, #[serde(skip_serializing_if = "Option::is_none")] id: Option<String> },
    #[serde(rename = "email")]
    Email { email: Option<String>, #[serde(skip_serializing_if = "Option::is_none")] id: Option<String> },
    #[serde(rename = "phone_number")]
    PhoneNumber { phone_number: Option<String>, #[serde(skip_serializing_if = "Option::is_none")] id: Option<String> },
    #[serde(rename = "files")]
    Files { files: Vec<FileItem>, #[serde(skip_serializing_if = "Option::is_none")] id: Option<String> },
    #[serde(rename = "created_time")]
    CreatedTime { created_time: String, #[serde(skip_serializing_if = "Option::is_none")] id: Option<String> },
    #[serde(rename = "created_by")]
    CreatedBy { created_by: UserItem, #[serde(skip_serializing_if = "Option::is_none")] id: Option<String> },
    #[serde(rename = "last_edited_time")]
    LastEditedTime { last_edited_time: String, #[serde(skip_serializing_if = "Option::is_none")] id: Option<String> },
    #[serde(rename = "last_edited_by")]
    LastEditedBy { last_edited_by: UserItem, #[serde(skip_serializing_if = "Option::is_none")] id: Option<String> },
    #[serde(rename = "unique_id")]
    UniqueId { unique_id: Option<UniqueIdValue>, #[serde(skip_serializing_if = "Option::is_none")] id: Option<String> },
    #[serde(rename = "rollup")]
    Rollup { rollup: RollupValue, #[serde(skip_serializing_if = "Option::is_none")] id: Option<String> },
    #[serde(rename = "people")]
    People { people: Vec<UserItem>, #[serde(skip_serializing_if = "Option::is_none")] id: Option<String> },
    #[serde(rename = "button")]
    Button { #[serde(skip_serializing_if = "Option::is_none")] id: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionPage {
    pub id: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub properties: HashMap<String, PropertyValue>,
    #[serde(default)]
    pub created_time: String,
    #[serde(default)]
    pub last_edited_time: String,
    #[serde(default)]
    pub parent: Option<ParentInfo>,
    #[serde(default)]
    pub icon: Option<Value>,
    #[serde(default)]
    pub cover: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentInfo {
    #[serde(rename = "type")]
    pub parent_type: Option<String>,
    pub database_id: Option<String>,
    pub page_id: Option<String>,
    pub data_source_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    pub object: String,
    pub results: Vec<NotionPage>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionDataSource {
    pub object: String,
    pub id: String,
    #[serde(default)]
    pub title: Option<Vec<RichTextItem>>,
    #[serde(default)]
    pub properties: HashMap<String, PropertySchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionDatabase {
    pub id: String,
    #[serde(default)]
    pub properties: Option<HashMap<String, PropertySchema>>,
    #[serde(default)]
    pub data_sources: Option<Vec<DataSourceRef>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceRef {
    pub id: String,
    #[serde(rename = "type")]
    pub ds_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySchema {
    pub id: String,
    #[serde(rename = "type")]
    pub prop_type: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub select: Option<SelectConfig>,
    #[serde(default)]
    pub status: Option<StatusConfig>,
    #[serde(default)]
    pub multi_select: Option<SelectConfig>,
    #[serde(default)]
    pub relation: Option<RelationConfig>,
    #[serde(default)]
    pub formula: Option<FormulaConfig>,
    #[serde(default)]
    pub rollup: Option<RollupConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectConfig {
    #[serde(default)]
    pub options: Option<Vec<SelectOption>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusConfig {
    #[serde(default)]
    pub options: Option<Vec<SelectOption>>,
    #[serde(default)]
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
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationConfig {
    pub database_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaConfig {
    #[serde(default)]
    pub expression: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollupConfig {
    #[serde(default)]
    pub function: Option<String>,
    #[serde(default)]
    pub relation_property_name: Option<String>,
    #[serde(default)]
    pub rollup_property_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichTextItem {
    #[serde(rename = "type")]
    pub rt_type: String,
    #[serde(default)]
    pub text: Option<TextContent>,
    #[serde(default)]
    pub annotations: Option<Annotations>,
    #[serde(default)]
    pub plain_text: Option<String>,
    #[serde(default)]
    pub href: Option<String>,
    #[serde(default)]
    pub equation: Option<EquationContent>,
    #[serde(default)]
    pub mention: Option<MentionContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    pub content: String,
    #[serde(default)]
    pub link: Option<LinkInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkInfo {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotations {
    #[serde(default)]
    pub bold: Option<bool>,
    #[serde(default)]
    pub italic: Option<bool>,
    #[serde(default)]
    pub strikethrough: Option<bool>,
    #[serde(default)]
    pub underline: Option<bool>,
    #[serde(default)]
    pub code: Option<bool>,
    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquationContent {
    pub expression: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentionContent {
    #[serde(rename = "type")]
    pub mention_type: String,
    #[serde(default)]
    pub user: Option<Value>,
    #[serde(default)]
    pub page: Option<Value>,
    #[serde(default)]
    pub database: Option<Value>,
    #[serde(default)]
    pub date: Option<DateValue>,
    #[serde(default)]
    pub template_mention: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateValue {
    pub start: String,
    #[serde(default)]
    pub end: Option<String>,
    #[serde(default)]
    pub time_zone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaValue {
    #[serde(rename = "type")]
    pub formula_type: String,
    #[serde(default)]
    pub string: Option<String>,
    #[serde(default)]
    pub number: Option<f64>,
    #[serde(default)]
    pub boolean: Option<bool>,
    #[serde(default)]
    pub date: Option<DateValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationItem {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileItem {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(rename = "type")]
    #[serde(default)]
    pub file_type: Option<String>,
    #[serde(default)]
    pub external: Option<ExternalFile>,
    #[serde(default)]
    pub file: Option<FileUrl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalFile {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUrl {
    pub url: String,
    #[serde(default)]
    pub expiry_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserItem {
    pub id: String,
    #[serde(default)]
    pub object: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniqueIdValue {
    #[serde(default)]
    pub prefix: Option<String>,
    pub number: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollupValue {
    #[serde(rename = "type")]
    pub rollup_type: String,
    #[serde(default)]
    pub number: Option<f64>,
    #[serde(default)]
    pub string: Option<String>,
    #[serde(default)]
    pub date: Option<DateValue>,
    #[serde(default)]
    pub array: Option<Vec<PropertyValue>>,
    #[serde(default)]
    pub function: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionBlock {
    #[serde(default)]
    pub object: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub block_type: String,
    #[serde(default)]
    pub paragraph: Option<BlockContent>,
    #[serde(default)]
    pub heading_1: Option<BlockContent>,
    #[serde(default)]
    pub heading_2: Option<BlockContent>,
    #[serde(default)]
    pub heading_3: Option<BlockContent>,
    #[serde(default)]
    pub bulleted_list_item: Option<BlockContent>,
    #[serde(default)]
    pub numbered_list_item: Option<BlockContent>,
    #[serde(default)]
    pub to_do: Option<ToDoContent>,
    #[serde(default)]
    pub code: Option<CodeContent>,
    #[serde(default)]
    pub quote: Option<BlockContent>,
    #[serde(default)]
    pub callout: Option<CalloutContent>,
    #[serde(default)]
    pub divider: Option<Value>,
    #[serde(default)]
    pub child_page: Option<ChildPageContent>,
    #[serde(default)]
    pub child_database: Option<ChildDatabaseContent>,
    #[serde(default)]
    pub image: Option<ImageContent>,
    #[serde(default)]
    pub video: Option<VideoContent>,
    #[serde(default)]
    pub bookmark: Option<BookmarkContent>,
    #[serde(default)]
    pub table: Option<TableContent>,
    #[serde(default)]
    pub table_row: Option<TableRowContent>,
    #[serde(default)]
    pub equation: Option<EquationBlockContent>,
    #[serde(default)]
    pub column_list: Option<Value>,
    #[serde(default)]
    pub column: Option<Value>,
    #[serde(default)]
    pub breadcrumb: Option<Value>,
    #[serde(default)]
    pub link_preview: Option<LinkPreviewContent>,
    #[serde(default)]
    pub template: Option<BlockContent>,
    #[serde(default)]
    pub synced_block: Option<SyncedBlockContent>,
    #[serde(default)]
    pub embed: Option<ImageContent>,
    #[serde(default)]
    pub pdf: Option<ImageContent>,
    #[serde(default)]
    pub file: Option<FileBlockContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockContent {
    #[serde(default)]
    pub rich_text: Option<Vec<RichTextItem>>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub children: Option<Vec<NotionBlock>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToDoContent {
    #[serde(default)]
    pub rich_text: Option<Vec<RichTextItem>>,
    #[serde(default)]
    pub checked: Option<bool>,
    #[serde(default)]
    pub children: Option<Vec<NotionBlock>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeContent {
    #[serde(default)]
    pub rich_text: Option<Vec<RichTextItem>>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub caption: Option<Vec<RichTextItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalloutContent {
    #[serde(default)]
    pub rich_text: Option<Vec<RichTextItem>>,
    #[serde(default)]
    pub icon: Option<Value>,
    #[serde(default)]
    pub children: Option<Vec<NotionBlock>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildPageContent {
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildDatabaseContent {
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageContent {
    #[serde(default)]
    pub image: Option<FileUrl>,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub external: Option<ExternalFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoContent {
    #[serde(default)]
    pub video: Option<FileUrl>,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub external: Option<ExternalFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkContent {
    pub url: String,
    #[serde(default)]
    pub caption: Option<Vec<RichTextItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableContent {
    #[serde(default)]
    pub table_width: Option<i64>,
    #[serde(default)]
    pub has_column_header: Option<bool>,
    #[serde(default)]
    pub has_row_header: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRowContent {
    #[serde(default)]
    pub cells: Option<Vec<Vec<RichTextItem>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquationBlockContent {
    pub expression: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkPreviewContent {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedBlockContent {
    #[serde(default)]
    pub synced_from: Option<SyncedFromInfo>,
    #[serde(default)]
    pub children: Option<Vec<NotionBlock>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedFromInfo {
    #[serde(default)]
    pub block_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileBlockContent {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub type_field: Option<String>,
    #[serde(default)]
    pub external: Option<ExternalFile>,
    #[serde(default)]
    pub file: Option<FileUrl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockListResponse {
    pub object: String,
    pub results: Vec<NotionBlock>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePageBody {
    pub parent: CreatePageParent,
    pub properties: HashMap<String, Value>,
    #[serde(default)]
    pub children: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePageParent {
    #[serde(rename = "type")]
    pub parent_type: String,
    #[serde(default)]
    pub page_id: Option<String>,
    #[serde(default)]
    pub database_id: Option<String>,
    #[serde(default)]
    pub data_source_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePageBody {
    #[serde(default)]
    pub properties: Option<HashMap<String, Value>>,
    #[serde(default)]
    pub archived: Option<bool>,
    #[serde(default)]
    pub icon: Option<Value>,
    #[serde(default)]
    pub cover: Option<Value>,
}