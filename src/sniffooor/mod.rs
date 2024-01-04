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

const PUBLIC_MAINNET_RPC : &str = "https://starknet-mainnet.public.blastapi.io";
const JEDISWAP : &str = "0x41fd22b238fa21cfcf5dd45a8548974d8263b3a531a60388411c5e230f97023";
const JEDISWAP_ADD_LIQUIDITY : &str = "0x2cfb12ff9e08412ec5009c65ea06e727119ad948d25c8a8cc2c86fec4adee70";

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

async fn get_token_name(contract_address : FieldElement, provider : &Arc<JsonRpcClient<HttpTransport>>) {
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

pub async fn sniffa(){
    // declare pseudo const
    let amm_contract = FieldElement::from_hex_be(JEDISWAP).unwrap();
    let add_liquidity = FieldElement::from_hex_be(JEDISWAP_ADD_LIQUIDITY).unwrap();
    
    // set provider and get last pending block
    let provider = SequencerGatewayProvider::starknet_alpha_mainnet();
    let rpc = Url::parse(PUBLIC_MAINNET_RPC).unwrap();
    let rpc_provider = Arc::new(JsonRpcClient::new(HttpTransport::new(rpc)));

    // set some flags
    let mut latest_block_hash :FieldElement = Default::default();
    let mut latest_block_len : usize = 0;

    // infinite loop to snifff
    loop {
        let latest_block = provider.get_block_with_txs(BlockId::Tag(BlockTag::Pending)).await;
        if let Ok(block) = latest_block {
            // println!("block : {:#?}", block);
            if let MaybePendingBlockWithTxs::PendingBlock(ref inner_block) = block {
                if latest_block_hash == inner_block.parent_hash {
                    println!("{:#x} : Same block as the old one, tx amount : {}",inner_block.parent_hash, inner_block.transactions.len());
                    thread::sleep(Duration::from_secs(5));
                } else {
                    println!();
                    latest_block_len = 0;
                }
                        
                for txs in &inner_block.transactions[(latest_block_len)..] {
                    let Invoke(V1(tx)) = txs else {continue ;};
                    if is_add_liquidity(&amm_contract, &add_liquidity, &tx.calldata){
                        println!();
                        println!("🚨 Jediswap 🚨 => Add Liquidity spotted :");
                        println!("📝 tx hash : {:#x}",tx.transaction_hash);

                        get_token_name(tx.calldata[tx.calldata.len() - 12], &rpc_provider).await;
                        get_token_name(tx.calldata[tx.calldata.len() - 11], &rpc_provider).await;
                        println!();
                    }
                }
                latest_block_hash = inner_block.parent_hash;
                latest_block_len = inner_block.transactions.len();
            }
        }
    }
}