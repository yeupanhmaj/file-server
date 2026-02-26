import { ServiceBase } from './ServiceBase';
import type {
	CreateFolderRequest,
	SearchRequest,
	SearchResponse,
	SortOptionRequest,
	RenameFolderRequest
} from './types';

export class FolderService extends ServiceBase {
	constructor() {
		super();
	}

	async createFolder(request: CreateFolderRequest) {
		return this.post<string>('/api/mkdir', request);
	}

	async searchFiles(request: SearchRequest) {
		return this.post<SearchResponse>('/api/search', request);
	}

	async sortedListFileAndFolder(request: SortOptionRequest) {
		return this.post<string[]>('/api/sort', request);
	}

	async renameFolder(request: RenameFolderRequest) {
		return this.post<string>('/api/rename-folder', request);
	}
}
