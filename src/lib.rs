pub mod store;
pub mod utils;

use bitcoin::blockdata::constants::genesis_block;
use bitcoin::consensus::encode::{serialize, serialize_hex};
use bitcoin::consensus::{self, deserialize};
use bitcoin::hashes::hex::{FromHex, ToHex};
use bitcoin::util::uint::Uint256;
use bitcoin::{Block, BlockHash, BlockHeader, Network};

use store::{AnyStore, Store, StoreError};

#[derive(Debug)]
pub enum BitcoindCacheError {
    Store(StoreError),
}

impl From<StoreError> for BitcoindCacheError {
    fn from(store_error: StoreError) -> BitcoindCacheError {
        BitcoindCacheError::Store(store_error)
    }
}

pub type BitcoindCacheResult<T> = Result<T, BitcoindCacheError>;

#[derive(Clone)]
pub struct BitcoindCache {
    pub store: AnyStore,
    pub network: Network,
}

impl BitcoindCache {
    pub fn new(network: Network, store: AnyStore) -> Self {
        Self { store, network }
    }

    pub fn best_block_hash_path(&self) -> String {
        "best-block-hash".to_string()
    }

    pub fn best_block_height_path(&self) -> String {
        "best-block-height".to_string()
    }

    pub fn header_path(&self, block_hash: String) -> String {
        format!("{}.header", block_hash)
    }

    pub fn block_path(&self, block_hash: String) -> String {
        format!("{}.block", block_hash)
    }

    pub async fn put_best_block_hash(&self, block_hash: &BlockHash) -> BitcoindCacheResult<()> {
        Ok(self
            .store
            .put_object(self.best_block_hash_path(), &serialize(block_hash))
            .await?)
    }

    pub async fn put_best_block_height(&self, block_height: u32) -> BitcoindCacheResult<()> {
        Ok(self
            .store
            .put_object(
                self.best_block_height_path(),
                block_height.to_string().as_bytes(),
            )
            .await?)
    }

    pub async fn put_block(&self, block: &Block) -> BitcoindCacheResult<()> {
        let block_hash_hex = block.block_hash().to_hex();

        Ok(self
            .store
            .put_object(self.block_path(block_hash_hex), &serialize(block))
            .await?)
    }

    pub async fn put_header(
        &self,
        header: &BlockHeader,
        height: u32,
        chainwork: Uint256,
    ) -> BitcoindCacheResult<()> {
        let block_hash_hex = header.block_hash().to_hex();
        let chainwork_hex = utils::hex_str(&consensus::serialize(&chainwork));
        let header_data = format!("{},{},{}", serialize_hex(header), height, chainwork_hex);

        Ok(self
            .store
            .put_object(self.header_path(block_hash_hex), header_data.as_bytes())
            .await?)
    }

    pub async fn get_header_by_hash(
        &self,
        hash: &BlockHash,
    ) -> BitcoindCacheResult<Option<(BlockHeader, u32, Uint256)>> {
        let header = self
            .store
            .get_object(self.header_path(hash.to_hex()))
            .await?;

        Ok(header.map(|header_bytes| {
            let header_data_string = String::from_utf8(header_bytes).unwrap();
            let header_parts: Vec<&str> = header_data_string.split(',').collect();
            let header: BlockHeader =
                deserialize(&Vec::<u8>::from_hex(header_parts[0]).unwrap()).unwrap();
            let height: u32 = header_parts[1].to_string().parse().unwrap();
            let chainwork: Uint256 = deserialize(&utils::to_vec(header_parts[2]).unwrap()).unwrap();
            (header, height, chainwork)
        }))
    }

    pub async fn get_block_by_hash(&self, hash: &BlockHash) -> BitcoindCacheResult<Option<Block>> {
        let block = self
            .store
            .get_object(self.block_path(hash.to_hex()))
            .await?;

        Ok(block.map(|serialized_block| {
            deserialize(&serialized_block).expect("data to be encoded correctly")
        }))
    }

    pub async fn get_best_block_hash(&self) -> BitcoindCacheResult<Option<BlockHash>> {
        let block_hash = self.store.get_object(self.best_block_hash_path()).await?;

        Ok(block_hash.map(|block_hash_bytes| {
            deserialize(&block_hash_bytes).expect("data to be encoded correctly")
        }))
    }

    pub async fn get_best_block_height(&self) -> BitcoindCacheResult<Option<u32>> {
        let height = self.store.get_object(self.best_block_height_path()).await?;

        Ok(height.map(|height_bytes| String::from_utf8(height_bytes).unwrap().parse().unwrap()))
    }

    pub async fn get_cached_best_block(&self) -> BitcoindCacheResult<(BlockHash, u32)> {
        let best_hash = self.get_best_block_hash().await?;
        let best_height = self.get_best_block_height().await?;

        if best_hash.is_none() || best_height.is_none() {
            let genesis_block = genesis_block(self.network);
            let genesis_hash = genesis_block.header.block_hash();
            self.block_connected(&genesis_block, 0, Uint256::from_u64(0).unwrap())
                .await?;
            Ok((genesis_hash, 0))
        } else {
            Ok((best_hash.unwrap(), best_height.unwrap()))
        }
    }

    pub async fn block_connected(
        &self,
        block: &Block,
        height: u32,
        chainwork: Uint256,
    ) -> BitcoindCacheResult<()> {
        self.put_block(block).await?;
        self.put_header(&block.header, height, chainwork).await?;
        self.put_best_block_hash(&block.block_hash()).await?;
        self.put_best_block_height(height).await?;

        Ok(())
    }
}
