export const FILE_UPLOADED = 'file-uploaded';
export const FILE_DELETED = 'file-deleted';
export const FOLDER_CREATED = 'folder-created';
export const FOLDER_CHANGED = 'folder-changed';
export const REFRESH_LIST = 'refresh-list';
export const REFRESH_STORAGE = 'refresh-storage';

export type EventTypes =
	| typeof FILE_UPLOADED
	| typeof FILE_DELETED
	| typeof FOLDER_CREATED
	| typeof FOLDER_CHANGED
	| typeof REFRESH_LIST
	| typeof REFRESH_STORAGE;
