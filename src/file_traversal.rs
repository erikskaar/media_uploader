use std::fs::{File, read_dir};
use std::{io};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use reqwest::Client;
use tokio::sync::Semaphore;
use tokio::task;
use crate::path_data::PathData;

pub(crate) async fn iterate_over_files_and_upload(path: &str, hashes_from_db: Vec<String>, client: Arc<Client>) {
    let root = path;
    let paths = get_files_in_directory(path).unwrap_or_else(|_| vec![]);
    let total_paths = paths.len();

    // Wrap hashes in Arc for shared access
    let hashes_from_db = Arc::new(hashes_from_db);

    // Set the number of concurrent tasks
    let concurrency_limit = 6;
    let semaphore = Arc::new(Semaphore::new(concurrency_limit));

    let mut tasks = Vec::new();

    for (index, path) in paths.into_iter().enumerate() {
        let client = client.clone();
        let hashes_from_db = hashes_from_db.clone();
        let root = root.to_string();
        let semaphore = semaphore.clone();

        let task = task::spawn(async move {
            let permit = semaphore.acquire().await.unwrap();
            let data = read_file(path.to_str().unwrap(), &root);
            if let Ok(data) = data {
                if !hashes_from_db.contains(&data.md5) {
                    println!("File {}/{}: Uploading", index + 1, total_paths);
                    data.upload(&client).await;
                } else {
                    println!("File {}/{}: Skipping", index + 1, total_paths);
                }
            }
            drop(permit); // Release the permit
        });

        tasks.push(task);
    }

    for task in tasks {
        let _ = task.await; // Handle or ignore the result/error here
    }
}

pub(crate) fn read_file(path: &str, root: &str) -> Result<PathData, std::fmt::Error> {
    // Set default acceptable usernames
    let acceptable_usernames: [String; 2] = [String::from("Erik"), String::from("Lasse")];

    // Split out the root
    let relative_path: String = path
        .split(root)
        .filter(|x| !x.is_empty())
        .next()
        .unwrap().to_owned();

    // create mutable copy to pop out different parts
    let mut mutable_relative_path: Vec<&str> = relative_path
        .split('/')
        .collect();

    let absolute_path = path.to_owned();
    let filename = mutable_relative_path.pop().unwrap().to_owned();
    let mut username: String;
    // If the file is in the root folder set it to default
    if mutable_relative_path.len() == 0 {
        username = String::from("Default_Uploader");
    } else {
        username = mutable_relative_path.remove(0).to_owned();
        if !acceptable_usernames.contains(&username) {
            username = String::from("Default_Uploader");
        }
    }
    let tags: Vec<String> = mutable_relative_path.iter().map(|x| x.to_lowercase()).collect();
    let file_buffer = get_file_buffer(path).unwrap();
    let file_buffer = Arc::new(file_buffer);
    let md5 = compute_md5_hash(&file_buffer).unwrap();

    // println!("Successfully read file into memory: {}", path);
    return Ok(PathData {
        absolute_path,
        relative_path,
        filename,
        username,
        tags,
        md5,
        file_buffer,
    });
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
    return Ok(format!("{:x}", digest));
}

pub fn get_file_buffer(path: &str) -> Result<Vec<u8>, io::Error> {
    let path = Path::new(path);
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    let _ = file.read_to_end(&mut buffer);
    return Ok(buffer);
}