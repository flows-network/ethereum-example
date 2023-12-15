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

pub mod ether_lib;
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
    
    let rpc_node_url = std::env::var("RPC_NODE_URL").unwrap_or("https://sepolia-rollup.arbitrum.io/rpc".to_string());
    let chain_id = std::env::var("CHAIN_ID").unwrap_or("421614".to_string()).parse::<u64>().unwrap_or(421614u64);
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
    
    let rpc_node_url = std::env::var("RPC_NODE_URL").unwrap_or("https://sepolia-rollup.arbitrum.io/rpc".to_string());
    let chain_id = std::env::var("CHAIN_ID").unwrap_or("421614".to_string()).parse::<u64>().unwrap_or(421614u64);
    let private_key = std::env::var("PRIVATE_KEY").unwrap_or("".to_string());
    let wallet: LocalWallet = private_key
    .parse::<LocalWallet>()
    .unwrap()
    .with_chain_id(chain_id);


    let reciver = NameOrAddress::from(H160::from_str(_qry.get("pay-to").expect("Require pay to address").to_string().as_str().trim_matches('"')).expect("Failed to parse address"));
    let contract_addrss = NameOrAddress::from(H160::from_str(std::env::var("CONTRACT_ADDRESS").unwrap_or("0x2ba7EA93b29286CB1f65c151ea0ad97FcCD41C91".to_string()).as_str()).expect("Failed to parse contract address"));
    let value = U256::from_dec_str("0").unwrap();
    let wei_to_eth = U256::from_dec_str("1000000000000000000").unwrap();
    let data = create_pbm_pay_data(reciver.as_address().unwrap().clone(), U256::from(10) * wei_to_eth).unwrap();

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
    
    let rpc_node_url = std::env::var("RPC_NODE_URL").unwrap_or("https://sepolia-rollup.arbitrum.io/rpc".to_string());
    let chain_id = std::env::var("CHAIN_ID").unwrap_or("421614".to_string()).parse::<u64>().unwrap_or(421614u64);
    let caller = NameOrAddress::from(H160::from_str(_qry.get("address").expect("Require an address").to_string().as_str().trim_matches('"')).expect("Failed to parse address"));
    let eth_balance = get_ethbalance(&rpc_node_url, format!("{:?}", caller.as_address().unwrap()).as_str()).await.unwrap();
    let resp:String;
    
    match chain_id{
        18 =>{
            resp = "Not implement.".to_string();
        },
        _ => {resp = "Not implement.".to_string();},
    }

    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/html"))],
        resp.into_bytes().to_vec(),
    );
}

pub async fn get_balance(_headers: Vec<(String, String)>, _qry: HashMap<String, Value>, _body: Vec<u8>){
    logger::init();
    log::info!("get txs Query -- {:?}", _qry);
    
    let rpc_node_url = std::env::var("RPC_NODE_URL").unwrap_or("https://sepolia-rollup.arbitrum.io/rpc".to_string());
    let chain_id = std::env::var("CHAIN_ID").unwrap_or("421614".to_string()).parse::<u64>().unwrap_or(421614u64);
    let caller = NameOrAddress::from(H160::from_str(_qry.get("address").expect("Require an address").to_string().as_str().trim_matches('"')).expect("Failed to parse address"));
    
    let resp = get_ethbalance(&rpc_node_url, format!("{:?}", caller.as_address().unwrap()).as_str()).await.unwrap().to_string();

    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/html"))],
        resp.into_bytes().to_vec(),
    );
}
