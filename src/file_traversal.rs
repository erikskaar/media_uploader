use std::fs::{read_dir};
use std::{io};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc};
use colored::Colorize;
use reqwest::{Client};
use tokio::sync::{Semaphore};
use tokio::task;
use crate::config::Config;
use crate::file_utils::{compute_hash_of_partial_file, FileExtension, get_file_buffer, get_file_size};
use crate::path_data::PathData;

pub(crate) async fn iterate_over_files_and_upload(
    path: &str,
    file_metadata_from_db: HashMap<u64, Vec<String>>,
    client: Arc<Client>,
    config: Config,
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

    for (index, path) in paths.into_iter().enumerate() {
        let acceptable_users = config.accepted_users.clone();
        let client = client.clone();
        let file_metadata_from_db = file_metadata_from_db.clone();
        let root = root.to_string();
        let semaphore = semaphore.clone();

        let task = task::spawn(async move {
            let permit = match semaphore.acquire().await {
                Ok(permit) => permit,
                Err(_) => {
                    println!("Could not acquire permit. Try limiting number_of_threads.");
                    panic!()
                }
            };

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
                let data = read_file(path_str, &root, &acceptable_users);
                if let Ok(data) = data {
                    println!("\t{}/{}:\t Uploading\t {}", index + 1, total_paths, path_str);
                    match data.upload(&client).await {
                        Ok(response) => {
                            if response.status() == 201 {
                                println!("\t{}/{}:\t Uploaded\t {}", index, total_paths, path_str.green());
                            } else {
                                println!("\t{}/{}:\t Failed  \t {}\t Response {}", index, total_paths, path_str, response.status().as_str().red())
                            }
                        }
                        Err(error) => {
                            println!("\t{}/{}:\t Failed  \t {}\t Response {}", index, total_paths, path_str, error.to_string().red())
                        }
                    };
                    drop(permit);
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
                    let data = read_file(path_str, &root, &acceptable_users);
                    if let Ok(data) = data {
                        match data.upload(&client).await {
                            Ok(response) => {
                                if response.status() == 201 {
                                    println!("\t{}/{}:\t Uploaded\t {}", index, total_paths, path_str.green());
                                } else {
                                    println!("\t{}/{}:\t Failed  \t {}\t Response {}", index, total_paths, path_str, response.status().as_str().red())
                                }
                            }
                            Err(error) => {
                                println!("\t{}/{}:\t Failed  \t {}\t Response {}", index, total_paths, path_str, error.to_string().red())
                            }
                        };
                        println!("\t{}/{}:\t Uploading\t {}", index + 1, total_paths, path_str);
                        drop(permit);
                    }
                } else {
                    println!("\t{}/{}:\t Skipping\t {}", index + 1, total_paths, path_str);
                    drop(permit);
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
