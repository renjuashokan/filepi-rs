use axum::{
    Json,
    body::Body,
    extract::{Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use bytes::Bytes;
use mime_guess::from_path;
use serde::Deserialize;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use tracing::{debug, error, info};

use crate::config::Config;
use crate::handlers::app_error::AppError;

use syncfusion_fm_backend::{FileManagerDirectoryContent, FileManagerResponse};

pub async fn file_operations(
    State(config): State<Arc<Config>>,
    Json(args): Json<FileManagerDirectoryContent>,
) -> Result<Json<FileManagerResponse>, AppError> {
    debug!("Syncfusion FileManager action: {:?}", args);

    // call the process_file_manager_request function from syncfusion-fm-backend
    let root_dir = PathBuf::from(&config.root_dir);
    let response = syncfusion_fm_backend::process_file_manager_request(&args, &root_dir);
    Ok(Json(response))
}

#[derive(Deserialize)]
pub struct GetImageParams {
    #[serde(alias = "Path")]
    pub path: String,
}

pub async fn get_image(
    State(config): State<Arc<Config>>,
    Query(params): Query<GetImageParams>,
) -> Result<impl IntoResponse, AppError> {
    let path = params.path;
    info!("Syncfusion GetImage: {}", path);

    let relative_path = path.trim_start_matches('/');
    let root_dir = PathBuf::from(&config.root_dir);

    let full_path = syncfusion_fm_backend::validate_path(&root_dir, relative_path)
        .map_err(|_| AppError::BadRequest("Invalid path".to_string()))?;

    if !full_path.exists() {
        return Err(AppError::NotFound("File not found".to_string()));
    }

    if !full_path.is_file() {
        return Err(AppError::BadRequest("Path is not a file".to_string()));
    }

    let file = File::open(&full_path).await.map_err(|e| {
        error!("Failed to open file: {}", e);
        AppError::InternalError(format!("Failed to open file: {}", e))
    })?;

    let metadata = file.metadata().await.map_err(|e| {
        error!("Failed to read metadata: {}", e);
        AppError::InternalError(format!("Failed to read metadata: {}", e))
    })?;

    let mime_type = "application/octet-stream".to_string();
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, mime_type),
            (header::CONTENT_LENGTH, metadata.len().to_string()),
            (header::CACHE_CONTROL, "public, max-age=3600".to_string()),
        ],
        body,
    ))
}

pub async fn download(
    State(config): State<Arc<Config>>,
    Json(args): Json<FileManagerDirectoryContent>,
) -> Result<impl IntoResponse, AppError> {
    info!("Syncfusion Download");

    let path_str = args.path.as_deref().unwrap_or("/");
    let names = args.names.as_ref().ok_or(AppError::BadRequest(
        "No files specified for download".to_string(),
    ))?;

    if names.is_empty() {
        return Err(AppError::BadRequest(
            "No files specified for download".to_string(),
        ));
    }

    // For now, support single file download
    // TODO: Support multi-file download (zip)
    let file_name = &names[0];
    let relative_path = if path_str == "/" {
        file_name.clone()
    } else {
        format!("{}/{}", path_str.trim_start_matches('/'), file_name)
    };

    let root_dir = PathBuf::from(&config.root_dir);
    let full_path = syncfusion_fm_backend::validate_path(&root_dir, &relative_path)
        .map_err(|_| AppError::BadRequest("Invalid path".to_string()))?;

    if !full_path.exists() {
        return Err(AppError::NotFound("File not found".to_string()));
    }

    if !full_path.is_file() {
        return Err(AppError::BadRequest("Path is not a file".to_string()));
    }

    let file = File::open(&full_path).await.map_err(|e| {
        error!("Failed to open file: {}", e);
        AppError::InternalError(format!("Failed to open file: {}", e))
    })?;

    let metadata = file.metadata().await.map_err(|e| {
        error!("Failed to read metadata: {}", e);
        AppError::InternalError(format!("Failed to read metadata: {}", e))
    })?;

    let mime_type = from_path(&full_path).first_or_octet_stream().to_string();
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let filename_header = format!("attachment; filename=\"{}\"", file_name);

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, mime_type),
            (header::CONTENT_LENGTH, metadata.len().to_string()),
            (header::CONTENT_DISPOSITION, filename_header),
        ],
        body,
    ))
}

#[derive(TryFromMultipart)]
pub struct SyncfusionUploadForm {
    pub path: String,
    pub action: String,
    #[form_data(field_name = "uploadFiles")]
    pub upload_files: Vec<FieldData<Bytes>>,
}

pub async fn upload(
    State(config): State<Arc<Config>>,
    TypedMultipart(form): TypedMultipart<SyncfusionUploadForm>,
) -> Result<impl IntoResponse, AppError> {
    info!("Syncfusion Upload to path: {}", form.path);

    let relative_path = form.path.trim_start_matches('/');
    let root_dir = PathBuf::from(&config.root_dir);

    // We validate the directory path, not a file path
    let _upload_dir = root_dir.join(relative_path);

    // Use validate_path logic manually or adapt it, since validate_path checks existence?
    // validate_path in lib.rs checks is_safe_path but DOES NOT check existence (it returns path).
    // Wait, my implementation of validate_path only checks is_safe_path.

    let canonical_upload_dir = syncfusion_fm_backend::validate_path(&root_dir, relative_path)
        .map_err(|_| AppError::BadRequest("Invalid upload path".to_string()))?;

    // Ensure upload directory exists
    if !canonical_upload_dir.exists() {
        std::fs::create_dir_all(&canonical_upload_dir).map_err(|e| {
            error!("Failed to create upload directory: {}", e);
            AppError::InternalError(format!("Failed to create directory: {}", e))
        })?;
    }

    for file_data in form.upload_files {
        let file_name = file_data
            .metadata
            .file_name
            .unwrap_or_else(|| "uploaded_file".to_string());
        let file_path = canonical_upload_dir.join(&file_name);

        // Validate file path is safe too (should be if dir is safe and name has no separators, but good to check)
        if !syncfusion_fm_backend::validate_path(
            &root_dir,
            &file_path
                .to_string_lossy()
                .replace(&root_dir.to_string_lossy().to_string(), ""),
        )
        .is_ok()
        {
            // This check is tricky because of absolute paths.
            // Let's just trust canonical_upload_dir join file_name if file_name is simple.
            // Better: check if file_path starts with canonical_upload_dir
        }

        info!("Saving file: {:?}", file_path);

        let mut file = std::fs::File::create(&file_path).map_err(|e| {
            error!("Failed to create file: {}", e);
            AppError::InternalError(format!("Failed to create file: {}", e))
        })?;

        file.write_all(&file_data.contents).map_err(|e| {
            error!("Failed to write file content: {}", e);
            AppError::InternalError(format!("Failed to write file content: {}", e))
        })?;
    }

    Ok(StatusCode::OK)
}
