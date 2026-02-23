# Automatic Service Generation 🚀

This project includes an **automatic code generation system** that keeps your frontend TypeScript services in sync with the Rust backend API.

## What It Does

Every time you run `npm run dev`, the system automatically:

1. 🔍 **Scans** your Rust backend for API endpoints
2. 📝 **Extracts** request/response types from Rust models
3. 🎯 **Generates** TypeScript interfaces
4. ✨ **Updates** service classes with properly typed methods
5. 🚀 **Starts** the dev server

## Features

✅ **Zero Manual Work** - Services update automatically  
✅ **Type Safety** - Full TypeScript types from Rust  
✅ **Always In Sync** - Backend changes instantly reflected  
✅ **FormData Support** - Handles file uploads  
✅ **Blob Support** - Handles file downloads  
✅ **Dynamic Imports** - Only imports used types  

## How It Works

### Backend Scanning

The script reads these Rust files:
- `server/src/main.rs` - Extracts route definitions
- `server/src/endpoints/*.rs` - Extracts handler functions
- `server/src/models/mod.rs` - Extracts type definitions

### Code Generation

Generates these TypeScript files:
- `client/src/lib/services/types.ts` - Type interfaces
- `client/src/lib/services/FileService.ts` - File operations
- `client/src/lib/services/FolderService.ts` - Folder operations

## Usage

### Running the Dev Server

```bash
cd client
npm run dev
```

This automatically generates services before starting Vite.

### Manual Generation

```bash
npm run generate:services
```

### Using the Services

```typescript
import { FileService, FolderService } from '$lib/services';

// Create instances
const fileService = new FileService();
const folderService = new FolderService();

// List files
const files = await fileService.getListFileAndFolder({ 
  path: '/some/path' 
});

// Create folder
await folderService.createFolder({ 
  path: '/parent', 
  folder_name: 'new-folder' 
});

// Upload file
const formData = new FormData();
formData.append('path', '/upload/path');
formData.append('file', file);
await fileService.uploadFile(formData);

// Download file
const blob = await fileService.downloadFile({ 
  file_path: '/path/to/file' 
});

// Delete file
await fileService.deleteFile({ 
  file_path: '/path/to/file' 
});

// Search files
const results = await folderService.searchFiles({ 
  search_string: 'query', 
  path: '/search/path' 
});

// Rename folder
await folderService.renameFolder({ 
  folder_name: '/old/path', 
  new_folder_name: '/new/path' 
});
```

## Current Endpoints

The following endpoints are automatically detected and generated:

### FileService
- `getListFileAndFolder(request)` - GET `/api/ls`
- `uploadFile(formData)` - POST `/api/upload`
- `downloadFile(request)` - POST `/api/download`
- `deleteFile(request)` - POST `/api/delete`

### FolderService
- `createFolder(request)` - POST `/api/mkdir`
- `searchFiles(request)` - POST `/api/search`
- `sortedListFileAndFolder(request)` - POST `/api/sort`
- `renameFolder(request)` - POST `/api/rename-folder`

## Adding New Endpoints

When you add a new endpoint to the Rust backend:

1. **Add the route** in `server/src/main.rs` (supports GET, POST, DELETE, PUT, PATCH):
   ```rust
   // POST endpoint
   .route("/api/create", post(create_handler))
   
   // GET endpoint
   .route("/api/list", get(list_handler))
   
   // DELETE endpoint
   .route("/api/remove", delete(remove_handler))
   ```

2. **Create the handler** in `server/src/endpoints/*.rs`:
   ```rust
   // For POST/PUT/PATCH with JSON body
   pub async fn create_handler(
       Json(req): Json<CreateRequest>,
   ) -> Result<Json<Response>, StatusCode> {
       // Implementation
   }
   
   // For GET requests (query params)
   pub async fn list_handler(
       Query(params): Query<ListParams>,
   ) -> Result<Json<Vec<Item>>, StatusCode> {
       // Implementation
   }
   
   // For DELETE requests
   pub async fn remove_handler(
       Json(req): Json<RemoveRequest>,
   ) -> Result<Json<String>, StatusCode> {
       // Implementation
   }
   ```

3. **Define the type** in `server/src/models/mod.rs`:
   ```rust
   #[derive(Deserialize, Serialize, ToSchema)]
   pub struct CreateRequest {
       pub field: String,
   }
   ```

4. **Run dev** - The services automatically update with the correct HTTP method!
   ```bash
   npm run dev
   ```

The generator automatically detects the HTTP method (GET, POST, DELETE, PUT, PATCH) and generates the appropriate TypeScript method:
- **GET** → `this.get(path, { params })`
- **POST/PUT/PATCH** → `this.post/put/patch(path, request)`
- **DELETE** → `this.delete(path, { data: request })`

## Customization

### Change Base URL

```typescript
const fileService = new FileService('https://api.example.com');
```

### Add Request Interceptors

Edit `client/src/lib/services/ServiceBase.ts`:

```typescript
constructor(baseURL: string = 'http://localhost:3000') {
    this.axiosInstance = axios.create({ baseURL });
    
    // Add auth token
    this.axiosInstance.interceptors.request.use(config => {
        config.headers.Authorization = `Bearer ${token}`;
        return config;
    });
}
```

### Custom Service Methods

You can add custom methods after the generated ones:

```typescript
export class FileService extends ServiceBase {
    // Auto-generated methods...
    
    // Your custom methods
    async bulkDelete(filePaths: string[]) {
        return Promise.all(
            filePaths.map(path => this.deleteFile({ file_path: path }))
        );
    }
}
```

## File Structure

```
client/
├── scripts/
│   └── generate-services.js    # Generator script
├── src/
│   └── lib/
│       └── services/
│           ├── ServiceBase.ts         # Base HTTP client
│           ├── FileService.ts         # Auto-generated
│           ├── FolderService.ts       # Auto-generated
│           ├── types.ts               # Auto-generated
│           ├── index.ts               # Exports
│           └── README.md              # Service docs
└── package.json                       # Includes generate:services script
```

## Troubleshooting

### Services not updating?

Run manual generation to see errors:
```bash
npm run generate:services
```

### Type mismatches?

Check the debug output in the console - it shows the type mapping.

### Custom types not detected?

Make sure your Rust types are:
- In `server/src/models/mod.rs`
- Use `pub struct`
- Have `#[derive(Deserialize, Serialize)]`

## Example Component

See [example-usage.svelte](../routes/example-usage.svelte) for a complete working example.

## Benefits

- 🚀 **Faster Development** - No manual service writing
- 🛡️ **Type Safety** - Catch errors at compile time
- 🔄 **Always Synced** - Backend changes auto-reflected
- 📚 **Self-Documenting** - Types show what's available
- 🐛 **Fewer Bugs** - TypeScript catches mismatches

---

**Made with ❤️ by the code generator**
