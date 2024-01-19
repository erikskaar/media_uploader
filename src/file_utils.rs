use std::fs::File;
use std::io;
use std::io::Read;
use std::path::{Path};
use crate::file_utils::FileExtension::{Avi, Mov, Mp4, Mpeg, Ogv, Unknown, Webm, Wmv};

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

#[derive(Debug, PartialEq)]
pub enum FileExtension {
    Mp4,
    Avi,
    Mpeg,
    Ogv,
    Webm,
    Mov,
    Wmv,
    Unknown,
}

impl FileExtension {
    pub fn from(path: &Path) -> FileExtension {
        let extension_str = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        if extension_str.eq_ignore_ascii_case("mp4") {
            Mp4
        } else if extension_str.eq_ignore_ascii_case("avi") {
            Avi
        } else if extension_str.eq_ignore_ascii_case("mpeg") {
            Mpeg
        } else if extension_str.eq_ignore_ascii_case("ogv") {
            Ogv
        } else if extension_str.eq_ignore_ascii_case("webm") {
            Webm
        } else if extension_str.eq_ignore_ascii_case("mov") {
            Mov
        } else if extension_str.eq_ignore_ascii_case("wmv") {
            Wmv
        } else {
            Unknown
        }
    }

    pub fn mime_type<'a>(&self) -> &'a str {
        match self {
            Mp4 => "video/mp4",
            Avi => "video/x-msvideo",
            Mpeg => "video/mpeg",
            Ogv => "video/ogg",
            Webm => "video/webm",
            Mov => "video/quicktime",
            Wmv => "video/x-ms-wmv",
            Unknown => ""
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_extension() {
        let path = Path::new("video.mP4");
        assert_eq!(FileExtension::from(path), Mp4);
    }

    #[test]
    fn test_unknown_extension() {
        let path = Path::new("file.xyz");
        assert_eq!(FileExtension::from(path), Unknown);
    }

    #[test]
    fn test_no_extension() {
        let path = Path::new("video.");
        assert_eq!(FileExtension::from(path), Unknown);
    }
}
