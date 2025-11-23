use axum::{Json, extract::State};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

use crate::config::Config;
use crate::handlers::app_error::AppError;

use syncfusion_fm_backend::{FileManagerDirectoryContent, FileManagerResponse};

pub async fn file_operations(
    State(config): State<Arc<Config>>,
    Json(args): Json<FileManagerDirectoryContent>,
) -> Result<Json<FileManagerResponse>, AppError> {
    info!("Syncfusion FileManager action");

    // call the process_file_manager_request function from syncfusion-fm-backend
    let root_dir = PathBuf::from(&config.root_dir);
    let response = syncfusion_fm_backend::process_file_manager_request(&args, &root_dir);
    Ok(Json(response))
}
