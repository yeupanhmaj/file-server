use crate::models::{CreateFolderRequest, RenameFolderRequest, SearchRequest};
use axum::{Json, http::StatusCode};

#[utoipa::path(
    post,
    path = "/api/mkdir",
    request_body = CreateFolderRequest,
    responses(
        (status = 200, description = "Folder created successfully", body = String),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_folder(
    Json(req): Json<CreateFolderRequest>,
) -> Result<Json<String>, StatusCode> {
    tokio::fs::create_dir(format!("{}/{}", req.path, req.folder_name))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json("Success".to_string()))
}

#[utoipa::path(
    post,
    path = "/api/rename-folder",
    request_body = RenameFolderRequest,
    responses(
        (status = 200, description = "Folder renamed successfully", body = String),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn rename_folder(
    Json(req): Json<RenameFolderRequest>,
) -> Result<Json<String>, StatusCode> {
    let old_path = &req.folder_name;
    let new_path = &req.new_folder_name;

    tokio::fs::rename(&old_path, &new_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json("Success".to_string()))
}

#[utoipa::path(
    post,
    path = "/api/search",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "File search results", body = String),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn search_files(Json(_req): Json<SearchRequest>) -> Result<Json<String>, StatusCode> {
    // Implement the search functionality here
    Ok(Json("Search results".to_string()))
}
