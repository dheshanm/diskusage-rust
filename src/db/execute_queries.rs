use polars::prelude::*;
use connectorx::prelude::*;
use std::{convert::TryFrom, vec};

/// Execute a list of queries in a transaction as a single transaction.
/// 
/// * `pool` - A reference to a sqlx::PgPool.
/// * `queries` - A vector of string slices.
/// * `debug` - A boolean value.
/// 
/// Returns a Result containing a Vec of DataFrames or an sqlx::Error.
pub async fn as_transaction(
    pool: &sqlx::PgPool,
    queries: Vec<&str>,
    debug: bool,
) -> Result<(), sqlx::Error> {
    let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = pool.begin().await?;

    for query in queries {
        if debug {
            log::info!("Executing query: {}", query);
        } else {
            log::info!("Executing query: {}", query);
            let result = sqlx::query(query).execute(&mut *tx).await?;
            log::info!("Query result: {:?}", result);
        }
    }

    tx.commit().await?;
    Ok(())
}

pub fn returning_df(
    db_uri: &str,
    queryies: Vec<&str>,
) -> Result<Vec<DataFrame>, PolarsError> {
    let source_conn = SourceConn::try_from(db_uri).expect("failed to create source connection");
    let mut dataframes: Vec<DataFrame> = vec![];

    for query in queryies {
        let queries = vec![ CXQuery::from(query) ];
        let destination = get_arrow2(&source_conn, None, &queries).expect("run failed");
        let df = destination.polars().unwrap();
        dataframes.push(df);
    }
    // let queries = vec![ CXQuery::from(query) ];
    // let destination = get_arrow2(&source_conn, None, &queries).expect("run failed");
    // let df = destination.polars().unwrap();

    Ok(dataframes)
}