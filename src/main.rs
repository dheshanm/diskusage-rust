mod db;
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    let database_url = "postgres://pipeline:piedpiper@localhost:5432/test";
    let pool_result = sqlx::postgres::PgPoolOptions::new().connect(database_url).await;

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

    let test_query = vec![
        "SELECT 'TEXT' AS result, 1 + 1 AS sum",
        "SELECT 'TEXT_2' AS result, 2 + 2 AS sum"
    ];
    db::execute_queries::as_transaction(&pool, test_query.clone(), false).await?;
    let _ = tokio::task::spawn_blocking(move || {
        let result = db::execute_queries::returning_df(database_url, test_query);
        match result {
            Ok(result) => {
                for df in result {
                    log::info!("DataFrame: {}", df);
                }
            }
            Err(e) => {
                log::error!("Failed to return DataFrame: {}", e);
            }
        }
    }).await?;

    // models::init_db::initialize(&pool, true).await?;

    Ok(())
}
