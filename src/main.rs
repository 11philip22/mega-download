    //! Download all files from a public MEGA folder (including subfolders).
    //!
    //! Usage:
    //!   mega-download <FOLDER_URL> [OUTPUT_DIR]

    use std::path::Path;

    use clap::Parser;
    use megalib::public::open_folder;
    use tracing_subscriber::{fmt, EnvFilter};

    #[derive(Debug, Parser)]
    #[command(name = "mega-download")]
    struct Args {
        /// MEGA folder URL (e.g. https://mega.nz/folder/ABC123#key)
        folder_url: String,

        /// Output directory (default: current directory)
        #[arg(default_value = ".")]
        output_dir: String,
    }

    fn init_tracing() {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("off"));
        fmt().with_env_filter(filter).with_target(false).init();
    }

    /// Build relative path from node path by stripping the root folder prefix.
    /// Rejects path traversal (..) for security.
    fn rel_path_from_node<'a>(node_path: &'a str, root_path: &str) -> Result<&'a str, Box<dyn std::error::Error>> {
        let root_trimmed = root_path.trim_end_matches('/');
        let remainder = node_path
            .strip_prefix(root_trimmed)
            .ok_or_else(|| format!("Node path {} does not start with root {}", node_path, root_path))?
            .trim_start_matches('/');

        if remainder.contains("..") {
            return Err("Path traversal (..) not allowed".into());
        }
        Ok(remainder)
    }

    #[tokio::main]
    async fn main() -> Result<(), Box<dyn std::error::Error>> {
        init_tracing();
        let args = Args::parse();

        println!("Opening public folder...");
        let folder = open_folder(&args.folder_url).await?;

        let root = folder.nodes().first().ok_or("Empty folder")?;
        let root_path = root.path().unwrap_or("/");
        println!("Root: {}\n", root.name);

        let output_dir = Path::new(&args.output_dir);
        std::fs::create_dir_all(output_dir)?;

        let nodes = folder.list(root_path, true);
        let file_count = nodes.iter().filter(|n| n.is_file()).count();
        println!("Downloading {} files...\n", file_count);

        for node in nodes {
            if !node.is_file() {
                continue;
            }

            let node_path = node
                .path()
                .ok_or_else(|| format!("Node {} has no path", node.name))?;

            let rel_path = rel_path_from_node(node_path, root_path)?;
            if rel_path.is_empty() {
                continue; // root itself, skip
            }

            let local_path = output_dir.join(rel_path);
            if let Some(parent) = local_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Skip if file already exists at this path
            if local_path.is_file() {
                println!("Skipped (already exists): {}", local_path.display());
                continue;
            }

            let mut file = std::fs::File::create(&local_path)?;
            folder.download(node, &mut file).await?;
            println!("Downloaded: {}", local_path.display());
        }

        println!("\nDone.");
        Ok(())
    }
