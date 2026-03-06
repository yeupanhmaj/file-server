use crate::models::{CreateFolderRequest, RenameFolderRequest};
use crate::utils::validate_and_resolve_path;
use axum::{http::StatusCode, Json};

#[utoipa::path(
    post,
    path = "/api/mkdir",
    request_body = CreateFolderRequest,
    responses(
        (status = 200, description = "Folder created successfully", body = String),
        (status = 403, description = "Forbidden - path outside allowed directory"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_folder(
    Json(req): Json<CreateFolderRequest>,
) -> Result<Json<String>, StatusCode> {
    let folder_path = format!("{}/{}", req.path, req.folder_name);
    let safe_path = validate_and_resolve_path(&folder_path)?;

    tokio::fs::create_dir(&safe_path)
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
        (status = 403, description = "Forbidden - path outside allowed directory"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn rename_folder(
    Json(req): Json<RenameFolderRequest>,
) -> Result<Json<String>, StatusCode> {
    let old_path = validate_and_resolve_path(&req.folder_name)?;
    let new_path = validate_and_resolve_path(&req.new_folder_name)?;

    tokio::fs::rename(&old_path, &new_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json("Success".to_string()))
}
