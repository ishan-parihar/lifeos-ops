use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: String,
    #[serde(rename = "type")]
    pub block_type: String,
    pub created_time: String,
    pub last_edited_time: String,
    pub archived: bool,
    pub in_trash: bool,
    #[serde(default)]
    pub parent: Option<serde_json::Value>,
    #[serde(default)]
    pub children: Option<Vec<Block>>,
    pub has_children: bool,
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichText {
    pub plain_text: String,
    pub href: Option<String>,
    #[serde(default)]
    pub annotations: Annotations,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Annotations {
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub underline: bool,
    pub code: bool,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileObject {
    pub url: Option<String>,
    pub expiry_time: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Icon {
    #[serde(rename = "type")]
    pub icon_type: String,
    pub emoji: Option<String>,
    pub external: Option<FileObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cover {
    #[serde(rename = "type")]
    pub cover_type: String,
    pub external: Option<FileObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    #[serde(rename = "type")]
    pub page_type: String,
    pub created_time: String,
    pub last_edited_time: String,
    pub archived: bool,
    pub in_trash: bool,
    pub url: String,
    pub public_url: Option<String>,
    #[serde(default)]
    pub properties: serde_json::Value,
    pub parent: Option<serde_json::Value>,
    pub icon: Option<Icon>,
    pub cover: Option<Cover>,
    #[serde(default)]
    pub children: Option<Vec<Block>>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePageRequest {
    pub parent: serde_json::Value,
    pub properties: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePageRequest {
    pub properties: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_trash: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub id: String,
    pub parent: Option<serde_json::Value>,
    pub created_time: String,
    pub last_edited_time: String,
    pub archived: bool,
    pub in_trash: bool,
    pub title: Vec<RichText>,
    pub description: Vec<RichText>,
    pub is_single: bool,
    pub is_inline: bool,
    #[serde(default)]
    pub properties: serde_json::Value,
}

impl Block {
    pub fn rich_text(&self) -> Option<Vec<RichText>> {
        self.data.get(&self.block_type)?
            .get("rich_text")?
            .as_array()?
            .iter()
            .map(|v| serde_json::from_value(v.clone()).ok())
            .collect()
    }
}

impl RichText {
    pub fn plain_text(&self) -> String {
        self.plain_text.clone()
    }
}

pub type PageProperty = serde_json::Value;
pub type BlockObject = Block;