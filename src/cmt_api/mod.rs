use serde_json::Value;
use serde_json::json;
use std::collections::HashMap;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn get_transaction(address: &str) -> Result<Value> {
	let result = get_request(format!("?module=account&action=txlist&address={}", address).as_str())
	.await
	.unwrap();
	Ok(result)
}

pub async fn get_erc20_balance(address: &str) -> Result<Value> {
	let result = get_request(format!("?module=account&action=tokenlist&address={}", address).as_str())
	.await
	.unwrap();
	let mut balance: Vec<Value> = vec!();
	for idx in 0..result.as_array().unwrap().len() {
		if result[idx]["type"].as_str().unwrap() == "ERC-20" {
			let new_vec = json!({
				"name": result[idx]["name"],
				"symbol": result[idx]["symbol"],
				"balance": result[idx]["balance"],
				"contractAddress": result[idx]["contractAddress"],
				"decimals": result[idx]["decimals"],
			});
			balance.push(new_vec);
		} 
	}

	Ok(balance.into())
}

pub async fn get_erc20_transfer(address: &str) -> Result<Value> {
	let result = get_request(format!("?module=account&action=tokentx&address={}", address).as_str())
	.await
	.unwrap();
	Ok(result)
}

async fn get_request(query: &str) -> Result<Value> {
	let url = "https://www.cmttracking.io/api".to_string();
	let client = reqwest::Client::new();
	let res = client
		.get(url + query)
		.header("Content-Type","application/json")
		.send()
		.await?;

	let body = res.text().await?;
	let map: HashMap<String, serde_json::Value> = serde_json::from_str(body.as_str())?;
	
	
	Ok(map["result"].clone())

} 