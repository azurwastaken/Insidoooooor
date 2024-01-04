use starknet::{
    core::types::{BlockId, BlockTag, MaybePendingBlockWithTxHashes, FieldElement, FunctionCall},
    providers::{jsonrpc::HttpTransport, JsonRpcClient, Provider, SequencerGatewayProvider},
    macros::{selector},
    providers::sequencer::models::TransactionType,
    core::utils::parse_cairo_short_string,
};

use std::thread;
use std::time::Duration;
use std::sync::Arc;
use url::Url;


const PUBLIC_MAINNET_RPC : &str = "https://starknet-mainnet.public.blastapi.io";

async fn get_token_name(contract_address : FieldElement) {
    let rpc = Url::parse(PUBLIC_MAINNET_RPC).unwrap();
    let provider = Arc::new(JsonRpcClient::new(HttpTransport::new(rpc)));

    let selector = selector!("name");

    let calldata = vec![];

    let result = provider
        .call(
            FunctionCall {
                contract_address,
                entry_point_selector: selector,
                calldata,
            },
            BlockId::Tag(BlockTag::Pending),
        )
        .await;

    let res = result.unwrap();
    println!("💰 Token {0:#?} => {1:#x}", parse_cairo_short_string(&res[0]).unwrap(), contract_address);
}

#[tokio::main]
async fn main() {
    
    // declare pseudo const
    let Ok(jediswap_amm_swap) = FieldElement::from_hex_be("0x41fd22b238fa21cfcf5dd45a8548974d8263b3a531a60388411c5e230f97023") else { todo!() };
    let Ok(add_liquidity) = FieldElement::from_hex_be("0x2cfb12ff9e08412ec5009c65ea06e727119ad948d25c8a8cc2c86fec4adee70") else { todo!() };
    
    // set provider and get last pending block
    let provider = SequencerGatewayProvider::starknet_alpha_mainnet();
    let mut latest_block_hash :FieldElement = Default::default();
    let mut latest_block_len : usize = 1;
    while true {
        let latest_block = provider.get_block_with_tx_hashes(BlockId::Tag(BlockTag::Pending)).await;
        if let Ok(block) = latest_block {
            if let MaybePendingBlockWithTxHashes::PendingBlock(ref inner_block) = block {
                if latest_block_hash == inner_block.parent_hash {
                    println!("{:#x} : Same block as the old one, tx amount : {}",inner_block.parent_hash, inner_block.transactions.len());
                    thread::sleep(Duration::from_secs(5));
                } else {
                    latest_block_len = 1;
                }
                
    
                for tx_hash in &inner_block.transactions[(latest_block_len - 1)..] {
                    // Fetch the full transaction details for each hash
                    // Note: Replace `get_transaction` with the actual method name
                    let transaction_details = provider.get_transaction(*tx_hash).await;
                    // println!("{transaction_details:#?}");
                    if let Ok(ref transac) = transaction_details {
                        if let Some(TransactionType::InvokeFunction(ref invoke_func_trans)) = transac.r#type {
                            // println!("{0:#?}", invoke_func_trans.calldata);
                            if invoke_func_trans.calldata.contains(&jediswap_amm_swap){
                            // if invoke_func_trans.calldata.contains(&add_liquidity){
                                for (index, data) in invoke_func_trans.calldata.iter().enumerate() {
                                    if *data == add_liquidity {
                                        
                                        println!("🚨 Jediswap 🚨 => Add Liquidity spotted :");
                                        println!("📝 tx hash : {:#x}",*tx_hash);
                                        // println!("data : {:#?}",invoke_func_trans.calldata);

                                        get_token_name(invoke_func_trans.calldata[invoke_func_trans.calldata.len() - 12]).await;
                                        get_token_name(invoke_func_trans.calldata[invoke_func_trans.calldata.len() - 11]).await;
                                        println!();
                                        println!();
                                    }
                                }
                            }
                        }
                    }
                }
                latest_block_hash = inner_block.parent_hash;
                latest_block_len = inner_block.transactions.len();
            }
        }
    }
}