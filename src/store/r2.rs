use std::time::Duration;

use crate::store::{AsyncStoreResult, Store, StoreError};

use s3::creds::Credentials;
use s3::Bucket;
use s3::Region;

#[derive(Clone)]
pub struct R2Store {
    bucket: s3::Bucket,
}

impl R2Store {
    pub fn new(
        access_key_id: String,
        secret_access_key: String,
        account_id: String,
        bucket_name: String,
    ) -> R2Store {
        let endpoint = format!("https://{}.r2.cloudflarestorage.com", account_id);
        let region = Region::Custom {
            region: String::from("auto"),
            endpoint,
        };
        let creds = Credentials::new(
            Some(&access_key_id),
            Some(&secret_access_key),
            None,
            None,
            None,
        )
        .unwrap();
        let mut bucket = Bucket::new(&bucket_name, region, creds).unwrap();
        bucket.set_request_timeout(Some(Duration::from_secs(10)));
        R2Store { bucket }
    }
}

impl Store for R2Store {
    fn get_object(&self, filename: String) -> AsyncStoreResult<Option<Vec<u8>>> {
        Box::pin(async move {
            let mut retries = 30;
            let mut success = false;
            let mut last_response = None;

            while !success && retries > 0 {
                let response = self.bucket.get_object(filename.clone()).await;
                success = response.is_err();
                last_response = Some(response);
                retries -= 1;
            }

            match last_response {
                Some(Ok(response)) => Ok(Some(response.bytes().to_vec())),
                Some(Err(e)) => Err(StoreError::R2(e)),
                None => Ok(None),
            }
        })
    }

    fn put_object<'a>(&'a self, filename: String, content: &'a [u8]) -> AsyncStoreResult<()> {
        Box::pin(async move {
            let mut retries = 30;
            let mut success = false;
            let mut last_response = None;

            while !success && retries > 0 {
                let response = self.bucket.put_object(filename.clone(), content).await;
                success = response.is_ok();
                last_response = Some(response);
                retries -= 1;
            }

            match last_response {
                Some(Ok(_)) => Ok(()),
                Some(Err(e)) => Err(StoreError::R2(e)),
                None => Ok(()),
            }
        })
    }
}
