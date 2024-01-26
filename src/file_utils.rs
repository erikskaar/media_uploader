use std::fs::File;
use std::{io, process};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use reqwest::Client;
use crate::path_data::PathData;
use crate::shared_state::SharedState;
use crate::tree_node;
use crate::tree_node::find_unique_files_in_directory;
use crate::upload_status::UploadStatus;

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
    let file_size_str = file_size.to_string(); // required to match MediaCMS' Python implementation
    let file_size_bytes = file_size_str.as_bytes();

    // Combine the chunk and file size for the final hash
    let mut buffer = chunk;
    buffer.extend_from_slice(file_size_bytes);

    let result = compute_md5_hash(&buffer).unwrap();
    Ok(result)
}

pub fn check_file_integrity(path: &PathBuf) -> bool {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg(path)
        .output()
        .expect("Failed to verify file integrity.");

    output.status.success()
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
                    shared_state
                        .lock()
                        .unwrap()
                        .append_to_processed_files((UploadStatus::Success, path_str.to_string()));
                } else {
                    shared_state
                        .lock()
                        .unwrap()
                        .append_to_processed_files((UploadStatus::Failed(response.status().as_u16()), path_str.to_string()));
                }
            }
            Err(error) => {
                shared_state
                    .lock()
                    .unwrap()
                    .append_to_processed_files((UploadStatus::Failed(error.status().unwrap().as_u16()), path_str.to_string()));
            }
        };
        shared_state.lock().unwrap().remove_from_currently_uploading(path_str.to_string());
    }
}

pub fn get_newest_files(root_folder: &str) -> Vec<PathBuf> {
    match tree_node::load_tree_from_file("tree.json") {
        Ok(old_node) => {
            let new_node = tree_node::get_files_in_directory(root_folder);
            let new_node = match new_node {
                Ok(node) => node,
                Err(_) => {
                    eprintln!("Root folder is not a directory.");
                    process::exit(1)
                }
            };
            let extra_files: Vec<PathBuf> = find_unique_files_in_directory(&new_node, &old_node)
                .iter()
                .map(|x| x.path.clone())
                .collect();

            match tree_node::save_tree_to_file(&new_node, "tree.json") {
                Ok(_) => println!("Saved tree to file."),
                Err(_) => println!("Could not save tree to file")
            };
            extra_files
        },
        Err(_) => {
            println!("No previous run detected.");
            let new_node = tree_node::get_files_in_directory(root_folder);
            let new_node = match new_node {
                Ok(node) => node,
                Err(_) => {
                    eprintln!("Root folder is not a directory.");
                    process::exit(1)
                }
            };
            match tree_node::save_tree_to_file(&new_node, "tree.json") {
                Ok(_) => println!("Saved tree to file."),
                Err(_) => println!("Could not save tree to file")
            };
            Vec::new()
        }
    }
}
