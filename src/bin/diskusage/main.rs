mod models;
mod filesystem;
mod users;

use clap::Parser;
use rayon::prelude::*;

#[derive(clap::Parser, Default, Debug)]
#[clap(
    author = "Dheshan Mohandass",
    version,
    about
)]
/// A CLI tool for tracking disk usage.
struct Arguments {
    /// The root directory to track.
    #[clap(short, long)]
    root_dir: String,  
    /// Enable debug mode.
    #[clap(short, long)]
    debug: bool,   
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    let args = Arguments::parse();
    log::info!("{:?}", args);

    let root_dir = args.root_dir;

    // Check if root directory exists
    if !std::path::Path::new(&root_dir).exists() {
        log::error!("Root directory does not exist: {}", root_dir);
        return Err("Root directory does not exist".into());
    }

    walkdir::WalkDir::new(root_dir)
        .into_iter()
        .par_bridge() // Allows rayon to process entries in parallel
        .filter_map(|entry| entry.ok())
        .for_each(|entry| {
            // Check if file or directory
            if entry.file_type().is_dir() {
                let dir_path = entry.path();
                let owner = filesystem::fetch::owner(dir_path).unwrap_or_default();
                let parent_dir = dir_path.parent().unwrap_or(std::path::Path::new("/") );
                
                let directory: models::definitions::Directory = models::definitions::Directory {
                    directory_id: dir_path.to_string_lossy().to_string(),
                    owner_id: owner as i32,
                    parent_id: Some(parent_dir.to_string_lossy().to_string()),
                };

                log::info!("{:?}", directory);
            } else {
                let file_path = entry.path();
                let owner = filesystem::fetch::owner(file_path).unwrap_or_default();
                let file_size = filesystem::fetch::file_size(file_path).unwrap_or_default();

                let parent_dir = file_path.parent().unwrap_or(std::path::Path::new("/") );
                let file: models::definitions::File = models::definitions::File {
                    file_id: file_path.to_string_lossy().to_string(),
                    name: file_path.file_name().unwrap().to_string_lossy().to_string(),
                    size: file_size as i64,
                    owner_id: owner as i32,
                    directory_id: parent_dir.to_string_lossy().to_string(),
                };

                log::info!("{:?}", file);
            }
        });

    Ok(())
}
