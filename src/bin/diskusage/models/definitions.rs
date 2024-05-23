use serde::Deserialize;
use serde::Serialize;
use sqlx::Row;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct User {
    pub user_id: i32,
    pub username: Option<String>
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
    pub owner_id: i32,
    pub directory_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct FileStructure {
    pub parent_dir_id: String,
    pub child_dir_id: String,
}

#[async_trait::async_trait]
pub trait DbModel {
    async fn insert(&self, pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<(), sqlx::Error>;
    async fn update(&self, pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<(), sqlx::Error>;
    async fn delete(&self, pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<(), sqlx::Error>;
    async fn select(&self, pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<Box<Self>, sqlx::Error>;
    async fn select_all(pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<Vec<Box<Self>>, sqlx::Error>;
    async fn select_where(pool: &sqlx::Pool<sqlx::postgres::Postgres>, where_clause: &str) -> Result<Vec<Box<Self>>, sqlx::Error>;
}

#[async_trait::async_trait]
impl DbModel for User {
    async fn insert(&self, pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO users (user_id, username) VALUES ($1, $2)",
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
        sqlx::query!(
            "DELETE FROM users WHERE user_id = $1",
            self.user_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn select(&self, pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<Box<Self>, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            "SELECT user_id, username FROM users WHERE user_id = $1",
            self.user_id
        )
        .fetch_one(pool)
        .await?;
        Ok(Box::new(user))
    }

    async fn select_all(pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<Vec<Box<Self>>, sqlx::Error> {
        let users = sqlx::query_as!(
            User,
            "SELECT user_id, username FROM users"
        )
        .fetch_all(pool)
        .await?;
        Ok(users.into_iter().map(|u| Box::new(u)).collect())
    }

    async fn select_where(pool: &sqlx::Pool<sqlx::postgres::Postgres>, where_clause: &str) -> Result<Vec<Box<User>>, sqlx::Error> {
        let query_string = format!("SELECT user_id, username FROM users WHERE {}", where_clause);
        let rows = sqlx::query(&query_string)
            .fetch_all(pool)
            .await?;
    
        let users: Vec<Box<User>> = rows.into_iter()
            .map(|row| {
                Box::new(User {
                    user_id: row.get("user_id"),
                    username: row.get("username"),
                })
            })
            .collect();
        Ok(users)
    }
}

#[async_trait::async_trait]
impl DbModel for Directory {
    async fn insert(&self, pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO directories (directory_id, owner_id, parent_id) VALUES ($1, $2, $3)",
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

    async fn select(&self, pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<Box<Self>, sqlx::Error> {
        let directory = sqlx::query_as!(
            Directory,
            "SELECT directory_id, owner_id, parent_id FROM directories WHERE directory_id = $1",
            self.directory_id
        )
        .fetch_one(pool)
        .await?;
        Ok(Box::new(directory))
    }

    async fn select_all(pool: &sqlx::Pool<sqlx::postgres::Postgres>) -> Result<Vec<Box<Self>>, sqlx::Error> {
        let directories = sqlx::query_as!(
            Directory,
            "SELECT directory_id, owner_id, parent_id FROM directories"
        )
        .fetch_all(pool)
        .await?;
        Ok(directories.into_iter().map(|d| Box::new(d)).collect())
    }

    async fn select_where(pool: &sqlx::Pool<sqlx::postgres::Postgres>, where_clause: &str) -> Result<Vec<Box<Directory>>, sqlx::Error> {
        let query_string = format!("SELECT directory_id, owner_id, parent_id FROM directories WHERE {}", where_clause);
        let rows = sqlx::query(&query_string)
            .fetch_all(pool)
            .await?;

        let directories: Vec<Box<Directory>> = rows.into_iter()
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
}