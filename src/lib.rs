use ethers_core::types::NameOrAddress;
use webhook_flows::{create_endpoint, request_handler, send_response};
use flowsnet_platform_sdk::logger;
use serde_json::Value;
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;
use ethers_signers::{LocalWallet, Signer};
use ethers_core::types::{Bytes, U256, U64, H160};
use ethers_core::{types::TransactionRequest, types::transaction::eip2718::TypedTransaction};
use ethers_core::utils::hex;
use ethers_core::abi::AbiEncode;
use hyper::{Client, Body};
use hyper::http::{Request, Method};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;


#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    create_endpoint().await;
}

#[request_handler]
async fn handler(_headers: Vec<(String, String)>, _subpath: String, qry: HashMap<String, Value>, _body: Vec<u8>) {
    logger::init();
    log::info!("Query -- {:?}", qry);
    
    let rpc_node_url = std::env::var("RPC_NODE_URL").unwrap_or("https://sepolia-rollup.arbitrum.io/rpc".to_string());
    let chain_id = std::env::var("CHAIN_ID").unwrap_or("421614".to_string()).parse::<u64>().unwrap_or(11155111u64);
    let private_key = std::env::var("PRIVATE_KEY").unwrap_or("".to_string());
    log::info!("ENV: {} {} {}", rpc_node_url, chain_id, private_key);
    let wallet: LocalWallet = private_key
    .parse::<LocalWallet>()
    .unwrap()
    .with_chain_id(chain_id);


    let address_from = wallet.address();
    let address_to = NameOrAddress::from(H160::from_str(qry.get("address_to").expect("Require address_to").to_string().as_str().trim_matches('"')).expect("Failed to parse address_to"));
    let mut value = U256::from_dec_str("0").unwrap();
    if let Some(_value) = qry.get("value") {
        value = U256::from_dec_str(qry.get("value").unwrap_or(&Value::Number(0.into())).as_str().unwrap().trim_matches('"')).expect("Failed to parse value.");
    }
    let nonce = get_nonce(&rpc_node_url, format!("{:?}", wallet.address()).as_str()).await.unwrap();
    let mut data = Bytes::from(vec![0u8; 32]);
    if let Some(qry_data) = qry.get("data") {      
        data = Bytes::from(hex::decode(qry_data.to_string().trim_matches('"').trim_start_matches("0x")).expect("Failed to parse data."));
    }

    log::info!("Parameter: {:#?} {:#?} {:#?}", data, nonce, address_to);
    
    let estimate_gas = get_estimate_gas(&rpc_node_url, format!("{:?}", address_from).as_str(), 
                                        format!("{:?}", address_to.as_address().expect("Failed to transfer address")).as_str(), 
                                        format!("0x{:x}", value).as_str(), format!("{:}", data).as_str())
                                        .await
                                        .expect("Failed to gat estimate gas.");
    
    let tx: TypedTransaction = TransactionRequest::new()
    .from(address_from)
    .to(address_to) 
    .nonce::<U256>(nonce.into())
    .gas_price::<U256>(get_gas_price(&rpc_node_url).await.expect("Failed to get gas price.").into())
    .gas::<U256>(estimate_gas.into())
    .chain_id::<U64>(chain_id.into())
    .data::<Bytes>(data.into())
    .value(value).into();    
    
    
    log::info!("Tx: {:#?}", tx);
    
    
    let signature = wallet.sign_transaction(&tx).await.expect("Failed to sign.");
    let params = json!([format!("0x{}", hex::encode(tx.rlp_signed(&signature))).as_str()]);
    let resp = json_rpc(&rpc_node_url, "eth_sendRawTransaction", params).await.expect("Failed to send raw transaction.");
	
    log::info!("resp: {:#?}", resp);
    
    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/html"))],
        resp.into_bytes().to_vec(),
    );
}

pub async fn get_gas_price(rpc_node_url: &str) -> Result<U256> {
    let params = json!([]);
    let result = json_rpc(rpc_node_url, "eth_gasPrice", params).await.expect("Failed to send json.");
    
    Ok(U256::from_str(&result)?)
}

pub async fn get_nonce(rpc_node_url: &str, address: &str) -> Result<U256> {
    let params = json!([address, "pending"]);
    let result = json_rpc(rpc_node_url, "eth_getTransactionCount", params).await.expect("Failed to send json.");
    
    Ok(U256::from_str(&result)?)
}

pub async fn get_estimate_gas(rpc_node_url: &str, from: &str, to: &str, value: &str, data: &str) -> Result<U256> {
    let params = json!([{"from": from, "to": to, "value":value, "data":data}]);
    let result = json_rpc(rpc_node_url, "eth_estimateGas", params).await.expect("Failed to send json.");
    
    Ok(U256::from_str(&result)?)
}

pub async fn json_rpc(url: &str, method: &str, params: Value) -> Result<String> {
    let https = wasmedge_hyper_rustls::connector::new_https_connector(
        wasmedge_rustls_api::ClientConfig::default(),
    );
    let client = Client::builder().build::<_, hyper::Body>(https);
    let req = Request::builder()
        .method(Method::POST)
        .uri(url)
        .header("Content-Type","application/json")
        .body(Body::from(json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        }).to_string()))?;

    let res = client.request(req).await?;
    let body = hyper::body::to_bytes(res.into_body()).await?;
    let map: HashMap<String, serde_json::Value> = serde_json::from_str(&String::from_utf8(body.into())?)?;
    
    if map.contains_key("error") {
        log::error!("{} request body: {:#?}", method, json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        }));
        log::error!("{} response body: {:#?}", method, map);
    }
    Ok(map["result"].as_str().expect("Failed to parse json.").to_string())
}