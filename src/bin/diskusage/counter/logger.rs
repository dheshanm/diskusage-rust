use crate::models;
use crate::models::definitions::DbModel;

/// A function to log the disk usage statistics during runtime.
/// Helps track the number of files and directories parsed per second.
/// 
/// Arguments
/// * `handle` - A tokio runtime handle to run the async functions.
/// * `pool` - A sqlx database pool to query the database.
pub async fn logger_thread(handle: tokio::runtime::Handle, pool: std::sync::Arc<sqlx::Pool<sqlx::Postgres>>) {
    std::thread::spawn(move || {
        let log_frequency = match std::env::var("DISK_USAGE_LOG_FREQUENCY") {
            Ok(frequency) => {
                match frequency.parse::<u64>() {
                    Ok(frequency) => {
                        log::info!("Found log frequency from environment: {} seconds", frequency);
                        std::time::Duration::from_secs(frequency)
                    }
                    Err(_) => {
                        std::time::Duration::from_secs(300)
                    }
                }
            }
            Err(_) => {
                std::time::Duration::from_secs(300)
            }
        };
        log::info!("Logging thread: Active");
        log::info!("Logging frequency: {:?}", log_frequency);
        let mut files_counter = 0;
        let mut directories_counter = 0;
        loop {
            let files_count = handle.block_on(async {
                let result = models::definitions::File::count_all(&pool).await.unwrap_or_default();
                result
            });

            let directories_count = handle.block_on(async {
                let result = models::definitions::Directory::count_all(&pool).await.unwrap_or_default();
                result
            });

            let files_count_diff = files_count - files_counter;
            let directories_count_diff = directories_count - directories_counter;

            files_counter = files_count;
            directories_counter = directories_count;

            let files_per_second = files_count_diff as f64 / log_frequency.as_secs() as f64;
            let directories_per_second = directories_count_diff as f64 / log_frequency.as_secs() as f64;

            log::info!("Parsed {} files ({} new) and {} directories ({} new): {:.2} files/s, {:.2} directories/s", files_count, files_count_diff, directories_count, directories_count_diff, files_per_second, directories_per_second);
            std::thread::sleep(log_frequency);
        }
    });
}
