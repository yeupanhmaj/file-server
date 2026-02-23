use crate::models::{
    DeleteFileRequest, DownloadFileRequest, FileSystemItem, GetListFileAndFolderRequest,
};
use crate::utils::validate_and_resolve_path;
use axum::{
    extract::Multipart,
    http::{header, StatusCode},
    response::Response,
    Json,
};
use std::time::SystemTime;

fn format_file_size(size: u64) -> String {
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

fn format_system_time(time: SystemTime) -> String {
    match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs();
            let datetime = chrono::DateTime::from_timestamp(secs as i64, 0)
                .unwrap_or_else(|| chrono::Utc::now());
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

        items.push(FileSystemItem {
            name,
            item_type: item_type.to_string(),
            modified,
            size,
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
        (status = 200, description = "File deleted successfully", body = String),
        (status = 403, description = "Forbidden - path outside allowed directory"),
        (status = 404, description = "File not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_file(Json(req): Json<DeleteFileRequest>) -> Result<Json<String>, StatusCode> {
    let safe_path = validate_and_resolve_path(&req.file_path)?;

    tokio::fs::remove_file(&safe_path).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok(Json("File deleted successfully".to_string()))
}
