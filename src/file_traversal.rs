use std::fs::{File, read_dir};
use std::{io};
use std::collections::HashMap;
use std::io::{Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc};
use reqwest::Client;
use tokio::sync::Semaphore;
use tokio::task;
use crate::config::Config;
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
    let concurrency_limit = config.number_of_threads;
    let semaphore = Arc::new(Semaphore::new(concurrency_limit as usize));

    let mut tasks = Vec::new();

    for (index, path) in paths.into_iter().enumerate() {
        let acceptable_users = config.accepted_users.clone();
        let client = client.clone();
        let file_metadata_from_db = file_metadata_from_db.clone();
        let root = root.to_string();
        let semaphore = semaphore.clone();

        let task = task::spawn(async move {
            let permit = semaphore.acquire().await.unwrap();
            let file_size = get_file_size(&path).unwrap();

            if !file_metadata_from_db.contains_key(&file_size) {
                let data = read_file(path.to_str().unwrap(), &root, &acceptable_users);
                if let Ok(data) = data {
                    println!("\t{}/{}:\t Uploading\t {}", index + 1, total_paths, path.to_str().unwrap());
                    data.upload(&client, index + 1, total_paths).await;
                    drop(permit);
                }
            } else {
                let partial_hash = compute_hash_of_partial_file(path.as_path()).unwrap();
                if !file_metadata_from_db.get(&file_size).unwrap().contains(&partial_hash) {
                    let data = read_file(path.to_str().unwrap(), &root, &acceptable_users);
                    if let Ok(data) = data {
                        println!("\t{}/{}:\t Uploading\t {}", index + 1, total_paths, path.to_str().unwrap());
                        data.upload(&client, index + 1, total_paths).await;
                        drop(permit);
                    }
                } else {
                    println!("\t{}/{}:\t Skipping\t {}", index + 1, total_paths, path.to_str().unwrap());
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
    acceptable_users: &Vec<String>,
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

    Ok(PathData {
        absolute_path,
        relative_path,
        filename,
        username,
        tags,
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
            if current_path
                .as_path()
                .to_str()
                .unwrap()
                .to_lowercase()
                .ends_with(".mp4") {
                file_paths.push(current_path);
            }
        } else if current_path.is_dir() {
            let mut sub_files = get_files_in_directory(current_path.as_path().to_str().unwrap().trim())?;
            file_paths.append(&mut sub_files);
        }
    }
    Ok(file_paths)
}

pub fn compute_md5_hash(buffer: &Vec<u8>) -> io::Result<String> {
    let digest = md5::compute(buffer);
    Ok(format!("{:x}", digest))
}

pub fn get_file_buffer(path: &str) -> Result<Vec<u8>, io::Error> {
    let path = Path::new(path);
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    let _ = file.read_to_end(&mut buffer);
    Ok(buffer)
}

pub fn get_file_size(path: &Path) -> io::Result<u64> {
    let file = File::open(path)?;
    Ok(file.metadata()?.len())
}

pub fn compute_hash_of_partial_file(path: &Path) -> io::Result<String> {
    const CHUNK_SIZE: usize = 128 * 1024; // 128 KB in bytes
    let mut file = File::open(path)?;

    // Read the first 128KB
    let mut chunk = vec![0; CHUNK_SIZE];
    file.read_exact(&mut chunk)?;
    let file_size = file.metadata()?.len();
    let file_size_str = file_size.to_string(); // required to match other languages
    let file_size_bytes = file_size_str.as_bytes();

    // Combine the chunk and file size for the final hash
    let mut buffer = chunk;
    buffer.extend_from_slice(&file_size_bytes);
    let result = compute_md5_hash(&buffer).unwrap();
    println!("Total MD5: {:?} {:?}", path, result);

    Ok(result)
}
