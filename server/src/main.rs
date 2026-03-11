mod api_doc;
mod endpoints;
mod models;
mod utils;

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use api_doc::ApiDoc;
use endpoints::{
    copy_file, create_folder, delete_file, download_file, empty_trash, get_folder_by_id,
    get_list_file_and_folder, get_storage_stats_endpoint, get_thumbnail, list_trash, move_file,
    rename_file, rename_folder, restore_file, search_files, sorted_list_file_and_folder,
    upload_chunk, upload_file,
};

#[tokio::main]
async fn main() {
    // Initialize trash directory
    utils::init_trash_directory().expect("Failed to initialize trash directory");

    // Define your CORS policy
    let cors = CorsLayer::new()
        .allow_origin(Any) // For debugging. In production, use "http://example.com".parse().unwrap()
        .allow_methods(Any)
        .allow_headers(Any);

    // Set a 150MB body size limit for file uploads (allows 50MB chunks + multipart overhead)
    let app = route_builder()
        .layer(DefaultBodyLimit::max(150 * 1024 * 1024)) // 150 MB
        .layer(cors);
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
        .route("/api/ls", post(get_list_file_and_folder))
        .route("/api/folder", post(get_folder_by_id))
        .route("/api/mkdir", post(create_folder))
        .route("/api/search", post(search_files))
        .route("/api/sort", post(sorted_list_file_and_folder))
        .route("/api/upload", post(upload_file))
        .route("/api/upload-chunk", post(upload_chunk))
        .route("/api/download", post(download_file))
        .route("/api/delete", post(delete_file))
        .route("/api/rename-file", post(rename_file))
        .route("/api/rename-folder", post(rename_folder))
        .route("/api/move-file", post(move_file))
        .route("/api/copy-file", post(copy_file))
        .route("/api/thumbnail", post(get_thumbnail))
        .route("/api/trash", get(list_trash))
        .route("/api/trash/restore", post(restore_file))
        .route("/api/trash/empty", post(empty_trash))
        .route("/api/storage", get(get_storage_stats_endpoint))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
