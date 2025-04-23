use cw_storage_plus::Map;
use sylvia::cw_multi_test::IntoAddr;
use sylvia::multitest::App;

use crate::{
    contract::sv::mt::{CodeId, DidContractProxy},
    state::{Controller, Did, DidDocument, DID_PREFIX},
};

#[test]
fn delete_existing_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let controller_to_delete = "controller_to_delete".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "delete_controller_did");
    let mut original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into(), controller_to_delete.to_string().into()],
        service: vec![],
    };

    // Create the original DID Document
    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Delete the controller
    let result = contract
        .delete_controller(Did::new(&did), controller_to_delete.to_string().into())
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

    assert_eq!(res.events[1].ty, "wasm-delete_controller");
    assert_eq!(res.events[1].attributes[0].key, "_contract_address");
    assert_eq!(
        res.events[1].attributes[0].value,
        contract.contract_addr.to_string()
    );
    assert_eq!(res.events[1].attributes[1].key, "did");
    assert_eq!(res.events[1].attributes[1].value, did.to_string());
    assert_eq!(res.events[1].attributes[2].key, "old_controller");
    assert_eq!(res.events[1].attributes[2].value, controller_to_delete.to_string());


    // Verify the updated DID Document
    original_did_doc.controller.retain(|c| c != &controller_to_delete.to_string().into());
    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(original_did_doc, updated_did_doc, "DID Document was not updated correctly");
}

#[test]
fn delete_non_existing_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let non_existing_controller = "non_existing_controller".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "delete_non_existing_controller_did");
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

    // Attempt to delete a non-existing controller
    let result = contract
        .delete_controller(Did::new(&did), non_existing_controller.to_string().into())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        format!("Did document controller not exists: {}", non_existing_controller),
        result.err().unwrap().to_string()
    );

    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(original_did_doc, updated_did_doc, "DID Document was not updated correctly");
}

#[test]
fn delete_last_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "delete_last_controller_did");
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

    // Attempt to delete the last controller
    let result = contract
        .delete_controller(Did::new(&did), owner.to_string().into())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        format!("Did document {} has no controller", did),
        result.err().unwrap().to_string()
    );
    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(original_did_doc, updated_did_doc, "DID Document was not updated correctly");
}

#[test]
fn delete_controller_unauthorized() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let unauthorized_user = "unauthorized_user".into_addr();
    let controller_to_delete = "controller_to_delete".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "unauthorized_delete_controller_did");
    let original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into(), controller_to_delete.to_string().into()],
        service: vec![],
    };

    // Create the original DID Document
    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Attempt to delete the controller by an unauthorized user
    let result = contract
        .delete_controller(Did::new(&did), controller_to_delete.to_string().into())
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
fn delete_controller_with_index_verification() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let controller_to_delete = "controller_to_delete".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "indexed_delete_controller_did");
    let mut original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into(), controller_to_delete.to_string().into()],
        service: vec![],
    };

    // Create the original DID Document
    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Verify the DID is indexed under the controller to delete
    let controlled_dids = contract
        .get_controlled_dids(controller_to_delete.to_string().into(), None, None)
        .unwrap();
    assert_eq!(
        controlled_dids,
        vec![did.clone().into()],
        "Expected the DID to be indexed under the controller to delete"
    );

    // Delete the controller
    let result = contract
        .delete_controller(Did::new(&did), controller_to_delete.to_string().into())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Verify the DID is no longer indexed under the deleted controller
    let controlled_dids = contract
        .get_controlled_dids(controller_to_delete.to_string().into(), None, None)
        .unwrap();
    assert!(
        controlled_dids.is_empty(),
        "Expected the DID to be removed from the deleted controller's index"
    );

    // Verify the updated DID Document
    original_did_doc.controller.retain(|c| c != &controller_to_delete.to_string().into());
    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(
        original_did_doc, updated_did_doc,
        "DID Document was not updated correctly"
    );
}

#[test]
fn delete_controller_with_invalid_did_format() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let controller_to_delete = "controller_to_delete".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Define a DID with an invalid format
    let invalid_did = "invalid_did_format";

    // Attempt to delete a controller from the invalid DID
    let result = contract
        .delete_controller(Did::new(invalid_did), controller_to_delete.to_string().into())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        format!("Did format error: {}", invalid_did),
        result.err().unwrap().to_string()
    );
}

#[test]
fn delete_controller_with_invalid_controller_format() {
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

    // Define a controller with an invalid format
    let invalid_controller = "invalid_controller_format";

    // Attempt to delete the invalid controller
    let result = contract
        .delete_controller(Did::new(&did), Controller::new(invalid_controller))
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        format!("Controller format error: {}", invalid_controller),
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
fn delete_address_controller_with_did_based_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let did_based_controller = format!("{}{}", DID_PREFIX, "did_based_controller");

    let contract = code_id.instantiate().call(&owner).unwrap();

    const DID_STORE: Map<String, DidDocument> = Map::new("dids");

    // Create a DID-based controller document
    let did_doc_based_controller = DidDocument {
        id: Did::new(&did_based_controller),
        controller: vec![did_based_controller.to_string().into()], // Self-controlled DID
        service: vec![],
    };

    {
        let mut app_mut = app.app_mut();
        let mut contract_store = app_mut.contract_storage_mut(&contract.contract_addr);
        let contract_store = contract_store.as_mut();

        let result = DID_STORE.save(
            contract_store,
            did_based_controller.to_string(),
            &did_doc_based_controller,
        );
        assert!(
            result.is_ok(),
            "Expected Ok, but got an Err: {}",
            result.unwrap_err()
        );
    }

    // Create the main DID Document with one address-based controller and one DID-based controller
    let did = format!("{}{}", DID_PREFIX, "main_did");
    let original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into(), did_based_controller.to_string().into()],
        service: vec![],
    };

    // Create the original DID Document
    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Attempt to delete the address-based controller
    let result = contract
        .delete_controller(Did::new(&did), owner.to_string().into())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        format!("Did document unsignable: {}", did),
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
fn delete_controller_in_did_controlled_by_another_did() {
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

    // Create the second DID Document controlled by the first DID
    let did = format!("{}{}", DID_PREFIX, "main_did");
    let mut original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![controller_did.clone().into(), owner2.to_string().into()],
        service: vec![],
    };

    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Attempt to delete the controller from the second DID Document
    let result = contract
        .delete_controller(Did::new(&did), Controller::new(&controller_did))
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

    assert_eq!(res.events[1].ty, "wasm-delete_controller");
    assert_eq!(res.events[1].attributes[0].key, "_contract_address");
    assert_eq!(
        res.events[1].attributes[0].value,
        contract.contract_addr.to_string()
    );
    assert_eq!(res.events[1].attributes[1].key, "did");
    assert_eq!(res.events[1].attributes[1].value, did.to_string());
    assert_eq!(res.events[1].attributes[2].key, "old_controller");
    assert_eq!(res.events[1].attributes[2].value, controller_did.clone());

    // Verify the updated DID Document
    original_did_doc.controller.retain(|c| c != &controller_did.clone().into());
    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(original_did_doc, updated_did_doc, "DID Document was not updated correctly");
}