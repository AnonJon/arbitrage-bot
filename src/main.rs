use crate::utils::{create_client, get_reserves, wei_to_eth, TokenPair};
use ethers::core::types::{Address, U256};
use eyre::Result;
mod contract_interfaces;
mod utils;

const LINK_SDL_ADDRESS: &str = "0xd27b7D42D24d8F7C1CF5C46cCD3b986C396FdE17";
const ETH_USDC_ADDRESS: &str = "0x397FF1542f962076d0BFE58eA045FfA2d347ACa0";

#[tokio::main]
async fn main() -> Result<()> {
    let client = create_client().await?;
    let x: U256 = get_reserves(client, ETH_USDC_ADDRESS.parse()?, TokenPair::Left).await?;

    println!("{:?}", x);

    Ok(())
}
