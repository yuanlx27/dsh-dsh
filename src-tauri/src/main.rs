fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("The desktop application could not start.");
}
