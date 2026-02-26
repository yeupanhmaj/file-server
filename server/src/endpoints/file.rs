use crate::models::{
    DeleteFileRequest, DownloadFileRequest, FileSystemItem, GetFolderByIdRequest,
    GetListFileAndFolderRequest, RestoreFileRequest, TrashItem,
};
use crate::utils::{
    generate_id_from_path, get_base_directory_canonical, get_trash_directory, resolve_id_to_path,
    validate_and_resolve_path,
};
use axum::{
    extract::Multipart,
    http::{header, StatusCode},
    response::Response,
    Json,
};
use std::time::SystemTime;

pub fn format_file_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.1} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

pub fn format_system_time(time: SystemTime) -> String {
    match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs();
            let datetime =
                chrono::DateTime::from_timestamp(secs as i64, 0).unwrap_or_else(chrono::Utc::now);
            datetime.format("%Y-%m-%d").to_string()
        }
        Err(_) => "Unknown".to_string(),
    }
}

#[utoipa::path(
    post,
    path = "/api/ls",
    request_body(content = crate::models::GetListFileAndFolderRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "List of files and folders", body = Vec<FileSystemItem>),
        (status = 403, description = "Forbidden - path outside allowed directory"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_list_file_and_folder(
    Json(req): Json<GetListFileAndFolderRequest>,
) -> Result<Json<Vec<FileSystemItem>>, StatusCode> {
    let requested_path = req.path.as_deref().unwrap_or(".");
    let safe_path = validate_and_resolve_path(requested_path)?;
    let base_dir = get_base_directory_canonical()?;

    let mut entries = tokio::fs::read_dir(&safe_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut items = Vec::new();

    // Calculate parent ID (convert to relative path)
    let current_path_relative = safe_path
        .strip_prefix(&base_dir)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());
    let parent_id = if current_path_relative.is_empty() || current_path_relative == "." {
        None
    } else {
        Some(generate_id_from_path(&current_path_relative))
    };

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

        let metadata = tokio::fs::metadata(&path)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let item_type = if path.is_dir() { "folder" } else { "file" };

        let size = if path.is_dir() {
            "-".to_string()
        } else {
            format_file_size(metadata.len())
        };

        let modified = metadata
            .modified()
            .ok()
            .map(format_system_time)
            .unwrap_or_else(|| "Unknown".to_string());

        // Generate relative path from base directory
        let item_path_relative = path
            .strip_prefix(&base_dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.to_string_lossy().to_string());

        let id = generate_id_from_path(&item_path_relative);

        items.push(FileSystemItem {
            id,
            name,
            item_type: item_type.to_string(),
            modified,
            size,
            parent_id: parent_id.clone(),
            path: item_path_relative,
        });
    }

    Ok(Json(items))
}

#[utoipa::path(
    post,
    path = "/api/folder",
    request_body = GetFolderByIdRequest,
    responses(
        (status = 200, description = "List of files and folders in the folder", body = Vec<FileSystemItem>),
        (status = 404, description = "Folder not found"),
        (status = 403, description = "Forbidden - path outside allowed directory"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_folder_by_id(
    Json(req): Json<GetFolderByIdRequest>,
) -> Result<Json<Vec<FileSystemItem>>, StatusCode> {
    // Resolve the ID to a path
    let folder_path = resolve_id_to_path(&req.folder_id).ok_or(StatusCode::NOT_FOUND)?;

    // Use the existing logic from get_list_file_and_folder
    let safe_path = validate_and_resolve_path(&folder_path)?;

    // Verify it's a directory
    if !safe_path.is_dir() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let base_dir = get_base_directory_canonical()?;
    let mut entries = tokio::fs::read_dir(&safe_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut items = Vec::new();

    // Calculate parent ID (convert to relative path)
    let current_path_relative = safe_path
        .strip_prefix(&base_dir)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());
    let parent_id = if current_path_relative.is_empty() || current_path_relative == "." {
        None
    } else {
        Some(generate_id_from_path(&current_path_relative))
    };

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

        let metadata = tokio::fs::metadata(&path)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let item_type = if path.is_dir() { "folder" } else { "file" };

        let size = if path.is_dir() {
            "-".to_string()
        } else {
            format_file_size(metadata.len())
        };

        let modified = metadata
            .modified()
            .ok()
            .map(format_system_time)
            .unwrap_or_else(|| "Unknown".to_string());

        // Generate relative path from base directory
        let item_path_relative = path
            .strip_prefix(&base_dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.to_string_lossy().to_string());

        let id = generate_id_from_path(&item_path_relative);

        items.push(FileSystemItem {
            id,
            name,
            item_type: item_type.to_string(),
            modified,
            size,
            parent_id: parent_id.clone(),
            path: item_path_relative,
        });
    }

    Ok(Json(items))
}

#[utoipa::path(
    post,
    path = "/api/upload",
    request_body(content = crate::models::UploadFileRequest, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "File uploaded successfully", body = String),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden - path outside allowed directory"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn upload_file(mut multipart: Multipart) -> Result<Json<String>, StatusCode> {
    let mut uploaded_files = Vec::new();
    let mut folder_path = String::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let field_name = field.name().unwrap_or("").to_string();

        // Handle the "path" field
        if field_name == "path" {
            folder_path = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            continue;
        }

        // Handle file fields
        let file_name = field
            .file_name()
            .ok_or(StatusCode::BAD_REQUEST)?
            .to_string();

        let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;

        // Validate the folder path
        let safe_folder_path = validate_and_resolve_path(&folder_path)?;
        let file_path = safe_folder_path.join(&file_name);

        // Double-check the final file path is still safe
        validate_and_resolve_path(&file_path.to_string_lossy())?;

        // Create the folder if it doesn't exist (does nothing if it already exists)
        tokio::fs::create_dir_all(&safe_folder_path)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        tokio::fs::write(&file_path, &data)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        uploaded_files.push(file_name);
    }

    if uploaded_files.is_empty() {
        Err(StatusCode::BAD_REQUEST)
    } else {
        Ok(Json(format!(
            "Uploaded {} file(s): {}",
            uploaded_files.len(),
            uploaded_files.join(", ")
        )))
    }
}

#[utoipa::path(
    post,
    path = "/api/download",
    request_body = DownloadFileRequest,
    responses(
        (status = 200, description = "File content", content_type = "application/octet-stream"),
        (status = 403, description = "Forbidden - path outside allowed directory"),
        (status = 404, description = "File not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn download_file(Json(req): Json<DownloadFileRequest>) -> Result<Response, StatusCode> {
    let safe_path = validate_and_resolve_path(&req.file_path)?;

    let contents = tokio::fs::read(&safe_path).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    // Extract just the filename for the download header
    let filename = safe_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download");

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(axum::body::Body::from(contents))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}

#[utoipa::path(
    post,
    path = "/api/delete",
    request_body = DeleteFileRequest,
    responses(
        (status = 200, description = "File moved to trash successfully", body = String),
        (status = 403, description = "Forbidden - path outside allowed directory"),
        (status = 404, description = "File not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_file(Json(req): Json<DeleteFileRequest>) -> Result<Json<String>, StatusCode> {
    let safe_path = validate_and_resolve_path(&req.file_path)?;

    // Check if file exists
    if !safe_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Get file metadata for trash info
    let metadata = tokio::fs::metadata(&safe_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create trash item ID (timestamp + filename)
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let filename = safe_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let trash_item_id = format!("{}_{}", timestamp, filename);

    // Create trash directory structure
    let trash_dir = get_trash_directory();
    let trash_files_dir = trash_dir.join("files");
    let trash_metadata_dir = trash_dir.join("metadata");

    tokio::fs::create_dir_all(&trash_files_dir)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    tokio::fs::create_dir_all(&trash_metadata_dir)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Move file to trash
    let trash_file_path = trash_files_dir.join(&trash_item_id);
    tokio::fs::rename(&safe_path, &trash_file_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Save metadata
    let trash_metadata = serde_json::json!({
        "id": trash_item_id,
        "original_path": req.file_path,
        "deleted_at": chrono::Utc::now().to_rfc3339(),
        "size": metadata.len(),
    });

    let metadata_path = trash_metadata_dir.join(format!("{}.json", trash_item_id));
    tokio::fs::write(&metadata_path, trash_metadata.to_string())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json("File moved to trash successfully".to_string()))
}

#[utoipa::path(
    get,
    path = "/api/trash",
    responses(
        (status = 200, description = "List of trash items", body = Vec<TrashItem>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_trash() -> Result<Json<Vec<TrashItem>>, StatusCode> {
    let trash_metadata_dir = get_trash_directory().join("metadata");

    if !trash_metadata_dir.exists() {
        return Ok(Json(vec![]));
    }

    let mut entries = tokio::fs::read_dir(&trash_metadata_dir)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut trash_items = Vec::new();

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let content = tokio::fs::read_to_string(&path)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&content) {
                let size = metadata["size"].as_u64().unwrap_or(0);
                trash_items.push(TrashItem {
                    id: metadata["id"].as_str().unwrap_or("").to_string(),
                    original_path: metadata["original_path"].as_str().unwrap_or("").to_string(),
                    name: metadata["original_path"]
                        .as_str()
                        .and_then(|p| std::path::Path::new(p).file_name())
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    deleted_at: metadata["deleted_at"].as_str().unwrap_or("").to_string(),
                    size: format_file_size(size),
                });
            }
        }
    }

    Ok(Json(trash_items))
}

#[utoipa::path(
    post,
    path = "/api/trash/restore",
    request_body = RestoreFileRequest,
    responses(
        (status = 200, description = "File restored successfully", body = String),
        (status = 404, description = "Trash item not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn restore_file(Json(req): Json<RestoreFileRequest>) -> Result<Json<String>, StatusCode> {
    let trash_dir = get_trash_directory();
    let trash_files_dir = trash_dir.join("files");
    let trash_metadata_dir = trash_dir.join("metadata");

    // Read metadata
    let metadata_path = trash_metadata_dir.join(format!("{}.json", req.trash_item_id));
    if !metadata_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    let content = tokio::fs::read_to_string(&metadata_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let metadata: serde_json::Value =
        serde_json::from_str(&content).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let original_path = metadata["original_path"]
        .as_str()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Validate and resolve the original path
    let restore_path = validate_and_resolve_path(original_path)?;

    // Create parent directory if it doesn't exist
    if let Some(parent) = restore_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Move file back from trash
    let trash_file_path = trash_files_dir.join(&req.trash_item_id);
    tokio::fs::rename(&trash_file_path, &restore_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Remove metadata
    tokio::fs::remove_file(&metadata_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json("File restored successfully".to_string()))
}

#[utoipa::path(
    post,
    path = "/api/trash/empty",
    responses(
        (status = 200, description = "Trash emptied successfully", body = String),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn empty_trash() -> Result<Json<String>, StatusCode> {
    let trash_dir = get_trash_directory();

    if trash_dir.exists() {
        tokio::fs::remove_dir_all(&trash_dir)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Recreate empty trash directory
        tokio::fs::create_dir_all(&trash_dir)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json("Trash emptied successfully".to_string()))
}
