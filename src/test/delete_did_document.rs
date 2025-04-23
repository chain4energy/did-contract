use sylvia::cw_multi_test::IntoAddr;
use sylvia::multitest::App;

use crate::{
    contract::sv::mt::{CodeId, DidContractProxy},
    state::{Did, DidDocument, Service, DID_PREFIX},
};

#[test]
fn delete_did_document_not_found() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = &format!("{}{}", DID_PREFIX, "did");
    let no_did = contract.delete_did_document(Did::new(did)).call(&owner);
    assert!(no_did.is_err(), "Expected Err, but got an Ok");
    assert_eq!("Did document not found: type: did_contract::state::DidDocument; key: [00, 04, 64, 69, 64, 73, 64, 69, 64, 3A, 63, 34, 65, 3A, 64, 69, 64] not found", no_did.err().unwrap().to_string());
}

#[test]
fn delete_did_document() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    // let did_owner = "did_owner";
    let owner_addr = "did_owner".into_addr();

    let contract = code_id.instantiate().call(&owner_addr).unwrap();

    // let did_owner = "did_owner";
    let did = &format!("{}{}", DID_PREFIX, "new_did");
    let new_did_doc = DidDocument {
        id: Did::new(did),
        // controller: Controllers(vec![owner_addr.to_string().into()]),
        controller: vec![owner_addr.to_string().into()],
        service: vec![Service {
            a_type: "".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "dfdsfs")),
            service_endpoint: "dfdsfs".to_string(),
        }],
    };
    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner_addr);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    let did_document = contract.get_did_document(Did::new(did)).unwrap();
    assert_eq!(new_did_doc.clone(), did_document.clone());

    let result = contract
        .delete_did_document(Did::new(did))
        .call(&owner_addr);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    let result = contract.get_did_document(Did::new(did));
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!(
        "Generic error: Querier contract error: Did document not found: type: did_contract::state::DidDocument; key: [00, 04, 64, 69, 64, 73, 64, 69, 64, 3A, 63, 34, 65, 3A, 6E, 65, 77, 5F, 64, 69, 64] not found",
        result.err().unwrap().to_string()
    );
}

#[test]
fn delete_did_document_wrong_owner() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    // let did_owner = "did_owner";
    let owner_addr = "did_owner".into_addr();
    let wrong_owner_addr = "wrong_did_owner".into_addr();

    let contract = code_id.instantiate().call(&owner_addr).unwrap();

    // let did_owner = "did_owner";
    let did = &format!("{}{}", DID_PREFIX, "new_did");
    let new_did_doc = DidDocument {
        id: Did::new(did),
        // controller: Controllers(vec![owner_addr.to_string().into()]),
        controller: vec![owner_addr.to_string().into()],
        service: vec![Service {
            a_type: "".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "dfdsfs")),
            service_endpoint: "dfdsfs".to_string(),
        }],
    };
    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner_addr);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    let did_document = contract.get_did_document(Did::new(did)).unwrap();
    assert_eq!(new_did_doc.clone(), did_document.clone());

    let result = contract
        .delete_did_document(Did::new(did))
        .call(&wrong_owner_addr);
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!(format!("Unauthorized: {}", wrong_owner_addr.to_string()), result.err().unwrap().to_string());
}

#[test]
fn delete_existing_did_document() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "delete_existing_did");
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

    // Delete the DID Document
    let result = contract.delete_did_document(Did::new(&did)).call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Verify the DID Document no longer exists
    let result = contract.get_did_document(Did::new(&did));
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!(
        "Generic error: Querier contract error: Did document not found: type: did_contract::state::DidDocument; key: [00, 04, 64, 69, 64, 73, 64, 69, 64, 3A, 63, 34, 65, 3A, 64, 65, 6C, 65, 74, 65, 5F, 65, 78, 69, 73, 74, 69, 6E, 67, 5F, 64, 69, 64] not found",
        result.err().unwrap().to_string()
    );
}

#[test]
fn delete_non_existing_did_document() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let non_existing_did = format!("{}{}", DID_PREFIX, "non_existing_did");

    // Attempt to delete a non-existing DID Document
    let result = contract.delete_did_document(Did::new(&non_existing_did)).call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        "Did document not found: type: did_contract::state::DidDocument; key: [00, 04, 64, 69, 64, 73, 64, 69, 64, 3A, 63, 34, 65, 3A, 6E, 6F, 6E, 5F, 65, 78, 69, 73, 74, 69, 6E, 67, 5F, 64, 69, 64] not found",
        result.err().unwrap().to_string()
    );
}

#[test]
fn delete_did_document_unauthorized() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let unauthorized_user = "unauthorized_user".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "unauthorized_delete_did");
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

    // Attempt to delete the DID Document by an unauthorized user
    let result = contract.delete_did_document(Did::new(&did)).call(&unauthorized_user);
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        format!("Unauthorized: {}", unauthorized_user.to_string()),
        result.err().unwrap().to_string()
    );

    // Verify the DID Document still exists
    let did_document = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(new_did_doc, did_document);
}

#[test]
fn delete_did_document_with_invalid_format() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Define a DID with an invalid format
    let invalid_did = "invalid_did_format";

    // Attempt to delete a DID Document with an invalid format
    let result = contract.delete_did_document(Did::new(invalid_did)).call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        format!("Did format error: {}", invalid_did),
        result.err().unwrap().to_string()
    );
}

#[test]
fn delete_did_document_that_is_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Create the first DID Document
    let controller_did = format!("{}{}", DID_PREFIX, "controller_did");
    let controller_did_doc = DidDocument {
        id: Did::new(&controller_did),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };

    let result = contract
        .create_did_document(controller_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Create the second DID Document controlled by the first DID
    let did = format!("{}{}", DID_PREFIX, "main_did");
    let new_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![controller_did.clone().into()],
        service: vec![],
    };

    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Attempt to delete the first DID Document
    let result = contract.delete_did_document(Did::new(&controller_did)).call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        format!("Did is controller of another document: {}", controller_did),
        result.err().unwrap().to_string()
    );

    // Verify the first DID Document still exists
    let did_document = contract.get_did_document(Did::new(&controller_did)).unwrap();
    assert_eq!(controller_did_doc, did_document);
}

#[test]
fn delete_did_document_with_event_verification() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "event_delete_did");
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

    // Delete the DID Document
    let result = contract.delete_did_document(Did::new(&did)).call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Check the emitted events
    let res = result.expect("Failed to get result");

    assert_eq!(res.events.len(), 2);

    assert_eq!(res.events[0].attributes.len(), 1);

    assert_eq!(res.events[0].ty, "execute");
    assert_eq!(res.events[0].attributes[0].key, "_contract_address");
    assert_eq!(
        res.events[0].attributes[0].value,
        contract.contract_addr.to_string()
    );

    assert_eq!(res.events[1].attributes.len(), 2);

    assert_eq!(res.events[1].ty, "wasm-delete_did_document");
    assert_eq!(res.events[1].attributes[0].key, "_contract_address");
    assert_eq!(
        res.events[1].attributes[0].value,
        contract.contract_addr.to_string()
    );
    assert_eq!(res.events[1].attributes[1].key, "did");
    assert_eq!(res.events[1].attributes[1].value, did.to_string());
}

#[test]
fn delete_did_document_controlled_by_another_did() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Create the controller DID Document
    let controller_did = format!("{}{}", DID_PREFIX, "controller_did");
    let controller_did_doc = DidDocument {
        id: Did::new(&controller_did),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };

    let result = contract
        .create_did_document(controller_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Create the main DID Document controlled by the controller DID
    let did = format!("{}{}", DID_PREFIX, "main_did");
    let new_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![controller_did.clone().into()],
        service: vec![],
    };

    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Attempt to delete the main DID Document
    let result = contract.delete_did_document(Did::new(&did)).call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Verify the main DID Document no longer exists
    let result = contract.get_did_document(Did::new(&did));
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!(
        "Generic error: Querier contract error: Did document not found: type: did_contract::state::DidDocument; key: [00, 04, 64, 69, 64, 73, 64, 69, 64, 3A, 63, 34, 65, 3A, 6D, 61, 69, 6E, 5F, 64, 69, 64] not found",
        result.err().unwrap().to_string()
    );

    // Verify the controller DID Document still exists
    let controller_did_doc_result = contract.get_did_document(Did::new(&controller_did)).unwrap();
    assert_eq!(controller_did_doc, controller_did_doc_result);
}

#[test]
fn delete_did_document_with_index_verification() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    // let controller = "controller".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Create the DID Document
    let did = format!("{}{}", DID_PREFIX, "indexed_did");
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

    // Verify the DID is indexed under the controller
    let controlled_dids = contract
        .get_controlled_dids(owner.to_string().into(), None, None)
        .unwrap();
    assert_eq!(
        controlled_dids,
        vec![did.clone().into()],
        "Expected the DID to be indexed under the controller"
    );

    // Delete the DID Document
    let result = contract.delete_did_document(Did::new(&did)).call(&owner);
    // Check if the result is Ok
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Verify the DID is no longer indexed under the controller
    let controlled_dids = contract
        .get_controlled_dids(owner.to_string().into(), None, None)
        .unwrap();
    assert!(
        controlled_dids.is_empty(),
        "Expected the DID to be removed from the controller's index"
    );

    // Verify the DID Document no longer exists
    let result = contract.get_did_document(Did::new(&did));
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!(
        "Generic error: Querier contract error: Did document not found: type: did_contract::state::DidDocument; key: [00, 04, 64, 69, 64, 73, 64, 69, 64, 3A, 63, 34, 65, 3A, 69, 6E, 64, 65, 78, 65, 64, 5F, 64, 69, 64] not found",
        result.err().unwrap().to_string()
    );
}