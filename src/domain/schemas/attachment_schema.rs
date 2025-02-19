use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UploadAttachmentSchema {
    #[schema(format = "binary")]
    pub file: String,
}
#[derive(Serialize, ToSchema)]
pub struct FileDownloadResponse {
    #[schema(format = "binary")]
    file: Vec<u8>,
}

#[derive(Serialize, ToSchema)]
pub struct DeleteAttachmentResponse {
    status: String,
    message: String,
}
