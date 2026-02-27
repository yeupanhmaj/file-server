use axum::http::StatusCode;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

// Global ID to path mapping cache
static ID_TO_PATH_CACHE: OnceLock<std::sync::Mutex<HashMap<String, String>>> = OnceLock::new();

fn get_cache() -> &'static std::sync::Mutex<HashMap<String, String>> {
    ID_TO_PATH_CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

/// Generate a stable ID from a file/folder path using SHA-256 and base64url encoding
/// This creates a Google Drive-like ID that's URL-safe and stable
pub fn generate_id_from_path(path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    let hash = hasher.finalize();

    // Take first 24 bytes (similar length to Google Drive IDs)
    let id = URL_SAFE_NO_PAD.encode(&hash[..24]);

    // Cache the mapping
    if let Ok(mut cache) = get_cache().lock() {
        cache.insert(id.clone(), path.to_string());
    }

    id
}

/// Resolve an ID back to its file path
pub fn resolve_id_to_path(id: &str) -> Option<String> {
    if let Ok(cache) = get_cache().lock() {
        cache.get(id).cloned()
    } else {
        None
    }
}

/// Get the base directory for file operations
/// This can be configured via environment variable or defaults to "./my-drive"
pub fn get_base_directory() -> PathBuf {
    std::env::var("FILE_SERVER_ROOT")
        .unwrap_or_else(|_| "./my-drive".to_string())
        .into()
}

/// Get the canonicalized base directory
pub fn get_base_directory_canonical() -> Result<PathBuf, StatusCode> {
    let base_dir = get_base_directory();

    // Create base directory if it doesn't exist
    if !base_dir.exists() {
        std::fs::create_dir_all(&base_dir).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    base_dir
        .canonicalize()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
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

/// Calculate the total size of a directory recursively
pub fn calculate_directory_size(path: &std::path::Path) -> std::io::Result<u64> {
    let mut total_size = 0u64;

    if path.is_file() {
        return Ok(path.metadata()?.len());
    }

    if path.is_dir() {
        let entries = fs::read_dir(path)?;
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_file() {
                total_size += entry_path.metadata()?.len();
            } else if entry_path.is_dir() {
                // Recursively calculate subdirectory size
                total_size += calculate_directory_size(&entry_path)?;
            }
        }
    }

    Ok(total_size)
}

/// Get storage statistics for the base directory
pub fn get_storage_stats() -> Result<(u64, u64), StatusCode> {
    let base_dir = get_base_directory();

    // Create base directory if it doesn't exist
    if !base_dir.exists() {
        std::fs::create_dir_all(&base_dir).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Calculate used space
    let used =
        calculate_directory_size(&base_dir).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // For MVP, set a fixed quota (15 GB like Google Drive free tier)
    // In production, this could come from a database or config per user
    let total = 15 * 1024 * 1024 * 1024u64; // 15 GB in bytes

    Ok((used, total))
}
