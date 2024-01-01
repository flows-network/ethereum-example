use serde_json::Value;
use serde_json::json;
// use std::time::Duration;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
use http_req::{
    request::{Method, Request},
    uri::Uri,
};
use std::convert::TryFrom;

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
	println!("get_erc20_transfer: {} {} {}", address, api_key, chain_id);
	let result = get_request(format!("{}/erc20/transfers?chain={:#x}", address, chain_id).as_str(), api_key)
	.await
	.unwrap();
	Ok(result)
}

async fn get_request(query: &str, api_key: &str) -> Result<Value> {
	log::info!("get_request: {} {}", query, api_key);
	// let client = reqwest::Client::new();
	// let res = client
	// 	.get(url + query)
	// 	// .timeout(Duration::from_secs(60))
	// 	.header("accept", "application/json")
	// 	.header("X-API-Key", api_key)
	// 	.send()
	// 	.await;
	// let res = match res {
	// 	Ok(ok) => ok,
	// 	Err(err) => {
	// 		log::error!("{err}");
	// 		panic!("{err}");
	// 	}
	// };
	let url = format!("https://deep-index.moralis.io/api/v2.2/{}", query);
	let addr = Uri::try_from(url.as_str()).unwrap();
    let mut writer:Vec<u8> = Vec::new();

    Request::new(&addr)
        .method(Method::GET)
        // .header("Connection", "Close")
        .header("accept", "application/json")
        .header("X-API-Key", api_key)
        .send(&mut writer)
        .unwrap();
    let body = std::str::from_utf8(&writer)?;
	let res_json: Value = serde_json::from_str(body)?;
	// let map: HashMap<String, serde_json::Value> = res_json; 
	
	if res_json.is_object() && res_json.get("result") != None {
		Ok(res_json.get("result").unwrap().clone())
	} else {
		Ok(res_json)
	}
} 
