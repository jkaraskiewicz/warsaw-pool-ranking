use std::io::Result;

fn main() -> Result<()> {
    let mut config = prost_build::Config::new();

    // Add Serde attributes to all messages
    config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    config.type_attribute(".", "#[serde(rename_all = \"camelCase\")]");

    config.compile_protos(&["proto/api.proto"], &["proto/"])?;
    Ok(())
}

