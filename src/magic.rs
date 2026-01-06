use crate::ArchiveInfo;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn check_magic<P: AsRef<Path>>(path: P) -> Option<ArchiveInfo> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return None,
    };

    let mut buffer = [0u8; 34000];
    let bytes_read = match file.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return None,
    };

    crate::generated::check_magic_signature(&buffer[..bytes_read])
}
