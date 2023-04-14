use crate::contracts::IERC20;
use ethers::core::types::{Address, U256};
use eyre::Result;

mod contracts;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let client = utils::create_client().await?;
    let erc20 = IERC20::new(
        "0x6b175474e89094c44da98b954eedeac495271d0f".parse::<Address>()?,
        client.clone(),
    );

    let answer: U256 = erc20.total_supply().call().await?;
    println!("Total supply: {}", answer);

    Ok(())
}
