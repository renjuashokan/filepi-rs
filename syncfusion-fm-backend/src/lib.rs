pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

// syncfusion-fm-backend/src/lib.rs
pub fn handle_filemanager_action(action: &str) -> String {
    match action {
        "read" => "Listing directory...".to_string(),
        "delete" => "Deleting file...".to_string(),
        _ => "Unknown action".to_string(),
    }
}