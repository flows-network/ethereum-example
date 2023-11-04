use ethers_core::types::NameOrAddress;
use webhook_flows::{create_endpoint, request_handler, send_response};
use flowsnet_platform_sdk::logger;
use serde_json::Value;
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;
use ethers_signers::{LocalWallet, Signer};
use ethers_core::types::{Bytes, U256, U64};
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

    let rpc_node_url = "https://light-still-glitter.ethereum-sepolia.quiknode.pro/f5d93df9549486a186b3e6e868499925b0340d9a/";
    let private_key = std::env::var("PRIVATE_KEY").unwrap();
    let wallet: LocalWallet = private_key
    .parse::<LocalWallet>()
    .unwrap()
    .with_chain_id(11155111u64);
    

    let address_from = wallet.address();
    let address_to = qry.get("address_to").unwrap().to_string().parse::<NameOrAddress>().unwrap();
    let value = U256::from_dec_str(qry.get("value").unwrap_or(&Value::Number(0.into())).as_str().unwrap()).unwrap();
    let wei_in_gwei = U256::from(10_u64.pow(9));
    
    let nonce = get_nonce(rpc_node_url, format!("{:?}", wallet.address()).as_str()).await.unwrap();
    let data: ethers_core::types::Bytes;
    if let Some(qry_data) = qry.get("data") {      
        data = Bytes::from(hex::decode(qry_data.to_string().trim_start_matches("0x")).unwrap());
    }else{
        data = Bytes::from(vec![0u8; 32]);
    }

    let estimate_gas = get_estimate_gas(rpc_node_url, format!("{:?}", address_from).as_str(), format!("{:?}", address_to.as_address().unwrap()).as_str(), format!("0x{:x}", value).as_str(), data.clone().encode_hex().as_str()).await.unwrap();
    
    let tx: TypedTransaction = TransactionRequest::new()
    .from(address_from)
    .to(address_to) 
        .nonce::<U256>(nonce.into())
        .gas_price::<U256>((get_gas_price(rpc_node_url).await.unwrap() * wei_in_gwei).into())
        .gas::<U256>(estimate_gas.into())
        .chain_id::<U64>(11155111.into())
        .data::<Bytes>(data.into())
        .value(value).into();    


    log::info!("Tx: {:#?}", tx);
    
    
    let signature = wallet.sign_transaction(&tx).await.unwrap();
    let params = json!([format!("0x{}", hex::encode(tx.rlp_signed(&signature))).as_str()]);
    let resp = json_rpc(rpc_node_url, "eth_sendRawTransaction", params).await.unwrap();
	
    log::info!("resp: {:#?}", resp);
    
    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/html"))],
        resp.into_bytes().to_vec(),
    );
}

pub async fn get_gas_price(rpc_node_url: &str) -> Result<U256> {
    let params = json!([]);
    let result = json_rpc(rpc_node_url, "eth_gasPrice", params).await.unwrap();
    
    Ok(U256::from_str(&result)?)
}

pub async fn get_nonce(rpc_node_url: &str, address: &str) -> Result<U256> {
    let params = json!([address, "pending"]);
    let result = json_rpc(rpc_node_url, "eth_getTransactionCount", params).await.unwrap();
    
    Ok(U256::from_str(&result)?)
}

pub async fn get_estimate_gas(rpc_node_url: &str, from: &str, to: &str, value: &str, data: &str) -> Result<U256> {
    let params = json!([{"from": from, "to": to, "value":value, "data":data}]);
    let result = json_rpc(rpc_node_url, "eth_estimateGas", params).await.unwrap();
    
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

    println!("Params: {:#?}", params);
    let res = client.request(req).await?;
    let body = hyper::body::to_bytes(res.into_body()).await?;
    let map: HashMap<String, serde_json::Value> = serde_json::from_str(&String::from_utf8(body.into())?)?;
    
    println!("Response body: {:#?}", map);
    
    Ok(map["result"].as_str().unwrap().to_string())
}