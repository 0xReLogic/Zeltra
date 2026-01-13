//! Utility to generate OpenAPI specification from backend code.
//!
//! This utility uses the `ApiDoc` struct from `zeltra-api` to generate a YAML
//! version of the OpenAPI specification and writes it to `contracts/openapi.yaml`.

use utoipa::OpenApi;
use zeltra_api::routes::ApiDoc;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating OpenAPI specification...");

    // Generate JSON
    let openapi_json = ApiDoc::openapi().to_pretty_json()?;

    // Convert JSON to YAML for better readability in openapi.yaml
    let json_value: serde_json::Value = serde_json::from_str(&openapi_json)?;
    let openapi_yaml = serde_yaml::to_string(&json_value)?;

    // Define output path (relative to where cargo run is executed, usually backend/)
    // We want to write to Zeltra/contracts/openapi.yaml
    let output_path = Path::new("../contracts/openapi.yaml");

    println!("Writing to {:?}...", output_path);

    let mut file = File::create(output_path)?;
    file.write_all(openapi_yaml.as_bytes())?;

    println!("Successfully generated OpenAPI specification at {:?}", output_path);

    Ok(())
}
