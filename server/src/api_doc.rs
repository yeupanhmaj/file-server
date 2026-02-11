use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::folder::create_folder,
        crate::handlers::folder::rename_folder,
        crate::handlers::folder::search_files,
        crate::handlers::file::get_list_file_and_folder,
        crate::handlers::file::upload_file,
        crate::handlers::file::download_file,
        crate::handlers::file::delete_file,
    ),
    components(
        schemas(
            crate::models::CreateFolderRequest,
            crate::models::RenameFolderRequest,
            crate::models::SearchRequest,
            crate::models::UploadFileRequest,
            crate::models::DownloadFileRequest,
            crate::models::DeleteFileRequest,
        )
    ),
    tags(
        (name = "file-server", description = "File server API endpoints")
    )
)]
pub struct ApiDoc;
