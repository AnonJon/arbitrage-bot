use arbitrage_bot::client::ArbClient;
use arbitrage_bot::utils::{check_arbitrage_opportunity, create_trade_amount_range, get_reserves};
use ethers::core::types::U256;
use eyre::Result;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exchanges = vec!["Sushiswap", "UniswapV2", "Pancakeswap", "Balancer"];
    let exchanges_vec: Vec<String> = exchanges.iter().map(|s| String::from(*s)).collect();
    let network = 1;
    let arb_client = ArbClient::new(network, exchanges_vec).await?;
    let common_pairs = arb_client.get_common_pairs(&exchanges)?;
    let mut arb_opportunities: Vec<String> = vec![];

    let alert_threshold = U256::from(10); // pbs to alert
    for (symbol_pair, exchange_addresses) in common_pairs {
        let (symbol_a, symbol_b) = symbol_pair;

        let mut prices_and_reserves_left = vec![];
        let mut prices_and_reserves_right = vec![];
        let mut decimals_a = 0;
        let mut decimals_b = 0;

        for (exchange, address) in exchange_addresses {
            let (left, right, reserve_a, reserve_b, token_a_decimal, token_b_decimal) =
                get_reserves(&arb_client.client, address, exchange.as_str()).await?;
            decimals_a = token_a_decimal;
            decimals_b = token_b_decimal;
            prices_and_reserves_left.push((exchange.clone(), left, reserve_a, reserve_b));
            prices_and_reserves_right.push((exchange.clone(), right, reserve_a, reserve_b));
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
