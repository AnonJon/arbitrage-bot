use crate::utils::{
    check_arbitrage_opportunity, create_client, get_reserves, get_symbols, read_exchanges_from_file,
};
use ethers::core::types::U256;
use eyre::Result;
mod config;
mod contract_interfaces;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = create_client().await?;
    let exchanges = vec!["Sushiswap", "UniswapV2"];
    let network = 1;
    let e = exchanges.clone();
    let pools = read_exchanges_from_file(exchanges, network)?;
    println!("Pools: {:?}", pools);

    let alert_threshold = U256::from(10);
    for pool_index in 0..pools[0].len() {
        let (symbol_a, symbol_b) = get_symbols(&client, pools[0][pool_index]).await?;

        let mut prices_and_reserves_left = vec![];
        let mut prices_and_reserves_right = vec![];

        for (exchange, exchange_pools) in e.iter().zip(pools.iter()) {
            let (left, right, reserve_a, reserve_b) =
                get_reserves(&client, exchange_pools[pool_index]).await?;
            prices_and_reserves_left.push((exchange.to_string(), left, reserve_a, reserve_b));
            prices_and_reserves_right.push((exchange.to_string(), right, reserve_a, reserve_b));
        }

        let trade_amount_range: (U256, U256, U256) = (
            U256::from(1) * U256::from(10).pow(18.into()), // For example, 1 ETH
            U256::from(5) * U256::from(10).pow(18.into()), // For example, 10 ETH
            U256::from(1) * U256::from(10).pow(18.into()), // For example, 0.1 ETH step
        );

        let mut trade_amount = trade_amount_range.0;
        let mut arbs = 0;
        while trade_amount <= trade_amount_range.1 {
            if check_arbitrage_opportunity(
                (&symbol_a, &symbol_b),
                &prices_and_reserves_left[..],
                alert_threshold,
                trade_amount.clone(),
                true,
            ) {
                arbs += 1
            }
            if check_arbitrage_opportunity(
                (&symbol_a, &symbol_b),
                &prices_and_reserves_right[..],
                alert_threshold,
                trade_amount.clone(),
                false,
            ) {
                arbs += 1
            }

            trade_amount += trade_amount_range.2;
        }
        println!(
            "Arbitrage opportunities for {}-{} pool: {}",
            symbol_a, symbol_b, arbs
        );
    }

    Ok(())
}
