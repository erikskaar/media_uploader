use std::fs::{read_dir};
use std::{io};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use reqwest::{Client};
use tokio::sync::{Semaphore};
use tokio::task;
use crate::config::Config;
use crate::{file_utils, SharedState};
use crate::file_extension::FileExtension;
use crate::file_utils::{compute_hash_of_partial_file, get_file_buffer, get_file_size};
use crate::path_data::PathData;
use crate::upload_status::UploadStatus;

pub(crate) async fn iterate_over_files_and_upload(
    path: &str,
    file_metadata_from_db: HashMap<u64, Vec<String>>,
    client: Arc<Client>,
    config: Config,
    shared_state: &Arc<Mutex<SharedState>>,
) {
    let root = path;
    let paths = get_files_in_directory(path).unwrap_or_else(|_| vec![]);
    let total_paths = paths.len();

    // Wrap hashes in Arc for shared access
    let file_metadata_from_db = Arc::new(file_metadata_from_db);

    // Set the number of concurrent tasks
    let concurrency_limit: usize = config.number_of_threads as usize;
    let semaphore = Arc::new(Semaphore::new(concurrency_limit));

    let mut tasks = Vec::new();

    shared_state.lock().unwrap().set_initial_remaining_files((total_paths) as i32);

    for path in paths.into_iter() {
        let acceptable_users = config.accepted_users.clone();
        let client = client.clone();
        let file_metadata_from_db = file_metadata_from_db.clone();
        let root = root.to_string();
        let semaphore = semaphore.clone();
        let shared_clone = shared_state.clone();

        let task = task::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            let file_size = match get_file_size(&path) {
                Ok(size) => size,
                Err(error) => {
                    println!("Could not get size of file {:?}. Reason: {}", path, error);
                    panic!()
                }
            };

            let path_str = match path.to_str() {
                Some(path_str) => path_str,
                None => {
                    println!("Could not get string slice from path {:?}", path);
                    panic!()
                }
            };

            let path_slice: &Path = path.as_path();

            if !file_metadata_from_db.contains_key(&file_size) {
                match file_utils::check_file_integrity(&path) {
                    true => {
                        shared_clone.lock().unwrap().append_to_currently_uploading(path.to_str().unwrap().to_string());
                        let data = read_file(path_str, &root, &acceptable_users);
                        upload_file(data, &client, path_str, shared_clone).await;
                    }
                    false => {
                        shared_clone.lock().unwrap().increment_corrupt_files();
                        shared_clone.lock().unwrap().append_to_processed_files((UploadStatus::Corrupt, path.to_str().unwrap().to_string()));
                    }
                }
            } else {
                let partial_hash = match compute_hash_of_partial_file(path_slice) {
                    Ok(hash) => hash,
                    Err(error) => {
                        println!("Could not get partial hash of file, {:?}. Reason: {}", path_slice, error);
                        panic!()
                    }
                };

                if !file_metadata_from_db.get(&file_size).unwrap().contains(&partial_hash) {
                    match file_utils::check_file_integrity(&path) {
                        true => {
                            let data = read_file(path_str, &root, &acceptable_users);
                            upload_file(data, &client, path_str, shared_clone).await
                        }
                        false => {
                            shared_clone.lock().unwrap().increment_corrupt_files();
                        }
                    }
                } else {
                    shared_clone.lock().unwrap().increment_skipped_files();
                    shared_clone
                        .lock()
                        .unwrap()
                        .append_to_processed_files((UploadStatus::Skipped, path.to_str().unwrap().to_string()));
                }
            }
        });
        tasks.push(task);
    }

    for task in tasks {
        let _ = task.await; // Handle or ignore the result/error here
    }
}

pub(crate) fn read_file(
    path: &str,
    root: &str,
    acceptable_users: &[String],
) -> Result<PathData, std::fmt::Error> {

    // Split out the root
    let relative_path: String = path
        .split(root)
        .find(|x| !x.is_empty())
        .unwrap()
        .to_owned();

    // create mutable copy to pop out different parts
    let mut mutable_relative_path: Vec<&str> = relative_path
        .split('/')
        .collect();

    let absolute_path = path.to_owned();
    let filename = mutable_relative_path.pop().unwrap().to_owned();
    let mut username: &str;
    // If the file is in the root folder set it to default
    if mutable_relative_path.is_empty() {
        username = "Default_Uploader";
    } else {
        username = mutable_relative_path.remove(0);
        if !acceptable_users.contains(&username.to_string()) {
            username = "Default_Uploader";
        }
    }
    let tags: Vec<String> = mutable_relative_path.iter().map(|x| x.to_lowercase()).collect();
    let file_buffer = get_file_buffer(path).unwrap();
    let file_buffer = Arc::new(file_buffer);

    let username = username.to_owned();

    let extension = FileExtension::from(Path::new(path));
    let mime_type = extension.mime_type().to_string();

    Ok(PathData {
        absolute_path,
        relative_path,
        filename,
        username,
        tags,
        mime_type,
        file_buffer,
    })
}

pub fn get_files_in_directory(path: &str) -> io::Result<Vec<PathBuf>> {
    let path = Path::new(path);
    let mut file_paths = Vec::new();

    for entry in read_dir(path)? {
        let entry = entry?;
        let current_path = entry.path();

        if current_path.is_file() {
            match FileExtension::from(&current_path) {
                FileExtension::Unknown => {}
                _ => {
                    file_paths.push(current_path)
                }
            }
        } else if current_path.is_dir() {
            let mut sub_files = get_files_in_directory(current_path.as_path().to_str().unwrap().trim())?;
            file_paths.append(&mut sub_files);
        }
    }
    Ok(file_paths)
}

pub async fn upload_file(
    data: Result<PathData, core::fmt::Error>,
    client: &Client,
    path_str: &str,
    shared_state: Arc<Mutex<SharedState>>,
) {
    if let Ok(data) = data {
        match data.upload(client).await {
            Ok(response) => {
                if response.status() == 201 {
                    shared_state.lock().unwrap().increment_uploaded_files();
                    shared_state
                        .lock()
                        .unwrap()
                        .append_to_processed_files((UploadStatus::Success, path_str.to_string()));
                } else {
                    shared_state.lock().unwrap().increment_failed_files();
                    shared_state
                        .lock()
                        .unwrap()
                        .append_to_processed_files((UploadStatus::Failed(response.status().as_u16()), path_str.to_string()));
                }
            }
            Err(error) => {
                shared_state.lock().unwrap().increment_failed_files();
                shared_state
                    .lock()
                    .unwrap()
                    .append_to_processed_files((UploadStatus::Failed(error.status().unwrap().as_u16()), path_str.to_string()));
            }
        };
        shared_state.lock().unwrap().remove_from_currently_uploading(path_str.to_string());
    }
}