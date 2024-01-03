use sqlx::{Error, Pool, Postgres};
use sqlx::postgres::PgPoolOptions;

pub async fn create_database_pool(database_url: &str) -> Result<Pool<Postgres>, Error> {
    match PgPoolOptions::new().max_connections(5).connect(database_url).await {
        Ok(pool) => {
            println!("Connected to database.");
            Ok(pool)
        }
        Err(error) => {
            println!("Could not connect to database. {:?}", error);
            Err(error)
        }
    }
}

pub async fn get_hashes_from_db(pool: Pool<Postgres>) -> Result<Vec<String>, Error> {
    match sqlx::query_as::<_, (Option<String>, )>("SELECT md5sum from files_media")
        .fetch_all(&pool).await {
        Ok(rows) => {
            let rows: Vec<String> = rows.into_iter()
                .filter_map(|x| x.0)
                .collect();
            println!("Successfully retrieved hashes from db");
            return Ok(rows);
        }
        Err(e) => println!("Failed to get hashes from db: {}", e),
    }
    Ok(vec![])
}

