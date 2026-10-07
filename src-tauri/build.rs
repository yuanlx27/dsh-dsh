fn main() {
    assert_eq!(
        std::env::var("TARGET").as_deref(),
        Ok("aarch64-apple-darwin"),
        "Only macOS Apple Silicon is supported."
    );
    tauri_build::build();
}
