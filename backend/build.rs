use std::io::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut config = prost_build::Config::new();

    // Add Serde attributes to all messages
    config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    config.type_attribute(".", "#[serde(rename_all = \"camelCase\")]");

    // Get directory containing Cargo.toml (backend/)
    let manifest_dir = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR not set")
    );

    // Try ./proto first (Docker), then ../proto (local)
    let proto_dir = if manifest_dir.join("proto").exists() {
        manifest_dir.join("proto")
    } else {
        manifest_dir.parent()
            .expect("No parent directory found")
            .join("proto")
    };

    if !proto_dir.exists() {
        panic!("Proto directory not found at {:?}", proto_dir);
    }

    let proto_file = proto_dir.join("api.proto");

    config.compile_protos(
        &[proto_file.to_str().unwrap()],
        &[proto_dir.to_str().unwrap()]
    )?;

    Ok(())
}

