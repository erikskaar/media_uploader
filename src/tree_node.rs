use std::{fs, io, process};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::file_extension::FileExtension;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum TreeNode {
    File(FileNode),
    Directory(DirectoryNode)

}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct FileNode {
    pub path: PathBuf,
}

impl FileNode {
    fn new(path: PathBuf) -> FileNode {
        FileNode {
            path
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DirectoryNode {
    path: PathBuf,
    children_count: usize,
    children: Vec<TreeNode>,
}

impl DirectoryNode {
    fn new(path: PathBuf, children: Vec<TreeNode>, children_count: usize) -> DirectoryNode {
        DirectoryNode {
            path,
            children,
            children_count,
        }
    }
}

impl TreeNode {
    fn add_node(&mut self, node: TreeNode) {
        if let TreeNode::Directory(directory_node) = self {
            directory_node.children.push(node);
            directory_node.children_count= TreeNode::count_descendants(&directory_node.children);
        }
    }

    pub fn count_descendants(children: &[TreeNode]) -> usize {
        children.iter().fold(0, |acc, child| {
            acc + match child {
                TreeNode::File(_) => 1,
                TreeNode::Directory(node) => TreeNode::count_descendants(&node.children),
            }
        })
    }
}

pub fn get_files_in_directory(path: &str) -> io::Result<TreeNode> {
    let path = Path::new(path);
    let mut directory_node = TreeNode::Directory(DirectoryNode::new(path.to_path_buf(), Vec::new(), 0));

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let current_path = entry.path();

        if current_path.is_file() {
            match FileExtension::from(&current_path) {
                FileExtension::Unknown => {}
                _ => {
                    if let TreeNode::Directory(_) = &directory_node {
                        directory_node.add_node(TreeNode::File(FileNode::new(current_path)));
                    }
                }
            }
        } else if current_path.is_dir() {
            let sub_tree = get_files_in_directory(current_path.to_str().unwrap())?;
            if let TreeNode::Directory(_) = &directory_node {
                directory_node.add_node(sub_tree);
            }
        }
    }
    Ok(directory_node)
}

pub fn save_tree_to_file(tree: &TreeNode, file_path: &str) -> Result<(), serde_json::Error> {
    let serialized = serde_json::to_string_pretty(&tree)?;
    let mut file = File::create(file_path).expect("Unable to create file");
    file.write_all(serialized.as_bytes()).expect("Unable to write data to file");
    Ok(())
}

pub fn load_tree_from_file(file_path: &str) -> Result<TreeNode, io::Error> {
   match File::open(file_path) {
        Ok(mut file) => {
            let mut contents = String::new();
            match file.read_to_string(&mut contents) {
                Ok(_) => {
                    let deserialized: TreeNode = serde_json::from_str(&contents)?;
                    Ok(deserialized)
                }
                Err(error) => Err(error)
            }
        },
        Err(x) => {
            Err(x)
        }
    }
}

pub fn find_unique_files_in_directory(tn1: &TreeNode, tn2: &TreeNode) -> Vec<FileNode> {
    let mut unique_files = Vec::new();
    compare_directory_nodes(tn1, tn2, &mut unique_files);
    unique_files
}

pub fn compare_directory_nodes(tn1: &TreeNode, tn2: &TreeNode, unique_files: &mut Vec<FileNode>) {
    match (tn1, tn2) {
        (TreeNode::Directory(dir1), TreeNode::Directory(dir2)) => {
            if dir1.children_count != dir2.children_count {
                for child in &dir1.children {
                    match child {
                        TreeNode::File(file_node) => {
                            if !dir_contains_file(dir2, file_node) {
                                unique_files.push(file_node.clone());
                            }
                        }
                        TreeNode::Directory(_) => {
                            let matching_dir = dir2.children.iter().find(|d| match d {
                                TreeNode::Directory(d) => d.path == *get_node_path(child),
                                _ => false,
                            });
                            match matching_dir {
                                Some(matching_dir) => compare_directory_nodes(child, matching_dir, unique_files),
                                None => unique_files.extend(flatten_directory(child)),
                            }
                        }
                    }
                }
            }
        }
        _ => {
            eprintln!("Root folder is not a directory!");
            process::exit(1)
        }
    }
}

// Helper function to check if a directory contains a specific file
fn dir_contains_file(dir: &DirectoryNode, file_node: &FileNode) -> bool {
    dir.children.iter().any(|child| match child {
        TreeNode::File(f) => f.path == file_node.path,
        _ => false,
    })
}

// Function to flatten DirectoryNode into a list of FileNodes
fn flatten_directory(node: &TreeNode) -> Vec<FileNode> {
    match node {
        TreeNode::File(file_node) => vec![file_node.clone()],
        TreeNode::Directory(dir_node) => {
            let mut files = Vec::new();
            for child in &dir_node.children {
                files.extend(flatten_directory(child));
            }
            files
        },
    }
}

pub fn get_node_path(node: &TreeNode) -> &PathBuf {
    match node {
        TreeNode::File(file_node) => &file_node.path,
        TreeNode::Directory(dir_node) => &dir_node.path,
    }
}
