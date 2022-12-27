use crate::store::{AsyncStoreResult, Store, StoreError};
use std::{fs, io};

#[derive(Clone)]
pub struct FileStore {
    root_dir: String,
}

impl FileStore {
    pub fn new(root_dir: &str) -> io::Result<Self> {
        fs::create_dir_all(root_dir)?;
        Ok(Self { root_dir: root_dir.to_string() })
    }
}

impl Store for FileStore {
    fn get_object(&self, filename: String) -> AsyncStoreResult<Option<Vec<u8>>> {
        Box::pin(async move {
            match tokio::fs::read(format!("{}/{}", self.root_dir, filename)).await {
                Ok(content) => Ok(Some(content)),
                Err(e) => {
                    if e.kind() == io::ErrorKind::NotFound {
                        Ok(None)
                    } else {
                        Err(StoreError::Io(e))
                    }
                }
            }
        })
    }

    fn put_object<'a>(&'a self, filename: String, content: &'a [u8]) -> AsyncStoreResult<()> {
        Box::pin(async move {
            tokio::fs::write(format!("{}/{}", self.root_dir, filename), content)
                .await
                .map_err(StoreError::Io)
        })
    }
}
