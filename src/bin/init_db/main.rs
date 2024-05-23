mod models;

use clap::Parser;

#[derive(clap::Parser, Default, Debug)]
#[clap(
    author = "Dheshan Mohandass",
    version,
    about
)]
/// A companion tool for the disk usage tracker to initialize the database.
struct Arguments {
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
    println!("{:?}", args);

    let debug = args.debug;

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

    // let test_query = vec![
    //     "SELECT 'TEXT' AS result, 1 + 1 AS sum",
    //     "SELECT 'TEXT_2' AS result, 2 + 2 AS sum"
    // ];
    // db::execute_queries::as_transaction(&pool, test_query.clone(), false).await?;
    // let _ = tokio::task::spawn_blocking(move || {
    //     let result = db::execute_queries::returning_df(database_url, test_query);
    //     match result {
    //         Ok(result) => {
    //             for df in result {
    //                 log::info!("DataFrame: {}", df);
    //             }
    //         }
    //         Err(e) => {
    //             log::error!("Failed to return DataFrame: {}", e);
    //         }
    //     }
    // }).await?;

    log::warn!("Dropping all tables...");
    models::init::drop_all(&pool, debug).await?;
    log::warn!("Initializing database...");
    models::init::initialize(&pool, debug).await?;
    log::info!("Database initialized.");

    Ok(())
}
