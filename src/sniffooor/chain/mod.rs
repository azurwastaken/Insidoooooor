pub mod strknet;
// pub mod solana;


use async_trait::async_trait;
use starknet::core::types::{FieldElement};

type StarknetBlock = starknet::core::types::PendingBlockWithTxs;
type StarknetLatestBlock = starknet::core::types::BlockWithTxs;
type StarknetTx = starknet::core::types::InvokeTransactionV1;


#[derive(Clone)]
pub enum DataType {
    FieldElement(FieldElement),
}

pub enum Block {
    Starknet(StarknetBlock),
    StarknetLatest(StarknetLatestBlock),
}

impl Block {
    pub fn get_current_hash(&self) -> String {
        match self {
            Block::Starknet(_starknet_block) => String::from("PENDING"),
            Block::StarknetLatest(starknet_latest_block) => format!("{:#x}",starknet_latest_block.block_hash),
        }
    }

    //TODO : maybe return a big int to optimize
    pub fn get_parent_hash(&self) -> String {
        match self {
            Block::Starknet(starknet_block) => format!("{:#x}",starknet_block.parent_hash),
            Block::StarknetLatest(starknet_latest_block) => format!("{:#x}",starknet_latest_block.parent_hash),
        }
    }

    pub fn get_txs(&self, chain_handler : &dyn Chain) -> Vec<Tx> {
        match self {
            Block::Starknet(_starknet_block) => chain_handler.extract_tx(self),
            Block::StarknetLatest(_starknet_latest_block) => chain_handler.extract_tx(self),
        }
    }
}

pub enum Tx {
    Starknet(StarknetTx),
}

impl Tx {
    pub fn is_add_liquidity(&self, chain_handler : &dyn Chain) -> bool {
        match self {
            Tx::Starknet(_starknet_tx) => chain_handler.is_add_liquidity(self),
        }
    }

    //TODO : maybe return a big int to optimize
    pub fn get_tx_hash(&self) -> String {
        match self {
            Tx::Starknet(starknet_tx) => format!("{:#x}",starknet_tx.transaction_hash),
        }
    }
}

pub struct Token{
    name : String,
    ticker : String,
    contract_address : String,
}

impl Token {
    fn new(ca : String, ticker : String, name : String) -> Self{
        Self{
            ticker : ticker,
            name : name,
            contract_address : ca,
        }
    }
}

#[async_trait]
pub trait Chain {
    
    // both should return Object with block hash and tx array
    async fn get_pending_block(&self) -> Block;
    async fn get_latest_block(&self) -> Block;
    fn extract_tx(&self, block: &Block) -> Vec<Tx>;
    async fn extract_tokens_from_calldata(&self, wrapped_tx : &Tx) -> (Token, Token);
    fn get_amm_parameter(&self) -> (DataType, DataType);
    fn is_add_liquidity(&self,wrapped_tx : &Tx) -> bool;
}