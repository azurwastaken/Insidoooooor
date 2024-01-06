use starknet::{
    core::types::{BlockId, BlockTag,MaybePendingBlockWithTxs,PendingBlockWithTxs, BlockWithTxs, FieldElement, FunctionCall,Transaction::Invoke, InvokeTransaction::V1},
    providers::{jsonrpc::HttpTransport, JsonRpcClient, Provider, SequencerGatewayProvider},
    macros::{selector},
    core::utils::parse_cairo_short_string,
};

use std::thread;
use std::time::Duration;
use std::sync::Arc;
use url::Url;
use std::any::TypeId;
use std::any::Any;

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
    async fn get_pending_block(&self) -> PendingBlockWithTxs{
        let latest_block = self.provider.get_block_with_txs(BlockId::Tag(BlockTag::Pending)).await;
        if let Ok(block) = latest_block {
            if let MaybePendingBlockWithTxs::PendingBlock(inner_block) = block {
                inner_block
            } else {
                todo!()
            }
        } else {
            todo!()
        }
    }

    async fn get_latest_block(&self) -> BlockWithTxs{
        let latest_block = self.provider.get_block_with_txs(BlockId::Tag(BlockTag::Latest)).await;
        if let Ok(block) = latest_block {
            if let MaybePendingBlockWithTxs::Block(inner_block) = block {
                inner_block
            } else {
                todo!()
            }
        } else {
            todo!()
        }
    }

    fn get_amm_parameter(&self) -> (DataType, DataType) {
        return (DataType::FieldElement(self.amm_contract), DataType::FieldElement(self.add_liquidity));
    }

    async fn extract_token_from_calldata(&self, contract_address : FieldElement) -> (String, String, String){
        let mut result = self.rpc_provider
        .call(
            FunctionCall {
                contract_address,
                entry_point_selector: selector!("name"),
                calldata: vec![],
            },
            BlockId::Tag(BlockTag::Pending),
        )
        .await;
        let name = result.unwrap()[0];

        result = self.rpc_provider
            .call(
                FunctionCall {
                    contract_address,
                    entry_point_selector: selector!("symbol"),
                    calldata: vec![],
                },
                BlockId::Tag(BlockTag::Pending),
            )
            .await;

        let ticker = result.unwrap()[0];
        
        println!("💰 Token {:#?} ({:#?}) => {:#x}", parse_cairo_short_string(&ticker).unwrap(), parse_cairo_short_string(&name).unwrap(), contract_address);
        return(parse_cairo_short_string(&ticker).unwrap(),parse_cairo_short_string(&name).unwrap(), contract_address.to_string());
    }
}