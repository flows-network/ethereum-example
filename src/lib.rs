use webhook_flows::{create_endpoint, request_handler, send_response};
use flowsnet_platform_sdk::logger;
use serde_json::Value;
use std::collections::HashMap;
use ethers::prelude::*;
use ethers::types::Bytes;
use ethers::utils;


// type Client = SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    create_endpoint().await;
}

#[request_handler]
async fn handler(_headers: Vec<(String, String)>, _subpath: String, qry: HashMap<String, Value>, _body: Vec<u8>) {
    logger::init();
    log::info!("Query -- {:?}", qry);
    
    let provider = Provider::<Http>::try_from("https://polygon-mumbai.blockpi.network/v1/rpc/public").unwrap();
    let chain_id = provider.get_chainid().await.unwrap();
    let private_key = std::env::var("PRIVATE_KEY").unwrap();
    let wallet: LocalWallet = private_key
    .parse::<LocalWallet>().unwrap()
    .with_chain_id(chain_id.as_u64());

    let address_from = wallet.address();
    let address_to = qry.get("address_to").unwrap_or(&Value::String("".to_string())).to_string();
    let value = qry.get("value").unwrap_or(&Value::Number(0.into())).to_string();
    let data: Bytes = Bytes::from(qry.get("data").unwrap_or(&Value::String("".to_string())).to_string().as_bytes().to_vec());


    let tx = TransactionRequest::new()
        .to(address_to)
        .value(U256::from(utils::parse_ether(value).unwrap()))
        .data(data)
        .from(address_from);

    log::info!("Tx: {:#?}", tx);

    let client = SignerMiddleware::new(provider.clone(), wallet.clone());
    let tx = client.send_transaction(tx, None).await.unwrap().await.unwrap();


    let resp = serde_json::to_string(&tx).unwrap();
    
    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/html"))],
        resp.as_bytes().to_vec(),
    );
}