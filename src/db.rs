use std::collections::HashMap;
use colored::Colorize;
use sqlx::{Error, Pool, Postgres};
use sqlx::postgres::PgPoolOptions;

pub async fn create_database_pool(database_url: &str) -> Result<Pool<Postgres>, Error> {
    match PgPoolOptions::new().max_connections(5).connect(database_url).await {
        Ok(pool) => {
            println!("{}", "Connected to database.".green());
            Ok(pool)
        }
        Err(error) => {
            println!("{} {}", "Could not connect to database.".red(), error);
            Err(error)
        }
    }
}

pub async fn get_file_details_from_db(pool: Pool<Postgres>) -> Result<HashMap<u64, Vec<String>>, Error> {
    let mut file_details = HashMap::new();

    match sqlx::query_as::<_, (Option<String>, Option<String>)>("SELECT size, md5sum from files_media")
        .fetch_all(&pool).await {
        Ok(rows) => {
            for (size, hash) in rows {
                if let (Some(size_str), Some(hash)) = (size, hash) {
                    if let Ok(size) = size_str.parse::<u64>() {
                        file_details.entry(size).or_insert_with(Vec::new).push(hash);
                    }
                }
            }
            println!("{}", "Successfully retrieved file metadata from db".green());
            Ok(file_details)
        }
        Err(e) => {
            println!("{} {}", "Failed to get file metadata from db:".red(), e);
            Err(e)
        }
    }
}



