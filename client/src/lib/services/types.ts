// Auto-generated types from Rust backend

export interface GetListFileAndFolderRequest {
	path?: string;
}

export interface GetFolderByIdRequest {
	folder_id: string;
}

export interface FileSystemItem {
	id: string;
	name: string;
	item_type: string;
	modified: string;
	size: string;
	parent_id?: string;
	path: string;
}

export interface UploadFileRequest {
	path?: string;
	file: string;
}

export interface CreateFolderRequest {
	path: string;
	folder_name: string;
}

export interface DownloadFileRequest {
	file_path: string;
}

export interface DeleteFileRequest {
	file_path: string;
}

export interface RenameFolderRequest {
	folder_name: string;
	new_folder_name: string;
}

export interface SearchRequest {
	search_string: string;
	path: string;
	page: number;
	limit: number;
}

export interface SearchResponse {
	results: FileSystemItem[];
	total: number;
	page: number;
	limit: number;
	has_more: boolean;
}

export interface SortOptionRequest {
	option?: string;
}

export interface RestoreFileRequest {
	trash_item_id: string;
}

export interface TrashItem {
	id: string;
	original_path: string;
	name: string;
	deleted_at: string;
	size: string;
}

export interface StorageStats {
	used_bytes: number;
	total_bytes: number;
	used_formatted: string;
	total_formatted: string;
	percentage: number;
}

export interface ChunkedUploadRequest {
	path?: string;
	chunk: string;
	file_id: string;
	chunk_index: number;
	total_chunks: number;
	filename: string;
}

export interface ChunkedUploadResponse {
	message: string;
	chunk_index: number;
	total_chunks: number;
	completed: boolean;
}

