use std::env;
use std::ops::Add;
use reqwest::{Body, Client, Error, multipart, Response};
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
    pub mime_type: String,
    pub file_buffer: Arc<Vec<u8>>,
}

impl<'a> PathData {
    pub async fn upload(&self, client: &Client) -> Result<Response, Error> {
        let url = env::var("API_URL").expect("API_URL must be set");

        let buffer_clone = Arc::clone(&self.file_buffer).to_vec();
        let stream = stream::once(async move {
            Ok::<Bytes, std::io::Error>(Bytes::from(buffer_clone))
        });

        let body = Body::wrap_stream(stream);

        let file_part = match multipart::Part::stream(body)
            .file_name(self.filename.clone())
            .mime_str("video/mp4") {
            Ok(part) => part,
            Err(ref e) => {
                println!("Error creating file part: {:?}", e);
                panic!()
            }
        };

        let description = self.tags.join(",");

        let form = multipart::Form::new()
            .part("media_file", file_part)
            .text("title", self.filename.clone())
            .text("description", description);

        let password = &self.username.to_uppercase().add("_PASSWORD");
        let password = env::var(password)
            .expect("Password not in env file");

        client
            .post(url)
            .basic_auth(&self.username, Some(password))
            .multipart(form)
            .send()
            .await
    }
}
