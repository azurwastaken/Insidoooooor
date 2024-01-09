pub mod chain;

use std::thread;
use std::time::Duration;

use crate::sniffooor::chain::*;
use crate::sniffooor::chain::strknet::StarknetChain;

async fn get_missing_tx<T: Chain>(latest_block_len : usize,  chain_handler : T) {
    let block = chain_handler.get_latest_block().await;
    let txs = block.get_txs(&chain_handler);

    if txs.len() > latest_block_len {
        println!("🚨 {} : {} tx missing processing ... 🚨",block.get_current_hash(), txs.len() - latest_block_len);
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
        let block : Block = chain_handler.get_pending_block().await;
        let txs : Vec<Tx> = block.get_txs(&chain_handler);
        if latest_block_hash == block.get_parent_hash() {
            println!("{} : Same parent as the old one, current block tx amount : {}",block.get_parent_hash(), txs.len());
            thread::sleep(Duration::from_secs(5));
        } else {
            let _external_thread = tokio::spawn(get_missing_tx(latest_block_len.clone(), chain_handler.clone()));
            println!();
            latest_block_len = 0;
        }
        if latest_block_len > txs.len(){
            latest_block_len = 0;
        }
        //parse transaction
        for tx in &txs[(latest_block_len)..] {
            if tx.is_add_liquidity(&chain_handler) {
                println!();
                println!("🚨 Jediswap 🚨 => Add Liquidity spotted :\n📝 tx hash : {:#?}", tx.get_tx_hash());
                let (_token_a, _token_b) = chain_handler.extract_tokens_from_calldata(tx).await;
                println!();
            }
        }
        latest_block_hash = block.get_parent_hash();
        latest_block_len = txs.len();
    }
}