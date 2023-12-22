use serde_json::Value;
use serde_json::json;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn get_transaction(address: &str, api_key: &str, chain_id: u64) -> Result<Value> {
	let result = get_request(format!("{}?chain={:#x}", address, chain_id).as_str(), api_key)
	.await
	.unwrap();
	Ok(result)
}

pub async fn get_erc20_balance(address: &str, api_key: &str, chain_id: u64) -> Result<Value> {
	let result = get_request(format!("{}/erc20?chain={:#x}", address, chain_id).as_str(), api_key)
	.await
	.unwrap();
	let mut balance: Vec<Value> = vec!();
	for idx in 0..result.as_array().unwrap().len() {
		let new_vec = json!({
			"name": result[idx]["name"],
			"symbol": result[idx]["symbol"],
			"balance": result[idx]["balance"],
			"contractAddress": result[idx]["token_address"],
			"decimals": result[idx]["decimals"],
		});
		balance.push(new_vec);
	}

	Ok(balance.into())
}

pub async fn get_erc20_transfer(address: &str, api_key: &str, chain_id: u64) -> Result<Value> {
	let result = get_request(format!("{}/erc20/transfers?chain={:#x}", address, chain_id).as_str(), api_key)
	.await
	.unwrap();
	Ok(result)
}

async fn get_request(query: &str, api_key: &str) -> Result<Value> {
	let url = "https://deep-index.moralis.io/api/v2.2/".to_string();
	let client = reqwest::Client::new();
	let res = client
		.get(url + query)
		.header("accept", "application/json")
		.header("X-API-Key", api_key)
		.send()
		.await?;

	let body = res.text().await?;
	let res_json: Value = serde_json::from_str(body.as_str())?;
	// let map: HashMap<String, serde_json::Value> = res_json; 
	
	if res_json.is_object() && res_json.get("result") != None {
		Ok(res_json.get("result").unwrap().clone())
	} else {
		Ok(res_json)
	}
} 
