use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CreateNoteSchema {
    pub title: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<bool>,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct UpdateNoteSchema {
    pub title: Option<String>,
    pub content: Option<String>,
    pub category: Option<String>,
    pub published: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Deserialize, Default, ToSchema)]
pub struct FilterOptions {
    pub limit: Option<usize>,
    pub page: Option<usize>,
    #[serde(default)]
    pub tags: Vec<String>,
}
