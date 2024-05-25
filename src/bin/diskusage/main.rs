mod counter;
mod filesystem;
mod models;
mod users;

use crate::models::definitions::DbModel;
use clap::Parser;
use rayon::prelude::*;

#[derive(clap::Parser, Default, Debug)]
#[clap(author = "Dheshan Mohandass", version, about)]
/// A CLI tool for tracking disk usage.
struct Arguments {
    /// The root directory to track.
    #[clap(short, long)]
    root_dir: String,
    /// Enable debug mode.
    #[clap(short, long)]
    debug: bool,
}

async fn ensure_user_exists(
    owner: Option<i32>,
    pool: std::sync::Arc<sqlx::Pool<sqlx::Postgres>>,
    cache: std::sync::Arc<dashmap::DashSet<i32>>,
) -> Result<(), sqlx::Error> {
    if let Some(owner_id) = owner {
        // Check if the user exists in the cache
        if let Some(_) = cache.get(&owner_id) {
            return Ok(());
        }
        let select_user_where_clause = format!("WHERE user_id = {}", owner_id);
        let user = models::definitions::User::select_where(&pool, &select_user_where_clause)
            .await
            .unwrap_or_default();

        if user.is_empty() {
            let user = models::definitions::User {
                user_id: owner_id,
                username: users::username::get_username(owner_id as u32),
            };
            user.insert(&pool).await?;
        }

        cache.insert(owner_id);
    }
    Ok(())
}

fn process_directory(
    entry: walkdir::DirEntry,
    pool: std::sync::Arc<sqlx::Pool<sqlx::Postgres>>,
    handle: tokio::runtime::Handle,
    cache: std::sync::Arc<dashmap::DashSet<i32>>,
) {
    let dir_path = entry.path();
    let owner = filesystem::fetch::owner(dir_path).map(|x| x as i32);
    let parent_dir = dir_path.parent().unwrap_or(std::path::Path::new("/"));

    let directory = models::definitions::Directory {
        directory_id: dir_path.to_string_lossy().to_string(),
        owner_id: owner,
        parent_id: Some(parent_dir.to_string_lossy().to_string()),
    };

    handle.block_on(async move {
        if let Err(e) = ensure_user_exists(owner, pool.clone(), cache.clone()).await {
            log::error!("Failed to insert user: {:?}", e);
            return;
        }
        if let Err(e) = directory.insert(&pool).await {
            log::error!("Error inserting directory: {:?}", e);
        }
    });
}

fn process_file(
    entry: walkdir::DirEntry,
    pool: std::sync::Arc<sqlx::Pool<sqlx::Postgres>>,
    handle: tokio::runtime::Handle,
    cache: std::sync::Arc<dashmap::DashSet<i32>>,
) {
    let file_path = entry.path();
    let owner = filesystem::fetch::owner(file_path).map(|x| x as i32);
    let file_size = filesystem::fetch::file_size(file_path).unwrap_or_default();
    let last_modified = filesystem::fetch::last_modified(file_path).ok();
    let parent_dir = file_path.parent().unwrap_or(std::path::Path::new("/"));

    let file = models::definitions::File {
        file_id: file_path.to_string_lossy().to_string(),
        name: file_path.file_name().unwrap().to_string_lossy().to_string(),
        size: file_size as i64,
        owner_id: owner,
        directory_id: parent_dir.to_string_lossy().to_string(),
        last_modified,
    };

    handle.block_on(async move {
        if let Err(e) = ensure_user_exists(owner, pool.clone(), cache.clone()).await {
            log::error!("Failed to insert user: {:?}", e);
            return;
        }

        loop {
            let result = file.insert(&pool).await;
            // If PgDatabaseError is returned, retry the insert
            if let Err(sqlx::Error::Database(_)) = result {
                // sleep for random time between 1 and 5 seconds
                let sleep_time = rand::random::<u64>() % 5 + 1;
                tokio::time::sleep(tokio::time::Duration::from_secs(sleep_time)).await;
                continue;
            }
            break;
        }

        if let Err(e) = file.insert(&pool).await {
            log::error!("Error inserting file: {:?}", e);
        }
    });
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

    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect(&database_url)
        .await
        .map_err(|e| {
            log::error!("Failed to connect to database: {}", e);
            e
        })?;

    log::info!("Connected to database: {}", database_url);
    let pool = std::sync::Arc::new(pool);
    let handle: tokio::runtime::Handle = tokio::runtime::Handle::current();

    // Spawn a thread to query files processed periodically
    let pool_c = std::sync::Arc::clone(&pool);
    let handle_c: tokio::runtime::Handle = handle.clone();
    counter::logger::logger_thread(handle_c, pool_c).await;

    let cache: std::sync::Arc<dashmap::DashSet<i32>> = std::sync::Arc::new(dashmap::DashSet::new());

    log::info!("Starting disk usage tracking for: {}", root_dir);
    walkdir::WalkDir::new(root_dir)
        .into_iter()
        .par_bridge() // Allows rayon to process entries in parallel
        .filter_map(|entry| entry.ok())
        .for_each(|entry| {
            let pool = std::sync::Arc::clone(&pool);
            let handle = handle.clone();
            if entry.file_type().is_dir() {
                process_directory(entry, pool, handle, cache.clone());
            } else if entry.file_type().is_file() {
                process_file(entry, pool, handle, cache.clone());
            }
        });

    Ok(())
}
