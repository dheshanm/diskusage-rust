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
    pub owner_id: i32,
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
