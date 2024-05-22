use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// Retrieves the metadata for the given file or directory.
///
/// * `path` - The path to the file or directory.
///
/// Returns
/// The size of the file or directory in bytes.
pub fn file_size(path: &Path) -> Result<u64, std::io::Error> {
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.len())
}

/// Get the owner of a file
///
/// * `path` - The path to the file.
///
/// Returns
/// The user ID of the owner of the file.
pub fn file_owner(path: &Path) -> Option<u32> {
    let metadata = std::fs::metadata(path).ok()?;
    Some(metadata.uid())
}
