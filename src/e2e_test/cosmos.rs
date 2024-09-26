use bollard::container::{
    Config, CreateContainerOptions, RemoveContainerOptions, StopContainerOptions,
};
use bollard::network::{CreateNetworkOptions /*, RemoveNetworkOptions*/};
use bollard::secret::{PortBinding, PortMap};
use bollard::Docker;
use cosmos_sdk_proto::cosmos::auth::v1beta1::query_client::QueryClient;
use cosmos_sdk_proto::cosmos::auth::v1beta1::{ModuleAccount, QueryAccountRequest};
use cosmos_sdk_proto::cosmwasm::wasm::v1::AccessConfig;
use cosmos_sdk_proto::prost::Name;
use cosmrs::cosmwasm::AccessType;
// use cosmrs::cosmwasm::AccessConfig;
use serde::Deserialize;
use serde_json::{from_str, json, Value};
// use futures::stream::TryStreamExt;
use cosmrs::proto::cosmos::auth::v1beta1::{BaseAccount, QueryAccountResponse};
use cosmrs::proto::cosmos::bank::v1beta1::QueryAllBalancesResponse;

use cosmrs::proto::cosmos::base::v1beta1::Coin;
use reqwest::blocking::Client as ReqwestClient;
use std::collections::HashMap;
use std::default::Default;
use std::env;
use std::error::Error;
use std::future::Future;
use std::io::{self, ErrorKind};
use std::process::Command;
use tokio::runtime::Runtime;

use cosmrs::crypto::secp256k1::SigningKey;
use cosmrs::proto::cosmos::bank::v1beta1::MsgSend;
use cosmrs::proto::cosmwasm::wasm::v1::MsgStoreCode;
use cosmrs::tx::{AuthInfo, Body, Fee, Raw, SignDoc, SignerInfo};

use cosmrs::tx::Msg;
// use cosmrs::proto::cosmos::base::v1beta1::{Coin};
// use cosmrs::Coin;
use cosmrs::Denom;

use tendermint_rpc::client::{Client, HttpClient};
// use cometbft_rpc::::{HttpClient, Client};
use cosmos_sdk_proto::traits::Message;
use cosmos_sdk_proto::Any;
use k256::ecdsa::signature::Signer;
use std::str::FromStr;
use tendermint::{block, chain};
use tendermint_rpc::endpoint::broadcast::tx_commit;
// use cosmrs::proto::cosmos::tx::v1beta1::{TxBody, Any};

use bip32::{DerivationPath, XPrv};
use bip39::{Language, Mnemonic};
use hdpath::StandardHDPath;

use hmac::Hmac;
use pbkdf2::pbkdf2;
use sha2::Sha512;
// use k256::ecdsa::SigningKey;
// use k256::SecretKey;
// use std::error::Error;

use tonic::transport::Channel;

// const LCD_URL: &str = "http://localhost:31317";
const RPC_URL: &str = "http://localhost:31657";
const GRPC_URL: &str = "http://localhost:31090";
const CHAIN_ID: &str = "c4e-chain-compose";

#[derive(Debug, Clone)]
struct ChainClient {
    query: Query,
    tx: Tx,
}

#[derive(Debug, Clone)]
struct Query {
    grpc_url: String,
}

#[derive(Debug, Clone)]
struct Tx {
    rpc_url: String,
    chain_id: String,
    query: Query,
}

impl ChainClient {
    pub fn new(rpc_url: &str, grpc_url: &str, chain_id: &str) -> ChainClient {
        let query = Query {
            grpc_url: grpc_url.to_string(),
        };
        let tx = Tx {
            chain_id: chain_id.to_string(),
            rpc_url: rpc_url.to_string(),
            query: query.clone(),
        };
        ChainClient {
            query: query,
            tx: tx,
        }
    }
}

impl Query {
    fn base_account(&self, account_address: &str) -> Result<BaseAccount, Box<dyn Error>> {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async { self.base_account_async(account_address).await })
    }

    async fn base_account_async(
        &self,
        account_address: &str,
    ) -> Result<BaseAccount, Box<dyn Error>> {
        let response: QueryAccountResponse = self.account(account_address).await?;
        let account_any = response.account.unwrap();
        if account_any.type_url == format!("/{}", BaseAccount::full_name()) {
            let base_account: BaseAccount = BaseAccount::decode(&*account_any.value)?;

            // Print the BaseAccount fields
            println!("Address: {:?}", base_account.address);
            println!("Account Number: {:?}", base_account.account_number);
            println!("Sequence: {:?}", base_account.sequence);
            return Ok(base_account);
        } else if account_any.type_url == format!("/{}", ModuleAccount::full_name()) {
            // Decode the BaseAccount from the Protobuf bytes
            let module_account: ModuleAccount = ModuleAccount::decode(&*account_any.value)?;
            let base_account = match module_account.base_account {
                None => return Err("module_account has no base_account".into()),
                Some(base_account) => base_account,
            };
            println!("Address: {:?}", base_account.address);
            println!("Account Number: {:?}", base_account.account_number);
            println!("Sequence: {:?}", base_account.sequence);
            return Ok(base_account);
        } else {
            println!("Unsupported account type: {}", account_any.type_url);
            Err("grpc error".into())
        }
    }

    async fn account(&self, account_address: &str) -> Result<QueryAccountResponse, Box<dyn Error>> {
        let mut client = self.create_auth_query_client().await?;

        let request = tonic::Request::new(QueryAccountRequest {
            address: account_address.to_string(),
        });
        let response: tonic::Response<QueryAccountResponse> = client.account(request).await?;
        Ok(response.into_inner())
    }

    async fn create_auth_query_client(&self) -> Result<QueryClient<Channel>, Box<dyn Error>> {
        let channel = Channel::from_shared(self.grpc_url.clone())?
            .connect()
            .await?;
        // Create a QueryClient for the Cosmos SDK auth module
        Ok(QueryClient::new(channel))
    }
}

impl Tx {
    fn broadcast_tx(
        &self,
        messages: Vec<Any>,
        fee: Fee,
        signing_key: SigningKey,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create a runtime to run async code in a sync function
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let signer_address = signing_key.public_key().account_id("c4e")?;
            let account = self
                .query
                .base_account_async(&signer_address.to_string())
                .await?;
            let chain_id = chain::Id::from_str(&self.chain_id)?;
            let tx_body = Body::new(messages, "Test transaction", 0u32);
            let signer_info = SignerInfo::single_direct(None, account.sequence);
            let auth_info: AuthInfo = AuthInfo {
                signer_infos: vec![signer_info],
                fee: fee,
            };
            let sign_doc = SignDoc::new(&tx_body, &auth_info, &chain_id, account.account_number)?;

            let tx_raw = sign_doc.sign(&signing_key)?;
            let tx_bytes = tx_raw.to_bytes()?;
            let rpc_client = HttpClient::new(&*self.rpc_url)?;
            let response = rpc_client.broadcast_tx_sync(tx_bytes).await?;
            if response.code.is_ok() {
                println!("Transaction included in block: {:?}", response.log);
                println!("Transaction hash: {:?}", response.hash);
                // println!("DeliverTx response: {:?}", response.tx_result);
            } else {
                println!("Transaction failed to be included in block: {:?}", response);
                return Err("tx failed".into());
            }
            Ok(())
        })
    }

    fn store_contract(
        &self,
        sender_address: &str,
        wasm_bytecode: Vec<u8>, // The compiled WASM contract
        signing_key: SigningKey,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create a MsgStoreCode transaction message
        let msg_store_code = MsgStoreCode {
            sender: sender_address.to_string(),
            wasm_byte_code: wasm_bytecode, // Compiled WASM bytecode
            instantiate_permission: None, // Optional, depending on contract instantiation restrictions
        };

        let mut msg_bytes = Vec::new();
        msg_store_code.encode(&mut msg_bytes)?;

        // Wrap the message in an Any type
        let msg_any = Any {
            type_url: "/cosmwasm.wasm.v1.MsgStoreCode".to_string(),
            value: msg_bytes,
        };

        let fee_coin = Coin {
            denom: "uc4e".to_string(),
            amount: 10000.to_string(), // Adjust fee as necessary
        };
        let fee = Fee::from_amount_and_gas(
            cosmrs::Coin::try_from(fee_coin).expect("coin conversion error"),
            30000000u64, // Adjust gas based on contract size
        );

        self.broadcast_tx(vec![msg_any], fee, signing_key)
    }

    fn bank_send(
        &self,
        sender_address: &str,
        recipient_address: &str,
        amount: u64,
        signing_key: SigningKey,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let coin = Coin {
            denom: "uc4e".to_string(),
            amount: amount.to_string(), // Sending `amount` in uatom (smallest denomination)
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
            amount: 5000.to_string(), // Sending `amount` in uatom (smallest denomination)
        };
        let fee = Fee::from_amount_and_gas(
            cosmrs::Coin::try_from(fee_coin).expect("coin conversion error"),
            200000u64,
        );
        self.broadcast_tx(vec![msg_any], fee, signing_key)
    }
}

#[test]
fn execute_grps_query() {
    let chain_client: ChainClient = ChainClient::new(RPC_URL, GRPC_URL, CHAIN_ID);
    let result = chain_client
        .query
        .base_account("c4e1au3vecfch0h5p3p90cxftrkwyfp63mj7lgxc4n");
    // assert_eq!(
    //         "Generic error: Querier contract error: Did document not found",
    //         result.err().unwrap().to_string()
    //     );
    assert!(result.is_ok(), "Expected Ok but is Err");
    println!("JSON: {}", json!(result.unwrap()));
}

const PBKDF2_ROUNDS: u32 = 2048;

fn derive_private_key_from_mnemonic(
    mnemonic: &str,
    derivation_path_str: &str,
) -> Result<SigningKey, Box<dyn Error>> {
    let mnemonic = Mnemonic::parse(mnemonic)?;

    // Generate the seed from the mnemonic
    let salt = format!("mnemonic{}", "");
    let mut seed = [0u8; 64]; // BIP-39 seed is 64 bytes
    pbkdf2::<Hmac<Sha512>>(
        mnemonic.to_string().as_bytes(),
        salt.as_bytes(),
        PBKDF2_ROUNDS,
        &mut seed,
    )?;

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
    assert_eq!(
        "c4e1au3vecfch0h5p3p90cxftrkwyfp63mj7lgxc4n",
        acc.to_string()
    );
}

#[test]
fn test_send_coins() {
    let result = derive_private_key_from_mnemonic(
        "harbor flee number sibling doll recycle brisk mask blanket orphan initial maze race flash limb sound wing ramp proud battle feature ceiling feel miss", 
        "m/44'/118'/0'/0/0");
    assert!(result.is_ok(), "Expected Ok but is Err");

    let chain_client: ChainClient = ChainClient::new(RPC_URL, GRPC_URL, CHAIN_ID);

    let result = chain_client.tx.bank_send(
        "c4e1au3vecfch0h5p3p90cxftrkwyfp63mj7lgxc4n",
        "c4e1yyjfd5cj5nd0jrlvrhc5p3mnkcn8v9q8fdd9gs",
        100,
        result.unwrap(),
    );
    assert!(result.is_ok(), "Expected Ok but is Err");
}

#[test]
fn test_store_contract() {
    let wasm_bytecode = std::fs::read("./artifacts/did_contract.wasm");
    assert!(wasm_bytecode.is_ok(), "Expected Ok but is Err");
    let wasm_bytecode = wasm_bytecode.unwrap();
    assert!(wasm_bytecode.len() > 0, "Expected Ok but is Err");

    println!("SSSSS {}", wasm_bytecode.len());

    let result = derive_private_key_from_mnemonic(
        "harbor flee number sibling doll recycle brisk mask blanket orphan initial maze race flash limb sound wing ramp proud battle feature ceiling feel miss", 
        "m/44'/118'/0'/0/0");
    assert!(result.is_ok(), "Expected Ok but is Err");

    let chain_client: ChainClient = ChainClient::new(RPC_URL, GRPC_URL, CHAIN_ID);

    let result = chain_client.tx.store_contract(
        "c4e1au3vecfch0h5p3p90cxftrkwyfp63mj7lgxc4n",
        wasm_bytecode,
        result.unwrap(),
        // "http://localhost:31657",
        // "c4e-chain-compose",
        // 6,
    );
    assert!(result.is_ok(), "Expected Ok but is Err");
}
