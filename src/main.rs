use std::env;
use std::sync::Arc;
use dotenv::dotenv;
use crate::api::{create_client};
use crate::db::create_database_pool;
use clap::Parser;

mod path_data;
mod file_traversal;
mod db;
mod api;
mod config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to config YML
    #[arg(short, long)]
    config: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = config::read_config(&args.config).unwrap();
    dotenv().ok();
    let root = env::var("ROOT_FOLDER").expect("ROOT_FOLDER must be set");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = create_database_pool(&database_url).await.unwrap();
    let hashes_from_db = db::get_hashes_from_db(pool).await.unwrap();
    let client = create_client();

    file_traversal::iterate_over_files_and_upload(&root, hashes_from_db, Arc::new(client), config).await;
}
