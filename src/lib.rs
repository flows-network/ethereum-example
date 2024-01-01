use webhook_flows::{create_endpoint, request_handler, send_response, route::{get, route, RouteError, Router}};
use flowsnet_platform_sdk::logger;
use ethers_core::rand;
use ethers_core::utils::hex;
use ethers_core::types::{NameOrAddress, Bytes, U256, H160};
use ethers_signers::{LocalWallet, Signer, MnemonicBuilder, coins_bip39::English};
use serde_json::Value;
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;
use ethers_core::abi::Token;
// use core::time::Duration;

pub mod ether_lib;
pub mod cmt_api;
pub mod moralis_api;
use ether_lib::*;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    create_endpoint().await;
}

#[request_handler]
async fn handler(_headers: Vec<(String, String)>, _subpath: String, _qry: HashMap<String, Value>, _body: Vec<u8>) {
    let mut router = Router::new();
    router
        .insert(
            "/sign-tx",
            vec![get(send_transaction)],
        )
        .unwrap();

    router
        .insert(
            "/gen-key",
            vec![get(gen_key)],
        )
        .unwrap();

    router
        .insert(
            "/pbm-pay",
            vec![get(pbm_pay)],
        )
        .unwrap();

    router
        .insert(
            "/get_txs",
            vec![get(get_txs)],
        )
        .unwrap();
    router
        .insert(
            "/get_balance",
            vec![get(get_balance)],
         )
        .unwrap();
    router
        .insert(
            "/get_pbm_from_txs",
            vec![get(get_pbm_from_txs)],
        )
        .unwrap();
    router
        .insert(
            "/get_pbm_balance",
            vec![get(get_pbm_balance)],
        )
        .unwrap();
    router
        .insert(
            "/get_pbm_to_txs",
            vec![get(get_pbm_to_txs)],
        )
        .unwrap();
    router
        .insert(
            "/get_erc20_balance",
            vec![get(get_erc20_balance)],
        )
        .unwrap();
    router
        .insert(
            "/get_erc20_from_txs",
            vec![get(get_erc20_from_txs)],
        )
        .unwrap();
    router
        .insert(
            "/get_erc20_to_txs",
            vec![get(get_erc20_to_txs)],
        )
        .unwrap();

    if let Err(e) = route(router).await {
        match e {
            RouteError::NotFound => {
                send_response(404, vec![], b"No route matched".to_vec());
            }
            RouteError::MethodNotAllowed => {
                send_response(405, vec![], b"Method not allowed".to_vec());
            }
        }
    }
}


async fn gen_key(_headers: Vec<(String, String)>, _qry: HashMap<String, Value>, _body: Vec<u8>){
    logger::init();
    log::info!("Gen key Query -- {:?}", _qry);
    let wallet;
    if let Some(_phrase) = _qry.get("phrase") {
        let phrase = _qry.get("phrase").unwrap().as_str().unwrap().trim_matches('"');
        wallet = MnemonicBuilder::<English>::default()
        .phrase(phrase)
        .build()
        .unwrap();
    } else {
        let mut rng = rand::thread_rng();
        wallet = MnemonicBuilder::<English>::default()
        .word_count(24)
        .derivation_path("m/44'/60'/0'/2/1")
        .unwrap()
        .build_random(&mut rng)
        .unwrap();   
    }

    log::info!("Your address is: {:?}, private key: 0x{}", wallet.address(), hex::encode(wallet.signer().to_bytes()));
    let resp = format!("Your address is: {:?}.", wallet.address()); 
    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/html"))],
        resp.into_bytes().to_vec(),
    );
}

async fn send_transaction(_headers: Vec<(String, String)>, _qry: HashMap<String, Value>, _body: Vec<u8>){
    logger::init();
    log::info!("Send trsaction Query -- {:?}", _qry);
    
    let rpc_node_url = std::env::var("RPC_NODE_URL").unwrap_or("https://mainnet.cybermiles.io".to_string());
    let chain_id = std::env::var("CHAIN_ID").unwrap_or("18".to_string()).parse::<u64>().unwrap_or(18u64);
    let private_key = std::env::var("PRIVATE_KEY").unwrap_or("".to_string());
    log::info!("ENV: {} {} {}", rpc_node_url, chain_id, private_key);
    let wallet: LocalWallet = private_key
    .parse::<LocalWallet>()
    .unwrap()
    .with_chain_id(chain_id);


    let address_to = NameOrAddress::from(H160::from_str(_qry.get("address_to").expect("Require address_to").to_string().as_str().trim_matches('"')).expect("Failed to parse address_to"));
    let mut value = U256::from_dec_str("0").unwrap();
    if let Some(_value) = _qry.get("value") {
        value = U256::from_dec_str(_qry.get("value").unwrap_or(&Value::Number(0.into())).as_str().unwrap().trim_matches('"')).expect("Failed to parse value.");
    }
    let mut data = Bytes::from(vec![0u8; 32]);
    if let Some(qry_data) = _qry.get("data") {      
        data = Bytes::from(hex::decode(qry_data.to_string().trim_matches('"').trim_start_matches("0x")).expect("Failed to parse data."));
    }

    log::info!("Parameter: {:#?} {:#?}", data, address_to);

    let params = json!([wrap_transaction(&rpc_node_url, chain_id, wallet, address_to, data, value).await.unwrap().as_str()]);
    let resp =json_rpc(&rpc_node_url, "eth_sendRawTransaction", params).await.expect("Failed to send raw transaction.");

    log::info!("resp: {:#?}", resp);

    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/html"))],
        resp.into_bytes().to_vec(),
    );
}

async fn pbm_pay(_headers: Vec<(String, String)>, _qry: HashMap<String, Value>, _body: Vec<u8>){
    logger::init();
    log::info!("PBM pay Query -- {:?}", _qry);
    
    let rpc_node_url = std::env::var("RPC_NODE_URL").unwrap_or("https://mainnet.cybermiles.io".to_string());
    let chain_id = std::env::var("CHAIN_ID").unwrap_or("18".to_string()).parse::<u64>().unwrap_or(18u64);
    let private_key = std::env::var("PRIVATE_KEY").unwrap_or("".to_string());
    let wallet: LocalWallet = private_key
    .parse::<LocalWallet>()
    .unwrap()
    .with_chain_id(chain_id);


    let reciver = NameOrAddress::from(H160::from_str(_qry.get("pay-to").expect("Require pay to address").to_string().as_str().trim_matches('"')).expect("Failed to parse address"));
    let contract_addrss = NameOrAddress::from(H160::from_str(std::env::var("CONTRACT_ADDRESS").unwrap_or("0xb1C1cEE9952e99f1d114f80E6a17fD598Ef106Af".to_string()).as_str()).expect("Failed to parse contract address"));
    let value = U256::from_dec_str("0").unwrap();
    let wei_to_eth = U256::from_dec_str("1000000000000000000").unwrap();
    let data = create_contract_call_data("pay",
     vec![Token::Address(reciver.as_address().unwrap().clone()), Token::Uint(U256::from(10) * wei_to_eth)])
        .unwrap();
    log::info!("Parameter: {:#?} {:#?}", data, reciver);

    let params = json!([wrap_transaction(&rpc_node_url, chain_id, wallet, contract_addrss, data, value).await.unwrap().as_str()]);
    let resp =json_rpc(&rpc_node_url, "eth_sendRawTransaction", params).await.expect("Failed to send raw transaction.");

    log::info!("resp: {:#?}", resp);

    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/html"))],
        resp.into_bytes().to_vec(),
    );
}



pub async fn get_txs(_headers: Vec<(String, String)>, _qry: HashMap<String, Value>, _body: Vec<u8>){
    logger::init();
    log::info!("get txs Query -- {:?}", _qry);
    
    let rpc_node_url = std::env::var("RPC_NODE_URL").unwrap_or("https://mainnet.cybermiles.io".to_string());
    let api_key = std::env::var("MORALIS_API_KEY").unwrap_or("".to_string());
    let chain_id = std::env::var("CHAIN_ID").unwrap_or("18".to_string()).parse::<u64>().unwrap_or(18u64);
    let caller = _qry.get("address").expect("Require an address").as_str().unwrap().trim_matches('"').to_string();
    let eth_balance = get_ethbalance(&rpc_node_url, &caller).await.unwrap();
    let mut transaction: Vec<Value> = vec!();

    match chain_id{
        18 =>{
            let query_tx = cmt_api::get_transaction(&caller).await.unwrap();
            for idx in 0..query_tx.as_array().unwrap().len() {
                if query_tx[idx]["from"].as_str().unwrap() == caller.to_lowercase() {
                    transaction.push(query_tx[idx].clone());
                } 
            }
        },
        _ => {
            let query_tx = moralis_api::get_transaction(&caller, &api_key, chain_id).await.unwrap();
            for idx in 0..query_tx.as_array().unwrap().len() {
                if query_tx[idx]["from_address"].as_str().unwrap() == caller.to_lowercase() {
                    transaction.push(query_tx[idx].clone());
                } 
            }
        },
    }
    let res_json:Value = json!({"transaction":Into::<Value>::into(transaction), "balance": eth_balance.to_string()});
    send_response(
        200,
        vec![(String::from("content-type"), String::from("application/json"))],
        serde_json::to_vec_pretty(&res_json).unwrap(),
    );
}

pub async fn get_balance(_headers: Vec<(String, String)>, _qry: HashMap<String, Value>, _body: Vec<u8>){
    logger::init();
    log::info!("get balance Query -- {:?}", _qry);
    
    let rpc_node_url = std::env::var("RPC_NODE_URL").unwrap_or("https://mainnet.cybermiles.io".to_string());
    let caller = H160::from_str(_qry.get("address").expect("Require an address").to_string().as_str().trim_matches('"')).expect("Failed to parse address");
    
    let resp = get_ethbalance(&rpc_node_url, format!("{:?}", caller).as_str()).await.unwrap().to_string();

    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/html"))],
        resp.into_bytes().to_vec(),
    );
}

pub async fn get_pbm_balance(_headers: Vec<(String, String)>, _qry: HashMap<String, Value>, _body: Vec<u8>){
    logger::init();
    log::info!("get pbm balance Query -- {:?}", _qry);
    
    let rpc_node_url = std::env::var("RPC_NODE_URL").unwrap_or("https://mainnet.cybermiles.io".to_string());
    let contract_addrss = H160::from_str(std::env::var("CONTRACT_ADDRESS").unwrap_or("0xb1C1cEE9952e99f1d114f80E6a17fD598Ef106Af".to_string()).as_str()).expect("Failed to parse contract address");
    let caller = H160::from_str(_qry.get("address").expect("Require an address").to_string().as_str().trim_matches('"')).expect("Failed to parse address");

    let data = create_contract_call_data("balanceOf", vec![Token::Address(caller.clone())]).unwrap();
    let resp = U256::from_str(
        eth_call(&rpc_node_url, "0x0000000000000000000000000000000000000000", format!("{:?}", contract_addrss).as_str(), format!("{:}", data).as_str())
        .await
        .unwrap()
        .as_str()
        )
        .unwrap()
        .to_string();

    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/html"))],
        resp.into_bytes().to_vec(),
    );
}

pub async fn get_pbm_from_txs(_headers: Vec<(String, String)>, _qry: HashMap<String, Value>, _body: Vec<u8>){
    logger::init();
    log::info!("get pbm from txs Query -- {:?}", _qry);
    
    let rpc_node_url = std::env::var("RPC_NODE_URL").unwrap_or("https://mainnet.cybermiles.io".to_string());
    let contract_addrss = std::env::var("CONTRACT_ADDRESS").unwrap_or("0xb1C1cEE9952e99f1d114f80E6a17fD598Ef106Af".to_string()).to_string();
    let query_address = H160::from_str(_qry.get("address").expect("Require an address").to_string().as_str().trim_matches('"')).expect("Failed to parse address");
    let data = create_contract_call_data("balanceOf", vec![Token::Address(query_address.clone())]).unwrap();
    let balance = U256::from_str(eth_call(&rpc_node_url, "0x0000000000000000000000000000000000000000", format!("{:?}", contract_addrss).as_str().trim_matches('"'), format!("{:}", data).as_str()).await.unwrap().as_str()).unwrap().to_string();
    let mut bytes = vec![0u8; 32];
    bytes[12..32].copy_from_slice(&query_address.0);
    let data = Bytes::from(bytes);
    // Keccak-256 payEvent(address,address,uint256)
    let log = get_log(&rpc_node_url, &contract_addrss, json!(["0x34882e90c95bfeaeb7e0738cfd8af3d1f6ab3d2065dd70f6660b404b9beb3505", format!("{:}", data).as_str()])).await.unwrap();
    let mut transaction: Vec<Value> = vec!();
    let len = log.as_array().unwrap().len();
    for idx in 0..len{
        let now = log.get(idx).unwrap();
        let pay_transaction = eth_get_tx_by_hash(&rpc_node_url, now["transactionHash"].as_str().unwrap()).await.unwrap();
        let new_vec = json!({
            "timestamp":U256::from_str(&now["data"].as_str().unwrap()[0..66]).unwrap().to_string(),
            "from": format!("0x{}", &(now["topics"][1].to_string()).trim_matches('"')[26..]),
            "to": format!("0x{}", &(now["topics"][2].to_string()).trim_matches('"')[26..]),
            "amount": U256::from_str(&now["data"].as_str().unwrap()[66..130]).unwrap().to_string(),
            "transaction_detail": pay_transaction,
        });
        transaction.push(new_vec);
    } 
    let res_json:Value = json!({"transaction":Into::<Value>::into(transaction), "balance": balance});
    
    send_response(
        200,
        vec![(String::from("content-type"), String::from("application/json"))],
        serde_json::to_vec_pretty(&res_json).unwrap(),
    );
}

pub async fn get_pbm_to_txs(_headers: Vec<(String, String)>, _qry: HashMap<String, Value>, _body: Vec<u8>){
    logger::init();
    log::info!("get pbm to txs Query -- {:?}", _qry);
    
    let rpc_node_url = std::env::var("RPC_NODE_URL").unwrap_or("https://mainnet.cybermiles.io".to_string());
    let contract_addrss = std::env::var("CONTRACT_ADDRESS").unwrap_or("0xb1C1cEE9952e99f1d114f80E6a17fD598Ef106Af".to_string()).to_string();
    let query_address = H160::from_str(_qry.get("address").expect("Require an address").to_string().as_str().trim_matches('"')).expect("Failed to parse address");
    let data = create_contract_call_data("balanceOf", vec![Token::Address(query_address.clone())]).unwrap();
    let balance = U256::from_str(eth_call(&rpc_node_url, "0x0000000000000000000000000000000000000000", format!("{:?}", contract_addrss).as_str().trim_matches('"'), format!("{:}", data).as_str()).await.unwrap().as_str()).unwrap().to_string();
    let mut bytes = vec![0u8; 32];
    bytes[12..32].copy_from_slice(&query_address.0);
    let data = Bytes::from(bytes);
    // Keccak-256 payEvent(uint256,address,address,uint256)
    let log = get_log(&rpc_node_url, &contract_addrss, json!(["0x34882e90c95bfeaeb7e0738cfd8af3d1f6ab3d2065dd70f6660b404b9beb3505", null, format!("{:}", data).as_str()])).await.unwrap();
    let mut transaction: Vec<Value> = vec!();
    let len = log.as_array().unwrap().len();
    for idx in 0..len{
        let now = log.get(idx).unwrap();
        let pay_transaction = eth_get_tx_by_hash(&rpc_node_url, now["transactionHash"].as_str().unwrap()).await.unwrap();
        let new_vec = json!({
            "timestamp":U256::from_str(&now["data"].as_str().unwrap()[0..66]).unwrap().to_string(),
            "from": format!("0x{}", &(now["topics"][1].to_string()).trim_matches('"')[26..]),
            "to": format!("0x{}", &(now["topics"][2].to_string()).trim_matches('"')[26..]),
            "amount": U256::from_str(&now["data"].as_str().unwrap()[66..130]).unwrap().to_string(),
            "transaction_detail": pay_transaction,
        });
        transaction.push(new_vec);
    } 
    let res_json:Value = json!({"transaction":Into::<Value>::into(transaction), "balance": balance});
    
    send_response(
        200,
        vec![(String::from("content-type"), String::from("application/json"))],
        serde_json::to_vec_pretty(&res_json).unwrap(),
    );
}

pub async fn get_erc20_balance(_headers: Vec<(String, String)>, _qry: HashMap<String, Value>, _body: Vec<u8>){
    logger::init();
    log::info!("get erc20 balance Query -- {:?}", _qry);
    
    let chain_id = std::env::var("CHAIN_ID").unwrap_or("18".to_string()).parse::<u64>().unwrap_or(18u64);
    let api_key = std::env::var("MORALIS_API_KEY").unwrap_or("".to_string());
    let query_address = _qry.get("address").expect("Require an address").as_str().unwrap().trim_matches('"').to_string();
    let res_json:Value;
    
    match chain_id{
        18 =>{
            res_json = cmt_api::get_erc20_balance(&query_address).await.unwrap();
            
        },
        _ => {
            res_json = moralis_api::get_erc20_balance(&query_address, &api_key, chain_id).await.unwrap();
        },
    }
    
    send_response(
        200,
        vec![(String::from("content-type"), String::from("application/json"))],
        serde_json::to_vec_pretty(&res_json).unwrap(),
    );
}

pub async fn get_erc20_from_txs(_headers: Vec<(String, String)>, _qry: HashMap<String, Value>, _body: Vec<u8>){
    logger::init();
    log::info!("get erc20 from txs Query -- {:?}", _qry);
    
    let chain_id = std::env::var("CHAIN_ID").unwrap_or("18".to_string()).parse::<u64>().unwrap_or(18u64);
    let api_key = std::env::var("MORALIS_API_KEY").unwrap_or("".to_string());
    let query_address = _qry.get("address").expect("Require an address").as_str().unwrap().trim_matches('"').to_string();
    let mut transaction: Vec<Value> = vec!();
    let balance: Value;

    match chain_id{
        18 =>{
            let txs = cmt_api::get_erc20_transfer(&query_address).await.unwrap();
            for idx in 0..txs.as_array().unwrap().len() {
                if txs[idx]["from"].as_str().unwrap() == query_address.to_lowercase() {
                    transaction.push(txs[idx].clone());
                } 
            }
            balance = cmt_api::get_erc20_balance(&query_address).await.unwrap();
        },
        _ => {
            let txs = moralis_api::get_erc20_transfer(&query_address,&api_key, chain_id).await.unwrap();
            for idx in 0..txs.as_array().unwrap().len() {
                if txs[idx]["from_address"].as_str().unwrap() == query_address.to_lowercase() {
                    transaction.push(txs[idx].clone());
                } 
            }
            balance = moralis_api::get_erc20_balance(&query_address, &api_key, chain_id).await.unwrap();
        },
    }
    
    let res_json:Value = json!({"transaction":Into::<Value>::into(transaction), "balance": balance});
    
    send_response(
        200,
        vec![(String::from("content-type"), String::from("application/json"))],
        serde_json::to_vec_pretty(&res_json).unwrap(),
    );
}

pub async fn get_erc20_to_txs(_headers: Vec<(String, String)>, _qry: HashMap<String, Value>, _body: Vec<u8>){
    logger::init();
    log::info!("get erc20 to txs Query -- {:?}", _qry);
    
    let chain_id = std::env::var("CHAIN_ID").unwrap_or("18".to_string()).parse::<u64>().unwrap_or(18u64);
    let api_key = std::env::var("MORALIS_API_KEY").unwrap_or("".to_string());
    let query_address = _qry.get("address").expect("Require an address").as_str().unwrap().trim_matches('"').to_string();
    let mut transaction: Vec<Value> = vec!();
    let balance: Value;

    match chain_id{
        18 =>{
            let txs = cmt_api::get_erc20_transfer(&query_address).await.unwrap();
            for idx in 0..txs.as_array().unwrap().len() {
                if txs[idx]["to"].as_str().unwrap() == query_address.to_lowercase() {
                    transaction.push(txs[idx].clone());
                } 
            }
            balance = cmt_api::get_erc20_balance(&query_address).await.unwrap();
        },
        _ => {
            let txs = moralis_api::get_erc20_transfer(&query_address,&api_key, chain_id).await.unwrap();
            for idx in 0..txs.as_array().unwrap().len() {
                if txs[idx]["to_address"].as_str().unwrap() == query_address.to_lowercase() {
                    transaction.push(txs[idx].clone());
                } 
            }
            balance = moralis_api::get_erc20_balance(&query_address, &api_key, chain_id).await.unwrap();
        },
    }
    
    let res_json:Value = json!({"transaction":Into::<Value>::into(transaction), "balance": balance});
    
    send_response(
        200,
        vec![(String::from("content-type"), String::from("application/json"))],
        serde_json::to_vec_pretty(&res_json).unwrap(),
    );
}