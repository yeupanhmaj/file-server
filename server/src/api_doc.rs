use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::endpoints::folder::create_folder,
        crate::endpoints::folder::rename_folder,
        crate::endpoints::folder::search_files,
        crate::endpoints::file::get_list_file_and_folder,
        crate::endpoints::file::get_folder_by_id,
        crate::endpoints::file::upload_file,
        crate::endpoints::file::upload_chunk,
        crate::endpoints::file::download_file,
        crate::endpoints::file::delete_file,
        crate::endpoints::folder::sorted_list_file_and_folder
    ),
    components(
        schemas(
            crate::models::CreateFolderRequest,
            crate::models::RenameFolderRequest,
            crate::models::SearchRequest,
            crate::models::UploadFileRequest,
            crate::models::ChunkedUploadRequest,
            crate::models::ChunkedUploadResponse,
            crate::models::DownloadFileRequest,
            crate::models::DeleteFileRequest,
            crate::models::SortOptionRequest,
            crate::models::GetFolderByIdRequest,
            crate::models::FileSystemItem
        )
    ),
    tags(
        (name = "file-server", description = "File server API endpoints")
    )
)]
pub struct ApiDoc;
