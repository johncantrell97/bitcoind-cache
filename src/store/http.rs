use crate::store::{AsyncStoreResult, Store, StoreError};
use reqwest::Body;

#[derive(Clone)]
pub struct HttpStore {
    host: String,
}

impl HttpStore {
    pub fn new(host: String) -> Self {
        Self { host }
    }
}

impl Store for HttpStore {
    fn get_object(&self, filename: String) -> AsyncStoreResult<Option<Vec<u8>>> {
        Box::pin(async move {
            match reqwest::get(format!("{}/{}", self.host, filename)).await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.bytes().await {
                            Ok(content) => Ok(Some(content.to_vec())),
                            Err(e) => Err(StoreError::Reqwest(e)),
                        }
                    } else {
                        Ok(None)
                    }
                }
                Err(e) => Err(StoreError::Reqwest(e)),
            }
        })
    }

    fn put_object<'a>(&'a self, filename: String, content: &'a [u8]) -> AsyncStoreResult<()> {
        Box::pin(async move {
            let body = content.to_vec();
            let body: Body = body.into();
            let client = reqwest::Client::new();
            client
                .put(format!("{}/{}", self.host, filename))
                .body(body)
                .send()
                .await
                .map(|_| ())
                .map_err(StoreError::Reqwest)
        })
    }
}
