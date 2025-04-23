use cw_storage_plus::Map;
use sylvia::cw_multi_test::IntoAddr;
use sylvia::multitest::App;

use crate::{
    contract::sv::mt::{CodeId, DidContractProxy},
    state::{Controller, Did, DidDocument, Service, DID_PREFIX},
};

#[test]
fn add_valid_service() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "valid_service_did");
    let mut original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };

    // Create the original DID Document
    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Add a valid service
    let new_service = Service {
        id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
        a_type: "ServiceType".to_string(),
        service_endpoint: "http://example.com".to_string(),
    };

    let result = contract
        .add_service(Did::new(&did), new_service.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Verify the updated DID Document
    original_did_doc.service.push(new_service);
    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(
        original_did_doc, updated_did_doc,
        "DID Document was not updated correctly"
    );
}

#[test]
fn add_duplicate_service() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "duplicate_service_did");
    let original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![Service {
            id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
            a_type: "ServiceType".to_string(),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    // Create the original DID Document
    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Attempt to add a duplicate service
    let duplicate_service = original_did_doc.service[0].clone();
    let result = contract
        .add_service(Did::new(&did), duplicate_service)
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        format!(
            "Did document service already exists: {}",
            original_did_doc.service[0].id
        ),
        result.err().unwrap().to_string()
    );

    // Verify that the DID Document was not updated
    let current_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(
        original_did_doc, current_did_doc,
        "DID Document was updated incorrectly"
    );
}

#[test]
fn add_service_with_invalid_id_format() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "invalid_service_id_did");
    let original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };

    // Create the original DID Document
    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Attempt to add a service with an invalid ID format
    let invalid_service = Service {
        id: Did::new("invalid_service_id"),
        a_type: "ServiceType".to_string(),
        service_endpoint: "http://example.com".to_string(),
    };

    let result = contract
        .add_service(Did::new(&did), invalid_service)
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        "Service id format error: Did format error: invalid_service_id",
        result.err().unwrap().to_string()
    );

    // Verify that the DID Document was not updated
    let current_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(
        original_did_doc, current_did_doc,
        "DID Document was updated incorrectly"
    );
}

#[test]
fn add_service_to_non_existing_did_document() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let non_existing_did = format!("{}{}", DID_PREFIX, "non_existing_did");

    // Attempt to add a service to a non-existing DID Document
    let new_service = Service {
        id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
        a_type: "ServiceType".to_string(),
        service_endpoint: "http://example.com".to_string(),
    };

    let result = contract
        .add_service(Did::new(&non_existing_did), new_service)
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        "Did document not found: type: did_contract::state::DidDocument; key: [00, 04, 64, 69, 64, 73, 64, 69, 64, 3A, 63, 34, 65, 3A, 6E, 6F, 6E, 5F, 65, 78, 69, 73, 74, 69, 6E, 67, 5F, 64, 69, 64] not found",
        result.err().unwrap().to_string()
    );

}

#[test]
fn add_service_unauthorized() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let unauthorized_user = "unauthorized_user".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "unauthorized_service_did");
    let original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };

    // Create the original DID Document
    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Attempt to add a service by an unauthorized user
    let new_service = Service {
        id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
        a_type: "ServiceType".to_string(),
        service_endpoint: "http://example.com".to_string(),
    };

    let result = contract
        .add_service(Did::new(&did), new_service)
        .call(&unauthorized_user);
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        format!("Unauthorized: {}", unauthorized_user.to_string()),
        result.err().unwrap().to_string()
    );

    // Verify that the DID Document was not updated
    let current_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(
        original_did_doc, current_did_doc,
        "DID Document was updated incorrectly"
    );
}

#[test]
fn add_service_with_event_verification() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "event_service_did");
    let mut original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };

    // Create the original DID Document
    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Add a valid service
    let new_service = Service {
        id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
        a_type: "ServiceType".to_string(),
        service_endpoint: "http://example.com".to_string(),
    };

    let result = contract
        .add_service(Did::new(&did), new_service.clone())
        .call(&owner);
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

    assert_eq!(res.events[1].attributes.len(), 3);

    assert_eq!(res.events[1].ty, "wasm-add_service");
    assert_eq!(res.events[1].attributes[0].key, "_contract_address");
    assert_eq!(
        res.events[1].attributes[0].value,
        contract.contract_addr.to_string()
    );
    assert_eq!(res.events[1].attributes[1].key, "did");
    assert_eq!(res.events[1].attributes[1].value, did.to_string());
    assert_eq!(res.events[1].attributes[2].key, "new_service");
    assert_eq!(
        res.events[1].attributes[2].value,
        new_service.id.to_string()
    );
}

#[test]
fn add_service_to_did_with_multiple_controllers() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let controller2 = "controller2".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "multi_controller_service_did");
    let mut original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into(), controller2.to_string().into()],
        service: vec![],
    };

    // Create the original DID Document
    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Add a valid service
    let new_service = Service {
        id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
        a_type: "ServiceType".to_string(),
        service_endpoint: "http://example.com".to_string(),
    };

    let result = contract
        .add_service(Did::new(&did), new_service.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Verify the updated DID Document
    original_did_doc.service.push(new_service);
    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(
        original_did_doc, updated_did_doc,
        "DID Document was not updated correctly"
    );
}

#[test]
fn add_service_to_invalid_did() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Define a DID with an invalid format
    let invalid_did = "invalid_did_format";

    // Attempt to add a service to the invalid DID
    let new_service = Service {
        id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
        a_type: "ServiceType".to_string(),
        service_endpoint: "http://example.com".to_string(),
    };

    let result = contract
        .add_service(Did::new(invalid_did), new_service)
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        format!("Did format error: {}", invalid_did),
        result.err().unwrap().to_string()
    );
}

#[test]
fn add_service_when_controller_is_did() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Create the controller DID Document
    let controller_did = format!("{}{}", DID_PREFIX, "controller_did");
    let controller_did_doc = DidDocument {
        id: Did::new(&controller_did),
        controller: vec![owner.to_string().into()], // Normal address as controller
        service: vec![],
    };

    let result = contract
        .create_did_document(controller_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Create the main DID Document controlled by the controller DID
    let did = format!("{}{}", DID_PREFIX, "main_did");
    let mut original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![controller_did.clone().into()],
        service: vec![],
    };

    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Add a service to the main DID Document
    let new_service = Service {
        id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
        a_type: "ServiceType".to_string(),
        service_endpoint: "http://example.com".to_string(),
    };

    let result = contract
        .add_service(Did::new(&did), new_service.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Verify the updated DID Document
    original_did_doc.service.push(new_service);
    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(
        original_did_doc, updated_did_doc,
        "DID Document was not updated correctly"
    );
}
