use crate::models::{DeleteFileRequest, DownloadFileRequest, GetListFileAndFolderRequest};
use axum::{
    Json,
    extract::Multipart,
    http::{StatusCode, header},
    response::Response,
};

#[utoipa::path(
    post,
    path = "/api/ls",
    request_body(content = crate::models::GetListFileAndFolderRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "List of files and folders", body = Vec<String>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_list_file_and_folder(
    Json(req): Json<GetListFileAndFolderRequest>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let root = req.path.clone().unwrap_or_else(|| ".".to_string());
    let mut entries = tokio::fs::read_dir(&root)
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

        if path.is_dir() {
            items.push(format!("[DIR] {}", name));
        } else {
            items.push(format!("[FILE] {}", name));
        }
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

        let file_path = format!("{}/{}", folder_path, file_name);

        // Create the folder if it doesn't exist (does nothing if it already exists)
        tokio::fs::create_dir_all(&folder_path)
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
        (status = 404, description = "File not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn download_file(Json(req): Json<DownloadFileRequest>) -> Result<Response, StatusCode> {
    let file_path = &req.file_path;

    let contents = tokio::fs::read(&file_path).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", req.file_path),
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
        (status = 404, description = "File not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_file(Json(req): Json<DeleteFileRequest>) -> Result<Json<String>, StatusCode> {
    tokio::fs::remove_file(&req.file_path).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok(Json("File deleted successfully".to_string()))
}
