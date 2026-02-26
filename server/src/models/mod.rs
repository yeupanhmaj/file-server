use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct GetListFileAndFolderRequest {
    // root folder to search, optional, if not passed, use "."
    pub path: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct GetFolderByIdRequest {
    pub folder_id: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct FileSystemItem {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub modified: String,
    pub size: String,
    pub parent_id: Option<String>,
    pub path: String,
}

/// Schema for file upload multipart form
#[derive(Deserialize, ToSchema)]
#[allow(unused)]
pub struct UploadFileRequest {
    /// Target folder path where files will be uploaded
    pub path: Option<String>,
    /// File(s) to upload
    #[schema(format = Binary, content_media_type = "application/octet-stream")]
    pub file: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateFolderRequest {
    pub path: String,
    pub folder_name: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct DownloadFileRequest {
    pub file_path: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct DeleteFileRequest {
    pub file_path: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct RenameFolderRequest {
    pub folder_name: String,
    pub new_folder_name: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct SearchRequest {
    pub search_string: String,
    pub path: String,
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_page() -> usize {
    1
}

fn default_limit() -> usize {
    5
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SearchResponse {
    pub results: Vec<FileSystemItem>,
    pub total: usize,
    pub page: usize,
    pub limit: usize,
    pub has_more: bool,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct SortOptionRequest {
    pub option: Option<String>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct RestoreFileRequest {
    pub trash_item_id: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct TrashItem {
    pub id: String,
    pub original_path: String,
    pub name: String,
    pub deleted_at: String,
    pub size: String,
}
