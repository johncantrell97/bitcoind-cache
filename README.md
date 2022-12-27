# bitcoind-cache

A library that helps facilitate storage and retrieval of data produced by bitcoind.  Currently supports storing data on a local filesystem, remote http kv-store, and s3-compatible object storage.  Currently only supports full blocks and their headers.  It probably makes sense to add support for compact blocks and their headers too.


# Usage Example

```rust
use bitcoin::blockdata::constants::genesis_block;
use bitcoin::Network;

use bitcoind_cache::{
    store::{AnyStore, FileStore},
    BitcoindCache,
};

#[tokio::main]
async fn main() {
    let filestore =
        FileStore::new("bitcoind-store-regtest").expect("path to be created if not exists");
    let store = AnyStore::File(filestore);
    let bitcoind_cache = BitcoindCache::new(Network::Regtest, store);

    let genesis_block = genesis_block(Network::Regtest);
    let genesis_block_hash = genesis_block.header.block_hash();

    // put the genesis block into the cache
    if let Err(e) = bitcoind_cache.put_block(&genesis_block).await {
        println!("failed to store block: {:?}", e);
    }

    // later, read the block out by hash
    let block = bitcoind_cache
        .get_block_by_hash(&genesis_block_hash)
        .await
        .expect("to read ok")
        .expect("to be some");

    assert_eq!(genesis_block, block);
}

```