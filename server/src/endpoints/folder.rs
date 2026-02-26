use crate::models::{
    CreateFolderRequest, FileSystemItem, RenameFolderRequest, SearchRequest, SearchResponse,
};
use crate::utils::{get_base_directory_canonical, validate_and_resolve_path};
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
        (status = 200, description = "File search results", body = SearchResponse),
        (status = 403, description = "Forbidden - path outside allowed directory"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn search_files(
    Json(req): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, StatusCode> {
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
                        crate::endpoints::file::format_file_size(metadata.len())
                    };

                    let modified = metadata
                        .modified()
                        .ok()
                        .map(crate::endpoints::file::format_system_time)
                        .unwrap_or_else(|| "Unknown".to_string());

                    // Generate relative path from base directory
                    let item_path_relative = entry_path
                        .strip_prefix(base_dir)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| entry_path.to_string_lossy().to_string());

                    let id = crate::utils::generate_id_from_path(&item_path_relative);

                    // Get parent path for parent_id
                    let parent_path = entry_path
                        .parent()
                        .and_then(|p| p.strip_prefix(base_dir).ok())
                        .map(|p| p.to_string_lossy().to_string());

                    let parent_id = parent_path
                        .filter(|p| !p.is_empty() && p != ".")
                        .map(|p| crate::utils::generate_id_from_path(&p));

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

    let response = SearchResponse {
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
