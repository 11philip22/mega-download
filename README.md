# mega-download

Download all files from a public MEGA folder, including subfolders. Uses [megalib](https://github.com/11philip22/megalib) (no login required).

## Usage

```bash
cargo run -- <FOLDER_URL> [OUTPUT_DIR]
```

**Examples:**

```bash
# Download to current directory
cargo run -- "https://mega.nz/folder/ABC123#key"

# Download to a specific directory
cargo run -- "https://mega.nz/folder/ABC123#key" ./downloads
```

## Requirements

- Rust 1.70+
- megalib crate (path: `../mega/megalib`)
