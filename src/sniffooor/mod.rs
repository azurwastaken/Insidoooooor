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
    let inner_block = chain_handler.get_latest_block().await;

    let (amm, add_liq) = match chain_handler.get_amm_parameter(){
        (DataType::FieldElement(a),DataType::FieldElement(b)) => (a,b),
        (_,_) => panic!("Unknown Type"),
    };

    if inner_block.transactions.len() != latest_block_len {
        println!("🚨 {:#x} : {} tx missing processing ... 🚨",inner_block.parent_hash, inner_block.transactions.len() - latest_block_len);

    }
    for txs in &inner_block.transactions[(latest_block_len)..] {
        let Invoke(V1(tx)) = txs else {continue ;};
        if is_add_liquidity(&amm, &add_liq, &tx.calldata){
            println!();

            println!("🚨 Jediswap 🚨 => Add Liquidity MISSED :\n📝 tx hash : {:#x}", tx.transaction_hash);
            chain_handler.extract_token_from_calldata(tx.calldata[tx.calldata.len() - 12]).await;
            chain_handler.extract_token_from_calldata(tx.calldata[tx.calldata.len() - 11]).await;
            
            println!();
        }
    }
}

pub async fn sniffa(){
    let chain_handler = StarknetChain::new();
    
    // set some flags
    let mut latest_block_hash :FieldElement = Default::default();
    let mut latest_block_len : usize = 0;
    // infinite loop to snifff
    loop {
        //get_pending_block
        let inner_block = chain_handler.get_pending_block().await;
        if latest_block_hash == inner_block.parent_hash {
            println!("{:#x} : Same block as the old one, tx amount : {}",inner_block.parent_hash, inner_block.transactions.len());
            thread::sleep(Duration::from_secs(5));
        } else {
            let external_thread = tokio::spawn(get_missing_tx(latest_block_len.clone(), chain_handler.clone()));
            latest_block_len = 0;
        }
        if latest_block_len > inner_block.transactions.len(){
            latest_block_len = 0;
        }
        //parse transaction
        for txs in &inner_block.transactions[(latest_block_len)..] {
            let Invoke(V1(tx)) = txs else {continue ;};
            if is_add_liquidity(&chain_handler.amm_contract, &chain_handler.add_liquidity, &tx.calldata){
                println!();
                println!("🚨 Jediswap 🚨 => Add Liquidity spotted :\n📝 tx hash : {:#x}", tx.transaction_hash);
                chain_handler.extract_token_from_calldata(tx.calldata[tx.calldata.len() - 12]).await;
                chain_handler.extract_token_from_calldata(tx.calldata[tx.calldata.len() - 11]).await;
                println!();
            }
        }
        latest_block_hash = inner_block.parent_hash;
        latest_block_len = inner_block.transactions.len();
    }
}