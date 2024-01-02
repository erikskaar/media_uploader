use std::env;
use std::ops::Add;
use reqwest::{Body, Client, multipart};
use std::sync::Arc;
use futures::{stream};
use tokio_util::bytes::Bytes;

#[derive(Clone)]
pub struct PathData {
    pub absolute_path: String,
    pub relative_path: String,
    pub filename: String,
    pub(crate) username: String,
    pub tags: Vec<String>,
    pub md5: String,
    pub file_buffer: Arc<Vec<u8>>,
}

impl<'a> PathData {
    pub async fn upload(&self, client: &Client) {
        let url = env::var("API_URL").expect("API_URL must be set");

        let buffer_clone = Arc::clone(&self.file_buffer).to_vec();
        let stream = stream::once(async move {
            Ok::<Bytes, std::io::Error>(Bytes::from(buffer_clone))
        });

        let body = Body::wrap_stream(stream);

        let file_part = multipart::Part::stream(body)
            .file_name(self.filename.clone())
            .mime_str("video/mp4");

        match file_part {
            Ok(ref _part) => {}
            Err(ref e) => {
                println!("Error creating file part: {:?}", e);
            }
        }

        let description = self.tags.join(",");

        let form = multipart::Form::new()
            .part("media_file", file_part.unwrap())
            .text("title", self.filename.clone())
            .text("description", description);

        let password = &self.username.to_uppercase().add("_PASSWORD");
        let password = env::var(password)
            .expect("Password not in env file");

        println!("Uploading: {}", self.absolute_path);
        match client
            .post(url)
            .basic_auth(&self.username, Some(password))
            .multipart(form)
            .send()
            .await {
            Ok(response) => {
                if response.status() == 201 {
                    println!("Uploaded, {}", self.absolute_path);
                } else {
                    println!("Should have uploaded {}, but did not get a 201. Response: {}", self.absolute_path, response.status())
                }
            }
            Err(error) => {
                println!("Error: {:?}", error);
            }
        }
    }
}
