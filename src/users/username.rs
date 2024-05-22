//// Get uid to username mapping
///
/// * `uid` - The user ID.
///
/// Returns
/// The username of the user.
pub fn get_username(uid: u32) -> Option<String> {
    let user = users::get_user_by_uid(uid)?;
    Some(user.name().to_string_lossy().to_string())
}
