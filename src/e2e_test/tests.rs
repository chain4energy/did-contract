
use cosmrs::crypto::secp256k1::SigningKey;
// use cw_multi_test::IntoAddr;
use serde_json::json;
use serial_test::serial;
use e2e_test_suite::{ADDR_PREFIX, derive_private_key_from_mnemonic};

use crate::state::{Did, DidDocument, DID_PREFIX};

const MENMONIC: &str = "harbor flee number sibling doll recycle brisk mask blanket orphan initial maze race flash limb sound wing ramp proud battle feature ceiling feel miss";
const HD_PATH: &str = "m/44'/118'/0'/0/0";

// const CONTRACT_PATH: &str = "./artifacts/did_contract.wasm";
const CONTRACT_PATH: &str = "./target/wasm32-unknown-unknown/release/did_contract.wasm";


#[test]
#[serial]
fn create_did_document() {
    init_suite();
    // setup_context();
    println!("RUN create_did_document");

    let context =  e2e_test_suite::CONTEXT.get().expect("Docker controller is not initialized");
    let context = context.lock().expect("Failed to lock Docker controller");
    
    let (key, address) = create_key_and_address();

    // let msg = r#"{
    //     "create_did_document": {
    //         "did_doc": {
    //             "id": "did:example:1234567890",
    //             "controller": ["did:user:1234567890"],
    //             "service": [
    //                 {
    //                     "id": "did:service:1234567890",
    //                     "type": "type",
    //                     "service_endpoint": "http://chargera.io"
    //                 }
    //             ]
    //         }
    //     }
    // }"#;

    // let msg = r#"
    //     {
    //         "create_did_document":{
    //             "did_doc":{
    //                 "controller":"didc4e:c4e:user:000131",
    //                 "id":"didc4e:c4e:example:000432",
    //                 "service":[
    //                     {
    //                         "id":"didc4e:c4e:service:000131",
    //                         "service_endpoint":"http://chargera.io",
    //                         "type":"Chargera"
    //                     }
    //                 ]
    //             }
    //         }
    //     }"#.to_string();

    let did = &format!("{}{}", DID_PREFIX, "example:000432");

    let did_controller = "c4e1ja3l8vxxjnq9reeal89pvt2mzf6cq9d7s5xk9d";
    let did_doc = DidDocument { 
        id: crate::state::Did::new(did), 
        // controller: Controllers(vec![format!("{}{}", DID_PREFIX, "user:000131").into(), format!("{}{}", DID_PREFIX, "user:000134").into()]), 
        // controller: vec![format!("{}{}", DID_PREFIX, "user:000131").into(), format!("{}{}", DID_PREFIX, "user:000134").into()], 
        // controller: vec![format!("{}{}", DID_PREFIX, "user:000131").into()], 
        controller: vec![did_controller.to_string().into()], 

        service: vec![crate::state::Service{
            id: crate::state::Did::new(&format!("{}{}", DID_PREFIX, "service:000131")),
            a_type: "Chargera".to_string(),
            service_endpoint: "http://chargera.io".to_string()
        }],
     };

    let create_msg = super::super::contract::sv::ExecMsg::CreateDidDocument { 
        did_doc: did_doc.clone()
    };
    
    let msg = json!(create_msg).to_string();
    println!("Message: {msg}");

    let result = context.chain.tx.execute_contract_msg(&address, &context.contract_address, &msg, vec![], &key);
    // let result = context.chain.tx.execute_contract_msg(&address, "c4e14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s86dt7n", &msg, vec![], &key);
    
    if result.is_err() {
        assert_eq!("Generic error: Querier contract error: Did document not found", result.err().unwrap().to_string());
    } else {
        assert!(result.is_ok(), "Expected Ok, but got an Err");
        
    }
    let query_msg = super::super::contract::sv::QueryMsg::GetDidDocument { did: Did::new(did) };
    let query = json!(query_msg).to_string();
    println!("Query: {query}");
    let result = context.chain.query.contract(&context.contract_address, &query);
    // if result.is_err() {
    //     assert_eq!("Generic error: Querier contract error: Did document not found", result.err().unwrap().to_string());
    // } else {
    assert!(result.is_ok(), "Expected Ok, but got an Err");
    let result = result.unwrap();

    let resp = String::from_utf8(result.data).expect("parse result error");
    println!("Resposne: {resp}");
    let resp_did_doc: DidDocument= serde_json::from_str(&resp).expect("desrializing did doc error");
    assert_eq!(did_doc.clone(), resp_did_doc);


    // }
}

#[test]
#[serial]
fn my_test_2() {
    init_suite();
    println!("RUN TEST 2")
}

#[test]
#[serial]
fn my_test_3() {
    init_suite();
    // setup_context();
    println!("RUN TEST 3");
}

fn init_suite() {
    e2e_test_suite::init_suite(MENMONIC, HD_PATH, CONTRACT_PATH, "c4e-chain-e2e-test:v1.4.3", "did-contract", "did");
}

fn create_key_and_address() -> (SigningKey, String){
    let key = derive_private_key_from_mnemonic(MENMONIC,    HD_PATH).expect("create key error");
    let address = key.public_key().account_id(ADDR_PREFIX).expect("cannot create address").to_string();
    (key, address)
}
