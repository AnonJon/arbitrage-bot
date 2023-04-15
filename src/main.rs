use crate::utils::{check_arbitrage_opportunity, create_client, get_reserves};
use ethers::core::types::U256;
use eyre::Result;
mod config;
mod contract_interfaces;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = create_client().await?;
    let alert_threshold = U256::from(1);

    let (sushi_left, sushi_right, sushi_reserve_a, sushi_reserve_b) =
        get_reserves(&client, config::SUSHI_LINK_ETH_ADDRESS.parse()?).await?;
    let (uni_left, uni_right, uni_reserve_a, uni_reserve_b) =
        get_reserves(&client, config::UNISWAP_LINK_ETH_ADDRESS.parse()?).await?;

    let prices_and_reserves_left = vec![
        (
            String::from("SushiSwap"),
            sushi_left,
            sushi_reserve_a,
            sushi_reserve_b,
        ),
        (
            String::from("Uniswap"),
            uni_left,
            uni_reserve_a,
            uni_reserve_b,
        ),
    ];

    let prices_and_reserves_right = vec![
        (
            String::from("SushiSwap"),
            sushi_right,
            sushi_reserve_a,
            sushi_reserve_b,
        ),
        (
            String::from("Uniswap"),
            uni_right,
            uni_reserve_a,
            uni_reserve_b,
        ),
    ];

    let trade_amount_range: (U256, U256, U256) = (
        U256::from(1) * U256::from(10).pow(18.into()), // For example, 1 ETH
        U256::from(10) * U256::from(10).pow(18.into()), // For example, 10 ETH
        U256::from(1) * U256::from(10).pow(17.into()), // For example, 0.1 ETH step
    );

    let mut trade_amount = trade_amount_range.0;
    let mut arbs = 0;
    while trade_amount <= trade_amount_range.1 {
        if check_arbitrage_opportunity(
            ("LINK", "ETH"),
            &prices_and_reserves_left,
            alert_threshold,
            trade_amount.clone(),
            true,
        ) {
            arbs += 1
        }
        if check_arbitrage_opportunity(
            ("LINK", "ETH"),
            &prices_and_reserves_right,
            alert_threshold,
            trade_amount.clone(),
            false,
        ) {
            arbs += 1
        }

        trade_amount += trade_amount_range.2;
    }
    println!("Arbitrage opportunities: {}", arbs);
    // let price_impact =
    //     calculate_price_impact(trade_amount, sushi_reserve_a, sushi_reserve_b).unwrap();
    // println!("Price impact: {}", price_impact);

    // let trade = calc_amount(trade_amount, sushi_reserve_a, sushi_reserve_b);
    // println!("Trade amount: {}", trade);

    Ok(())
}
