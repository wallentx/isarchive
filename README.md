# isarchive

A lightweight Rust library and CLI tool to detect archive types based on magic signatures and file extensions.

## Why `isarchive`?

Standard tools like `file` are excellent for general purpose identification, but they often lack:
1.  **Programmatic simplicity:** Easily checking if a file is an archive and getting a structured result.
2.  **Granular archive details:** Specificity for niche game data archives, compression streams, and container formats.
3.  **Comprehensive category mapping:** Distinguishing between a "package" (like `.deb`), a "compressed archive" (like `.7z`), and a "stream" (like `.zst`).

`isarchive` was born from the need for a tool that answers one simple question reliably: *"Is this an archive, and if so, exactly what kind?"*

## Features

- **Signature-First Detection:** Validates files using a database of over 360 magic number signatures.
- **Intelligent Fallback:** Uses extension matching when magic signatures are missing or the file is empty.
- **Structured Categories:** Maps results to MIME-like categories (e.g., `archive/storage`, `archive/package`).
- **Zero-Dependency Core:** The library is extremely lightweight and compiles quickly.

## CLI Usage

Install and run the tool to identify files:

```bash
# Basic usage
isarchive my_file.zip

# Brief output (just the category)
isarchive --brief my_file.zip

# MIME style output
isarchive --mime my_file.zip
```

### Output Example
```text
my_file.zip: ZIP compressed archive
Type: archive/compressed-archive
```

## Library Usage

Add `isarchive` to your `Cargo.toml`:

```toml
[dependencies]
isarchive = { git = "https://github.com/wallentx/isarchive.git" }
```

Use the `analyze` function in your code:

```rust
use isarchive::analyze;
use std::path::Path;

fn main() {
    let path = Path::new("backup.tar.gz");
    
    if let Some(info) = analyze(path) {
        println!("Detected: {}", info.description);
        println!("Category: {}", info.category);
    } else {
        println!("Not a recognized archive format.");
    }
}
```

## How it Works

The project uses a `build.rs` script to compile the `archive_signatures.yaml` into highly efficient, hard-coded matching logic at compile time. This ensures that lookups are nearly instantaneous and the resulting binary is self-contained.

## License

MIT
