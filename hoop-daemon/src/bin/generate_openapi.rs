//! Generate OpenAPI spec for HOOP REST API
//!
//! This binary generates the OpenAPI 3.1 spec and writes it to stdout as YAML.

use hoop_daemon::openapi::ApiDoc;
use utoipa::OpenApi;

fn main() -> anyhow::Result<()> {
    // The openapi feature must be enabled for this binary to work
    #[cfg(not(feature = "openapi"))]
    compile_error!("The 'openapi' feature must be enabled to build this binary");

    let openapi = ApiDoc::openapi();
    let json = serde_json::to_value(&openapi)?;
    let yaml = serde_yaml::to_string(&json)?;
    println!("{}", yaml);
    Ok(())
}
