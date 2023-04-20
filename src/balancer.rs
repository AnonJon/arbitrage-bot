use crate::config::BALANCER_VAULT_ADDRESS;
use crate::contract_interfaces::{IBalancerPool, IBalancerVault};
use crate::utils::bytes32_from_hex;
use ethers::{
    core::types::{Address, U256},
    providers::{Http, Provider},
    utils::hex::ToHex,
};
use eyre::Result;
use std::sync::Arc;

pub async fn balancer_pair(
    client: &Arc<Provider<Http>>,
    contract_address: Address,
) -> Result<(Vec<Address>, Vec<U256>)> {
    let pool_id = get_pool_id(client, contract_address).await?;
    let contract = IBalancerVault::new(BALANCER_VAULT_ADDRESS.parse::<Address>()?, client.clone());
    let pool_id_info: (Vec<Address>, Vec<U256>, U256) = contract
        .get_pool_tokens(bytes32_from_hex(pool_id.as_str()))
        .call()
        .await?;

    Ok((pool_id_info.0, pool_id_info.1))
}

async fn get_pool_id(client: &Arc<Provider<Http>>, contract_address: Address) -> Result<String> {
    let contract = IBalancerPool::new(contract_address, client.clone());
    let pool_id: [u8; 32] = contract.get_pool_id().call().await?;
    let hex_string: String = ToHex::encode_hex(&pool_id);
    Ok(hex_string)
}
