use std::path::Path;
use crate::file_extension::FileExtension::{Avi, Mov, Mp4, Mpeg, Ogv, Unknown, Webm, Wmv};

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
