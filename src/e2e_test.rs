use bollard::container::{Config, CreateContainerOptions, RemoveContainerOptions, StopContainerOptions};
use bollard::network::{CreateNetworkOptions/*, RemoveNetworkOptions*/};
use bollard::secret::{PortBinding, PortMap};
use bollard::Docker;
use serde::Deserialize;
use serde_json::from_str;
// use futures::stream::TryStreamExt;
use std::collections::HashMap;
use std::default::Default;
use std::env;
use std::io::{self, ErrorKind};
use std::process::Command;
use tokio::runtime::Runtime;
use cosmrs::proto::cosmos::bank::v1beta1::{QueryAllBalancesResponse};
use cosmrs::proto::cosmos::base::v1beta1::{Coin};
use reqwest::blocking::Client as ReqwestClient;
use std::error::Error;

use cosmrs::tx::{Body, Fee, SignDoc, SignerInfo, AuthInfo, Raw};
use cosmrs::crypto::secp256k1::SigningKey;
use cosmrs::proto::cosmos::bank::v1beta1::MsgSend;
use cosmrs::tx::Msg;
// use cosmrs::proto::cosmos::base::v1beta1::{Coin};
// use cosmrs::Coin;
use cosmrs::Denom;

use tendermint_rpc::client::{HttpClient, Client};
// use cometbft_rpc::::{HttpClient, Client};
use k256::ecdsa::signature::Signer;
use std::str::FromStr;
use tendermint::{block, chain};
use tendermint_rpc::endpoint::broadcast::tx_commit;
use cosmos_sdk_proto::traits::Message;
use cosmos_sdk_proto::Any;
// use cosmrs::proto::cosmos::tx::v1beta1::{TxBody, Any};

use bip39::{Mnemonic, Language};
use hdpath::{StandardHDPath};
use bip32::{XPrv, DerivationPath};

use pbkdf2::pbkdf2;
use hmac::Hmac;
use sha2::Sha512;
// use k256::ecdsa::SigningKey;
// use k256::SecretKey;
// use std::error::Error;

#[test]
fn connect_docker() {
    let docker = Docker::connect_with_local_defaults();
    assert!(docker.is_ok(), "Expected Ok, but got an Err");
    // run_chain(&docker.expect("failed to get docker"));

    let rt = Runtime::new().unwrap();

    // Run the async function and block until it completes
    let result = rt.block_on(run_chain(&docker.expect("failed to get docker")));
    assert!(result.is_ok(), "Expected Ok, but got an Err");
    // assert!(result.is_err(), "Expected Err, but got an Ok");
    // assert_eq!("Generic error: Querier contract error: Did document not found", result.err().unwrap().to_string());
}
#[test]
fn execute_query() {
    let result = query_balance_sync("c4e1au3vecfch0h5p3p90cxftrkwyfp63mj7lgxc4n", "http://localhost:31317");
}

fn query_balance_sync(account_address: &str, node_url: &str) -> Result<Vec<Coin>, Box<dyn Error>> {

    let client = ReqwestClient::new();
    let url = format!("{}/cosmos/bank/v1beta1/balances/{}", node_url, account_address);
    let response = client.get(&url).send()?.error_for_status()?;

    let response_text = response.text()?;
    println!("MESSAGE: {}", response_text);
    let balance_response: QueryAllBalancesResponse = from_str(&response_text)?;
    Ok(balance_response.balances)
}

// Function to replicate `run_chain` using the `bollard` library
async fn run_chain(docker: &Docker) -> Result<(), bollard::errors::Error> {
    // Create the network
    // let create_network_options = CreateNetworkOptions {
    //     name: "did",
    //     ..Default::default()
    // };
    // docker.create_network(create_network_options).await?;

    let uid = Command::new("id")
        .arg("-u")
        .output()
        .expect("Failed to get user ID")
        .stdout;
    let gid = Command::new("id")
        .arg("-g")
        .output()
        .expect("Failed to get group ID")
        .stdout;

    let uid_str = String::from_utf8(uid).expect("Failed to convert UID to string").trim().to_string();
    let gid_str = String::from_utf8(gid).expect("Failed to convert GID to string").trim().to_string();
    let user_str = format!("{}:{}", uid_str, gid_str); // "uid:gid"

    let mut labels = HashMap::new();
    labels.insert("com.docker.compose.project", "did-contract");

    let current_dir = env::current_dir().expect("Failed to get current directory");

    // Create and start containers
    for (name, volume, port_bindings) in [
        ("chain-node-did-1", ".e2e/node1", [("26657/tcp", "31657"), ("1317/tcp", "31317")]),
        ("chain-node-did-2", ".e2e/node2", [("26657/tcp", "32657"), ("1317/tcp", "32317")]),
        ("chain-node-did-3", ".e2e/node3", [("26657/tcp", "33657"), ("1317/tcp", "33317")]),
        ("chain-node-did-4", ".e2e/node4", [("26657/tcp", "34657"), ("1317/tcp", "34317")]),
    ] {

        let volume_absolute_path = current_dir.join(volume).to_string_lossy().into_owned();
        let bind = format!("{}:/chain4energy/.c4e-chain/", volume_absolute_path);
        let create_options = CreateContainerOptions { name, platform: Some("linux/amd64") };
        let config = create_container_config(&bind, &user_str, &port_bindings);
        docker.create_container(Some(create_options), config).await?;
        docker.start_container(name, None::<bollard::container::StartContainerOptions<String>>).await?;
        println!("Running container: {}", name);
    }

    Ok(())
}

fn create_container_config(volume: &str, user_str: &str, port_bindings: &[(&str, &str)]) -> Config<String> {
    let mut labels = HashMap::new();
    labels.insert("com.docker.compose.project".to_string(), "did-contract".to_string());

    let mut port_map: PortMap = PortMap::new();
    for (container_port, host_port) in port_bindings {
        let host_binding = vec![PortBinding {
            host_ip: Some("0.0.0.0".to_string()),
            host_port: Some(host_port.to_string()),
        }];
        port_map.insert(container_port.to_string(), Some(host_binding));
    }


    Config {
        image: Some("c4e-chain-did:v1.4.3".to_string()),
        labels: Some(labels),
        host_config: Some(bollard::service::HostConfig {
            binds: Some(vec![volume.to_string()]),
            network_mode: Some("did".to_string()),
            port_bindings: Some(port_map),
            
            ..Default::default()
        }),
        exposed_ports: Some(
            port_bindings
                .iter()
                .map(|(container_port, _)| (container_port.to_string(), HashMap::new()))
                .collect(),
        ),
        user: Some(user_str.to_string()),
        ..Default::default()
    }
}

// // Function to replicate `stop_chain` using the `bollard` library
// async fn stop_chain(docker: &Docker) -> Result<(), bollard::errors::Error> {
//     let containers = docker
//         .list_containers::<String>(None)
//         .await?
//         .into_iter()
//         .filter(|container| {
//             if let Some(labels) = &container.labels {
//                 return labels.get("com.docker.compose.project") == Some(&"did-contract".to_string());
//             }
//             false
//         })
//         .collect::<Vec<_>>();

//     // Stop and remove containers with the label
//     for container in containers {
//         if let Some(container_id) = &container.id {
//             println!("Stopping container: {}", container_id);
//             docker
//                 .stop_container(container_id, Some(StopContainerOptions { t: 10 }))
//                 .await?;
//             docker
//                 .remove_container(container_id, Some(RemoveContainerOptions { force: true, ..Default::default() }))
//                 .await?;
//         }
//     }

//     // Remove the network
//     println!("Removing network 'did'...");
//     docker.remove_network("did", Some(RemoveNetworkOptions::default())).await?;

//     Ok(())
// }



// --------------------------------------------------------------------


fn bank_send(
    sender_address: &str,
    recipient_address: &str,
    amount: u64,
    signing_key: SigningKey,
    rpc_url: &str,
    chain_id: &str,
    sequence: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    // let coin = Coin {
    //     denom: Denom::from_str("uc4e").expect("denom error"),
    //     amount: amount.into(),  // Sending `amount` in uatom (smallest denomination)
    // };

    let coin = Coin {
        denom: "uc4e".to_string(),
        amount: amount.to_string(),  // Sending `amount` in uatom (smallest denomination)
    };

    // Create a MsgSend transaction message
    let msg_send = MsgSend {
        from_address: sender_address.to_string(),
        to_address: recipient_address.to_string(),
        amount: vec![coin.into()],
    };

    let mut msg_bytes = Vec::new();
    msg_send.encode(&mut msg_bytes)?;

    // Wrap the message in an Any type
    let msg_any = Any {
        type_url: "/cosmos.bank.v1beta1.MsgSend".to_string(),
        value: msg_bytes,
    };

    let fee_coin = Coin {
        denom: "uc4e".to_string(),
        amount: 5000.to_string(),  // Sending `amount` in uatom (smallest denomination)
    };
    let fee = Fee::from_amount_and_gas(
        cosmrs::Coin::try_from(fee_coin).expect("coin conversion error"),
        200000u64,
    );
    // let fee = Fee::from_amount_and_gas(
    //     Coin {
    //         denom: Denom::from_str("uc4e").expect("denom error"),
    //         amount: 5000u64.into(),  // Small gas fee
    //     }.into(),
    //     200000u64,
    // );

    broadcast_tx(vec![msg_any], fee, signing_key, rpc_url, chain_id, sequence)
}


fn broadcast_tx(
    messages: Vec<Any>,
    fee: Fee,
    signing_key: SigningKey,
    rpc_url: &str,
    chain_id: &str,
    sequence: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a runtime to run async code in a sync function
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let chain_id = chain::Id::from_str(chain_id)?;
        let tx_body = Body::new(messages, "Test transaction", 0u32);
        let signer_info = SignerInfo::single_direct(None, sequence);
        let auth_info: AuthInfo = AuthInfo { signer_infos: vec![signer_info], fee: fee };
        let sign_doc = SignDoc::new(&tx_body, &auth_info, &chain_id, 0)?;

        let tx_raw = sign_doc.sign(&signing_key)?;
        let tx_bytes = tx_raw.to_bytes()?;
        let rpc_client = HttpClient::new(rpc_url)?; 
        let response = rpc_client.broadcast_tx_commit(tx_bytes).await?;
        if  response.tx_result.code.is_ok() {
            println!("Transaction included in block: {:?}", response.height);
            println!("Transaction hash: {:?}", response.hash);
            println!("DeliverTx response: {:?}", response.tx_result);
        } else {
            println!("Transaction failed to be included in block: {:?}", response.check_tx);
            return Err("tx failed".into());
        }

        Ok(())
    })
}

const PBKDF2_ROUNDS: u32 = 2048;

fn derive_private_key_from_mnemonic(mnemonic: &str, derivation_path_str: &str) -> Result<SigningKey, Box<dyn Error>> {
    let mnemonic = Mnemonic::parse(mnemonic)?;

    // Generate the seed from the mnemonic
    let salt = format!("mnemonic{}", "");
    let mut seed = [0u8; 64]; // BIP-39 seed is 64 bytes
    pbkdf2::<Hmac<Sha512>>(mnemonic.to_string().as_bytes(), salt.as_bytes(), PBKDF2_ROUNDS, &mut seed)?;

    // Derive the extended private key (XPrv) using BIP-32
    let derivation_path = DerivationPath::from_str(derivation_path_str)?;
    let xprv = XPrv::derive_from_path(&seed, &derivation_path)?;

    // Step 6: Extract the signing key (secp256k1 private key) from the extended private key
    let signing_key = SigningKey::from_slice(&xprv.to_bytes().as_slice())?;

    Ok(signing_key)
}

#[test]
fn test_mnemonic_derivation() {
    let result = derive_private_key_from_mnemonic(
        "harbor flee number sibling doll recycle brisk mask blanket orphan initial maze race flash limb sound wing ramp proud battle feature ceiling feel miss", 
        "m/44'/118'/0'/0/0");
    assert!(result.is_ok(), "Expected Ok but is Err");



    let result = result.unwrap();
    let acc = result.public_key().account_id("c4e").expect("acc err");
    assert_eq!("c4e1au3vecfch0h5p3p90cxftrkwyfp63mj7lgxc4n", acc.to_string());

}

#[test]
fn test_send_coins() {
    let result = derive_private_key_from_mnemonic(
        "harbor flee number sibling doll recycle brisk mask blanket orphan initial maze race flash limb sound wing ramp proud battle feature ceiling feel miss", 
        "m/44'/118'/0'/0/0");
    assert!(result.is_ok(), "Expected Ok but is Err");

    let result = bank_send("c4e1au3vecfch0h5p3p90cxftrkwyfp63mj7lgxc4n", 
        "c4e1yyjfd5cj5nd0jrlvrhc5p3mnkcn8v9q8fdd9gs", 
        100, result.unwrap(), "http://localhost:31657", "c4e-chain-compose", 4 );
    assert!(result.is_ok(), "Expected Ok but is Err");
}