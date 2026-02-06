use axum::{
    Json, Router,
    extract::{Multipart, Path},
    http::{StatusCode, header},
    response::Response,
    routing::{get, post},
};

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(get_list_file_and_folder, create_folder, upload_file, download_file),
    tags(
        (name = "file-server", description = "File server API endpoints")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    let app = route_builder();

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
        .route("/api/mkdir/:path/:folder_name", post(create_folder))
        .route("/api/upload/:path", post(upload_file))
        .route("/api/download/:path/:file_name", get(download_file))
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
    path = "/api/mkdir/{path}/{folder_name}",
    params(
        ("path" = String, Path, description = "Base path"),
        ("folder_name" = String, Path, description = "Folder name to create")
    ),
    responses(
        (status = 200, description = "Folder created successfully", body = String),
        (status = 500, description = "Internal server error")
    )
)]
async fn create_folder(
    Path((path, folder_name)): Path<(String, String)>,
) -> Result<Json<String>, StatusCode> {
    tokio::fs::create_dir(format!("{}/{}", path, folder_name))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json("Success".to_string()))
}

#[utoipa::path(
    post,
    path = "/api/upload/{path}",
    params(
        ("path" = String, Path, description = "Target folder path")
    ),
    request_body(content = String, description = "File to upload", content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "File uploaded successfully", body = String),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
async fn upload_file(
    Path(path): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<String>, StatusCode> {
    let mut uploaded_files = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let file_name = field
            .file_name()
            .ok_or(StatusCode::BAD_REQUEST)?
            .to_string();

        let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;

        let file_path = format!("{}/{}", path, file_name);
        tokio::fs::write(&file_path, &data)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        uploaded_files.push(file_name);
    }

    if uploaded_files.is_empty() {
        Err(StatusCode::BAD_REQUEST)
    } else {
        Ok(Json(format!("Uploaded {} file(s): {}", uploaded_files.len(), uploaded_files.join(", "))))
    }
}

#[utoipa::path(
    get,
    path = "/api/download/{path}/{file_name}",
    params(
        ("path" = String, Path, description = "Folder path"),
        ("file_name" = String, Path, description = "File name to download")
    ),
    responses(
        (status = 200, description = "File content", content_type = "application/octet-stream"),
        (status = 404, description = "File not found"),
        (status = 500, description = "Internal server error")
    )
)]
async fn download_file(
    Path((path, file_name)): Path<(String, String)>,
) -> Result<Response, StatusCode> {
    let file_path = format!("{}/{}", path, file_name);

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
            format!("attachment; filename=\"{}\"", file_name),
        )
        .body(axum::body::Body::from(contents))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}
