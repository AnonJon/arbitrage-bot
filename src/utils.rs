use crate::contract_interfaces::{IUniswapV2Pair, IERC20};
use ethers::{
    core::types::{Address, U256},
    providers::{Http, Provider},
};
use eyre::Result;
use serde::Deserialize;
use std::env;
use std::fs::File;
use std::io::Read;
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
#[derive(Debug, Deserialize)]
pub struct Pair {
    pub pair: Vec<String>,
    pub address: Address,
}

pub async fn get_symbols(
    client: &Arc<Provider<Http>>,
    contract_address: Address,
) -> Result<(String, String)> {
    let contract = IUniswapV2Pair::new(contract_address, client.clone());

    let token_a = IERC20::new(contract.token_0().call().await?, client.clone());
    let token_b = IERC20::new(contract.token_1().call().await?, client.clone());

    let token_a_name = token_a.symbol().call().await? as String;
    let token_b_name = token_b.symbol().call().await? as String;

    Ok((token_a_name, token_b_name))
}

// 1 token0 = x token1
pub async fn get_reserves(
    client: &Arc<Provider<Http>>,
    contract_address: Address,
) -> Result<(U256, U256, U256, U256)> {
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
    let current_price_left: U256;
    let current_price_right: U256;

    if token_a_decimals < 18 {
        current_price_right = (reserve1 * diviser) / reserve0;
    } else {
        current_price_right = (reserve1 * precision) / reserve0;
    }

    if token_b_decimals < 18 {
        current_price_left = (reserve0 * diviser) / reserve1;
    } else {
        current_price_left = (reserve0 * precision) / reserve1;
    }

    Ok((current_price_left, current_price_right, reserve0, reserve1))
}

pub fn check_arbitrage_opportunity(
    token_pair: (&str, &str),
    prices: &[(String, U256, U256, U256)],
    threshold: U256,
    trade_amount: U256,
    token0_to_token1: bool,
) -> bool {
    let mut buy_opportunity: Option<(String, U256, U256, U256)> = None;
    let mut sell_opportunity: Option<(String, U256, U256, U256)> = None;
    let selling = if token0_to_token1 {
        token_pair.0
    } else {
        token_pair.1
    };
    let buying = if token0_to_token1 {
        token_pair.1
    } else {
        token_pair.0
    };

    for (exchange, price, reserve_a, reserve_b) in prices {
        let calc_prices =
            calculate_price_impact(trade_amount, *reserve_a, *reserve_b, token0_to_token1);
        if let Some((price_impact, execution_price)) = calc_prices {
            println!(
                "Exchange: {} ({selling} -> {buying})| Price Per {buying}: {} {selling} | Total Selling: {} {selling} | Price Impact: {} | Execution Price: {} {buying}",
                exchange,
                price,
                trade_amount,
                price_impact,
                execution_price
            );

            if buy_opportunity
                .as_ref()
                .map_or(true, |(_, _, current_best_impact, _)| {
                    price_impact < *current_best_impact
                })
            {
                buy_opportunity = Some((exchange.clone(), *price, price_impact, execution_price));
            }
            if sell_opportunity
                .as_ref()
                .map_or(true, |(_, _, current_best_impact, _)| {
                    price_impact > *current_best_impact
                })
            {
                sell_opportunity = Some((exchange.clone(), *price, price_impact, execution_price));
            }
        }
    }

    if let (
        Some((buy_exchange, buy_price, buy_impact, _buy_execution_price)),
        Some((sell_exchange, sell_price, sell_impact, _sell_execution_price)),
    ) = (buy_opportunity, sell_opportunity)
    {
        let buy_exchange_reserves = prices
            .iter()
            .find(|(exchange, _, _, _)| exchange == &buy_exchange)
            .map(|(_, _, reserve_a, reserve_b)| (*reserve_a, *reserve_b))
            .unwrap();
        let sell_exchange_reserves = prices
            .iter()
            .find(|(exchange, _, _, _)| exchange == &sell_exchange)
            .map(|(_, _, reserve_a, reserve_b)| (*reserve_a, *reserve_b))
            .unwrap();

        let amount_out_buy = calc_amount(
            trade_amount,
            buy_exchange_reserves.0,
            buy_exchange_reserves.1,
            token0_to_token1,
        );

        let amount_out_sell = calc_amount(
            amount_out_buy,
            sell_exchange_reserves.0,
            sell_exchange_reserves.1,
            !token0_to_token1,
        );
        println!("Trade Route: {trade_amount} {selling} -> {amount_out_buy} {buying} -> {amount_out_sell} {selling}");

        if amount_out_sell > trade_amount {
            let arbitrage_profit = amount_out_sell - trade_amount;
            if arbitrage_profit >= threshold {
                let p = wei_to_eth(arbitrage_profit);
                println!(
                    "Arbitrage opportunity found! Profit {} ETH | Buy {} with {} from {} for price {} (Price Impact: {}), sell {} for {} to {} for price {} (Price Impact: {}).",
                    p,
                    selling,
                    buying,
                    buy_exchange,
                    buy_price,
                    buy_impact,
                    selling,
                    buying,
                    sell_exchange,
                    sell_price,
                    sell_impact
                );
                true
            } else {
                println!("Arbitrage opportunity found, but profit is below the threshold.");
                false
            }
        } else {
            println!("No arbitrage opportunity found with a positive profit.\n");
            false
        }
    } else {
        println!("No arbitrage opportunity found.\n");
        false
    }
}

fn calc_amount(amount: U256, reserve0: U256, reserve1: U256, token0_to_token1: bool) -> U256 {
    let in_amount_fee_adjusted = amount * U256::from(997);
    let (numerator, denominator) = if token0_to_token1 {
        (
            in_amount_fee_adjusted * reserve1,
            reserve0 * U256::from(1000) + in_amount_fee_adjusted,
        )
    } else {
        (
            in_amount_fee_adjusted * reserve0,
            reserve1 * U256::from(1000) + in_amount_fee_adjusted,
        )
    };

    let out_amount = numerator / denominator;

    out_amount
}

fn calculate_price_impact(
    amount: U256,
    reserve_a: U256,
    reserve_b: U256,
    token0_to_token1: bool,
) -> Option<(U256, U256)> {
    if amount.is_zero() || reserve_a.is_zero() || reserve_b.is_zero() {
        return None;
    }

    // Calculate mid price (without any trade)
    let mid_price = if token0_to_token1 {
        reserve_b * U256::exp10(18) / reserve_a
    } else {
        reserve_a * U256::exp10(18) / reserve_b
    };

    // Calculate out amount after the trade
    let out_amount = calc_amount(amount, reserve_a, reserve_b, token0_to_token1);

    // Calculate execution price (with slippage)
    let execution_price = out_amount * U256::exp10(18) / amount;

    // Calculate price impact
    let price_impact = if execution_price > mid_price {
        (execution_price - mid_price) * U256::from(10000) / mid_price
    } else {
        (mid_price - execution_price) * U256::from(10000) / mid_price
    };

    Some((price_impact, execution_price))
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

pub fn get_exchange_groups(exchanges: Vec<&str>, pools: Vec<Vec<Address>>) -> Vec<Vec<Address>> {
    let mut exchange_pools = vec![];
    for (i, exchange) in exchanges.iter().enumerate() {
        println!("{} liquidity pools:", exchange);

        let mut pools_for_exchange = vec![];
        for pool in &pools[i] {
            println!("{}", pool);
            pools_for_exchange.push(*pool);
        }
        exchange_pools.push(pools_for_exchange);
    }

    exchange_pools
}

pub fn read_exchanges_from_file(
    exchanges: Vec<&str>,
    network: u16,
) -> Result<Vec<Vec<Address>>, Box<dyn std::error::Error>> {
    let mut pools: Vec<Vec<Address>> = Vec::new();
    for exchange in exchanges {
        let mut x: Vec<Address> = Vec::new();
        let mut file = File::open(format!("./src/data/{network}/{exchange}.config.json"))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let pairs: Vec<Pair> = serde_json::from_str(&contents)?;
        for pair in &pairs {
            x.push(pair.address);
        }
        pools.push(x);
    }

    Ok(pools)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arbitrage_opportunity() {
        let token_pair = ("TokenA", "TokenB");
        let prices = vec![
            (
                String::from("Exchange1"),
                U256::from(100),
                U256::from(1000),
                U256::from(1000),
            ),
            (
                String::from("Exchange2"),
                U256::from(200),
                U256::from(1000),
                U256::from(500),
            ),
        ];
        let threshold = U256::from(1);
        let trade_amount = U256::from(10);
        let token0_to_token1 = true;

        let op = check_arbitrage_opportunity(
            token_pair,
            &prices,
            threshold,
            trade_amount,
            token0_to_token1,
        );
        assert_eq!(op, true);
    }

    #[test]
    fn test_no_arbitrage_opportunity() {
        let token_pair = ("TokenA", "TokenB");
        let prices = vec![
            (
                String::from("Exchange1"),
                U256::from(100),
                U256::from(1000),
                U256::from(1000),
            ),
            (
                String::from("Exchange2"),
                U256::from(100),
                U256::from(1000),
                U256::from(1000),
            ),
        ];
        let threshold = U256::from(1);
        let trade_amount = U256::from(10);
        let token0_to_token1 = true;

        let op = check_arbitrage_opportunity(
            token_pair,
            &prices,
            threshold,
            trade_amount,
            token0_to_token1,
        );
        assert_eq!(op, false);
    }
}
