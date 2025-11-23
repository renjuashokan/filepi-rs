use serde::Deserialize;

#[derive(Deserialize)]
pub struct FileManagerRequest {
    pub action: String,
    #[serde(default)]
    pub path: Option<String>,
    // ... other fields
}
