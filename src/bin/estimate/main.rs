use clap::Parser;
use sqlx::Row;
use sqlx::types::BigDecimal;

#[derive(clap::Parser, Default, Debug)]
#[clap(
    author = "Dheshan Mohandass",
    version,
    about
)]
/// A companion tool for the disk usage tracker to estimate a directory's size.
struct Arguments {
    /// The path to the directory to estimate.
    #[clap(short, long)]
    path: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    let args = Arguments::parse();
    log::info!("{:?}", args);

    let path = args.path;

    // get the database url from the environment
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            log::error!("DATABASE_URL environment variable not set.");
            return Err("DATABASE_URL environment variable not set.".into());
        }
    };

    let pool_result = sqlx::postgres::PgPoolOptions::new().connect(&database_url).await;

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

    let query = format!(
        r#"
        WITH RECURSIVE directory_tree AS (
            SELECT d.directory_id
            FROM directories d
            WHERE d.directory_id = '{}'

            UNION ALL
            
            SELECT d.directory_id
            FROM directories d
            INNER JOIN directory_tree dt ON d.parent_id = dt.directory_id
        )
        SELECT COALESCE(SUM(f.size), 0) / 1024 AS total_size
        FROM files f
        WHERE f.directory_id IN (SELECT directory_id FROM directory_tree)
        "#,
        path
    );

    let result = sqlx::query(&query)
        .fetch_one(&pool)
        .await?;

    // NUMERIC type
    let total_size: BigDecimal = result.try_get("total_size")?;

    log::info!("Estimated size of directory:");
    log::info!("{:.2} TB", total_size.clone() / 1024 / 1024 / 1024);
    log::info!("{:.2} GB", total_size.clone() / 1024 / 1024);
    log::info!("{:.2} MB", total_size.clone() / 1024);
    log::info!("{:.2} KB", total_size);

    Ok(())
}

