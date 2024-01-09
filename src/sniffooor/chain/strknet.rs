use starknet::{
    core::types::{BlockId, BlockTag,MaybePendingBlockWithTxs, FieldElement, FunctionCall,Transaction,InvokeTransaction},
    providers::{jsonrpc::HttpTransport, JsonRpcClient, Provider, SequencerGatewayProvider},
    macros::{selector},
    core::utils::parse_cairo_short_string,
};


use std::sync::Arc;
use url::Url;



use crate::sniffooor::chain::*;
use async_trait::async_trait;


pub const PUBLIC_MAINNET_RPC : &str = "https://starknet-mainnet.public.blastapi.io";
pub const JEDISWAP : &str = "0x41fd22b238fa21cfcf5dd45a8548974d8263b3a531a60388411c5e230f97023";
pub const JEDISWAP_ADD_LIQUIDITY : &str = "0x2cfb12ff9e08412ec5009c65ea06e727119ad948d25c8a8cc2c86fec4adee70";

#[derive(Clone)]
pub struct StarknetChain {
    // declare pseudo const
    pub amm_contract : FieldElement,
    pub add_liquidity : FieldElement,
    
    // set provider and get last pending block
    pub provider : SequencerGatewayProvider,
    pub rpc_provider : Arc<JsonRpcClient<HttpTransport>>,
}

impl StarknetChain {
    pub fn new() -> Self {
        // declare pseudo const
        let rpc = Url::parse(PUBLIC_MAINNET_RPC).unwrap();

        Self{
            amm_contract : FieldElement::from_hex_be(JEDISWAP).unwrap(),
            add_liquidity : FieldElement::from_hex_be(JEDISWAP_ADD_LIQUIDITY).unwrap(),
            provider : SequencerGatewayProvider::starknet_alpha_mainnet(),
            rpc_provider : Arc::new(JsonRpcClient::new(HttpTransport::new(rpc))),
        }
    }

}

#[async_trait]
impl Chain for StarknetChain {
    async fn get_pending_block(&self) -> Block {
        let latest_block = self.provider.get_block_with_txs(BlockId::Tag(BlockTag::Pending)).await;
        if let Ok(block) = latest_block {
            if let MaybePendingBlockWithTxs::PendingBlock(inner_block) = block {
                Block::Starknet(inner_block)
            } else {
                todo!()
            }
        } else {
            todo!()
        }
    }

    async fn get_latest_block(&self) -> Block {
        let latest_block = self.rpc_provider.get_block_with_txs(BlockId::Tag(BlockTag::Latest)).await;
        if let Ok(block) = latest_block {
            if let MaybePendingBlockWithTxs::Block(inner_block) = block {
                Block::StarknetLatest(inner_block)
            } else {
                todo!()
            }
        } else {
            todo!()
        }
    }

    fn extract_tx(&self, block: &Block) -> Vec<Tx> {
        match block {
            Block::Starknet(starknet_block) => {
                starknet_block.transactions.clone().into_iter().filter_map(|transaction| {
                    if let Transaction::Invoke(InvokeTransaction::V1(v1_tx)) = transaction {
                        Some(Tx::Starknet(v1_tx))
                    } else {
                        None
                    }
                }).collect()
            },
            Block::StarknetLatest(starknet_block) => {
                starknet_block.transactions.clone().into_iter().filter_map(|transaction| {
                    if let Transaction::Invoke(InvokeTransaction::V1(v1_tx)) = transaction {
                        Some(Tx::Starknet(v1_tx))
                    } else {
                        None
                    }
                }).collect()
            },
            // Handle other block types (Solana, Avax, etc.)
        }
    }

    fn get_amm_parameter(&self) -> (DataType, DataType) {
        return (DataType::FieldElement(self.amm_contract), DataType::FieldElement(self.add_liquidity));
    }


    //TODO : refacto
    async fn extract_tokens_from_calldata(&self, wrapped_tx : &Tx) -> (Token, Token){
        let Tx::Starknet(tx) = wrapped_tx;
        let token_a_ca = tx.calldata[tx.calldata.len() - 12] ;
        let token_b_ca = tx.calldata[tx.calldata.len() - 11];

        let mut result = self.rpc_provider
        .call(
            FunctionCall {
                contract_address: token_a_ca,
                entry_point_selector: selector!("name"),
                calldata: vec![],
            },
            BlockId::Tag(BlockTag::Pending),
        )
        .await;
        let mut name = result.unwrap()[0];

        result = self.rpc_provider
            .call(
                FunctionCall {
                    contract_address: token_a_ca,
                    entry_point_selector: selector!("symbol"),
                    calldata: vec![],
                },
                BlockId::Tag(BlockTag::Pending),
            )
            .await;

        let mut ticker = result.unwrap()[0];

        let token_a = Token::new(format!("{:#x}",token_a_ca), parse_cairo_short_string(&ticker).unwrap(), parse_cairo_short_string(&name).unwrap());

        result = self.rpc_provider
        .call(
            FunctionCall {
                contract_address: token_b_ca,
                entry_point_selector: selector!("name"),
                calldata: vec![],
            },
            BlockId::Tag(BlockTag::Pending),
        )
        .await;
        name = result.unwrap()[0];

        result = self.rpc_provider
            .call(
                FunctionCall {
                    contract_address: token_b_ca,
                    entry_point_selector: selector!("symbol"),
                    calldata: vec![],
                },
                BlockId::Tag(BlockTag::Pending),
            )
            .await;

        ticker = result.unwrap()[0];

        let token_b = Token::new(format!("{:#x}",token_b_ca), parse_cairo_short_string(&ticker).unwrap(), parse_cairo_short_string(&name).unwrap());

        println!("💰 Token {:#?} ({:#?}) => {:#?}", token_a.name, token_a.ticker, token_a.contract_address);
        println!("💰 Token {:#?} ({:#?}) => {:#?}", token_b.name, token_b.ticker, token_b.contract_address);

        
        return(token_a,token_b);
    }

    fn is_add_liquidity(&self, wrapped_tx : &Tx) -> bool {
        let Tx::Starknet(tx) = wrapped_tx;
        let mut value1_found = false;
        let mut value2_found = false;
        
    
        for data in &tx.calldata {
            if *data == self.amm_contract {
                value1_found = true;
            } else if *data == self.add_liquidity {
                value2_found = true;
            }
    
            // Break early if both values are found
            if value1_found && value2_found {
                return true;
            }
        }
        return false;
    }
}