// filepi-server/src/main.rs
use syncfusion_fm_backend::handle_filemanager_action;

fn main() {
    println!("Starting FilePi-like server...");

    let result = handle_filemanager_action("read");
    println!("Result: {}", result);
}