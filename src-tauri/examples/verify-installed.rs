use deepseek_harness_desktop::bundle::{self, BundlePaths};
#[tokio::main]
async fn main() {
    let app = std::path::PathBuf::from(
        std::env::args_os()
            .nth(1)
            .expect("Owned candidate path required."),
    )
    .canonicalize()
    .expect("Candidate unavailable.");
    let paths = BundlePaths::installed(app.join("Contents/Resources"), app.join("Contents/MacOS"));
    match bundle::verify(&paths).await {
        Ok(manifest) => println!("Verified installed runtime {}.", manifest.dsh_version),
        Err(_) => {
            eprintln!("Installed closure rejected.");
            std::process::exit(1)
        }
    }
}
