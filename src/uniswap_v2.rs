use crate::contract_interfaces::IUniswapV2Pair;
use ethers::{
    contract::{Multicall, MulticallVersion},
    core::types::{Address, U256},
    providers::{Http, Provider},
};
use eyre::Result;
use std::sync::Arc;

pub async fn uniswap_v2_pair(
    client: &Arc<Provider<Http>>,
    contract_address: Address,
) -> Result<(Vec<Address>, Vec<U256>)> {
    let mut tokens: Vec<Address> = vec![];
    let mut reserves: Vec<U256> = vec![];
    let mut multicall: Multicall<Provider<Http>> = Multicall::new(client.clone(), None)
        .await?
        .version(MulticallVersion::Multicall3);

    let contract = IUniswapV2Pair::new(contract_address, client.clone());
    let token_res: ((bool, Address), (bool, Address)) = multicall
        .add_call(contract.token_0(), false)
        .add_call(contract.token_1(), false)
        .call()
        .await?;
    tokens.push(token_res.0 .1);
    tokens.push(token_res.1 .1);
    let reserve_res: (u128, u128, u32) = contract.get_reserves().call().await?;
    reserves.push(U256::from(reserve_res.0));
    reserves.push(U256::from(reserve_res.1));

    Ok((tokens, reserves))
}
