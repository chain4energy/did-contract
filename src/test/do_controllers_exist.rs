use sylvia::cw_multi_test::IntoAddr;
use sylvia::multitest::App;

use crate::{
    contract::sv::mt::{CodeId, DidContractProxy},
    state::{Controller, Did, DidDocument, DID_PREFIX},
};

#[test]
fn do_controllers_exist_test() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let controller1 = format!("{}{}", DID_PREFIX, "controller1");
    let controller2 = format!("{}{}", DID_PREFIX, "controller2");
    let non_existing_controller = format!("{}{}", DID_PREFIX, "non_existing_controller");

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Create the first DID Document (controller1)
    let controller1_doc = DidDocument {
        id: Did::new(&controller1),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };

    let result = contract
        .create_did_document(controller1_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Create the second DID Document (controller2)
    let controller2_doc = DidDocument {
        id: Did::new(&controller2),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };

    let result = contract
        .create_did_document(controller2_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Check if both controllers exist
    let controllers = vec![
        Controller::new(&controller1),
        Controller::new(&controller2),
    ];
    let result = contract.do_controllers_exist(controllers.clone()).unwrap();
    assert!(
        result,
        "Expected controllers to exist, but got false"
    );

    // Check if one existing and one non-existing controller exist
    let controllers = vec![
        Controller::new(&controller1),
        Controller::new(&non_existing_controller),
    ];
    let result = contract.do_controllers_exist(controllers.clone()).unwrap();
    assert!(
        !result,
        "Expected controllers to not exist, but got true"
    );

    // Check if no controllers exist
    let controllers = vec![Controller::new(&non_existing_controller)];
    let result = contract.do_controllers_exist(controllers.clone()).unwrap();
    assert!(
        !result,
        "Expected controllers to not exist, but got true"
    );
}

#[test]
fn all_controllers_exist() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let controller1 = format!("{}{}", DID_PREFIX, "controller1");
    let controller2 = format!("{}{}", DID_PREFIX, "controller2");

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Create the first DID Document (controller1)
    let controller1_doc = DidDocument {
        id: Did::new(&controller1),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };
    let result = contract
        .create_did_document(controller1_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Create the second DID Document (controller2)
    let controller2_doc = DidDocument {
        id: Did::new(&controller2),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };
    let result = contract
        .create_did_document(controller2_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Check if both controllers exist
    let controllers = vec![
        Controller::new(&controller1),
        Controller::new(&controller2),
    ];
    let result = contract.do_controllers_exist(controllers.clone()).unwrap();
    assert!(result, "Expected controllers to exist, but got false");
}

#[test]
fn some_controllers_do_not_exist() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let controller1 = format!("{}{}", DID_PREFIX, "controller1");
    let non_existing_controller = format!("{}{}", DID_PREFIX, "non_existing_controller");

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Create the first DID Document (controller1)
    let controller1_doc = DidDocument {
        id: Did::new(&controller1),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };
    let result = contract
        .create_did_document(controller1_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Check if one existing and one non-existing controller exist
    let controllers = vec![
        Controller::new(&controller1),
        Controller::new(&non_existing_controller),
    ];
    let result = contract.do_controllers_exist(controllers.clone()).unwrap();
    assert!(!result, "Expected controllers to not exist, but got true");
}

#[test]
fn no_controllers_exist() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let non_existing_controller1 = format!("{}{}", DID_PREFIX, "non_existing_controller1");
    let non_existing_controller2 = format!("{}{}", DID_PREFIX, "non_existing_controller2");

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Check if no controllers exist
    let controllers = vec![
        Controller::new(&non_existing_controller1),
        Controller::new(&non_existing_controller2),
    ];
    let result = contract.do_controllers_exist(controllers.clone()).unwrap();
    assert!(!result, "Expected controllers to not exist, but got true");
}

#[test]
fn empty_controller_list() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Check with an empty controller list
    let controllers: Vec<Controller> = vec![];
    let result = contract.do_controllers_exist(controllers.clone());
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        "Generic error: Querier contract error: No controllers",
        result.err().unwrap().to_string()
    );
}

#[test]
fn invalid_controller_format() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let invalid_controller = "invalid_controller_format";

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Attempt to check with an invalid controller format
    let controllers = vec![Controller::new(invalid_controller)];
    let result = contract.do_controllers_exist(controllers.clone());
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        format!("Generic error: Querier contract error: Controller format error: {}", invalid_controller),
        result.err().unwrap().to_string()
    );

 
}

#[test]
fn mixed_valid_and_invalid_controllers() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let valid_controller = format!("{}{}", DID_PREFIX, "valid_controller");
    let invalid_controller = "invalid_controller_format";

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Create the valid controller DID Document
    let valid_controller_doc = DidDocument {
        id: Did::new(&valid_controller),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };
    let result = contract
        .create_did_document(valid_controller_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Attempt to check with a mix of valid and invalid controllers
    let controllers = vec![
        Controller::new(&valid_controller),
        Controller::new(invalid_controller),
    ];
    let result = contract.do_controllers_exist(controllers.clone());
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        format!("Generic error: Querier contract error: Controller format error: {}", invalid_controller),
        result.err().unwrap().to_string()
    );
}

#[test]
fn did_based_controllers() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let did_controller = format!("{}{}", DID_PREFIX, "did_controller");

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Create the DID-based controller
    let did_controller_doc = DidDocument {
        id: Did::new(&did_controller),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };
    let result = contract
        .create_did_document(did_controller_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Check if the DID-based controller exists
    let controllers = vec![Controller::new(&did_controller)];
    let result = contract.do_controllers_exist(controllers.clone()).unwrap();
    assert!(result, "Expected the DID-based controller to exist, but got false");
}

#[test]
fn address_based_controllers() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Check if the address-based controller exists
    let controllers = vec![Controller::new(&owner.to_string())];
    let result = contract.do_controllers_exist(controllers.clone()).unwrap();
    assert!(result, "Expected the address-based controller to exist, but got false");
}