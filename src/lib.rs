use std::path::Path;

pub mod magic;

// Include generated code
include!(concat!(env!("OUT_DIR"), "/extensions.rs"));

/// Analyzes the file and returns detailed archive info.
pub fn analyze<P: AsRef<Path>>(path: P) -> Option<ArchiveInfo> {
    let path = path.as_ref();

    // 1. Magic Number Check (Priority)
    if let Some(info) = magic::check_magic(path) {
        return Some(info);
    }

    // 2. Extension Check (Fallback)
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        let name_lower = name.to_lowercase();
        let indices: Vec<_> = name_lower.match_indices('.').map(|(i, _)| i).collect();

        // Try longest suffix first
        for i in indices {
            let suffix = &name_lower[i..];
            if let Some(info) = get_extension_info(suffix) {
                return Some(info);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_magic_signature_zip() {
        let zip_magic = [0x50, 0x4B, 0x03, 0x04];
        let info = check_magic_signature(&zip_magic);
        assert!(info.is_some(), "ZIP magic should be detected");
        let info = info.unwrap();
        // The category should be one of the known archive categories
        assert!(
            info.category.contains("archive"),
            "Category should contain 'archive'"
        );
        assert!(
            info.description.len() > 0,
            "Description should not be empty"
        );
    }

    #[test]
    fn test_extension_zip() {
        let info = get_extension_info(".zip");
        assert!(info.is_some(), ".zip extension should be detected");
        let info = info.unwrap();
        assert!(
            info.category.contains("archive"),
            "Category should contain 'archive'"
        );
    }

    #[test]
    fn test_extension_nonexistent() {
        let info = get_extension_info(".nonexistent_extension_xyz");
        assert!(
            info.is_none(),
            "Non-existent extension should not be detected"
        );
    }

    #[test]
    fn test_analyze_file_magic() {
        let filename = "test_magic_analyze.dat";
        {
            let mut file = File::create(filename).unwrap();
            // Write ZIP magic
            file.write_all(&[0x50, 0x4B, 0x03, 0x04]).unwrap();
        }

        let result = analyze(filename);
        std::fs::remove_file(filename).unwrap();

        assert!(result.is_some(), "File with ZIP magic should be analyzed");
        assert!(result.unwrap().category.contains("archive"));
    }

    #[test]
    fn test_analyze_file_extension() {
        let filename = "test_extension_analyze.zip";
        {
            let _file = File::create(filename).unwrap();
            // Empty file, so magic fails, extension should match
        }

        let result = analyze(filename);
        std::fs::remove_file(filename).unwrap();

        assert!(
            result.is_some(),
            "File with .zip extension should be analyzed"
        );
        assert!(result.unwrap().category.contains("archive"));
    }
}
