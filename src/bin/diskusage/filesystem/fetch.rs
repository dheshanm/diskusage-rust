use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// Retrieves the metadata for the given file or directory.
///
/// * `path` - The path to the file or directory.
///
/// Returns
/// The size of the file or directory in bytes.
pub fn file_size(path: &Path) -> Result<u64, std::io::Error> {
    let metadata = std::fs::metadata(path);
    match metadata {
        Ok(metadata) => Ok(metadata.len()),
        Err(e) => {
            log::error!("Error getting file size: {:?}", e);
            Err(e)
        }
    }
}

/// Get the owner of a file or directory.
///
/// * `path` - The path to the file.
///
/// Returns
/// The user ID of the owner of the file.
pub fn owner(path: &Path) -> Option<u32> {
    let metadata = std::fs::metadata(path);
    match metadata {
        Ok(metadata) => Some(metadata.uid()),
        Err(_) => {
            log::error!("Error getting owner of file: {:?}", path);
            None
        }
    }
}

/// Get the last modified time of a file or directory.
///
/// * `path` - The path to the file.
///
/// Returns
/// The last modified time of the file in chrono::DateTime<chrono::Utc>.
pub fn last_modified(path: &Path) -> Result<chrono::NaiveDateTime, std::io::Error> {
    let metadata = std::fs::metadata(path);
    match metadata {
        Ok(metadata) => {
            let modified = metadata.modified();
            match modified {
                Ok(modified) => {
                    let modified = modified.duration_since(std::time::UNIX_EPOCH).unwrap();
                    let modified = std::time::UNIX_EPOCH + modified;
                    Ok(chrono::DateTime::<chrono::Utc>::from(modified).naive_utc())
                }
                Err(e) => {
                    log::error!("Error getting modified time: {:?}", e);
                    Err(e)
                }
            }
        }
        Err(e) => Err(e),
    }
}
