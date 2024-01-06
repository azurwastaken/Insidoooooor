pub mod strknet;
// pub mod solana;

use async_trait::async_trait;
use starknet::core::types::{PendingBlockWithTxs,BlockWithTxs, FieldElement};

#[derive(Clone)]
pub enum DataType {
    FieldElement(FieldElement),
}

struct Block {
    pub parent_hash : DataType,
    pub hash : DataType,
    pub transaction : Vec<DataType>,
}

impl Block {
    fn new(phash : DataType, hash : DataType, tx : &[DataType]) -> Self {
        Self {
            parent_hash : phash,
            hash : hash,
            transaction : tx.to_vec(),
        }
    }
}

#[async_trait]
pub trait Chain {
    
    // both should return Object with block hash and tx array
    async fn get_pending_block(&self) -> PendingBlockWithTxs;
    async fn get_latest_block(&self) -> BlockWithTxs;
    async fn extract_token_from_calldata(&self, contract_address : FieldElement) -> (String, String, String);
    fn get_amm_parameter(&self) -> (DataType, DataType);
}