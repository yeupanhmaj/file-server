import { FileService } from './FileService';
import { FolderService } from './FolderService';
const fileService = new FileService();
const folderService = new FolderService();

export * from './types';
export { fileService, folderService };
