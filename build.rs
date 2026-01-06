use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("extensions.rs");

    println!("cargo:rerun-if-changed=archive_signatures.yaml");

    let content = fs::read_to_string("archive_signatures.yaml")
        .expect("Failed to read archive_signatures.yaml");

    // Categories Mapping
    let category_map: HashMap<&str, &str> = [
        ("archiveOnly", "archive/storage"),
        ("compressOnly", "archive/stream-compression"),
        ("archiveAndCompress", "archive/compressed-archive"),
        ("dataRecovery", "archive/recovery"),
        ("packaging", "archive/package"),
        ("containers", "archive/container"),
    ]
    .iter()
    .cloned()
    .collect();

    #[derive(Clone)]
    struct SigData {
        bytes: Vec<u8>,
        offset: usize,
        description: String,
        hexdump_str: String,
    }

    struct Entry {
        ext: String,
        category_mime: String,
        signatures: Vec<SigData>,
    }

    let mut entries: Vec<Entry> = Vec::new();

    let mut current_category_mime = String::new();
    let mut current_ext = String::new();

    struct PendingSig {
        bytes: Option<Vec<u8>>,
        offset: usize,
        description: String,
        hexdump_str: String,
    }
    let mut current_sig: Option<PendingSig> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.ends_with(':')
            && !line.starts_with("hexdump")
            && !line.starts_with("offset")
            && !line.starts_with("description")
            && !line.starts_with("- description")
        {
            // Push pending
            if let Some(s) = current_sig.take()
                && let Some(bytes) = s.bytes
                && bytes.len() >= 2
            {
                if let Some(entry) = entries
                    .iter_mut()
                    .find(|e| e.ext == current_ext && e.category_mime == current_category_mime)
                {
                    entry.signatures.push(SigData {
                        bytes,
                        offset: s.offset,
                        description: s.description,
                        hexdump_str: s.hexdump_str,
                    });
                } else {
                    entries.push(Entry {
                        ext: current_ext.clone(),
                        category_mime: current_category_mime.clone(),
                        signatures: vec![SigData {
                            bytes,
                            offset: s.offset,
                            description: s.description,
                            hexdump_str: s.hexdump_str,
                        }],
                    });
                }
            }

            let key = line.trim_end_matches(':');
            if let Some(mime) = category_map.get(key) {
                current_category_mime = mime.to_string();
            } else if key.starts_with('.') {
                current_ext = key.to_string();
            }
        } else if line.starts_with("- description:") {
            // Push pending
            if let Some(s) = current_sig.take()
                && let Some(bytes) = s.bytes
                && bytes.len() >= 2
            {
                if let Some(entry) = entries
                    .iter_mut()
                    .find(|e| e.ext == current_ext && e.category_mime == current_category_mime)
                {
                    entry.signatures.push(SigData {
                        bytes,
                        offset: s.offset,
                        description: s.description,
                        hexdump_str: s.hexdump_str,
                    });
                } else {
                    entries.push(Entry {
                        ext: current_ext.clone(),
                        category_mime: current_category_mime.clone(),
                        signatures: vec![SigData {
                            bytes,
                            offset: s.offset,
                            description: s.description,
                            hexdump_str: s.hexdump_str,
                        }],
                    });
                }
            }

            let desc = line
                .trim_start_matches("- description:")
                .trim()
                .trim_matches('"')
                .to_string();
            let desc_escaped = desc.replace("\"", "\\\"");

            current_sig = Some(PendingSig {
                bytes: None,
                offset: 0,
                description: desc_escaped,
                hexdump_str: String::new(),
            });
        } else if line.starts_with("hexdump:") {
            if let Some(s) = current_sig.as_mut()
                && let Some(start) = line.find('"')
                && let Some(end) = line.rfind('"')
                && end > start
            {
                let hex_str = &line[start + 1..end];
                s.hexdump_str = hex_str.to_string();
                let bytes: Vec<u8> = hex_str
                    .split_whitespace()
                    .filter_map(|s| u8::from_str_radix(s, 16).ok())
                    .collect();
                s.bytes = Some(bytes);
            }
        } else if line.starts_with("offset:")
            && let Some(s) = current_sig.as_mut()
            && let Some(val_str) = line.strip_prefix("offset:").map(|s| s.trim())
            && let Ok(val) = val_str.parse::<usize>()
        {
            s.offset = val;
        }
    }
    // Push final
    if let Some(s) = current_sig.take()
        && let Some(bytes) = s.bytes
        && bytes.len() >= 2
    {
        if let Some(entry) = entries
            .iter_mut()
            .find(|e| e.ext == current_ext && e.category_mime == current_category_mime)
        {
            entry.signatures.push(SigData {
                bytes,
                offset: s.offset,
                description: s.description,
                hexdump_str: s.hexdump_str,
            });
        } else {
            entries.push(Entry {
                ext: current_ext.clone(),
                category_mime: current_category_mime.clone(),
                signatures: vec![SigData {
                    bytes,
                    offset: s.offset,
                    description: s.description,
                    hexdump_str: s.hexdump_str,
                }],
            });
        }
    }

    // --- Generate Code ---
    let mut output = String::new();
    // VERSION 2
    output.push_str("pub struct ArchiveInfo {");
    output.push('\n');
    output.push_str("    pub category: &'static str,");
    output.push('\n');
    output.push_str("    pub description: &'static str,");
    output.push('\n');
    output.push_str("    pub hexdump: &'static str,");
    output.push('\n');
    output.push('}');
    output.push('\n');
    output.push('\n');

    let priority = [
        "archive/package",
        "archive/recovery",
        "archive/compressed-archive",
        "archive/stream-compression",
        "archive/storage",
        "archive/container",
    ];

    let mut ext_to_info: HashMap<String, (String, String)> = HashMap::new();
    for entry in &entries {
        let desc = if let Some(sig) = entry.signatures.first() {
            sig.description.clone()
        } else {
            format!("{} archive", entry.ext)
        };

        let ext_lower = entry.ext.to_lowercase();

        if let Some((existing_mime, _)) = ext_to_info.get(&ext_lower) {
            let old_p = priority
                .iter()
                .position(|&p| p == existing_mime)
                .unwrap_or(999);
            let new_p = priority
                .iter()
                .position(|&p| p == entry.category_mime)
                .unwrap_or(999);
            if new_p < old_p {
                ext_to_info.insert(ext_lower, (entry.category_mime.clone(), desc));
            }
        } else {
            ext_to_info.insert(ext_lower, (entry.category_mime.clone(), desc));
        }
    }

    output.push_str("pub fn get_extension_info(ext: &str) -> Option<ArchiveInfo> {");
    output.push('\n');
    output.push_str("    match ext.to_lowercase().as_str() {");
    output.push('\n');

    let mut sorted_exts: Vec<_> = ext_to_info.keys().collect();
    sorted_exts.sort();

    for ext in sorted_exts {
        let (mime, desc) = &ext_to_info[ext];
        // Ensure desc doesn't break string literal
        output.push_str(&format!("        \"{}\" => Some(ArchiveInfo {{ category: \"{}\", description: \"{}\", hexdump: \"\" }}),\n", ext, mime, desc));
    }

    output.push_str("        _ => None,");
    output.push('\n');
    output.push_str("    }");
    output.push('\n');
    output.push('}');
    output.push('\n');
    output.push('\n');

    struct SigMatch {
        bytes: Vec<u8>,
        offset: usize,
        mime: String,
        description: String,
        hexdump_str: String,
    }

    let mut all_signatures: Vec<SigMatch> = Vec::new();
    for entry in &entries {
        for sig in &entry.signatures {
            all_signatures.push(SigMatch {
                bytes: sig.bytes.clone(),
                offset: sig.offset,
                mime: entry.category_mime.clone(),
                description: sig.description.clone(),
                hexdump_str: sig.hexdump_str.clone(),
            });
        }
    }

    all_signatures.sort_by(|a, b| b.bytes.len().cmp(&a.bytes.len()));

    output.push_str("pub fn check_magic_signature(data: &[u8]) -> Option<ArchiveInfo> {");
    output.push('\n');

    for sig in all_signatures {
        output.push_str(&format!(
            "    if data.len() >= {} && ",
            sig.offset + sig.bytes.len()
        ));

        if sig.offset == 0 {
            output.push_str("data.starts_with(&[");
        } else {
            output.push_str(&format!("data[{}..].starts_with(&[", sig.offset));
        }

        for (i, byte) in sig.bytes.iter().enumerate() {
            if i > 0 {
                output.push_str(", ");
            }
            output.push_str(&format!("0x{:02X}", byte));
        }
        output.push_str("]){ ");
        output.push('\n');
        output.push_str("        return Some(ArchiveInfo {");
        output.push('\n');
        // Explicitly adding commas
        output.push_str(&format!("            category: \"{}\",", sig.mime));
        output.push('\n');
        output.push_str(&format!(
            "            description: \"{}\",",
            sig.description
        ));
        output.push('\n');
        output.push_str(&format!("            hexdump: \"{}\"", sig.hexdump_str));
        output.push('\n');
        output.push_str("        });");
        output.push('\n');
        output.push_str("    }");
        output.push('\n');
    }

    output.push_str("    None");
    output.push('\n');
    output.push('}');
    output.push('\n');

    fs::write(&dest_path, output).unwrap();
}
