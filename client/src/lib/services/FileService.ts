import { ServiceBase } from './ServiceBase';
import type {
	GetListFileAndFolderRequest,
	FileSystemItem,
	DownloadFileRequest,
	DeleteFileRequest
} from './types';

export class FileService extends ServiceBase {
	constructor() {
		super();
	}

	async getListFileAndFolder(request: GetListFileAndFolderRequest) {
		return this.post<FileSystemItem[]>('/api/ls', request);
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
}
