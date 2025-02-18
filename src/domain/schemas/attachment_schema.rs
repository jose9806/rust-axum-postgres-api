use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct UploadAttachmentSchema {
    pub original_filename: Option<String>,
}
