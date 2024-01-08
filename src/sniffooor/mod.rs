pub mod chain;

use starknet::{
    core::types::{BlockId, BlockTag,MaybePendingBlockWithTxs, FieldElement, FunctionCall,Transaction::Invoke, InvokeTransaction::V1},
    providers::{jsonrpc::HttpTransport, JsonRpcClient, Provider, SequencerGatewayProvider},
    macros::{selector},
    core::utils::parse_cairo_short_string,
};

use std::thread;
use std::time::Duration;
use std::sync::Arc;
use url::Url;
use crate::sniffooor::chain::*;

use crate::sniffooor::chain::strknet::StarknetChain;

fn is_add_liquidity(amm : &FieldElement, add_liquidity : &FieldElement, calldata : &[FieldElement]) -> bool {
    let mut value1_found = false;
    let mut value2_found = false;

    for &data in calldata {
        if data == *amm {
            value1_found = true;
        } else if data == *add_liquidity {
            value2_found = true;
        }

        // Break early if both values are found
        if value1_found && value2_found {
            return true;
        }
    }
    return false;
}

async fn get_missing_tx<T: Chain>(latest_block_len : usize,  chain_handler : T) {
    let block = chain_handler.get_latest_block().await;
    let txs = block.get_txs(&chain_handler);

    if txs.len() != latest_block_len {
        println!("🚨 {} : {} tx missing processing ... 🚨",block.get_parent_hash(), txs.len() - latest_block_len);
        for tx in &txs[(latest_block_len)..] {
            if tx.is_add_liquidity(&chain_handler) {
                println!();
                println!("🚨 Jediswap 🚨 => Add Liquidity MISSED :\n📝 tx hash : {:#?}", tx.get_tx_hash());
                chain_handler.extract_tokens_from_calldata(tx).await;
                // chain_handler.extract_token_from_calldata(tx).await;
                println!();
            }
        }
    }
}

pub async fn sniffa(){
    let chain_handler = StarknetChain::new();
    
    // set some flags
    let mut latest_block_hash : String = Default::default();
    let mut latest_block_len : usize = 0;
    // infinite loop to snifff
    loop {
        //get_pending_block
        let block = chain_handler.get_pending_block().await;
        let txs = block.get_txs(&chain_handler);
        if latest_block_hash == block.get_parent_hash() {
            println!("{} : Same block as the old one, tx amount : {}",block.get_parent_hash(), txs.len());
            thread::sleep(Duration::from_secs(5));
        } else {
            let external_thread = tokio::spawn(get_missing_tx(latest_block_len.clone(), chain_handler.clone()));
            println!();
            latest_block_len = 0;
        }
        if latest_block_len > txs.len(){
            latest_block_len = 0;
        }
        //parse transaction
        for tx in &txs[(latest_block_len)..] {
            // let Invoke(V1(tx)) = txs else {continue ;};
            if tx.is_add_liquidity(&chain_handler) {
                println!();
                println!("🚨 Jediswap 🚨 => Add Liquidity spotted :\n📝 tx hash : {:#?}", tx.get_tx_hash());
                chain_handler.extract_tokens_from_calldata(tx).await;
                // chain_handler.extract_token_from_calldata(tx).await;
                println!();
            }
        }
        latest_block_hash = block.get_parent_hash();
        latest_block_len = txs.len();
    }
}