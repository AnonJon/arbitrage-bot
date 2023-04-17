use crate::utils::{
    check_arbitrage_opportunity, create_client, create_trade_amount_range, get_common_pairs,
    get_reserves, get_symbols,
};
use ethers::core::types::U256;
use eyre::Result;
mod config;
mod contract_interfaces;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = create_client().await?;
    let exchanges = vec!["Sushiswap", "UniswapV2", "Pancakeswap"];
    let network = 1;
    let e = exchanges.clone();
    let pools = get_common_pairs(&exchanges, network)?;
    let mut arb_opportunities: Vec<String> = vec![];

    let alert_threshold = U256::from(10);
    for pool_index in 0..pools[0].len() {
        let (symbol_a, symbol_b) = get_symbols(&client, pools[0][pool_index]).await?;

        let mut prices_and_reserves_left = vec![];
        let mut prices_and_reserves_right = vec![];
        let mut decimals_a = 0;
        let mut decimals_b = 0;

        for (exchange, exchange_pools) in e.iter().zip(pools.iter()) {
            let (left, right, reserve_a, reserve_b, token_a_decimal, token_b_decimal) =
                get_reserves(&client, exchange_pools[pool_index]).await?;
            decimals_a = token_a_decimal;
            decimals_b = token_b_decimal;
            prices_and_reserves_left.push((exchange.to_string(), left, reserve_a, reserve_b));
            prices_and_reserves_right.push((exchange.to_string(), right, reserve_a, reserve_b));
        }

        // Determine the trade amount range for token A and token B
        let trade_amount_range_a = create_trade_amount_range(decimals_a);
        let trade_amount_range_b = create_trade_amount_range(decimals_b);

        let mut trade_amount_a = trade_amount_range_a.0;
        let mut trade_amount_b = trade_amount_range_b.0;
        let mut arbs = 0;

        while trade_amount_a <= trade_amount_range_a.1 && trade_amount_b <= trade_amount_range_b.1 {
            if check_arbitrage_opportunity(
                (&symbol_a, &symbol_b),
                &prices_and_reserves_left[..],
                alert_threshold,
                trade_amount_a.clone(),
                true,
                decimals_a,
                decimals_b,
            ) {
                arbs += 1
            }
            if check_arbitrage_opportunity(
                (&symbol_a, &symbol_b),
                &prices_and_reserves_right[..],
                alert_threshold,
                trade_amount_b.clone(),
                false,
                decimals_a,
                decimals_b,
            ) {
                arbs += 1
            }

            trade_amount_a += trade_amount_range_a.2;
            trade_amount_b += trade_amount_range_b.2;
        }
        arb_opportunities.push(format!(
            "Arbitrage opportunities for {}-{} pool: {}",
            symbol_a, symbol_b, arbs
        ));
    }
    arb_opportunities.iter().for_each(|s| println!("{}", s));
    Ok(())
}
