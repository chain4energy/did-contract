use sylvia::cw_multi_test::IntoAddr;
use sylvia::multitest::App;

use crate::{
    contract::sv::mt::{CodeId, DidContractProxy},
    state::{Did, DidDocument, Service, DID_PREFIX},
};

#[test]
fn get_document_not_found() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = "did";
    let no_did = contract.get_did_document(Did::new(did));
    assert!(no_did.is_err(), "Expected Err, but got an Ok");
    assert_eq!(
        "Generic error: Querier contract error: Did format error: did",
        no_did.err().unwrap().to_string()
    );

    let did = format!("{}{}", DID_PREFIX, "did");
    let no_did = contract.get_did_document(Did::new(&did));
    assert!(no_did.is_err(), "Expected Err, but got an Ok");
    assert_eq!(
       "Generic error: Querier contract error: Did document not found: type: did_contract::state::DidDocument; key: [00, 04, 64, 69, 64, 73, 64, 69, 64, 3A, 63, 34, 65, 3A, 64, 69, 64] not found",
        no_did.err().unwrap().to_string()
    );
}

#[test]
fn get_existing_did_document() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "existing_did");
    let new_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };

    // Create the DID Document
    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Retrieve the DID Document
    let retrieved_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(
        new_did_doc, retrieved_did_doc,
        "DID Document does not match"
    );
}

#[test]
fn get_non_existing_did_document() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let non_existing_did = format!("{}{}", DID_PREFIX, "non_existing_did");

    // Attempt to retrieve a non-existing DID Document
    let result = contract.get_did_document(Did::new(&non_existing_did));
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        "Generic error: Querier contract error: Did document not found: type: did_contract::state::DidDocument; key: [00, 04, 64, 69, 64, 73, 64, 69, 64, 3A, 63, 34, 65, 3A, 6E, 6F, 6E, 5F, 65, 78, 69, 73, 74, 69, 6E, 67, 5F, 64, 69, 64] not found",
        result.err().unwrap().to_string()
    );
}

#[test]
fn get_did_document_with_invalid_format() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Define a DID with an invalid format
    let invalid_did = "invalid_did_format";

    // Attempt to retrieve a DID Document with an invalid format
    let result = contract.get_did_document(Did::new(invalid_did));
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        format!("Generic error: Querier contract error: Did format error: {}", invalid_did),
        result.err().unwrap().to_string()
    );
}

#[test]
fn get_did_document_with_multiple_controllers() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let controller2 = "controller2".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "multi_controller_did");
    let new_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into(), controller2.to_string().into()],
        service: vec![],
    };

    // Create the DID Document
    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Retrieve the DID Document
    let retrieved_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(
        new_did_doc, retrieved_did_doc,
        "DID Document does not match"
    );
}

#[test]
fn get_did_document_with_services() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "service_did");
    let new_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![Service {
            id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
            a_type: "ServiceType".to_string(),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    // Create the DID Document
    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Retrieve the DID Document
    let retrieved_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(
        new_did_doc, retrieved_did_doc,
        "DID Document does not match"
    );
}
