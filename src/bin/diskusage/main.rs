mod models;
mod filesystem;
mod users;

use clap::Parser;
use rayon::prelude::*;
use crate::models::definitions::DbModel;

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

    // get the database url from the environment
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            log::error!("DATABASE_URL environment variable not set.");
            return Err("DATABASE_URL environment variable not set.".into());
        }
    };

    let pool_result = sqlx::postgres::PgPoolOptions::new().connect(&database_url).await;
    let handle = tokio::runtime::Handle::current();

    let pool = match pool_result {
        Ok(pool) => {
            log::info!("Connected to database: {}", database_url);
            pool
        }
        Err(e) => {
            log::error!("Failed to connect to database: {}", e);
            return Err(e.into());
        }
    };

    walkdir::WalkDir::new(root_dir)
        .into_iter()
        .par_bridge() // Allows rayon to process entries in parallel
        .filter_map(|entry| entry.ok())
        .for_each(|entry| {
            // Check if file or directory
            if entry.file_type().is_dir() {
                let dir_path = entry.path();
                let owner: Option<u32> = filesystem::fetch::owner(dir_path);
                let owner_i32: Option<i32> = owner.map(|x| x as i32);
                let parent_dir = dir_path.parent().unwrap_or(std::path::Path::new("/") );

                let handle = handle.clone();
                let select_user_where_clause = format!("WHERE user_id = {}", owner_i32.unwrap_or_default());
                let pool_c = pool.clone();
                handle.block_on(async move {
                    let user = models::definitions::User::select_where(&pool_c, &select_user_where_clause).await.unwrap_or_default();
                    if user.is_empty() {
                        let user = models::definitions::User {
                            user_id: owner_i32.unwrap_or_default(),
                            username: users::username::get_username(owner.unwrap_or_default())
                        };
                        user.insert(&pool_c).await.unwrap_or_default();
                    }
                });
                
                let directory: models::definitions::Directory = models::definitions::Directory {
                    directory_id: dir_path.to_string_lossy().to_string(),
                    owner_id: owner_i32,
                    parent_id: Some(parent_dir.to_string_lossy().to_string()),
                };

                log::info!("directory: {:?}", directory);
            } else {
                let file_path = entry.path();
                let owner: Option<u32> = filesystem::fetch::owner(file_path);
                let owner_i32: Option<i32> = owner.map(|x| x as i32);
                let file_size = filesystem::fetch::file_size(file_path).unwrap_or_default();

                let parent_dir = file_path.parent().unwrap_or(std::path::Path::new("/") );
                let file: models::definitions::File = models::definitions::File {
                    file_id: file_path.to_string_lossy().to_string(),
                    name: file_path.file_name().unwrap().to_string_lossy().to_string(),
                    size: file_size as i64,
                    owner_id: owner_i32.unwrap_or_default(),
                    directory_id: parent_dir.to_string_lossy().to_string(),
                };

                log::info!("file: {:?}", file);
            }
        });

    Ok(())
}
