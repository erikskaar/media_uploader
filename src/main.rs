use std::collections::HashMap;
use std::{env, process};
use std::io::{stdout, Write};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use dotenv::dotenv;
use crate::api::{create_client};
use crate::db::{create_database_pool};
use clap::Parser;
use crossterm::{
    terminal::{Clear, ClearType},
    cursor::MoveTo,
    ExecutableCommand,
};
use crate::shared_state::SharedState;

mod path_data;
mod file_traversal;
mod db;
mod api;
mod config;
mod file_utils;
mod shared_state;
mod upload_status;
mod file_extension;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to config YML
    #[arg(short, long)]
    config: String,

    /// True for testing, not fetching anything from db. Will still try to upload files.
    #[arg(short, long, default_value_t = false)]
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

    let shared_state = Arc::new(Mutex::new(SharedState {
        files_retrieved: 0,
        uploaded_files: 0,
        corrupt_files_counter: 0,
        remaining_files: i32::MAX,  // example number
        failed_files_counter: 0,
        skipped_files: 0,
        last_processed_files: vec![],
        currently_uploading: vec![],
        corrupt_files: vec![],
        failed_files: vec![],
    }));
    shared_state.lock().unwrap().set_files_retrieved(file_metadata_from_db.values().len());

    let client = create_client();

    let shared_state_clone = shared_state.clone();

    tokio::spawn(async move {
        file_traversal::iterate_over_files_and_upload(
            &root,
            file_metadata_from_db,
            Arc::new(client),
            config,
            &shared_state_clone,
        ).await;
    });

    let mut stdout = stdout();

    let start_time = Instant::now();

    loop {
        let elapsed = start_time.elapsed();
        let hours = elapsed.as_secs() / 3600;
        let minutes = (elapsed.as_secs() % 3600) / 60;
        let seconds = elapsed.as_secs() % 60;

        stdout.execute(Clear(ClearType::All)).unwrap();
        stdout.execute(MoveTo(0, 0)).unwrap();

        let should_break = {
            let state = shared_state.lock().unwrap();
            println!("Runtime: {:02}:{:02}:{:02}\n", hours, minutes, seconds);
            state.print_status();
            state.remaining_files == 0
        };

        stdout.flush().unwrap();

        if should_break {
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
