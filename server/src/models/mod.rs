use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct GetListFileAndFolderRequest {
    // root folder to search, optional, if not passed, use "."
    pub path: Option<String>,
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
    pub query: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct SortOptionRequest {
    pub option: Option<String>,
}
