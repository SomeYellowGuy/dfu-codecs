use std::path::Path;
use std::{env, fs};

mod data_result;

fn main() {
    // Use CARGO_MANIFEST_DIR to get the absolute path to the crate directory
    let out_dir_path = env::var("OUT_DIR").unwrap();

    let out_dir = Path::new(&out_dir_path).join("generated");

    // Create the generated directory if it doesn't exist
    if !out_dir.exists() {
        fs::create_dir(&out_dir).unwrap();
    }

    let path = out_dir.join("data_result.rs");
    let content = data_result::build().to_string();
    fs::write(&path, content).unwrap();

    // Rerun build script when any file in the build/ directory changes
    println!("cargo:rerun-if-changed=build/");
}
