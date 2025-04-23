use sylvia::cw_multi_test::IntoAddr;
use sylvia::multitest::App;

use crate::{
    contract::sv::mt::{CodeId, DidContractProxy},
    state::{Controller, Did, DidDocument, DID_PREFIX},
};

#[test]
fn add_valid_address_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let new_controller = "new_controller".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "add_controller_did");
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

    // Add a new controller
    let result = contract
        .add_controller(Did::new(&did), new_controller.to_string().into())
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

    assert_eq!(res.events[1].ty, "wasm-add_controller");
    assert_eq!(res.events[1].attributes[0].key, "_contract_address");
    assert_eq!(
        res.events[1].attributes[0].value,
        contract.contract_addr.to_string()
    );
    assert_eq!(res.events[1].attributes[1].key, "did");
    assert_eq!(res.events[1].attributes[1].value, did.to_string());
    assert_eq!(res.events[1].attributes[2].key, "new_controller");
    assert_eq!(res.events[1].attributes[2].value, new_controller.to_string());

    
    // Verify the updated DID Document
    original_did_doc.controller.push(new_controller.to_string().into());
    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(original_did_doc, updated_did_doc, "DID Document was not updated correctly");
}

#[test]
fn add_valid_did_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Create the first DID Document (controller DID)
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

    // Create the second DID Document
    let did = format!("{}{}", DID_PREFIX, "main_did");
    let mut original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };

    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Add the first DID as a controller to the second DID Document
    let result = contract
        .add_controller(Did::new(&did), Controller::new(&controller_did))
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

    assert_eq!(res.events[1].ty, "wasm-add_controller");
    assert_eq!(res.events[1].attributes[0].key, "_contract_address");
    assert_eq!(
        res.events[1].attributes[0].value,
        contract.contract_addr.to_string()
    );
    assert_eq!(res.events[1].attributes[1].key, "did");
    assert_eq!(res.events[1].attributes[1].value, did.to_string());
    assert_eq!(res.events[1].attributes[2].key, "new_controller");
    assert_eq!(res.events[1].attributes[2].value, controller_did);

    // Verify the updated DID Document
    original_did_doc.controller.push(controller_did.into());
    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(original_did_doc, updated_did_doc, "DID Document was not updated correctly");
}

#[test]
fn add_duplicate_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "duplicate_controller_did");
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

    // Attempt to add the same controller again
    let result = contract
        .add_controller(Did::new(&did), owner.to_string().into())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!(
        format!("Did document controller already exists: {}", owner.to_string()),
        result.err().unwrap().to_string()
    );
}

#[test]
fn add_nonexistent_did_as_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let nonexistent_did = format!("{}{}", DID_PREFIX, "nonexistent_did");

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "add_nonexistent_controller_did");
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

    // Attempt to add a nonexistent DID as a controller
    let result = contract
        .add_controller(Did::new(&did), nonexistent_did.to_string().into())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!(
        format!("Did controller not found: {}", nonexistent_did),
        result.err().unwrap().to_string()
    );
}

#[test]
fn add_self_controlled_did_as_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "self_controlled_did");
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

    // Attempt to add the DID itself as a controller
    let result = contract
        .add_controller(Did::new(&did), did.to_string().into())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!(
        format!("Self controlled did document not allowed: {}", did),
        result.err().unwrap().to_string()
    );
}

#[test]
fn add_controller_to_non_existing_did_document() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let new_controller = "new_controller".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Define a non-existing DID
    let non_existing_did = format!("{}{}", DID_PREFIX, "non_existing_did");

    // Attempt to add a controller to the non-existing DID Document
    let result = contract
        .add_controller(Did::new(&non_existing_did), new_controller.to_string().into())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        "Did document not found: type: did_contract::state::DidDocument; key: [00, 04, 64, 69, 64, 73, 64, 69, 64, 3A, 63, 34, 65, 3A, 6E, 6F, 6E, 5F, 65, 78, 69, 73, 74, 69, 6E, 67, 5F, 64, 69, 64] not found",
        result.err().unwrap().to_string()
    );
}

#[test]
fn add_controller_to_did_with_invalid_format() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let new_controller = "new_controller".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Define a DID with an invalid format
    let invalid_did = "invalid_did_format";

    // Attempt to add a controller to the invalid DID
    let result = contract
        .add_controller(Did::new(invalid_did), new_controller.to_string().into())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        format!("Did format error: {}", invalid_did),
        result.err().unwrap().to_string()
    );
}

#[test]
fn add_invalid_format_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "valid_did");
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

    // Attempt to add a controller with an invalid format
    let invalid_controller = "invalid_controller_format";
    let result = contract
        .add_controller(Did::new(&did), Controller::new(invalid_controller))
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        format!("Controller format error: {}", invalid_controller),
        result.err().unwrap().to_string()
    );
}

#[test]
fn add_controller_unauthorized() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let unauthorized_user = "unauthorized_user".into_addr();
    let new_controller = "new_controller".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "unauthorized_add_controller_did");
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

    // Attempt to add a controller by an unauthorized user
    let result = contract
        .add_controller(Did::new(&did), new_controller.to_string().into())
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
fn add_controller_with_index_verification() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let new_controller = "new_controller".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "indexed_did");
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

    // Verify the DID is indexed under the original controller
    let controlled_dids = contract
        .get_controlled_dids(owner.to_string().into(), None, None)
        .unwrap();
    assert_eq!(
        controlled_dids,
        vec![did.clone().into()],
        "Expected the DID to be indexed under the original controller"
    );

    // Verify the DID is now indexed under the new controller
    let controlled_dids = contract
        .get_controlled_dids(new_controller.to_string().into(), None, None)
        .unwrap();
    assert_eq!(
        controlled_dids,
        vec![],
        "No DIDs controlled expected"
    );

    // Add a new controller
    let result = contract
        .add_controller(Did::new(&did.clone()), new_controller.to_string().into())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Verify the DID is still indexed under the original controller
    let controlled_dids = contract
        .get_controlled_dids(owner.to_string().into(), None, None)
        .unwrap();
    assert_eq!(
        controlled_dids,
        vec![did.clone().into()],
        "Expected the DID to remain indexed under the original controller"
    );

    // Verify the DID is now indexed under the new controller
    let controlled_dids = contract
        .get_controlled_dids(new_controller.to_string().into(), None, None)
        .unwrap();
    assert_eq!(
        controlled_dids,
        vec![did.clone().into()],
        "Expected the DID to be indexed under the new controller"
    );

    // Verify the updated DID Document
    original_did_doc.controller.push(new_controller.to_string().into());
    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(
        original_did_doc, updated_did_doc,
        "DID Document was not updated correctly"
    );
}

#[test]
fn add_as_did_controller_success() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let owner2 = "owner2".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Create the first DID Document (controller DID)
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

    // Create the second DID Document
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

    let result = contract
        .add_controller(Did::new(&did), Controller::new(&owner2.to_string()))
        .call(&owner);
    // if result.is_err() {
    //     println!("Error: {}", result.err().unwrap());
    //     return;
    // }
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

    assert_eq!(res.events[1].ty, "wasm-add_controller");
    assert_eq!(res.events[1].attributes[0].key, "_contract_address");
    assert_eq!(
        res.events[1].attributes[0].value,
        contract.contract_addr.to_string()
    );
    assert_eq!(res.events[1].attributes[1].key, "did");
    assert_eq!(res.events[1].attributes[1].value, did.to_string());
    assert_eq!(res.events[1].attributes[2].key, "new_controller");
    assert_eq!(res.events[1].attributes[2].value, owner2.to_string());

    // Verify the updated DID Document
    original_did_doc.controller.push(Controller::new(&owner2.to_string()));
    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(original_did_doc, updated_did_doc, "DID Document was not updated correctly");
}