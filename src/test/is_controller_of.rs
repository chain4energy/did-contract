use sylvia::cw_multi_test::IntoAddr;
use sylvia::multitest::App;

use crate::{
    contract::sv::mt::{CodeId, DidContractProxy},
    state::{Controller, Did, DidDocument, DID_PREFIX},
};

#[test]
fn is_valid_controller_in_list() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let controller = "controller".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let controllers = vec![Controller::new(&controller.to_string())];

    // Check if the controller is in the list of controllers
    let is_controller = contract
        .is_controller_of(controllers.clone(), Controller::new(&controller.to_string()))
        .unwrap();
    assert!(
        is_controller,
        "Expected the controller to be in the list of controllers"
    );
}

#[test]
fn is_non_controller_in_list() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let non_controller = "non_controller".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let controllers = vec![Controller::new(&owner.to_string())];

    // Check if the non-controller is in the list of controllers
    let is_controller = contract
        .is_controller_of(controllers.clone(), Controller::new(&non_controller.to_string()))
        .unwrap();
    assert!(
        !is_controller,
        "Expected the non-controller to not be in the list of controllers"
    );
}

#[test]
fn is_did_in_list_of_controllers() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Create a DID-based controller
    let did_controller = format!("{}{}", DID_PREFIX, "did_controller");
    let controllers = vec![Controller::new(&did_controller)];

    // Check if the DID is in the list of controllers
    let is_controller = contract
        .is_controller_of(controllers.clone(), Controller::new(&did_controller))
        .unwrap();
    assert!(
        is_controller,
        "Expected the DID to be in the list of controllers"
    );
}

#[test]
fn is_non_existing_did_not_in_list_of_controllers() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Define a non-existing DID
    let non_existing_did = format!("{}{}", DID_PREFIX, "non_existing_did");
    let controllers = vec![Controller::new(&owner.to_string())];

    // Check if the non-existing DID is in the list of controllers
    let is_controller = contract
        .is_controller_of(controllers.clone(), Controller::new(&non_existing_did))
        .unwrap();
    assert!(
        !is_controller,
        "Expected the non-existing DID to not be in the list of controllers"
    );
}

#[test]
fn is_controller_with_invalid_format() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Define a controller with an invalid format
    let invalid_controller = "invalid_controller_format";
    let controllers = vec![Controller::new(&owner.to_string())];

    // Attempt to check if the invalid controller is in the list of controllers
    let result = contract.is_controller_of(controllers.clone(), Controller::new(invalid_controller));
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        "Generic error: Querier contract error: Controller format error: invalid_controller_format",
        result.err().unwrap().to_string()
    );
}


#[test]
fn is_controller_in_vec_with_invalid_format() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Define a controller with an invalid format
    let invalid_controller = "invalid_controller_format";
    let controllers = vec![Controller::new(&invalid_controller)];

    // Attempt to check if the invalid controller is in the list of controllers
    let result = contract.is_controller_of(controllers.clone(), Controller::new(&owner.to_string()));
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        "Generic error: Querier contract error: Controller format error: invalid_controller_format",
        result.err().unwrap().to_string()
    );
}

#[test]
fn is_controller_in_empty_list() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Define an empty list of controllers
    let controllers: Vec<Controller> = vec![];

    // Check if any controller is in the empty list
    let is_controller = contract
        .is_controller_of(controllers.clone(), Controller::new(&owner.to_string()))
        .unwrap();
    assert!(
        !is_controller,
        "Expected no controller to be in the empty list of controllers"
    );
}

#[test]
fn is_controller_of_with_looped_controllers() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Create the first DID Document (loop start)
    let did1 = format!("{}{}", DID_PREFIX, "did1");
    let mut did_doc1 = DidDocument {
        id: Did::new(&did1),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };

    let result = contract.create_did_document(did_doc1.clone()).call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Create the second DID Document
    let did2 = format!("{}{}", DID_PREFIX, "did2");
    let did_doc2 = DidDocument {
        id: Did::new(&did2),
        controller: vec![did1.clone().into()],
        service: vec![],
    };

    let result = contract.create_did_document(did_doc2.clone()).call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Create the third DID Document (loop back to did1)
    let did3 = format!("{}{}", DID_PREFIX, "did3");
    let did_doc3 = DidDocument {
        id: Did::new(&did3),
        controller: vec![did2.clone().into()],
        service: vec![],
    };

    let result = contract.create_did_document(did_doc3.clone()).call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    let result = contract
        .add_controller(did1.clone().into(), did3.clone().into())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    did_doc1
        .controller
        .push(did3.clone().into());

    // Check if did1 is a controller of itself through the loop
    let is_controller = contract
        .is_controller_of(did_doc1.controller.clone(), Controller::new(&did1))
        .unwrap();
    assert!(
        is_controller,
        "Expected did1 to be a controller of itself through the loop"
    );

    // Check if did2 is a controller of did1
    let is_controller = contract
        .is_controller_of(did_doc1.controller.clone(), Controller::new(&did2))
        .unwrap();
    assert!(
        is_controller,
        "Expected did2 to be a controller of did1 through the loop"
    );

    // Check if did3 is a controller of did1
    let is_controller = contract
        .is_controller_of(did_doc1.controller.clone(), Controller::new(&did3))
        .unwrap();
    assert!(
        is_controller,
        "Expected did3 to be a controller of did1 through the loop"
    );

    let is_controller = contract
        .is_controller_of(did_doc1.controller.clone(), Controller::new(&owner.to_string()))
        .unwrap();
    assert!(
        is_controller,
        "Expected did3 to be a controller of did1 through the loop"
    );

    let is_controller = contract
        .is_controller_of(did_doc2.controller.clone(), Controller::new(&did1))
        .unwrap();
    assert!(
        is_controller,
        "Expected did1 to be a controller of itself through the loop"
    );

    let is_controller = contract
        .is_controller_of(did_doc2.controller.clone(), Controller::new(&did2))
        .unwrap();
    assert!(
        is_controller,
        "Expected did2 to be a controller of did1 through the loop"
    );

    let is_controller = contract
        .is_controller_of(did_doc2.controller.clone(), Controller::new(&did3))
        .unwrap();
    assert!(
        is_controller,
        "Expected did3 to be a controller of did1 through the loop"
    );

    let is_controller = contract
        .is_controller_of(did_doc2.controller.clone(), Controller::new(&owner.to_string()))
        .unwrap();
    assert!(
        is_controller,
        "Expected did3 to be a controller of did1 through the loop"
    );

    let is_controller = contract
        .is_controller_of(did_doc3.controller.clone(), Controller::new(&did1))
        .unwrap();
    assert!(
        is_controller,
        "Expected did1 to be a controller of itself through the loop"
    );

    let is_controller = contract
        .is_controller_of(did_doc3.controller.clone(), Controller::new(&did2))
        .unwrap();
    assert!(
        is_controller,
        "Expected did2 to be a controller of did1 through the loop"
    );

    let is_controller = contract
        .is_controller_of(did_doc3.controller.clone(), Controller::new(&did3))
        .unwrap();
    assert!(
        is_controller,
        "Expected did3 to be a controller of did1 through the loop"
    );

    let is_controller = contract
        .is_controller_of(did_doc3.controller.clone(), Controller::new(&owner.to_string()))
        .unwrap();
    assert!(
        is_controller,
        "Expected did3 to be a controller of did1 through the loop"
    );

    // Check if an unrelated DID is not a controller
    let unrelated_did = format!("{}{}", DID_PREFIX, "unrelated_did");
    let is_controller = contract
        .is_controller_of(did_doc1.controller, Controller::new(&unrelated_did))
        .unwrap();
    assert!(
        !is_controller,
        "Expected unrelated_did to not be a controller of did1"
    );
}