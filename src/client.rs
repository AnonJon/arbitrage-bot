use crate::utils::Pair;
use ethers::{
    core::types::Address,
    providers::{Http, Provider},
};
use eyre::Result;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;

pub struct ArbClient {
    pub client: Arc<Provider<Http>>,
    pub network: u64,
    pub exchanges: Vec<String>,
}

impl ArbClient {
    pub async fn new(network: u64, exchanges: Vec<String>) -> Result<Self> {
        let client = create_client().await?;

        Ok(Self {
            client,
            network,
            exchanges,
        })
    }

    pub fn get_common_pairs(
        &self,
        exchanges: &[&str],
    ) -> Result<HashMap<(String, String), Vec<(String, Address)>>, Box<dyn std::error::Error>> {
        let mut all_pairs: Vec<HashMap<(String, String), Address>> = Vec::new();
        let network = self.network;
        for exchange in exchanges {
            let mut file = File::open(format!("./src/data/{network}/{exchange}.config.json"))?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            let data: Vec<Pair> = serde_json::from_str(&contents)?;
            let pairs_map = data
                .into_iter()
                .map(|pair_data| {
                    (
                        (pair_data.pair[0].clone(), pair_data.pair[1].clone()),
                        pair_data.address,
                    )
                })
                .collect();
            all_pairs.push(pairs_map);
        }

        let mut common_pairs: HashMap<(String, String), Vec<(String, Address)>> = HashMap::new();

        for (exchange, pairs) in exchanges.iter().zip(all_pairs.iter()) {
            for (symbol_pair, address) in pairs {
                common_pairs
                    .entry(symbol_pair.clone())
                    .or_insert_with(Vec::new)
                    .push((exchange.to_string(), *address));
            }
        }

        common_pairs.retain(|_, v| v.len() >= 2);

        Ok(common_pairs)
    }
}

async fn create_client() -> Result<Arc<Provider<Http>>> {
    let rpc_url = &env::var("ETHEREUM_RPC_URL").expect("ETHEREUM_RPC_URL must be set");
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let client = Arc::new(provider);

    Ok(client)
}
