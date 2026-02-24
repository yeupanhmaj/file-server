use crate::models::{CreateFolderRequest, RenameFolderRequest, SearchRequest};
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

#[utoipa::path(
    post,
    path = "/api/search",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "File search results", body = Vec<String>),
        (status = 403, description = "Forbidden - path outside allowed directory"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn search_files(Json(req): Json<SearchRequest>) -> Result<Json<Vec<String>>, StatusCode> {
    let safe_path = validate_and_resolve_path(&req.path)?;

    let mut entries = tokio::fs::read_dir(&safe_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut results = Vec::new();

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        let entry_path = entry.path();
        let name = entry_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Skip .trash directory
        if name == ".trash" {
            continue;
        }

        if entry_path.to_str() == Some(&req.search_string) {
            let display_path = entry_path.display().to_string();

            if entry_path.is_dir() {
                results.push(format!("[DIR] {}", display_path));
            } else {
                results.push(format!("[FILE] {}", display_path));
            }
        }
    }

    Ok(Json(results))
}

#[utoipa::path(
    post,
    path = "/api/ls",
    request_body(content = crate::models::GetListFileAndFolderRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "List of files and folders", body = Vec<String>),
        (status = 403, description = "Forbidden - path outside allowed directory"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn sorted_list_file_and_folder(
    Json(req): Json<crate::models::SortOptionRequest>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let requested_path = req.option.as_deref().unwrap_or(".");
    let safe_path = validate_and_resolve_path(requested_path)?;

    let mut entries = tokio::fs::read_dir(&safe_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut items = Vec::new();

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip .trash directory
        if name == ".trash" {
            continue;
        }

        if path.is_dir() {
            items.push(format!("[DIR] {}", name));
        } else {
            items.push(format!("[FILE] {}", name));
        }
    }

    // Sort items alphabetically
    items.sort();

    Ok(Json(items))
}
