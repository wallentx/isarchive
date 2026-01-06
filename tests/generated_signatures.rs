use isarchive::analyze;
use std::collections::HashMap;
use std::fs;
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;

#[derive(serde::Deserialize)]
struct Signature {
    description: String,
    hexdump: String,
    offset: Option<usize>,
}

#[derive(serde::Deserialize)]
struct Entry {
    signatures: Option<Vec<Signature>>,
}

// Top level is Key -> Key -> Entry
type Manifest = HashMap<String, HashMap<String, Entry>>;

#[test]
fn test_all_signatures_generated() {
    let manifest_path = PathBuf::from("archive_signatures.yaml");
    if !manifest_path.exists() {
        eprintln!("Skipping generated signature tests: archive_signatures.yaml not found");
        return;
    }

    let manifest_str =
        fs::read_to_string(&manifest_path).expect("Failed to read archive_signatures.yaml");
    let manifest: Manifest = serde_yaml::from_str(&manifest_str).expect("Failed to parse YAML");

    // Use a unique temp directory for this test run
    let test_dir = std::env::temp_dir().join("isarchive_gen_tests");
    if test_dir.exists() {
        fs::remove_dir_all(&test_dir).unwrap();
    }
    fs::create_dir_all(&test_dir).unwrap();

    let mut total_checked = 0;
    let mut failures = Vec::new();

    for (category, extensions) in manifest {
        for (ext, entry) in extensions {
            if let Some(signatures) = entry.signatures {
                for (i, sig) in signatures.iter().enumerate() {
                    // Parse hex
                    let bytes: Vec<u8> = sig
                        .hexdump
                        .split_whitespace()
                        .map(|s| u8::from_str_radix(s, 16).expect("Invalid hex"))
                        .collect();

                    if bytes.len() < 2 {
                        // build.rs ignores signatures shorter than 2 bytes to avoid false positives
                        continue;
                    }

                    // Use a generic extension to force Magic detection, NOT extension detection
                    // We append the original extension to the filename for debugging visibility,
                    // but end with .bin or .tmp
                    let safe_ext = if ext.starts_with('.') {
                        &ext[1..]
                    } else {
                        &ext
                    };
                    let filename = format!("{}_{}_{}.bin", category, safe_ext, i);
                    let file_path = test_dir.join(&filename);

                    let offset = sig.offset.unwrap_or(0);

                    // Create file
                    {
                        let mut file = fs::File::create(&file_path).expect("Failed to create file");
                        if offset > 0 {
                            file.seek(SeekFrom::Start(offset as u64)).unwrap();
                        }
                        file.write_all(&bytes).unwrap();
                    }

                    // Test
                    let result = analyze(&file_path);

                    if result.is_none() {
                        failures.push(format!(
                            "Failed to detect {} ({}) \n  File: {:?}\n  Hex: {}\n  Offset: {}",
                            ext, sig.description, file_path, sig.hexdump, offset
                        ));
                    } else {
                        // Cleanup success cases to save space
                        let _ = fs::remove_file(&file_path);
                    }

                    total_checked += 1;
                }
            }
        }
    }

    // Cleanup dir if everything passed
    if failures.is_empty() {
        fs::remove_dir_all(&test_dir).unwrap();
    }

    if !failures.is_empty() {
        panic!(
            "{} failures out of {} checks:\n{}",
            failures.len(),
            total_checked,
            failures.join("\n\n")
        );
    }

    println!("Successfully verified {} signatures", total_checked);
}
