
use cosmrs::crypto::secp256k1::SigningKey;
use ctor::dtor;
use once_cell::sync::OnceCell;
use serde_json::json;
use serial_test::serial;
// use tokio::time::sleep;
use std::{sync::{Mutex, Once}, thread::sleep, time::Duration};

use crate::e2e_test::{cosmos::ADDR_PREFIX, docker::DockerControler};

use super::cosmos::{derive_private_key_from_mnemonic, ChainClient};

// static DOCKER_SETUP: Lazy<Mutex<()>> = Lazy::new(|| {
//     setup();
//     Mutex::new(())
// });


const MENMONIC: &str = "harbor flee number sibling doll recycle brisk mask blanket orphan initial maze race flash limb sound wing ramp proud battle feature ceiling feel miss";
const HD_PATH: &str = "m/44'/118'/0'/0/0";

const CONTRACT_PATH: &str = "./artifacts/did_contract.wasm";

#[test]
#[serial]
fn create_did_document() {
    // init_suite();
    setup_context();
    println!("RUN TEST 1");

    let context = CONTEXT.get().expect("Docker controller is not initialized");
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

    let create_msg = super::super::contract::sv::ExecMsg::CreateDidDocument { 
        did_doc: crate::state::DidDocument { 
            id: crate::state::Did::new("did:example:000131"), 
            controller: vec![crate::state::Did::new("did:user:000131")], 
            service: vec![crate::state::Service{
                id: crate::state::Did::new("did:service:000131"),
                a_type: "Chargera".to_string(),
                service_endpoint: "http://chargera.io".to_string()
            }],
         } 
    };
    
    let msg = json!(create_msg).to_string();
    println!("Message: {msg}");

    // let result = context.chain.tx.execute_contract_msg(&address, &context.contract_address, msg, vec![], &key);
    let result = context.chain.tx.execute_contract_msg(&address, "c4e14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s86dt7n", &msg, vec![], &key);
    
    if result.is_err() {
        assert_eq!("Generic error: Querier contract error: Did document not found", result.err().unwrap().to_string());
    } else {
        assert!(result.is_ok(), "Expected Ok, but got an Err");
        
    }
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


static INIT: Once = Once::new();
// static DOCKER: OnceCell<Mutex<DockerControler>> = OnceCell::new();
// static BLOCKCHAIN: OnceCell<Mutex<ChainClient>> = OnceCell::new();

static CONTEXT: OnceCell<Mutex<TestSuiteContext>> = OnceCell::new();

fn init_suite() {
    INIT.call_once(|| {
        setup(); // Ensures setup runs only once
    });
}

fn setup_context() {
    CONTEXT.set(Mutex::new(TestSuiteContext::new())).expect("msg")

}

// fn setup_bc() {
//     BLOCKCHAIN.set(Mutex::new(ChainClient::new_for_test())).expect("Failed to initialize blockchain client");
// }

fn setup() {
    println!("SETUP SUITE");
    // let my_docker = DockerControler::new().expect("cannot create docker controller");
    // Initialize the Docker controller and store it in the OnceCell
    // DOCKER.set(Mutex::new(my_docker)).expect("Failed to initialize Docker controller");
    setup_context();

    let context = CONTEXT.get().expect("Docker controller is not initialized");
    let mut context = context.lock().expect("Failed to lock Docker controller");
    context.docker.run_chain().expect("run chain error");

    let (key, address) = create_key_and_address();
    // let address = key.public_key().account_id(ADDR_PREFIX).expect("cannot create address");
    loop {

        let result = context.chain.tx.bank_send(&address, "c4e1yyjfd5cj5nd0jrlvrhc5p3mnkcn8v9q8fdd9gs", 100, &key);
        match result {
            Ok(_) => {
                println!("bank_send ok");
                break
            },
            Err(_) => {
                println!("bank_send error");
                sleep(Duration::from_nanos(1000*1000));
                ()
            },
        }
    }
    
    let wasm_bytecode = std::fs::read(CONTRACT_PATH).expect("cannot read contract");
    println!("Contract Size {}", wasm_bytecode.len());

    let result = context.chain.tx.store_contract(
        &address.to_string(),
        wasm_bytecode,
        &key,
    ).expect("store contract error");
    context.contract_code_id = result.code_id;
    println!("RESULT code: {}",context.contract_code_id);

    let result = context.chain.tx.instantiate_contract(&address, &address, 1 , "{}", "did-contract", vec![],  &key).expect("instantiate contract error");
    context.contract_address = result.address;
    println!("RESULT contract: {}",context.contract_address);
    // assert!(result.is_ok(), "Expected Ok but is Err");
    // println!("Contract: {}", result.unwrap().address);
}

#[dtor]
fn teardown() {
    println!("TEARDOWN SUITE");
    // let context = CONTEXT.get().expect("Docker controller is not initialized");
    // let context = context.lock().expect("Failed to lock Docker controller");
    // context.docker.stop_chain().expect("stop chain error");
  
}

fn create_key_and_address() -> (SigningKey, String){
    let key = derive_private_key_from_mnemonic(MENMONIC,    HD_PATH).expect("create key error");
    let address = key.public_key().account_id(ADDR_PREFIX).expect("cannot create address").to_string();
    (key, address)
}

#[derive(Debug, Clone)]
struct TestSuiteContext {
    pub docker : DockerControler,
    pub chain: ChainClient,
    pub contract_code_id: u64,
    pub contract_address: String,
}

impl TestSuiteContext {
    fn new() -> TestSuiteContext{
        let my_docker = DockerControler::new().expect("cannot create docker controller");
        let chain = ChainClient::new_for_test();

        TestSuiteContext {
            chain: chain,
            docker: my_docker,
            contract_address: "".to_string(),
            contract_code_id: 0,
        }
    }

    // fn set_contract_code_id(&mut self, code_id: u64) {
    //     self.contract_code_id = code_id;
    // }

    // fn set_contract_address(&mut self, contract_address: &str) {
    //     self.contract_address = contract_address.to_string();
    // }
}
 