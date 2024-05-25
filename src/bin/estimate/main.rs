use clap::Parser;
use sqlx::types::BigDecimal;
use sqlx::Row;

use db;

#[derive(clap::Parser, Default, Debug)]
#[clap(author = "Dheshan Mohandass", version, about)]
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

    let pool_result = sqlx::postgres::PgPoolOptions::new()
        .connect(&database_url)
        .await;

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

    let result = sqlx::query(&query).fetch_one(&pool).await?;

    // NUMERIC type
    let total_size: BigDecimal = result.try_get("total_size")?;
    let size_kb: BigDecimal = total_size.clone() / 1024;
    let size_mb: BigDecimal = size_kb.clone() / 1024;
    let size_gb: BigDecimal = size_mb.clone() / 1024;
    let size_tb: BigDecimal = size_gb.clone() / 1024;

    log::info!("Estimated size: {:.2} TB = {:.2} GB = {:.2} MB = {:.2} KB = {:.2} bytes", size_tb, size_gb, size_mb, size_kb, total_size);

    let largest_files_query = format!(
        r#"
        WITH RECURSIVE directory_tree AS (
            SELECT d.directory_id
            FROM directories d
            WHERE d.directory_id = '{path}'

            UNION ALL
            
            SELECT d.directory_id
            FROM directories d
            INNER JOIN directory_tree dt ON d.parent_id = dt.directory_id
        )
        SELECT f.file_id, f.size / 1024 / 1024 AS size_mb, f.owner_id, f.last_modified
        FROM files f
        WHERE f.directory_id IN (SELECT directory_id FROM directory_tree)
        ORDER BY f.size DESC
        LIMIT 5
        "#,
    );

    let _ = tokio::task::spawn_blocking(move || {
        let dfs = db::execute_queries::returning_df(&database_url, vec![&largest_files_query]);
        if let Err(e) = dfs {
            log::error!("Failed to return DataFrame: {}", e);
            return;
        }

        let dfs = dfs.unwrap();
        let df = &dfs[0];

        // get col names
        let mut col_names: Vec<&str> = vec![];
        for series in df.get_columns() {
            col_names.push(series.name());
        }

        let mut table = comfy_table::Table::new();
        table.load_preset(comfy_table::presets::UTF8_FULL);
        table.set_header(&col_names);

        // iterate over the rows
        let df_row_count = df.height();
        for i in 0..df_row_count {
            let mut row: Vec<String> = vec![];
            for series in df.get_columns() {
                let value: Result<polars::prelude::AnyValue, polars::prelude::PolarsError> = series.get(i);
                let value_str = match value {
                    Ok(value) => value.to_string(),
                    Err(_) => "".to_string(),
                };
                row.push(value_str);
            }
            table.add_row(row);
        }

        println!("Largest files in directory:");
        println!("{table}")
    }).await?;

    Ok(())
}
