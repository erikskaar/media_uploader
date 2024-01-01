use reqwest::{Client};

pub fn create_client() -> Client {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    return client;
}
