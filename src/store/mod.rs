use std::{future::Future, pin::Pin};

#[cfg(feature = "fs")]
use std::io;

#[cfg(feature = "fs")]
pub mod filesystem;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "r2")]
pub mod r2;

#[cfg(feature = "fs")]
pub use filesystem::FileStore;
#[cfg(feature = "http")]
pub use http::HttpStore;
#[cfg(feature = "r2")]
pub use r2::R2Store;

#[derive(Clone)]
pub enum AnyStore {
    #[cfg(feature = "fs")]
    File(FileStore),
    #[cfg(feature = "http")]
    HTTP(HttpStore),
    #[cfg(feature = "r2")]
    R2(R2Store),
}

impl Store for AnyStore {
    fn put_object<'a>(&'a self, filename: String, content: &'a [u8]) -> AsyncStoreResult<()> {
        match self {
            #[cfg(feature = "fs")]
            AnyStore::File(store) => store.put_object(filename, content),
            #[cfg(feature = "http")]
            AnyStore::HTTP(store) => store.put_object(filename, content),
            #[cfg(feature = "r2")]
            AnyStore::R2(store) => store.put_object(filename, content),
        }
    }

    fn get_object(&self, filename: String) -> AsyncStoreResult<Option<Vec<u8>>> {
        match self {
            #[cfg(feature = "fs")]
            AnyStore::File(store) => store.get_object(filename),
            #[cfg(feature = "http")]
            AnyStore::HTTP(store) => store.get_object(filename),
            #[cfg(feature = "r2")]
            AnyStore::R2(store) => store.get_object(filename),
        }
    }
}

#[derive(Debug)]
pub enum StoreError {
    #[cfg(feature = "fs")]
    Io(io::Error),
    #[cfg(feature = "http")]
    Reqwest(reqwest::Error),
    #[cfg(feature = "r2")]
    R2(s3::error::S3Error),
}

pub type AsyncStoreResult<'a, T> = Pin<Box<dyn Future<Output = Result<T, StoreError>> + 'a + Send>>;

pub trait Store {
    fn put_object<'a>(&'a self, filename: String, content: &'a [u8]) -> AsyncStoreResult<'a, ()>;
    fn get_object(&self, filename: String) -> AsyncStoreResult<Option<Vec<u8>>>;
}
