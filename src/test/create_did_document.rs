use cw_storage_plus::Map;
use sylvia::cw_multi_test::IntoAddr;
use sylvia::multitest::App;

use crate::{
    contract::sv::mt::{CodeId, DidContractProxy},
    state::{Did, DidDocument, Service, DID_PREFIX},
};

#[test]
fn create_and_get_document() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // let did_owner = "did_owner";
    let did = "new_did";
    let mut new_did_doc = DidDocument {
        id: Did::new(did),
        controller: vec![owner.to_string().into()],
        // controller: Controllers(vec![owner.to_string().into()]),
        service: vec![Service {
            a_type: "".to_string(),
            id: Did::new("dfdsfs"),
            service_endpoint: "dfdsfs".to_string(),
        }],
    };
    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!("Did format error", result.err().unwrap().to_string());

    let did = format!("{}{}", DID_PREFIX, "new_did");
    new_did_doc.id = Did::new(&did);

    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!("Did format error", result.err().unwrap().to_string());

    new_did_doc.service[0].id = Did::new(&format!("{}{}", DID_PREFIX, "ffffff"));

    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    let did_document = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(new_did_doc.clone(), did_document.clone());
}


#[test]
fn create_document_with_duplicate_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "new_did");
    let new_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![
            owner.to_string().into(),
            owner.to_string().into(), // Duplicate controller
        ],
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!(
        format!("Duplicated controller: {}", owner.to_string()),
        result.err().unwrap().to_string()
    );
}

#[test]
fn create_document_with_invalid_did_format() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = "invalid_did_format";
    let new_did_doc = DidDocument {
        id: Did::new(did),
        controller: vec![owner.to_string().into()],
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!("Did format error", result.err().unwrap().to_string());
}
