# Security Features

## Path Protection

The file server implements strict path validation to prevent directory traversal attacks and unauthorized file access.

### Base Directory Restriction

All file operations are restricted to a **base directory**. By default, this is the `./shared` folder in the project root.

**Configuration:**
- Default: `./shared`
- Environment Variable: Set `FILE_SERVER_ROOT` to customize the base directory

Example:
```bash
export FILE_SERVER_ROOT=/path/to/your/files
cargo run
```

### Path Validation

The server validates all incoming paths to ensure:

1. **No Directory Traversal**: Paths containing `..` or absolute paths like `/` are resolved and validated
2. **Confined to Base Directory**: All resolved paths must remain within the configured base directory
3. **Canonical Path Resolution**: Uses `canonicalize()` to resolve symbolic links and relative paths
4. **Forbidden Responses**: Returns `403 Forbidden` for any attempt to access files outside the allowed directory

### Protected Endpoints

All file and folder operations are protected:

- `POST /api/ls` - List files and folders
- `POST /api/upload` - Upload files
- `POST /api/download` - Download files
- `POST /api/delete` - Delete files
- `POST /api/mkdir` - Create folders
- `POST /api/rename-folder` - Rename folders
- `POST /api/search` - Search files
- `POST /api/sort` - Sorted file listing

### Example Attack Prevention

❌ **Blocked Attempts:**
```json
// Trying to access root filesystem
{"path": "/"}

// Directory traversal
{"path": "../../../etc/passwd"}

// Absolute paths outside base
{"path": "/home/user/secrets"}
```

✅ **Allowed Access:**
```json
// Access within shared directory
{"path": "."}
{"path": "documents"}
{"path": "documents/2024"}
```

### Implementation Details

The security implementation is located in:
- `/server/src/utils.rs` - Path validation utilities
- Applied in all endpoints in `/server/src/endpoints/`

Key function: `validate_and_resolve_path(requested_path: &str) -> Result<PathBuf, StatusCode>`
