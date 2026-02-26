import { ServiceBase } from './ServiceBase';
import type {
	GetListFileAndFolderRequest,
	GetFolderByIdRequest,
	FileSystemItem,
	DownloadFileRequest,
	DeleteFileRequest,
	TrashItem,
	RestoreFileRequest
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

	async downloadFile(request: DownloadFileRequest) {
		return this.post<Blob>('/api/download', request, { responseType: 'blob' });
	}

	async deleteFile(request: DeleteFileRequest) {
		return this.post<string>('/api/delete', request);
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
}
