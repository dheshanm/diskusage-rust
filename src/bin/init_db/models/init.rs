use db;

/// Drops all tables in the database.
/// 
/// * `pool` - A reference to a sqlx::PgPool.
/// * `debug` - A boolean value. If True, log the queries (does not execute them).
/// 
/// Returns a Result containing unit or an sqlx::Error.
pub async fn drop_all(pool: &sqlx::PgPool, debug: bool) -> Result<(), sqlx::Error> {
    let drop_user_table = r#"
        DROP TABLE IF EXISTS users;
    "#;

    let drop_directory_table = r#"
        DROP TABLE IF EXISTS directories;
    "#;

    let drop_file_table = r#"
        DROP TABLE IF EXISTS files;
    "#;

    let drop_file_structure_table = r#"
        DROP TABLE IF EXISTS dir_structure;
    "#;

    let drop_queries = vec![
        drop_file_structure_table,
        drop_file_table,
        drop_directory_table,
        drop_user_table,
    ];

    db::execute_queries::as_transaction(pool, drop_queries, debug).await?;

    Ok(())
}

/// Initializes the database with the necessary tables.
/// 
/// * `pool` - A reference to a sqlx::PgPool.
/// * `debug` - A boolean value. If True, log the queries (does not execute them).
/// 
/// Returns a Result containing unit or an sqlx::Error.
pub async fn initialize(pool: &sqlx::PgPool, debug: bool) -> Result<(), sqlx::Error> {
    let create_user_table = r#"
        CREATE TABLE users (
            user_id INT PRIMARY KEY,
            username TEXT
        );
    "#;

    let create_directory_table = r#"
        CREATE TABLE directories (
            directory_id TEXT PRIMARY KEY,
            owner_id INT,
            parent_id TEXT,
            FOREIGN KEY (owner_id) REFERENCES users(user_id),
            FOREIGN KEY (parent_id) REFERENCES directories(directory_id)
        );
    "#;

    let create_directory_owner_index = r#"
        CREATE INDEX directory_owner_id ON directories(owner_id);
    "#;

    let create_directory_parent_index = r#"
        CREATE INDEX directory_parent_id ON directories(parent_id);
    "#;

    let create_file_table = r#"
        CREATE TABLE files (
            file_id TEXT PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            size BIGINT NOT NULL,
            owner_id INT,
            directory_id TEXT,
            FOREIGN KEY (owner_id) REFERENCES users(user_id),
            FOREIGN KEY (directory_id) REFERENCES directories(directory_id)
        );
    "#;

    let create_file_directory_index = r#"
        CREATE INDEX file_directory_id ON files(directory_id);
    "#;

    let create_file_structure_table = r#"
        CREATE TABLE dir_structure (
            parent_dir_id TEXT,
            child_dir_id TEXT,
            FOREIGN KEY (parent_dir_id) REFERENCES directories(directory_id),
            FOREIGN KEY (child_dir_id) REFERENCES directories(directory_id),
            PRIMARY KEY (parent_dir_id, child_dir_id)
        );
    "#;

    let create_file_structure_parent_index = r#"
        CREATE INDEX file_structure_parent_id ON dir_structure(parent_dir_id);
    "#;

    let init_queries = vec![
        create_user_table,
        create_directory_table,
        create_directory_owner_index,
        create_directory_parent_index,
        create_file_table,
        create_file_directory_index,
        create_file_structure_table,
        create_file_structure_parent_index,
    ];

    db::execute_queries::as_transaction(pool, init_queries, debug).await?;

    Ok(())
}