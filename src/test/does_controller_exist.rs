use sylvia::cw_multi_test::IntoAddr;
use sylvia::multitest::App;

use crate::{
    contract::sv::mt::{CodeId, DidContractProxy},
    state::{Controller, Did, DidDocument, DID_PREFIX},
};

#[test]
fn controller_exists_as_did() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let controller_did = format!("{}{}", DID_PREFIX, "controller_did");

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Create the DID Document for the controller
    let controller_did_doc = DidDocument {
        id: Did::new(&controller_did),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };

    let result = contract
        .create_did_document(controller_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Check if the controller DID exists
    let result = contract
        .does_controller_exist(Controller::new(&controller_did))
        .unwrap();
    assert!(result, "Expected the controller DID to exist, but got false");
}

#[test]
fn controller_exists_as_address() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Check if the address-based controller exists
    let result = contract
        .does_controller_exist(Controller::new(&owner.to_string()))
        .unwrap();
    assert!(result, "Expected the address-based controller to exist, but got false");
}

#[test]
fn controller_does_not_exist() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let non_existing_controller = format!("{}{}", DID_PREFIX, "non_existing_controller");

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Check if the non-existing controller exists
    let result = contract
        .does_controller_exist(Controller::new(&non_existing_controller))
        .unwrap();
    assert!(!result, "Expected the non-existing controller to not exist, but got true");
}

#[test]
fn invalid_controller_format() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let invalid_controller = "invalid_controller_format";

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Attempt to check with an invalid controller format
    let result = contract.does_controller_exist(Controller::new(invalid_controller));
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        format!("Generic error: Querier contract error: Controller format error: {}", invalid_controller),
        result.err().unwrap().to_string()
    );
}

#[test]
fn empty_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let empty_controller = "";

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Attempt to check with an empty controller
    let result = contract.does_controller_exist(Controller::new(empty_controller));
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        format!("Generic error: Querier contract error: Controller format error: {}", empty_controller),
        result.err().unwrap().to_string()
    );
}

#[test]
fn controller_exists_in_complex_did_hierarchy() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Create the first DID Document (controller1)
    let controller1 = format!("{}{}", DID_PREFIX, "controller1");
    let controller1_doc = DidDocument {
        id: Did::new(&controller1),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };

    let result = contract
        .create_did_document(controller1_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Create the second DID Document (controller2 controlled by controller1)
    let controller2 = format!("{}{}", DID_PREFIX, "controller2");
    let controller2_doc = DidDocument {
        id: Did::new(&controller2),
        controller: vec![controller1.clone().into()],
        service: vec![],
    };

    let result = contract
        .create_did_document(controller2_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Check if controller1 exists
    let result = contract
        .does_controller_exist(Controller::new(&controller1))
        .unwrap();
    assert!(result, "Expected controller1 to exist, but got false");

    // Check if controller2 exists
    let result = contract
        .does_controller_exist(Controller::new(&controller2))
        .unwrap();
    assert!(result, "Expected controller2 to exist, but got false");
}