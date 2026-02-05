use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::get};

#[tokio::main]
async fn main() {
    let app = route_builder();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Server running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}

fn route_builder() -> Router {
    Router::new()
        .route("/", get(hello_world))
        .route("/api/ls", get(get_list_file_and_folder))
}

async fn hello_world() -> impl IntoResponse {
    "Hello, World!"
}

async fn get_list_file_and_folder() -> Result<Json<Vec<String>>, StatusCode> {
    let mut entries = tokio::fs::read_dir(".")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut items = Vec::new();

    while let Some(entry) = entries.next_entry()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
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
