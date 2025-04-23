use cw_storage_plus::Map;
use sylvia::cw_multi_test::IntoAddr;
use sylvia::multitest::App;

use crate::{
    contract::sv::mt::{CodeId, DidContractProxy},
    state::{Controller, Did, DidDocument, Service, DID_PREFIX},
};

#[test]
fn is_did_controller() {
    let app = App::default();

    let code_id = CodeId::store_code(&app);
    // did_docs: Map::new("dids"),
    const DID_STORE: Map<String, DidDocument> = Map::new("dids");

    let owner = "owner".into_addr();
    let unknow_addr = "unknown".into_addr();
    let unknow_did = &format!("{}{}", DID_PREFIX, "unknown");
    let service_did = &format!("{}{}", DID_PREFIX, "dfdsfs");

    let contract = code_id.instantiate().call(&owner).unwrap();

    // let did_owner = "did_owner";
    let did_simple = format!("{}{}", DID_PREFIX, "did_simple");
    let did_doc_simple = DidDocument {
        id: Did::new(&did_simple),
        // controller: Controllers(vec![owner.to_string().into()]),
        controller: vec![owner.to_string().into()],
        service: vec![Service {
            a_type: "".to_string(),
            id: Did::new(service_did),
            service_endpoint: "dfdsfs".to_string(),
        }],
    };

    let result = contract
        .create_did_document(did_doc_simple.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    let is_controller = contract.is_did_controller(Did::new(&did_simple), owner.to_string().into());
    assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
    assert!(is_controller.unwrap(), "Expected true, but got false");

    let is_controller =
        contract.is_did_controller(Did::new(&did_simple), unknow_addr.to_string().into());
    assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
    assert!(!is_controller.unwrap(), "Expected false, but got true");

    let is_controller = contract.is_did_controller(Did::new(&did_simple), unknow_did.into());
    assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
    assert!(!is_controller.unwrap(), "Expected false, but got true");

    let did_controlled_by_itself = format!("{}{}", DID_PREFIX, "did_controlled_by_himself");
    let did_doc_controlled_by_itself = DidDocument {
        id: Did::new(&did_controlled_by_itself),
        // controller: Controllers(vec![did_controlled_by_itself.to_string().into()]),
        controller: vec![did_controlled_by_itself.to_string().into()],
        service: vec![Service {
            a_type: "".to_string(),
            id: Did::new(service_did),
            service_endpoint: "dfdsfs".to_string(),
        }],
    };

    {
        let mut app_mut = app.app_mut();
        let mut contract_store = app_mut.contract_storage_mut(&contract.contract_addr);
        let contract_store = contract_store.as_mut();

        let result = DID_STORE.save(
            contract_store,
            did_controlled_by_itself.to_string(),
            &did_doc_controlled_by_itself,
        );
        assert!(
            result.is_ok(),
            "Expected Ok, but got an Err: {}",
            result.unwrap_err()
        );
    }
    // let result = contract.create_did_document(did_doc_controlled_by_itself.clone()).call(&owner);
    // assert!(result.is_err(), "Expected Err, but got an Ok");
    // assert_eq!("Did controller not found", result.err().unwrap().to_string());
    // // assert!(result.is_ok(), "Expected Ok, but got an Err: {}", result.unwrap_err());

    let is_controller = contract.is_did_controller(
        Did::new(&did_controlled_by_itself),
        did_controlled_by_itself.to_string().into(),
    );
    assert!(
        is_controller.is_ok(),
        "Expected Ok, but got an Err: {}",
        is_controller.unwrap_err()
    );
    assert!(is_controller.unwrap(), "Expected true, but got false");

    let is_controller =
        contract.is_did_controller(Did::new(&did_controlled_by_itself), unknow_did.into());
    assert!(
        is_controller.is_ok(),
        "Expected Ok, but got an Err: {}",
        is_controller.unwrap_err()
    );
    assert!(!is_controller.unwrap(), "Expected false, but got true");

    let did_looped_1 = &format!("{}{}", DID_PREFIX, "did_looped_1");
    let did_looped_2 = &format!("{}{}", DID_PREFIX, "did_looped_2");
    let did_doc_looped_1 = DidDocument {
        id: Did::new(did_looped_1),
        // controller: Controllers(vec![did_looped_2.to_string().into()]),
        controller: vec![did_looped_2.to_string().into()],
        service: vec![Service {
            a_type: "".to_string(),
            id: Did::new(service_did),
            service_endpoint: "dfdsfs".to_string(),
        }],
    };

    let did_doc_looped_2 = DidDocument {
        id: Did::new(did_looped_2),
        // controller: Controllers(vec![did_looped_1.to_string().into()]),
        controller: vec![did_looped_1.to_string().into()],
        service: vec![Service {
            a_type: "".to_string(),
            id: Did::new(service_did),
            service_endpoint: "dfdsfs".to_string(),
        }],
    };

    {
        let mut app_mut = app.app_mut();
        let mut contract_store = app_mut.contract_storage_mut(&contract.contract_addr);
        let contract_store = contract_store.as_mut();

        let result = DID_STORE.save(
            contract_store,
            did_doc_looped_1.id.to_string(),
            &did_doc_looped_1,
        );
        assert!(
            result.is_ok(),
            "Expected Ok, but got an Err: {}",
            result.unwrap_err()
        );

        let result = DID_STORE.save(
            contract_store,
            did_doc_looped_2.id.to_string(),
            &did_doc_looped_2,
        );
        assert!(
            result.is_ok(),
            "Expected Ok, but got an Err: {}",
            result.unwrap_err()
        );
    }

    // let result = contract.create_did_document(did_doc_looped_1.clone()).call(&owner);
    // assert!(result.is_ok(), "Expected Ok, but got an Err: {}", result.unwrap_err());

    // let result = contract.create_did_document(did_doc_looped_2.clone()).call(&owner);
    // assert!(result.is_ok(), "Expected Ok, but got an Err: {}", result.unwrap_err());

    let is_controller = contract.is_did_controller(Did::new(did_looped_1), unknow_did.into());
    assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
    assert!(!is_controller.unwrap(), "Expected false, but got true");

    let is_controller =
        contract.is_did_controller(Did::new(did_looped_1), did_looped_2.to_string().into());
    assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
    assert!(is_controller.unwrap(), "Expected true, but got false");

    let is_controller =
        contract.is_did_controller(Did::new(did_looped_1), did_looped_1.to_string().into());
    assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
    assert!(is_controller.unwrap(), "Expected true, but got false");

    let did_controlled_by_simple = &format!("{}{}", DID_PREFIX, "did_controlled_by_simple");
    let did_doc_controlled_by_simple = DidDocument {
        id: Did::new(did_controlled_by_simple),
        // controller: Controllers(vec![did_simple.to_string().into()]),
        controller: vec![did_simple.to_string().into()],
        service: vec![Service {
            a_type: "".to_string(),
            id: Did::new(service_did),
            service_endpoint: "dfdsfs".to_string(),
        }],
    };

    let result = contract
        .create_did_document(did_doc_controlled_by_simple.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    let is_controller =
        contract.is_did_controller(Did::new(did_controlled_by_simple), unknow_did.into());
    assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
    assert!(!is_controller.unwrap(), "Expected false, but got true");

    let is_controller = contract.is_did_controller(
        Did::new(did_controlled_by_simple),
        did_simple.to_string().into(),
    );
    assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
    assert!(is_controller.unwrap(), "Expected true, but got false");

    let is_controller =
        contract.is_did_controller(Did::new(did_controlled_by_simple), owner.to_string().into());
    assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
    assert!(is_controller.unwrap(), "Expected true, but got false");
}

#[test]
fn is_valid_controller_of_did() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let controller = "controller".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "valid_did");
    let new_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![controller.to_string().into()],
        service: vec![],
    };

    // Create the DID Document
    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Check if the controller is a controller of the DID
    let is_controller = contract
        .is_did_controller(Did::new(&did), Controller::new(&controller.to_string()))
        .unwrap();
    assert!(
        is_controller,
        "Expected the controller to be a valid controller of the DID"
    );
}

#[test]
fn is_non_controller_of_did() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let non_controller = "non_controller".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "valid_did");
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

    // Check if the non-controller is a controller of the DID
    let is_controller = contract
        .is_did_controller(Did::new(&did), Controller::new(&non_controller.to_string()))
        .unwrap();
    assert!(
        !is_controller,
        "Expected the non-controller to not be a controller of the DID"
    );
}

#[test]
fn is_did_controller_of_another_did() {
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

    // Check if the controller DID is a controller of the main DID
    let is_controller = contract
        .is_did_controller(Did::new(&did), Controller::new(&controller_did))
        .unwrap();
    assert!(
        is_controller,
        "Expected the controller DID to be a controller of the main DID"
    );
}

#[test]
fn is_non_existing_did_not_a_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let non_existing_did = format!("{}{}", DID_PREFIX, "non_existing_did");

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "valid_did");
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

    // Check if the non-existing DID is a controller of the DID
    let is_controller = contract
        .is_did_controller(Did::new(&did), Controller::new(&non_existing_did))
        .unwrap();
    assert!(
        !is_controller,
        "Expected the non-existing DID to not be a controller of the DID"
    );
}

#[test]
fn is_did_controller_with_invalid_did_format() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let controller = "controller".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Define a DID with an invalid format
    let invalid_did = "invalid_did_format";

    // Attempt to check if the controller is a controller of the invalid DID
    let result = contract.is_did_controller(
        Did::new(invalid_did),
        Controller::new(&controller.to_string()),
    );
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        "Generic error: Querier contract error: Did format error: invalid_did_format",
        result.err().unwrap().to_string()
    );
}

#[test]
fn is_did_controller_with_invalid_controller_format() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "valid_did");
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

    // Define a controller with an invalid format
    let invalid_controller = "invalid_controller_format";

    // Attempt to check if the invalid controller is a controller of the DID
    let result = contract.is_did_controller(Did::new(&did), Controller::new(invalid_controller));
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        "Generic error: Querier contract error: Controller format error: invalid_controller_format",
        result.err().unwrap().to_string()
    );
}

#[test]
fn is_did_controller_with_looped_controllers() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Create the first DID Document (loop start)
    let did1 = format!("{}{}", DID_PREFIX, "did1");
    let did_doc1 = DidDocument {
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

    // Check if did1 is a controller of itself through the loop
    let is_controller = contract
        .is_did_controller(Did::new(&did1), Controller::new(&did1))
        .unwrap();
    assert!(
        is_controller,
        "Expected did1 to be a controller of itself through the loop"
    );

    // Check if did2 is a controller of did1
    let is_controller = contract
        .is_did_controller(Did::new(&did1), Controller::new(&did2))
        .unwrap();
    assert!(
        is_controller,
        "Expected did2 to be a controller of did1 through the loop"
    );

    // Check if did3 is a controller of did1
    let is_controller = contract
        .is_did_controller(Did::new(&did1), Controller::new(&did3))
        .unwrap();
    assert!(
        is_controller,
        "Expected did3 to be a controller of did1 through the loop"
    );

    let is_controller = contract
        .is_did_controller(Did::new(&did1), Controller::new(&owner.to_string()))
        .unwrap();
    assert!(
        is_controller,
        "Expected did3 to be a controller of did1 through the loop"
    );

    let is_controller = contract
        .is_did_controller(Did::new(&did2), Controller::new(&did1))
        .unwrap();
    assert!(
        is_controller,
        "Expected did1 to be a controller of itself through the loop"
    );

    let is_controller = contract
        .is_did_controller(Did::new(&did2), Controller::new(&did2))
        .unwrap();
    assert!(
        is_controller,
        "Expected did2 to be a controller of did1 through the loop"
    );

    let is_controller = contract
        .is_did_controller(Did::new(&did2), Controller::new(&did3))
        .unwrap();
    assert!(
        is_controller,
        "Expected did3 to be a controller of did1 through the loop"
    );

    let is_controller = contract
        .is_did_controller(Did::new(&did2), Controller::new(&owner.to_string()))
        .unwrap();
    assert!(
        is_controller,
        "Expected did3 to be a controller of did1 through the loop"
    );

    let is_controller = contract
        .is_did_controller(Did::new(&did3), Controller::new(&did1))
        .unwrap();
    assert!(
        is_controller,
        "Expected did1 to be a controller of itself through the loop"
    );

    let is_controller = contract
        .is_did_controller(Did::new(&did3), Controller::new(&did2))
        .unwrap();
    assert!(
        is_controller,
        "Expected did2 to be a controller of did1 through the loop"
    );

    let is_controller = contract
        .is_did_controller(Did::new(&did3), Controller::new(&did3))
        .unwrap();
    assert!(
        is_controller,
        "Expected did3 to be a controller of did1 through the loop"
    );

    let is_controller = contract
        .is_did_controller(Did::new(&did3), Controller::new(&owner.to_string()))
        .unwrap();
    assert!(
        is_controller,
        "Expected did3 to be a controller of did1 through the loop"
    );

    // Check if an unrelated DID is not a controller
    let unrelated_did = format!("{}{}", DID_PREFIX, "unrelated_did");
    let is_controller = contract
        .is_did_controller(Did::new(&did1), Controller::new(&unrelated_did))
        .unwrap();
    assert!(
        !is_controller,
        "Expected unrelated_did to not be a controller of did1"
    );
}
