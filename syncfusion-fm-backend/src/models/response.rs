use serde::Serialize;

#[derive(Serialize)]
pub struct FileManagerResponse {
    pub cwd: Option<FileEntry>,
    pub files: Vec<FileEntry>,
}

#[derive(Serialize)]
pub struct FileEntry {
    pub name: String,
    pub size: u64,
    #[serde(rename = "isFile")]
    pub is_file: bool,
    #[serde(rename = "dateModified")]
    pub date_modified: String,
    #[serde(rename = "type")]
    pub mime_type: String,
}
