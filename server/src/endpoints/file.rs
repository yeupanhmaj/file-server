use crate::models::{
    ChunkedUploadResponse, CopyFileRequest, DeleteFileRequest, DownloadFileRequest, FileSystemItem,
    GetFolderByIdRequest, GetListFileAndFolderRequest, MoveFileRequest, RenameFileRequest,
    RestoreFileRequest, StorageStats, TrashItem,
};
use crate::utils::{
    generate_id_from_path, get_base_directory_canonical, get_storage_stats, get_trash_directory,
    resolve_id_to_path, validate_and_resolve_path,
};
use axum::{
    extract::Multipart,
    http::{header, HeaderMap, StatusCode},
    response::Response,
    Json,
};
use std::time::SystemTime;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

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

        // Skip .trash and .chunks directories
        if name == ".trash" || name == ".chunks" {
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

        // Validate the folder path
        let safe_folder_path = validate_and_resolve_path(&folder_path)?;
        let file_path = safe_folder_path.join(&file_name);

        // Double-check the final file path is still safe
        validate_and_resolve_path(&file_path.to_string_lossy())?;

        // Create the folder if it doesn't exist (does nothing if it already exists)
        tokio::fs::create_dir_all(&safe_folder_path)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Stream file data directly to disk (memory-efficient)
        let mut file = tokio::fs::File::create(&file_path)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        use tokio::io::AsyncWriteExt;
        let mut stream = field;
        while let Some(chunk) = stream.chunk().await.map_err(|_| StatusCode::BAD_REQUEST)? {
            file.write_all(&chunk)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

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
    path = "/api/upload-chunk",
    request_body(content = crate::models::ChunkedUploadRequest, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Chunk uploaded successfully", body = ChunkedUploadResponse),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden - path outside allowed directory"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn upload_chunk(
    mut multipart: Multipart,
) -> Result<Json<ChunkedUploadResponse>, StatusCode> {
    let mut folder_path = String::from("."); // Default to current directory
    let mut file_id = String::new();
    let mut chunk_index: usize = 0;
    let mut total_chunks: usize = 0;
    let mut filename = String::new();
    let mut chunk_data: Option<axum::body::Bytes> = None;

    // Parse multipart fields
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "path" => {
                folder_path = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            }
            "file_id" => {
                file_id = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            }
            "chunk_index" => {
                let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                chunk_index = text.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
            }
            "total_chunks" => {
                let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                total_chunks = text.parse().map_err(|_| StatusCode::BAD_REQUEST)?;
            }
            "filename" => {
                filename = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            }
            "chunk" => {
                // Read chunk bytes - this might be a large field
                let bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                chunk_data = Some(bytes);
            }
            _ => {}
        }
    }

    // Validate required fields
    if file_id.is_empty() || filename.is_empty() || chunk_data.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate the folder path
    let safe_folder_path = validate_and_resolve_path(&folder_path)?;

    // Create chunks directory in the base directory
    let base_dir = get_base_directory_canonical()?;
    let chunks_dir = base_dir.join(".chunks").join(&file_id);
    tokio::fs::create_dir_all(&chunks_dir)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Save the chunk
    let chunk_file = chunks_dir.join(format!("chunk_{}", chunk_index));
    tokio::fs::write(&chunk_file, chunk_data.unwrap())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Check if all chunks have been uploaded
    let mut received_chunks = 0;
    for i in 0..total_chunks {
        let chunk_path = chunks_dir.join(format!("chunk_{}", i));
        if chunk_path.exists() {
            received_chunks += 1;
        }
    }

    let completed = received_chunks == total_chunks;

    // If all chunks received, merge them
    if completed {
        // Create the target directory
        tokio::fs::create_dir_all(&safe_folder_path)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let final_file_path = safe_folder_path.join(&filename);

        // Validate the final file path
        validate_and_resolve_path(&final_file_path.to_string_lossy())?;

        // Merge chunks
        let mut final_file = tokio::fs::File::create(&final_file_path)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        for i in 0..total_chunks {
            let chunk_path = chunks_dir.join(format!("chunk_{}", i));
            let chunk_data = tokio::fs::read(&chunk_path)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            use tokio::io::AsyncWriteExt;
            final_file
                .write_all(&chunk_data)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        // Clean up chunks
        tokio::fs::remove_dir_all(&chunks_dir)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(ChunkedUploadResponse {
        message: if completed {
            format!("File '{}' uploaded successfully", filename)
        } else {
            format!("Chunk {}/{} uploaded", chunk_index + 1, total_chunks)
        },
        chunk_index,
        total_chunks,
        completed,
    }))
}

// Helper function to detect MIME type based on file extension
fn detect_mime_type(filename: &str) -> &'static str {
    let extension = std::path::Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    match extension.to_lowercase().as_str() {
        // Images
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        // Documents
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        // Videos
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "ogg" => "video/ogg",
        // Audio
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        // Archives
        "zip" => "application/zip",
        "rar" => "application/x-rar-compressed",
        "7z" => "application/x-7z-compressed",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",
        // Default
        _ => "application/octet-stream",
    }
}

/// Parse Range header and return (start, end) bytes
/// Format: "bytes=start-end" or "bytes=start-" or "bytes=-suffix"
fn parse_range_header(range_header: &str, file_size: u64) -> Option<(u64, u64)> {
    let range_header = range_header.trim();

    // Check if it starts with "bytes="
    if !range_header.starts_with("bytes=") {
        return None;
    }

    let range_str = &range_header[6..]; // Skip "bytes="

    // Split by comma (take only first range for simplicity)
    let first_range = range_str.split(',').next()?.trim();

    if let Some((start_str, end_str)) = first_range.split_once('-') {
        let start_str = start_str.trim();
        let end_str = end_str.trim();

        if start_str.is_empty() && !end_str.is_empty() {
            // Suffix range: "-500" means last 500 bytes
            if let Ok(suffix_length) = end_str.parse::<u64>() {
                let start = file_size.saturating_sub(suffix_length);
                return Some((start, file_size - 1));
            }
        } else if !start_str.is_empty() {
            // Normal range or open-ended range
            if let Ok(start) = start_str.parse::<u64>() {
                if start >= file_size {
                    return None; // Invalid range
                }

                let end = if end_str.is_empty() {
                    file_size - 1 // Open-ended: "500-"
                } else {
                    end_str
                        .parse::<u64>()
                        .unwrap_or(file_size - 1)
                        .min(file_size - 1)
                };

                if start <= end {
                    return Some((start, end));
                }
            }
        }
    }

    None
}

#[utoipa::path(
    post,
    path = "/api/download",
    request_body = DownloadFileRequest,
    responses(
        (status = 200, description = "File content", content_type = "application/octet-stream"),
        (status = 206, description = "Partial content", content_type = "application/octet-stream"),
        (status = 403, description = "Forbidden - path outside allowed directory"),
        (status = 404, description = "File not found"),
        (status = 416, description = "Range not satisfiable"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn download_file(
    headers: HeaderMap,
    Json(req): Json<DownloadFileRequest>,
) -> Result<Response, StatusCode> {
    let safe_path = validate_and_resolve_path(&req.file_path)?;

    // Open file for streaming (doesn't load into memory)
    let file = tokio::fs::File::open(&safe_path).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    // Get file size for Content-Length header
    let metadata = file
        .metadata()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let file_size = metadata.len();

    // Extract just the filename for the download header
    let filename = safe_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download");

    // Detect MIME type based on file extension
    let mime_type = detect_mime_type(filename);

    // Check for Range header
    let range_header = headers.get(header::RANGE).and_then(|h| h.to_str().ok());

    // Build response based on whether Range header is present
    if let Some(range_str) = range_header {
        // Parse range
        if let Some((start, end)) = parse_range_header(range_str, file_size) {
            let content_length = end - start + 1;

            // Seek to start position
            let mut file = file;
            file.seek(std::io::SeekFrom::Start(start))
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Create limited reader for the range
            let limited_file = file.take(content_length);
            let stream = ReaderStream::new(limited_file);
            let body = axum::body::Body::from_stream(stream);

            let response = Response::builder()
                .status(StatusCode::PARTIAL_CONTENT)
                .header(header::CONTENT_TYPE, mime_type)
                .header(header::CONTENT_LENGTH, content_length.to_string())
                .header(
                    header::CONTENT_RANGE,
                    format!("bytes {}-{}/{}", start, end, file_size),
                )
                .header(header::ACCEPT_RANGES, "bytes")
                .header(
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", filename),
                )
                .body(body)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            return Ok(response);
        } else {
            // Invalid range request
            let response = Response::builder()
                .status(StatusCode::RANGE_NOT_SATISFIABLE)
                .header(header::CONTENT_RANGE, format!("bytes */{}", file_size))
                .body(axum::body::Body::empty())
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            return Ok(response);
        }
    }

    // No Range header - return full file
    let stream = ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type)
        .header(header::CONTENT_LENGTH, file_size.to_string())
        .header(header::ACCEPT_RANGES, "bytes")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(body)
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

#[utoipa::path(
    get,
    path = "/api/storage",
    responses(
        (status = 200, description = "Storage statistics", body = StorageStats),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_storage_stats_endpoint() -> Result<Json<StorageStats>, StatusCode> {
    let (used_bytes, total_bytes) = get_storage_stats()?;

    let used_formatted = format_file_size(used_bytes);
    let total_formatted = format_file_size(total_bytes);
    let percentage = if total_bytes > 0 {
        (used_bytes as f64 / total_bytes as f64) * 100.0
    } else {
        0.0
    };

    Ok(Json(StorageStats {
        used_bytes,
        total_bytes,
        used_formatted,
        total_formatted,
        percentage,
    }))
}

#[utoipa::path(
    post,
    path = "/api/search",
    request_body = crate::models::SearchRequest,
    responses(
        (status = 200, description = "File search results", body = crate::models::SearchResponse),
        (status = 403, description = "Forbidden - path outside allowed directory"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn search_files(
    Json(req): Json<crate::models::SearchRequest>,
) -> Result<Json<crate::models::SearchResponse>, StatusCode> {
    let safe_path = validate_and_resolve_path(&req.path)?;
    let base_dir = get_base_directory_canonical()?;
    let search_term = req.search_string.to_lowercase();

    let page = req.page.max(1); // Ensure page is at least 1
    let limit = req.limit.max(1); // Ensure limit is at least 1
    let skip = (page - 1) * limit;
    let max_items = skip + limit + 1; // Collect one extra to check if there are more

    let mut all_results = Vec::new();

    // Recursive search function with early termination
    fn search_recursive(
        path: std::path::PathBuf,
        base_dir: &std::path::Path,
        search_term: &str,
        results: &mut Vec<FileSystemItem>,
        max_items: usize,
    ) -> std::io::Result<bool> {
        // Return true if we should stop searching
        if results.len() >= max_items {
            return Ok(true); // Stop recursion
        }

        if !path.is_dir() {
            return Ok(false);
        }

        let entries = std::fs::read_dir(&path)?;

        for entry in entries.flatten() {
            if results.len() >= max_items {
                return Ok(true); // Stop recursion
            }

            let entry_path = entry.path();
            let name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            // Skip .trash directory
            if name == ".trash" {
                continue;
            }

            // Case-insensitive substring match
            if name.to_lowercase().contains(search_term) {
                // Get metadata
                if let Ok(metadata) = entry_path.metadata() {
                    let item_type = if entry_path.is_dir() {
                        "folder"
                    } else {
                        "file"
                    };

                    let size = if entry_path.is_dir() {
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
                    let item_path_relative = entry_path
                        .strip_prefix(base_dir)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| entry_path.to_string_lossy().to_string());

                    let id = generate_id_from_path(&item_path_relative);

                    // Get parent path for parent_id
                    let parent_path = entry_path
                        .parent()
                        .and_then(|p| p.strip_prefix(base_dir).ok())
                        .map(|p| p.to_string_lossy().to_string());

                    let parent_id = parent_path
                        .filter(|p| !p.is_empty() && p != ".")
                        .map(|p| generate_id_from_path(&p));

                    results.push(FileSystemItem {
                        id,
                        name: name.to_string(),
                        item_type: item_type.to_string(),
                        modified,
                        size,
                        parent_id,
                        path: item_path_relative,
                    });

                    if results.len() >= max_items {
                        return Ok(true); // Stop recursion
                    }
                }
            }

            // Recurse into subdirectories
            if entry_path.is_dir() {
                let should_stop =
                    search_recursive(entry_path, base_dir, search_term, results, max_items)?;
                if should_stop {
                    return Ok(true); // Propagate stop signal
                }
            }
        }

        Ok(false)
    }

    // Start recursive search
    search_recursive(
        safe_path,
        &base_dir,
        &search_term,
        &mut all_results,
        max_items,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Calculate pagination metadata
    let total_found = all_results.len();
    let has_more = total_found > skip + limit;

    // Slice results for the requested page
    let end_idx = (skip + limit).min(total_found);
    let page_results: Vec<FileSystemItem> = if skip < total_found {
        all_results[skip..end_idx].to_vec()
    } else {
        Vec::new()
    };

    let response = crate::models::SearchResponse {
        results: page_results,
        total: total_found.min(skip + limit), // Don't report more than we've searched
        page,
        limit,
        has_more,
    };

    Ok(Json(response))
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

#[utoipa::path(
    post,
    path = "/api/rename-file",
    request_body = RenameFileRequest,
    responses(
        (status = 200, description = "File renamed successfully", body = String),
        (status = 403, description = "Forbidden - path outside allowed directory"),
        (status = 404, description = "File not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn rename_file(Json(req): Json<RenameFileRequest>) -> Result<Json<String>, StatusCode> {
    let safe_path = validate_and_resolve_path(&req.file_path)?;

    // Check if file exists
    if !safe_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Get parent directory
    let parent = safe_path
        .parent()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create new path with new name
    let new_path = parent.join(&req.new_name);

    // Validate the new path is still safe
    validate_and_resolve_path(&new_path.to_string_lossy())?;

    // Check if destination already exists
    if new_path.exists() {
        return Err(StatusCode::CONFLICT);
    }

    // Rename the file
    tokio::fs::rename(&safe_path, &new_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json("File renamed successfully".to_string()))
}

#[utoipa::path(
    post,
    path = "/api/move-file",
    request_body = MoveFileRequest,
    responses(
        (status = 200, description = "File moved successfully", body = String),
        (status = 403, description = "Forbidden - path outside allowed directory"),
        (status = 404, description = "File or destination not found"),
        (status = 409, description = "File already exists at destination"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn move_file(Json(req): Json<MoveFileRequest>) -> Result<Json<String>, StatusCode> {
    let safe_source = validate_and_resolve_path(&req.file_path)?;
    let safe_dest_dir = validate_and_resolve_path(&req.destination)?;

    // Check if source exists
    if !safe_source.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Check if destination directory exists
    if !safe_dest_dir.exists() || !safe_dest_dir.is_dir() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Get filename from source
    let filename = safe_source
        .file_name()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create destination path
    let destination = safe_dest_dir.join(filename);

    // Check if file already exists at destination
    if destination.exists() {
        return Err(StatusCode::CONFLICT);
    }

    // Move the file
    tokio::fs::rename(&safe_source, &destination)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json("File moved successfully".to_string()))
}

#[utoipa::path(
    post,
    path = "/api/copy-file",
    request_body = CopyFileRequest,
    responses(
        (status = 200, description = "File copied successfully", body = String),
        (status = 403, description = "Forbidden - path outside allowed directory"),
        (status = 404, description = "File or destination not found"),
        (status = 409, description = "File already exists at destination"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn copy_file(Json(req): Json<CopyFileRequest>) -> Result<Json<String>, StatusCode> {
    let safe_source = validate_and_resolve_path(&req.file_path)?;
    let safe_dest_dir = validate_and_resolve_path(&req.destination)?;

    // Check if source exists
    if !safe_source.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Check if destination directory exists
    if !safe_dest_dir.exists() || !safe_dest_dir.is_dir() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Get filename from source
    let filename = safe_source
        .file_name()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create destination path
    let destination = safe_dest_dir.join(filename);

    // Check if file already exists at destination
    if destination.exists() {
        return Err(StatusCode::CONFLICT);
    }

    // Copy the file (works for both files and directories)
    if safe_source.is_dir() {
        copy_dir_recursive(&safe_source, &destination).await?;
    } else {
        tokio::fs::copy(&safe_source, &destination)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json("File copied successfully".to_string()))
}

// Helper function to recursively copy directories
fn copy_dir_recursive<'a>(
    src: &'a std::path::Path,
    dst: &'a std::path::Path,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), StatusCode>> + Send + 'a>> {
    Box::pin(async move {
        // Create destination directory
        tokio::fs::create_dir_all(dst)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Read source directory
        let mut entries = tokio::fs::read_dir(src)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        {
            let src_path = entry.path();
            let filename = entry.file_name();
            let dst_path = dst.join(filename);

            if src_path.is_dir() {
                // Recursively copy subdirectory
                copy_dir_recursive(&src_path, &dst_path).await?;
            } else {
                // Copy file
                tokio::fs::copy(&src_path, &dst_path)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            }
        }

        Ok(())
    })
}
