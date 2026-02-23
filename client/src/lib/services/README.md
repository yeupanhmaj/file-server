# Auto-Generated Services

This directory contains auto-generated TypeScript service classes that mirror the Rust backend API endpoints.

## How It Works

When you run `npm run dev`, the `generate-services.js` script automatically:

1. **Scans** the Rust backend files (`server/src/main.rs`, `server/src/endpoints/*.rs`, `server/src/models/mod.rs`)
2. **Extracts** all API routes and their request/response types
3. **Generates** TypeScript interfaces in `types.ts`
4. **Updates** `FileService.ts` and `FolderService.ts` with properly typed methods

## Usage

```typescript
import { FileService, FolderService } from '$lib/services';

// Create service instances
const fileService = new FileService();
const folderService = new FolderService();

// Use the services
const files = await fileService.getListFileAndFolder({ path: '/some/path' });
await folderService.createFolder({ path: '/parent', folder_name: 'new-folder' });
await fileService.uploadFile(formData);
```

## Files

- **types.ts** - Auto-generated TypeScript interfaces from Rust request types
- **ServiceBase.ts** - Base class with axios instance and HTTP methods  
- **FileService.ts** - Auto-generated file operations (ls, upload, download, delete)
- **FolderService.ts** - Auto-generated folder operations (mkdir, rename, search, sort)

## Customization

To change the base URL or add custom logic:

```typescript
// Custom base URL
const fileService = new FileService('http://api.example.com');

// Add interceptors in ServiceBase.ts
this.axiosInstance.interceptors.request.use(config => {
  // Add auth tokens, etc.
  return config;
});
```

## Manual Regeneration

```bash
npm run generate:services
```

This happens automatically on `npm run dev`, but you can run it manually anytime.
