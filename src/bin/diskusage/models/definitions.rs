use serde::Deserialize;
use serde::Serialize;
use sqlx::Row;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct User {
    pub user_id: i32,
    pub username: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Directory {
    pub directory_id: String,
    pub owner_id: Option<i32>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct File {
    pub file_id: String,
    pub name: String,
    pub size: i64,
    pub owner_id: Option<i32>,
    pub directory_id: String,
    pub last_modified: Option<chrono::NaiveDateTime>,
}

#[async_trait::async_trait]
pub trait DbModel {
    async fn insert(&self, pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<(), sqlx::Error>;
    async fn update(&self, pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<(), sqlx::Error>;
    async fn delete(&self, pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<(), sqlx::Error>;
    async fn select(
        &self,
        pool: &sqlx::Pool<sqlx::postgres::Postgres>,
    ) -> Result<Box<Self>, sqlx::Error>;
    async fn select_all(
        pool: &sqlx::Pool<sqlx::postgres::Postgres>,
    ) -> Result<Vec<Box<Self>>, sqlx::Error>;
    async fn select_where(
        pool: &sqlx::Pool<sqlx::postgres::Postgres>,
        where_clause: &str,
    ) -> Result<Vec<Box<Self>>, sqlx::Error>;
    async fn count_all(pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<i64, sqlx::Error>;
}

#[async_trait::async_trait]
pub trait DbEstimateRow {
    async fn estimate_count(
        pool: &sqlx::Pool<sqlx::postgres::Postgres>,
    ) -> Result<i64, sqlx::Error>;
}

#[async_trait::async_trait]
impl DbModel for User {
    async fn insert(&self, pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO users (user_id, username) VALUES ($1, $2) ON CONFLICT (user_id) DO UPDATE SET username = $2",
            self.user_id,
            self.username
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn update(&self, pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE users SET username = $2 WHERE user_id = $1",
            self.user_id,
            self.username
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM users WHERE user_id = $1", self.user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn select(
        &self,
        pool: &sqlx::Pool<sqlx::postgres::Postgres>,
    ) -> Result<Box<Self>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            "SELECT user_id, username FROM users WHERE user_id = $1",
            self.user_id
        )
        .fetch_one(pool)
        .await?;
        Ok(Box::new(user))
    }

    async fn select_all(
        pool: &sqlx::Pool<sqlx::postgres::Postgres>,
    ) -> Result<Vec<Box<Self>>, sqlx::Error> {
        let users = sqlx::query_as!(User, "SELECT user_id, username FROM users")
            .fetch_all(pool)
            .await?;
        Ok(users.into_iter().map(|u| Box::new(u)).collect())
    }

    async fn select_where(
        pool: &sqlx::Pool<sqlx::postgres::Postgres>,
        where_clause: &str,
    ) -> Result<Vec<Box<User>>, sqlx::Error> {
        let query_string = format!("SELECT user_id, username FROM users WHERE {}", where_clause);
        let rows = sqlx::query(&query_string).fetch_all(pool).await?;

        let users: Vec<Box<User>> = rows
            .into_iter()
            .map(|row| {
                Box::new(User {
                    user_id: row.get("user_id"),
                    username: row.get("username"),
                })
            })
            .collect();
        Ok(users)
    }

    async fn count_all(pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<i64, sqlx::Error> {
        let count = sqlx::query("SELECT COUNT(*) FROM users")
            .fetch_one(pool)
            .await?;
        Ok(count.get("count"))
    }
}

#[async_trait::async_trait]
impl DbModel for Directory {
    async fn insert(&self, pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO directories (directory_id, owner_id, parent_id) VALUES ($1, $2, $3) ON CONFLICT (directory_id) DO UPDATE SET owner_id = $2, parent_id = $3",
            self.directory_id,
            self.owner_id,
            self.parent_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn update(&self, pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE directories SET owner_id = $2, parent_id = $3 WHERE directory_id = $1",
            self.directory_id,
            self.owner_id,
            self.parent_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM directories WHERE directory_id = $1",
            self.directory_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn select(
        &self,
        pool: &sqlx::Pool<sqlx::postgres::Postgres>,
    ) -> Result<Box<Self>, sqlx::Error> {
        let directory = sqlx::query_as!(
            Directory,
            "SELECT directory_id, owner_id, parent_id FROM directories WHERE directory_id = $1",
            self.directory_id
        )
        .fetch_one(pool)
        .await?;
        Ok(Box::new(directory))
    }

    async fn select_all(
        pool: &sqlx::Pool<sqlx::postgres::Postgres>,
    ) -> Result<Vec<Box<Self>>, sqlx::Error> {
        let directories = sqlx::query_as!(
            Directory,
            "SELECT directory_id, owner_id, parent_id FROM directories"
        )
        .fetch_all(pool)
        .await?;
        Ok(directories.into_iter().map(|d| Box::new(d)).collect())
    }

    async fn select_where(
        pool: &sqlx::Pool<sqlx::postgres::Postgres>,
        where_clause: &str,
    ) -> Result<Vec<Box<Directory>>, sqlx::Error> {
        let query_string = format!(
            "SELECT directory_id, owner_id, parent_id FROM directories WHERE {}",
            where_clause
        );
        let rows = sqlx::query(&query_string).fetch_all(pool).await?;

        let directories: Vec<Box<Directory>> = rows
            .into_iter()
            .map(|row| {
                Box::new(Directory {
                    directory_id: row.get("directory_id"),
                    owner_id: row.get("owner_id"),
                    parent_id: row.get("parent_id"),
                })
            })
            .collect();

        Ok(directories)
    }

    async fn count_all(pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<i64, sqlx::Error> {
        let count = sqlx::query("SELECT COUNT(*) FROM directories")
            .fetch_one(pool)
            .await?;
        Ok(count.get("count"))
    }
}

#[async_trait::async_trait]
impl DbModel for File {
    async fn insert(&self, pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO files (file_id, name, size, owner_id, directory_id, last_modified) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (file_id) DO UPDATE SET name = $2, size = $3, owner_id = $4, directory_id = $5",
            self.file_id,
            self.name,
            self.size,
            self.owner_id,
            self.directory_id,
            self.last_modified
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn update(&self, pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE files SET name = $2, size = $3, owner_id = $4, directory_id = $5 WHERE file_id = $1",
            self.file_id,
            self.name,
            self.size,
            self.owner_id,
            self.directory_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM files WHERE file_id = $1", self.file_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn select(
        &self,
        pool: &sqlx::Pool<sqlx::postgres::Postgres>,
    ) -> Result<Box<Self>, sqlx::Error> {
        let file = sqlx::query_as!(
            File,
            "SELECT file_id, name, size, owner_id, directory_id, last_modified FROM files WHERE file_id = $1",
            self.file_id
        )
        .fetch_one(pool)
        .await?;
        Ok(Box::new(file))
    }

    async fn select_all(
        pool: &sqlx::Pool<sqlx::postgres::Postgres>,
    ) -> Result<Vec<Box<Self>>, sqlx::Error> {
        let files = sqlx::query_as!(
            File,
            "SELECT file_id, name, size, owner_id, directory_id, last_modified FROM files"
        )
        .fetch_all(pool)
        .await?;
        Ok(files.into_iter().map(|f| Box::new(f)).collect())
    }

    async fn select_where(
        pool: &sqlx::Pool<sqlx::postgres::Postgres>,
        where_clause: &str,
    ) -> Result<Vec<Box<File>>, sqlx::Error> {
        let query_string = format!(
            "SELECT file_id, name, size, owner_id, directory_id FROM files WHERE {}",
            where_clause
        );
        let rows = sqlx::query(&query_string).fetch_all(pool).await?;

        let files: Vec<Box<File>> = rows
            .into_iter()
            .map(|row| {
                Box::new(File {
                    file_id: row.get("file_id"),
                    name: row.get("name"),
                    size: row.get("size"),
                    owner_id: row.get("owner_id"),
                    directory_id: row.get("directory_id"),
                    last_modified: row.get("last_modified"),
                })
            })
            .collect();

        Ok(files)
    }

    async fn count_all(pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<i64, sqlx::Error> {
        let count = sqlx::query("SELECT COUNT(*) FROM files")
            .fetch_one(pool)
            .await?;
        Ok(count.get("count"))
    }
}

#[async_trait::async_trait]
impl DbEstimateRow for File {
    async fn estimate_count(
        pool: &sqlx::Pool<sqlx::postgres::Postgres>,
    ) -> Result<i64, sqlx::Error> {
        let query = format!(
            r#"
            SELECT reltuples AS estimated_rows
            FROM pg_class
            WHERE relname = 'files';
            "#
        );

        let row = sqlx::query(&query).fetch_one(pool).await?;
        let estimated_rows: i64 = row.get("estimated_rows");
        Ok(estimated_rows)
    }
}

#[async_trait::async_trait]
impl DbEstimateRow for Directory {
    async fn estimate_count(
        pool: &sqlx::Pool<sqlx::postgres::Postgres>,
    ) -> Result<i64, sqlx::Error> {
        let query = format!(
            r#"
            SELECT reltuples AS estimated_rows
            FROM pg_class
            WHERE relname = 'directories';
            "#
        );

        let row = sqlx::query(&query).fetch_one(pool).await?;
        let estimated_rows: i64 = row.get("estimated_rows");
        Ok(estimated_rows)
    }
}
