use starknet::{
    core::types::{BlockId, BlockTag, MaybePendingBlockWithTxHashes, FieldElement},
    providers::{Provider, SequencerGatewayProvider},
    providers::sequencer::models::TransactionType,
};


#[tokio::main]
async fn main() {
    
    // declare pseudo const
    let Ok(jediswap_amm_swap) = FieldElement::from_hex_be("0x41fd22b238fa21cfcf5dd45a8548974d8263b3a531a60388411c5e230f97023") else { todo!() };
    let Ok(_add_liquidity) = FieldElement::from_hex_be("0x2cfb12ff9e08412ec5009c65ea06e727119ad948d25c8a8cc2c86fec4adee70") else { todo!() };
    
    // set provider and get last pending block
    let provider = SequencerGatewayProvider::starknet_alpha_mainnet();
    let latest_block = provider.get_block_with_tx_hashes(BlockId::Tag(BlockTag::Pending)).await;

    if let Ok(block) = latest_block {
        if let MaybePendingBlockWithTxHashes::PendingBlock(ref inner_block) = block {
            println!("parent hash : {:#x}",inner_block.parent_hash);


            for tx_hash in &inner_block.transactions {
                // Fetch the full transaction details for each hash
                // Note: Replace `get_transaction` with the actual method name
                let transaction_details = provider.get_transaction(*tx_hash).await;
                // println!("{transaction_details:#?}");
                if let Ok(transac) = transaction_details {
                    if let Some(TransactionType::InvokeFunction(invoke_func_trans)) = transac.r#type {
                        // println!("{0:#?}", invoke_func_trans.calldata);
                        if invoke_func_trans.calldata.contains(&jediswap_amm_swap){
                            println!("tx hash : {:#x}",*tx_hash);
                            for data in invoke_func_trans.calldata{
                                println!("{0:#x}", data);
                            }
                            println!();
                            println!();
                        }
                    }
                }
            }
        }
    }
}