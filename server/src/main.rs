use axum::{
    Json, Router,
    extract::Multipart,
    http::{StatusCode, header},
    response::Response,
    routing::{get, post},
};

use tower_http::cors::{Any, CorsLayer};

use serde::{Deserialize, Serialize};

use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(get_list_file_and_folder, create_folder, upload_file, download_file),
    tags(
        (name = "file-server", description = "File server API endpoints")
    )
)]
struct ApiDoc;

/// Schema for file upload multipart form
#[derive(Deserialize, ToSchema)]
#[allow(unused)]
struct UploadFileRequest {
    /// Target folder path where files will be uploaded
    path: String,
    /// File(s) to upload
    #[schema(format = Binary, content_media_type = "application/octet-stream")]
    file: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
struct CreateFolderRequest {
    path: String,
    folder_name: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
struct DownloadFileRequest {
    file_path: String,
}

#[tokio::main]
async fn main() {
    // 1. Define your CORS policy
    let cors = CorsLayer::new()
        .allow_origin(Any) // For debugging. In production, use "http://example.com".parse().unwrap()
        .allow_methods(Any)
        .allow_headers(Any);
    let app = route_builder().layer(cors);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Server running on http://localhost:3000");
    println!("API docs available at http://localhost:3000/swagger-ui/");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C handler");
}

fn route_builder() -> Router {
    Router::new()
        .route("/api/ls", get(get_list_file_and_folder))
        .route("/api/mkdir", post(create_folder))
        .route("/api/upload", post(upload_file))
        .route("/api/download", post(download_file))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}

#[utoipa::path(
    get,
    path = "/api/ls",
    responses(
        (status = 200, description = "List of files and folders", body = Vec<String>),
        (status = 500, description = "Internal server error")
    )
)]
async fn get_list_file_and_folder() -> Result<Json<Vec<String>>, StatusCode> {
    let mut entries = tokio::fs::read_dir(".")
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
    path = "/api/mkdir",
    request_body = CreateFolderRequest,
    responses(
        (status = 200, description = "Folder created successfully", body = String),
        (status = 500, description = "Internal server error")
    )
)]
async fn create_folder(Json(req): Json<CreateFolderRequest>) -> Result<Json<String>, StatusCode> {
    tokio::fs::create_dir(format!("{}/{}", req.path, req.folder_name))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json("Success".to_string()))
}

#[utoipa::path(
    post,
    path = "/api/upload",
    request_body(content = UploadFileRequest, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "File uploaded successfully", body = String),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
async fn upload_file(mut multipart: Multipart) -> Result<Json<String>, StatusCode> {
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
async fn download_file(Json(req): Json<DownloadFileRequest>) -> Result<Response, StatusCode> {
    let file_path = format!("{}", req.file_path);

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

async fn delete() {}
