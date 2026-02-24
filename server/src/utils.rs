use axum::http::StatusCode;
use std::fs;
use std::path::PathBuf;

/// Get the base directory for file operations
/// This can be configured via environment variable or defaults to "./shared"
pub fn get_base_directory() -> PathBuf {
    std::env::var("FILE_SERVER_ROOT")
        .unwrap_or_else(|_| "./shared".to_string())
        .into()
}

/// Get the trash directory path
pub fn get_trash_directory() -> PathBuf {
    get_base_directory().join(".trash")
}

/// Initialize trash directory on application startup
pub fn init_trash_directory() -> std::io::Result<()> {
    let trash_dir = get_trash_directory();
    if !trash_dir.exists() {
        fs::create_dir_all(&trash_dir)?;
        println!("✅ Created trash directory at: {}", trash_dir.display());
    }
    Ok(())
}

/// Validates and resolves a path to ensure it's within the base directory
/// Returns the canonicalized path if valid, or an error if the path is invalid or tries to escape
pub fn validate_and_resolve_path(requested_path: &str) -> Result<PathBuf, StatusCode> {
    let base_dir = get_base_directory();

    // Create base directory if it doesn't exist
    if !base_dir.exists() {
        std::fs::create_dir_all(&base_dir).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Canonicalize the base directory
    let base_canonical = base_dir
        .canonicalize()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Normalize the requested path (remove "..", ".", etc.)
    let requested = requested_path.trim_start_matches('/');
    let full_path = if requested.is_empty() || requested == "." {
        base_canonical.clone()
    } else {
        base_canonical.join(requested)
    };

    // For paths that don't exist yet (e.g., for mkdir or upload), we need to check parent
    let path_to_check = if full_path.exists() {
        full_path
            .canonicalize()
            .map_err(|_| StatusCode::FORBIDDEN)?
    } else {
        // Check that the parent directory is valid
        if let Some(parent) = full_path.parent() {
            if parent.exists() {
                let canonical_parent = parent.canonicalize().map_err(|_| StatusCode::FORBIDDEN)?;
                if !canonical_parent.starts_with(&base_canonical) {
                    return Err(StatusCode::FORBIDDEN);
                }
            }
        }
        full_path
    };

    // Ensure the resolved path is within the base directory
    if path_to_check.starts_with(&base_canonical) {
        Ok(path_to_check)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}
