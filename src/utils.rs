use ethers::{
    core::types::Address,
    providers::{Http, Provider},
};
use eyre::Result;
use serde::Deserialize;
use std::env;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(rename = "contractCreation")]
    pub contract_creation_block: u64,
    #[serde(rename = "contractAddress")]
    pub contract_address: Address,
    #[serde(rename = "blockHeight")]
    pub block_height: u64,
    #[serde(rename = "tokenAddresses")]
    pub token_addresses: Vec<Address>,
    #[serde(rename = "tokenNames")]
    pub token_names: Vec<String>,
}

// async fn set_config() -> Result<Config> {
//     let rpc_url = &env::var("ETHEREUM_RPC_URL").expect("ETHEREUM_RPC_URL must be set");
//     let provider = Provider::<Http>::try_from(rpc_url)?;
//     let client = Arc::new(provider);

//     Ok(())
// }

pub async fn create_client() -> Result<Arc<Provider<Http>>> {
    let rpc_url = &env::var("ETHEREUM_RPC_URL").expect("ETHEREUM_RPC_URL must be set");
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let client = Arc::new(provider);

    Ok(client)
}
