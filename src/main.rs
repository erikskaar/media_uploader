use std::collections::HashMap;
use std::{env, process};
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
mod file_utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to config YML
    #[arg(short, long)]
    config: String,

    /// True for testing, not fetching anything from db. Will still try to upload files.
    #[arg(short, long, default_value_t=false)]
    dry: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = config::read_config(&args.config).unwrap();
    dotenv().ok();
    let root = env::var("ROOT_FOLDER").expect("ROOT_FOLDER must be set");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let file_metadata_from_db = if !args.dry {
        let pool = create_database_pool(&database_url).await.unwrap();
        match db::get_file_details_from_db(pool).await {
            Ok(metadata) => {
                println!("Successfully mapped {} files from database", metadata.values().len());
                metadata
            }
            Err(error) => {
                println!("Could not get rows from database. Reason:, {}", error);
                process::exit(1)
            }
        }
    } else {
        HashMap::new()
    };


    let client = create_client();

    file_traversal::iterate_over_files_and_upload(&root, file_metadata_from_db, Arc::new(client), config).await;
}
