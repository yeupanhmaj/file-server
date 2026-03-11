import { ServiceBase } from './ServiceBase';
import type {
	GetListFileAndFolderRequest,
	FileSystemItem,
	GetFolderByIdRequest,
	DownloadFileRequest,
	DeleteFileRequest,
	RenameFileRequest,
	MoveFileRequest,
	CopyFileRequest,
	ThumbnailRequest,
	TrashItem,
	RestoreFileRequest,
	StorageStats
} from './types';

export class FileService extends ServiceBase {
	constructor() {
		super();
	}

	async getListFileAndFolder(request: GetListFileAndFolderRequest) {
		return this.post<FileSystemItem[]>('/api/ls', request);
	}

	async getFolderById(request: GetFolderByIdRequest) {
		return this.post<FileSystemItem[]>('/api/folder', request);
	}

	async uploadFile(formData: FormData) {
		return this.post<unknown>('/api/upload', formData, {
			headers: { 'Content-Type': 'multipart/form-data' }
		});
	}

	async uploadChunk(formData: FormData) {
		return this.post<unknown>('/api/upload-chunk', formData, {
			headers: { 'Content-Type': 'multipart/form-data' }
		});
	}

	async downloadFile(request: DownloadFileRequest) {
		return this.post<Blob>('/api/download', request, { responseType: 'blob' });
	}

	async deleteFile(request: DeleteFileRequest) {
		return this.post<string>('/api/delete', request);
	}

	async renameFile(request: RenameFileRequest) {
		return this.post<string>('/api/rename-file', request);
	}

	async moveFile(request: MoveFileRequest) {
		return this.post<string>('/api/move-file', request);
	}

	async copyFile(request: CopyFileRequest) {
		return this.post<string>('/api/copy-file', request);
	}

	async getThumbnail(request: ThumbnailRequest) {
		return this.post<Blob>('/api/thumbnail', request);
	}

	async listTrash() {
		return this.get<TrashItem[]>('/api/trash');
	}

	async restoreFile(request: RestoreFileRequest) {
		return this.post<string>('/api/trash/restore', request);
	}

	async emptyTrash() {
		return this.post<string>('/api/trash/empty');
	}

	async getStorageStatsEndpoint() {
		return this.get<StorageStats>('/api/storage');
	}
}
