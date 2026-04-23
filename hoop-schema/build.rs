use std::{
    collections::HashMap,
    env,
    fs,
    path::Path,
};
use typify::{TypeSpace, TypeSpaceSettings};

fn main() {
    println!("cargo:rerun-if-changed=schemas");
    println!("cargo:rerun-if-changed=br-compat.toml");

    let schemas_dir = Path::new("schemas");
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);

    let mut settings_binding = TypeSpaceSettings::default();
    let settings = settings_binding.with_struct_builder(true);
    let mut type_space = TypeSpace::new(settings);

    // First pass: read all schemas into a map
    let mut schema_map: HashMap<String, serde_json::Value> = HashMap::new();
    let entries = fs::read_dir(schemas_dir).expect("Failed to read schemas directory");
    let schema_files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
        .collect();

    for entry in &schema_files {
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        let schema_content = fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Failed to read schema: {:?}", path));

        let schema_value: serde_json::Value =
            serde_json::from_str(&schema_content).unwrap_or_else(|_| {
                panic!("Failed to parse schema JSON: {:?}", path)
            });

        schema_map.insert(file_name, schema_value);
    }

    // Second pass: resolve refs and convert to schemars Schema
    for entry in &schema_files {
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();

        // Clone and resolve refs in this schema
        let mut schema_value = schema_map.get(&file_name).unwrap().clone();
        resolve_refs(&mut schema_value, &schema_map, schemas_dir);

        // Convert to schemars Schema
        let schema: schemars::schema::Schema =
            serde_json::from_value(schema_value).unwrap_or_else(|e| {
                panic!("Failed to convert to Schema for {:?}: {}", path, e)
            });

        // Add the type
        type_space
            .add_type(&schema)
            .unwrap_or_else(|e| panic!("Failed to add type from {:?}: {}", path, e));
    }

    // Generate the Rust code
    let content = type_space.to_stream();

    // Format the code
    let formatted =
        rustfmt_wrapper::rustfmt(content.to_string()).unwrap_or_else(|e| {
            eprintln!("rustfmt failed: {}", e);
            content.to_string()
        });

    // Add PartialEq derives to all structs for round-trip tests
    let with_derives = add_partial_eq_derives(&formatted);

    // Add #[allow(clippy::clone_on_copy)] before each From<&T> impl — typify
    // generates `value.clone()` for Copy types, which triggers the lint.
    let with_allows = add_clippy_allows(&with_derives);

    // Write to the output file
    let output_path = out_path.join("types.rs");
    fs::write(&output_path, with_allows)
        .unwrap_or_else(|_| panic!("Failed to write {:?}", output_path));

    // Parse br-compat.toml and emit the pinned minimum br version
    let compat_toml = fs::read_to_string("br-compat.toml")
        .expect("Failed to read br-compat.toml");
    let br_min_version = compat_toml
        .lines()
        .find(|line| line.trim().starts_with("br_min_version"))
        .and_then(|line| line.split('=').nth(1))
        .map(|v| v.trim().trim_matches('"'))
        .expect("br_min_version not found in br-compat.toml");
    let compat_rs = format!(
        "/// Minimum pinned br version (from br-compat.toml)\npub const BR_MIN_VERSION: &str = \"{}\";\n",
        br_min_version
    );
    fs::write(out_path.join("br_compat.rs"), compat_rs)
        .expect("Failed to write br_compat.rs");
}

/// Resolve $ref references in a JSON Schema by inlining the referenced schemas
fn resolve_refs(
    value: &mut serde_json::Value,
    schema_map: &HashMap<String, serde_json::Value>,
    schemas_dir: &Path,
) {
    match value {
        serde_json::Value::Object(map) => {
            // Check if this object has a $ref
            if let Some(ref_value) = map.get("$ref") {
                if let Some(ref_str) = ref_value.as_str() {
                    // Resolve the reference
                    let referenced_schema = resolve_ref(ref_str, schema_map, schemas_dir);

                    // Replace the entire object with the referenced schema
                    // But preserve properties that aren't part of the ref resolution
                    // For simplicity, we'll just merge the referenced schema in
                    if let Some(referenced_obj) = referenced_schema.as_object() {
                        // Remove the $ref key
                        map.remove("$ref");

                        // Merge all properties from the referenced schema
                        for (key, val) in referenced_obj {
                            // Skip $schema and $id as they're metadata
                            if key != "$schema" && key != "$id" && key != "title" && key != "description" {
                                map.insert(key.clone(), val.clone());
                            }
                        }

                        // Recursively resolve any refs in the merged schema
                        resolve_refs(value, schema_map, schemas_dir);
                    }
                    return;
                }
            }

            // Recursively resolve refs in all values
            for (_key, val) in map.iter_mut() {
                resolve_refs(val, schema_map, schemas_dir);
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                resolve_refs(item, schema_map, schemas_dir);
            }
        }
        _ => {}
    }
}

/// Resolve a single $ref reference
fn resolve_ref(
    ref_path: &str,
    schema_map: &HashMap<String, serde_json::Value>,
    schemas_dir: &Path,
) -> serde_json::Value {
    // Extract just the filename from the ref path
    // Refs can be like "worker_data.json" or "#/definitions/foo" or a full URL
    let filename = if ref_path.contains('/') {
        ref_path.split('/').next_back().unwrap_or(ref_path)
    } else {
        ref_path
    };

    // Look up the schema in our map
    if let Some(schema) = schema_map.get(filename) {
        // Clone and recursively resolve any refs in this schema
        let mut resolved = schema.clone();
        resolve_refs(&mut resolved, schema_map, schemas_dir);
        resolved
    } else {
        // If we can't find it, try reading from disk
        let full_path = schemas_dir.join(filename);
        if full_path.exists() {
            let content = fs::read_to_string(&full_path)
                .unwrap_or_else(|_| panic!("Failed to read referenced schema: {}", filename));
            let mut schema: serde_json::Value = serde_json::from_str(&content)
                .unwrap_or_else(|_| panic!("Failed to parse referenced schema: {}", filename));
            resolve_refs(&mut schema, schema_map, schemas_dir);
            schema
        } else {
            panic!("Could not resolve $ref: {}", ref_path);
        }
    }
}

/// Insert `#[allow(clippy::clone_on_copy)]` before every `impl From<&…>` block.
///
/// typify generates `value.clone()` inside these impls even for Copy types;
/// the allow suppresses the resulting lint in generated code.
fn add_clippy_allows(code: &str) -> String {
    let mut result = String::with_capacity(code.len() + 1024);
    for line in code.lines() {
        if line.trim().starts_with("impl From<&") {
            result.push_str("#[allow(clippy::clone_on_copy)]\n");
        }
        result.push_str(line);
        result.push('\n');
    }
    result
}

/// Add #[derive(PartialEq)] to struct definitions that don't already have it.
/// This is post-processing on the generated Rust code to enable round-trip tests.
fn add_partial_eq_derives(code: &str) -> String {
    let mut result = String::new();

    for line in code.lines() {
        let trimmed = line.trim();

        // Only process derive attributes, not serde or other attributes
        if trimmed.starts_with("#[derive(") && !trimmed.contains("PartialEq") {
            // Add PartialEq to the derive macro
            // Find the closing ) which marks the end of the derive list
            if let Some(derive_end) = trimmed.find(')') {
                result.push_str(&trimmed[..derive_end]);
                result.push_str(", PartialEq");
                result.push_str(&trimmed[derive_end..]);
                result.push('\n');
            } else {
                // If we can't find a closing ), just pass the line through
                result.push_str(line);
                result.push('\n');
            }
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    // Clean up any double PartialEq that might have been created
    result.replace("PartialEq, PartialEq", "PartialEq")
}
