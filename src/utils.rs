use crate::contract_interfaces::{IUniswapV2Pair, IERC20};
use ethers::{
    core::types::{Address, U256},
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
#[derive(PartialEq)]
pub enum TokenPair {
    Left,
    Right,
}

// async fn set_config() -> Result<Config> {
//     let rpc_url = &env::var("ETHEREUM_RPC_URL").expect("ETHEREUM_RPC_URL must be set");
//     let provider = Provider::<Http>::try_from(rpc_url)?;
//     let client = Arc::new(provider);

//     Ok(())
// }

// 1 token0 = x token1
pub async fn get_reserves(
    client: Arc<Provider<Http>>,
    contract_address: Address,
    token: TokenPair,
) -> Result<U256> {
    let contract = IUniswapV2Pair::new(contract_address, client.clone());
    let token_a = IERC20::new(contract.token_0().call().await?, client.clone());
    let token_b = IERC20::new(contract.token_1().call().await?, client.clone());
    let token_a_decimals = token_a.decimals().call().await? as u32;
    let token_b_decimals = token_b.decimals().call().await? as u32;
    let reserves = contract.get_reserves().call().await?;
    let reserve0: U256 = reserves.0.into();
    let reserve1: U256 = reserves.1.into();
    let precision = U256::exp10(18);
    let diviser = U256::from(10).pow((18 - token_b_decimals + token_a_decimals).into());
    let current_price: U256;
    if token == TokenPair::Right {
        if token_a_decimals < 18 {
            current_price = (reserve1 * diviser) / reserve0;
        } else {
            current_price = (reserve1 * precision) / reserve0;
        }
    } else {
        if token_b_decimals < 18 {
            current_price = (reserve0 * diviser) / reserve1;
        } else {
            current_price = (reserve0 * precision) / reserve1;
        }
    }

    println!(
        "Reserve0: {}, Reserve1: {}, Token A decimals: {}, Token B decimals: {}, Diviser: {}, Price: {}",
        reserve0, reserve1, token_a_decimals, token_b_decimals, diviser, current_price
    );

    Ok(current_price)
}

pub async fn create_client() -> Result<Arc<Provider<Http>>> {
    let rpc_url = &env::var("ETHEREUM_RPC_URL").expect("ETHEREUM_RPC_URL must be set");
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let client = Arc::new(provider);

    Ok(client)
}

pub fn wei_to_eth(wei: U256) -> f64 {
    let divisor = U256::exp10(18);
    let wei_f64 = wei.to_string().parse::<f64>().unwrap();
    let divisor_f64 = divisor.to_string().parse::<f64>().unwrap();
    wei_f64 / divisor_f64
}
