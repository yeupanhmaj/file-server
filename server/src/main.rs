mod api_doc;
mod handlers;
mod models;

use axum::{Router, routing::post};
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use api_doc::ApiDoc;
use handlers::{
    create_folder, delete_file, download_file, get_list_file_and_folder, rename_folder,
    search_files, sorted_list_file_and_folder, upload_file,
};

#[tokio::main]
async fn main() {
    // Define your CORS policy
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
        .route("/api/ls", post(get_list_file_and_folder))
        .route("/api/mkdir", post(create_folder))
        .route("/api/search", post(search_files))
        .route("/api/sort", post(sorted_list_file_and_folder))
        .route("/api/upload", post(upload_file))
        .route("/api/download", post(download_file))
        .route("/api/delete", post(delete_file))
        .route("/api/rename-folder", post(rename_folder))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
