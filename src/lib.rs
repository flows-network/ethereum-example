use ethers_core::types::NameOrAddress;
use webhook_flows::{create_endpoint, request_handler, send_response};
use flowsnet_platform_sdk::logger;
use serde_json::Value;
use serde_json::json;
use std::collections::HashMap;
use ethers_signers::{LocalWallet, Signer};
use ethers_core::types::{Bytes, U256, U64};
use ethers_core::{types::TransactionRequest, types::transaction::eip2718::TypedTransaction};
use ethers_core::utils::hex;
use hyper::{Client, Body};
use hyper::body::HttpBody;
use hyper::http::{Request, Method};


#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    create_endpoint().await;
}

#[request_handler]
async fn handler(_headers: Vec<(String, String)>, _subpath: String, qry: HashMap<String, Value>, _body: Vec<u8>) {
    logger::init();
    log::info!("Query -- {:?}", qry);

    let https = wasmedge_hyper_rustls::connector::new_https_connector(
        wasmedge_rustls_api::ClientConfig::default(),
    );
    let client = Client::builder().build::<_, hyper::Body>(https);

    let rpc_node_url = "https://api.zan.top/node/v1/polygon/mumbai/public";    
    let private_key = std::env::var("PRIVATE_KEY").unwrap();
    let wallet: LocalWallet = private_key
    .parse::<LocalWallet>().unwrap();

    let address_to = qry.get("address_to").unwrap().to_string().parse::<NameOrAddress>().unwrap();
    let value = U256::from_dec_str(qry.get("value").unwrap_or(&Value::Number(0.into())).as_str().unwrap()).unwrap();
    
    let mut tx: TypedTransaction = TransactionRequest::new()
        .to(address_to) // this will use ENS
        .nonce::<U256>(1.into())
        .value(value).into();
    tx.set_gas::<U256>(21000.into());
    tx.set_gas_price::<U256>(U256::from_dec_str("20000000000").unwrap().into());
    tx.set_chain_id::<U64>(80001.into());

    if let Some(data) = qry.get("data") {
        tx.set_data(Bytes::from(data.to_string().as_bytes().to_vec()));
    }



    log::info!("Tx: {:#?}", tx);


    let signature = wallet.sign_transaction(&tx).await.unwrap();

    let req = Request::builder()
        .method(Method::POST)
        .uri(rpc_node_url)
        .body(Body::from(json!({
            "jsonrpc": "2.0",
            "method": "eth_sendRawTransaction",
            "params": [hex::encode(tx.rlp_signed(&signature))],
            "id": 1
        }).to_string())).unwrap();

    let mut res = client.request(req).await.unwrap();

    let mut resp_data = Vec::new();
    while let Some(next) = res.data().await {
        let chunk = next.unwrap();
        resp_data.extend_from_slice(&chunk);
    }

    let resp = serde_json::to_string(&resp_data).unwrap();
    
    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/html"))],
        resp.as_bytes().to_vec(),
    );
}