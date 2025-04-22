use cw_storage_plus::Map;
use sylvia::cw_multi_test::IntoAddr;
use sylvia::multitest::App;

use crate::{
    contract::sv::mt::{CodeId, DidContractProxy},
    state::{Did, DidDocument, Service, DID_PREFIX},
};

#[test]
fn update_did_document_modify_services() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "valid_update_did");
    let mut original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    // Create the original DID Document
    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Update the DID Document
    original_did_doc.service.push(Service {
        a_type: "NewServiceType".to_string(),
        id: Did::new(&format!("{}{}", DID_PREFIX, "service2")),
        service_endpoint: "http://new-service.com".to_string(),
    });

    let result = contract
        .update_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Verify the updated DID Document
    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(original_did_doc, updated_did_doc, "DID Document was not updated correctly");
}


#[test]
fn update_did_document_modify_controllers_and_services() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "valid_update_did");
    let mut original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    // Create the original DID Document
    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    let controller = "controller".into_addr();

    original_did_doc.controller.push(controller.to_string().into());

    // Update the DID Document
    original_did_doc.service.push(Service {
        a_type: "NewServiceType".to_string(),
        id: Did::new(&format!("{}{}", DID_PREFIX, "service2")),
        service_endpoint: "http://new-service.com".to_string(),
    });

    let result = contract
        .update_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Verify the updated DID Document
    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(original_did_doc, updated_did_doc, "DID Document was not updated correctly");
}


#[test]
fn update_did_document_modify_controllers() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "valid_update_did");
    let mut original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    // Create the original DID Document
    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    let controller = "controller".into_addr();

    original_did_doc.controller.push(controller.to_string().into());

    let result = contract
        .update_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Verify the updated DID Document
    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(original_did_doc, updated_did_doc, "DID Document was not updated correctly");
}

#[test]
fn update_nonexistent_did_document() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "nonexistent_did");
    let nonexistent_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };

    let result = contract
        .update_did_document(nonexistent_did_doc.clone())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!(
        "Did document not found: type: did_contract::state::DidDocument; key: [00, 04, 64, 69, 64, 73, 64, 69, 64, 3A, 63, 34, 65, 3A, 6E, 6F, 6E, 65, 78, 69, 73, 74, 65, 6E, 74, 5F, 64, 69, 64] not found",
        result.err().unwrap().to_string()
    );

    let did_doc = contract.get_did_document(Did::new(&did));
    assert!(did_doc.is_err(), "Expected Err, but got an Ok");


}

#[test]
fn update_did_document_with_invalid_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "invalid_controller_did");
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

    let mut update_did_doc = original_did_doc.clone();
    // Attempt to update with an invalid controller
    update_did_doc.controller.push("invalid_controller".to_string().into());

    let result = contract
        .update_did_document(update_did_doc.clone())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!(
        "Controller format error: invalid_controller",
        result.err().unwrap().to_string()
    );

    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(original_did_doc, updated_did_doc, "DID Document was not updated correctly");

}

#[test]
fn update_did_document_with_duplicate_controllers() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "duplicate_controllers_did");
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

    let mut update_did_doc = original_did_doc.clone();

    // Attempt to update with duplicate controllers
    update_did_doc.controller.push(owner.to_string().into());

    let result = contract
        .update_did_document(update_did_doc.clone())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!(
        format!("Duplicated controller: {}", owner.to_string()),
        result.err().unwrap().to_string()
    );

    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(original_did_doc, updated_did_doc, "DID Document was not updated correctly");

}

#[test]
fn update_did_document_to_self_controlled() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "self_controlled_update_did");
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

    let mut update_did_doc = original_did_doc.clone();
    // Attempt to update to self-controlled
    update_did_doc.controller.push(did.to_string().into());

    let result = contract
        .update_did_document(update_did_doc.clone())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!(
        format!("Self controlled did document not allowed: {}", did),
        result.err().unwrap().to_string()
    );

    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(original_did_doc, updated_did_doc, "DID Document was not updated correctly");

}

#[test]
fn update_did_document_unauthorized() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let unauthorized_user = "unauthorized_user".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "unauthorized_update_did");
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

    let mut update_did_doc = original_did_doc.clone();

    // Attempt to update the DID Document as an unauthorized user
    update_did_doc.service.push(Service {
        a_type: "UnauthorizedService".to_string(),
        id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
        service_endpoint: "http://unauthorized-service.com".to_string(),
    });

    let result = contract
        .update_did_document(update_did_doc.clone())
        .call(&unauthorized_user);
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!(
        format!("Unauthorized: {}", unauthorized_user.to_string()),
        result.err().unwrap().to_string()
    );

    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(original_did_doc, updated_did_doc, "DID Document was not updated correctly");

}

#[test]
fn update_did_document_with_did_as_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Create the first DID Document (controller DID)
    let controller_did = format!("{}{}", DID_PREFIX, "controller_did");
    let controller_did_doc = DidDocument {
        id: Did::new(&controller_did),
        controller: vec![owner.to_string().into()], // Normal address as controller
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    let result = contract
        .create_did_document(controller_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Create the second DID Document
    let did = format!("{}{}", DID_PREFIX, "update_did_with_did_controller");
    let mut original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![controller_did.into()],
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "service2")),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Update the second DID Document to use the first DID as a controller
    original_did_doc.controller.push("new_controller".into_addr().to_string().into());

    let result = contract
        .update_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Verify the updated DID Document
    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(original_did_doc, updated_did_doc, "DID Document was not updated correctly");
}

#[test]
fn update_did_document_by_two_separate_controllers() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner1 = "owner1".into_addr();
    let owner2 = "owner2".into_addr();
    let second_controller_did = format!("{}{}", DID_PREFIX, "second_controller_did");

    let contract = code_id.instantiate().call(&owner1).unwrap();

    // Create the first DID Document (second controller DID)
    let second_controller_did_doc = DidDocument {
        id: Did::new(&second_controller_did),
        controller: vec![owner2.to_string().into()], // Normal address as controller
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    let result = contract
        .create_did_document(second_controller_did_doc.clone())
        .call(&owner2);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Create the main DID Document with two controllers (address and DID)
    let did = format!("{}{}", DID_PREFIX, "main_did");
    let mut original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner1.to_string().into(), second_controller_did.to_string().into()],
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "service2")),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner1);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // First update by the address controller
    original_did_doc.service.push(Service {
        a_type: "NewServiceTypeByOwner".to_string(),
        id: Did::new(&format!("{}{}", DID_PREFIX, "service3")),
        service_endpoint: "http://owner-service.com".to_string(),
    });

    let result = contract
        .update_did_document(original_did_doc.clone())
        .call(&owner1);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Verify the updated DID Document after the first update
    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(original_did_doc, updated_did_doc, "DID Document was not updated correctly after the first update");

    // Second update by the DID controller
    let mut updated_did_doc_by_did = updated_did_doc.clone();
    updated_did_doc_by_did.service.push(Service {
        a_type: "NewServiceTypeByDID".to_string(),
        id: Did::new(&format!("{}{}", DID_PREFIX, "service4")),
        service_endpoint: "http://did-service.com".to_string(),
    });

    let result = contract
        .update_did_document(updated_did_doc_by_did.clone())
        .call(&owner2);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Verify the updated DID Document after the second update
    let final_updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(updated_did_doc_by_did, final_updated_did_doc, "DID Document was not updated correctly after the second update");
}

#[test]
fn update_did_document_with_no_controllers() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "no_controllers_update_did");
    let original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    // Create the original DID Document
    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    let mut update_did_doc = original_did_doc.clone();
    // Attempt to update the DID Document with no controllers
    update_did_doc.controller.clear();

    let result = contract
        .update_did_document(update_did_doc.clone())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!(
        format!("Did document {} has no controller", did),
        result.err().unwrap().to_string()
    );

    // Verify that the DID Document was not updated
    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(original_did_doc, updated_did_doc, "DID Document was updated incorrectly");
}

#[test]
fn update_document_with_unsignable_did() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    const DID_STORE: Map<String, DidDocument> = Map::new("dids");

    let did_controlled_by_itself = format!("{}{}", DID_PREFIX, "did_controlled_by_himself");
    let did_doc_controlled_by_itself = DidDocument {
        id: Did::new(&did_controlled_by_itself),
        controller: vec![did_controlled_by_itself.to_string().into()],
        service: vec![],
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

    let did = format!("{}{}", DID_PREFIX, "unsignable_did");
    let original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    // Create the original DID Document
    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Attempt to update the DID Document with an unsignable controller
    let mut updated_did_doc = original_did_doc.clone();
    updated_did_doc.controller = vec![did_controlled_by_itself.into()];

    let result = contract
        .update_did_document(updated_did_doc.clone())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!(
        format!("Did document unsignable: {}", did),
        result.err().unwrap().to_string()
    );

    // Verify that the DID Document was not updated
    let current_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(original_did_doc, current_did_doc, "DID Document was updated incorrectly");
}

#[test]
fn update_did_document_unauthorized_when_did_is_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let unauthorized_user = "unauthorized_user".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Create the first DID Document (controller DID)
    let controller_did = format!("{}{}", DID_PREFIX, "controller_did");
    let controller_did_doc = DidDocument {
        id: Did::new(&controller_did),
        controller: vec![owner.to_string().into()], // Normal address as controller
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    let result = contract
        .create_did_document(controller_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Create the second DID Document with the first DID as its controller
    let did = format!("{}{}", DID_PREFIX, "main_did");
    let original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![controller_did.to_string().into()], // DID as controller
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "service2")),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Attempt to update the DID Document by an unauthorized user
    let mut updated_did_doc = original_did_doc.clone();
    updated_did_doc.service.push(Service {
        a_type: "NewServiceType".to_string(),
        id: Did::new(&format!("{}{}", DID_PREFIX, "service3")),
        service_endpoint: "http://new-service.com".to_string(),
    });

    let result = contract
        .update_did_document(updated_did_doc.clone())
        .call(&unauthorized_user);
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!(
        format!("Unauthorized: {}", unauthorized_user.to_string()),
        result.err().unwrap().to_string()
    );

    // Verify that the DID Document was not updated
    let current_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(original_did_doc, current_did_doc, "DID Document was updated incorrectly");
}

#[test]
fn update_did_document_with_event_verification() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "event_verification_did");
    let service1_did = format!("{}{}", DID_PREFIX, "service_did");
    let mut original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&service1_did),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    // Create the original DID Document
    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Update the DID Document
    let service2_did = format!("{}{}", DID_PREFIX, "service2_did");
    original_did_doc.service = vec![Service {
        a_type: "NewServiceType".to_string(),
        id: Did::new(&service2_did),
        service_endpoint: "http://new-service.com".to_string(),
    }];

    let result = contract
        .update_did_document(original_did_doc.clone())
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

    assert_eq!(res.events[1].attributes.len(), 6);

    assert_eq!(res.events[1].ty, "wasm-update_did_document");
    assert_eq!(res.events[1].attributes[0].key, "_contract_address");
    assert_eq!(
        res.events[1].attributes[0].value,
        contract.contract_addr.to_string()
    );
    assert_eq!(res.events[1].attributes[1].key, "did");
    assert_eq!(res.events[1].attributes[1].value, did.to_string());
    assert_eq!(res.events[1].attributes[2].key, "old_controllers");
    assert_eq!(
        res.events[1].attributes[2].value,
         owner.to_string()
    );

    assert_eq!(res.events[1].attributes[3].key, "new_controllers");
    assert_eq!(
        res.events[1].attributes[3].value,
         owner.to_string()
    );
    assert_eq!(res.events[1].attributes[4].key, "old_services");
    assert_eq!(
        res.events[1].attributes[4].value,
        service1_did.to_string()
    );


    assert_eq!(res.events[1].attributes[5].key, "new_services");
    assert_eq!(
        res.events[1].attributes[5].value,
        service2_did.to_string()
    );
    // Verify the updated DID Document
    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(original_did_doc, updated_did_doc, "DID Document was not updated correctly");
}


#[test]
fn update_did_document_with_event_verification_many_controller_and_services() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let owner2 = "owner2".into_addr();
    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "event_verification_did");
    let service1_did = format!("{}{}", DID_PREFIX, "service_did");
    let mut original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&service1_did),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    // Create the original DID Document
    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Update the DID Document
    original_did_doc.controller.push(owner2.to_string().into());
    
    let service2_did = format!("{}{}", DID_PREFIX, "service2_did");
    original_did_doc.service.push(Service {
        a_type: "NewServiceType".to_string(),
        id: Did::new(&service2_did),
        service_endpoint: "http://new-service.com".to_string(),
    });

    let result = contract
        .update_did_document(original_did_doc.clone())
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

    assert_eq!(res.events[1].attributes.len(), 6);

    assert_eq!(res.events[1].ty, "wasm-update_did_document");
    assert_eq!(res.events[1].attributes[0].key, "_contract_address");
    assert_eq!(
        res.events[1].attributes[0].value,
        contract.contract_addr.to_string()
    );
    assert_eq!(res.events[1].attributes[1].key, "did");
    assert_eq!(res.events[1].attributes[1].value, did.to_string());
    assert_eq!(res.events[1].attributes[2].key, "old_controllers");
    assert_eq!(
        res.events[1].attributes[2].value,
        owner.to_string()
    );
    assert_eq!(res.events[1].attributes[3].key, "new_controllers");
    assert_eq!(
        res.events[1].attributes[3].value,
         format!("{},{}", owner.to_string(), owner2.to_string())
    );
    assert_eq!(res.events[1].attributes[4].key, "old_services");
    assert_eq!(
        res.events[1].attributes[4].value,
        service1_did.to_string()
    );

    assert_eq!(res.events[1].attributes[5].key, "new_services");
    assert_eq!(
        res.events[1].attributes[5].value,
        format!("{},{}", service1_did.to_string(), service2_did.to_string())
    );

    // Verify the updated DID Document
    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(original_did_doc, updated_did_doc, "DID Document was not updated correctly");
}


#[test]
fn update_did_document_with_index_verification() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let new_controller = "new_controller".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "index_verification_did");
    let mut original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    // Create the original DID Document
    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Verify the original controller is indexed
    let controlled_dids = contract
        .get_controlled_dids(owner.to_string().into(), None, None)
        .unwrap();
    assert_eq!(
        controlled_dids,
        vec![did.clone().into()],
        "Expected the DID to be indexed under the original controller"
    );

    // Update the DID Document: Add a new controller and remove the old one
    original_did_doc.controller = vec![new_controller.to_string().into()];
    original_did_doc.service.push(Service {
        a_type: "NewServiceType".to_string(),
        id: Did::new(&format!("{}{}", DID_PREFIX, "service2")),
        service_endpoint: "http://new-service.com".to_string(),
    });

    let result = contract
        .update_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Verify the old controller no longer indexes the DID
    let controlled_dids = contract
        .get_controlled_dids(owner.to_string().into(), None, None)
        .unwrap();
    assert!(
        controlled_dids.is_empty(),
        "Expected the DID to be removed from the old controller's index"
    );

    // Verify the new controller indexes the DID
    let controlled_dids = contract
        .get_controlled_dids(new_controller.to_string().into(), None, None)
        .unwrap();
    assert_eq!(
        controlled_dids,
        vec![did.clone().into()],
        "Expected the DID to be indexed under the new controller"
    );

    // Verify the updated DID Document
    let updated_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(
        original_did_doc, updated_did_doc,
        "DID Document was not updated correctly"
    );
}


#[test]
fn update_did_document_with_invalid_service_id_format() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "invalid_service_id_did");
    let original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    // Create the original DID Document
    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Attempt to update the DID Document with an invalid service ID format
    let mut updated_did_doc = original_did_doc.clone();
    updated_did_doc.service.push(Service {
        a_type: "InvalidServiceType".to_string(),
        id: Did::new("invalid_service_id"), // Invalid service ID format
        service_endpoint: "http://invalid-service.com".to_string(),
    });

    let result = contract
        .update_did_document(updated_did_doc.clone())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!(
        "Service id format error: Did format error: invalid_service_id",
        result.err().unwrap().to_string()
    );

    // Verify that the DID Document was not updated
    let current_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(original_did_doc, current_did_doc, "DID Document was updated incorrectly");
}

#[test]
fn update_did_document_add_service() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "add_service_did");
    let original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![], // Initially no services
    };

    // Create the original DID Document
    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Update the DID Document to add a service
    let mut updated_did_doc = original_did_doc.clone();
    updated_did_doc.service.push(Service {
        a_type: "NewServiceType".to_string(),
        id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
        service_endpoint: "http://new-service.com".to_string(),
    });

    let result = contract
        .update_did_document(updated_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Verify the updated DID Document
    let current_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(
        updated_did_doc, current_did_doc,
        "DID Document was not updated correctly"
    );

    // Verify the added service
    assert_eq!(
        current_did_doc.service.len(),
        1,
        "Expected one service to be added"
    );
    assert_eq!(
        current_did_doc.service[0].id,
        Did::new(&format!("{}{}", DID_PREFIX, "service1")),
        "Service ID does not match"
    );
    assert_eq!(
        current_did_doc.service[0].a_type,
        "NewServiceType",
        "Service type does not match"
    );
    assert_eq!(
        current_did_doc.service[0].service_endpoint,
        "http://new-service.com",
        "Service endpoint does not match"
    );
}

#[test]
fn update_did_document_remove_service() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "remove_service_did");
    let original_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
            service_endpoint: "http://example.com".to_string(),
        }], // Initially has one service
    };

    // Create the original DID Document
    let result = contract
        .create_did_document(original_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Update the DID Document to remove the service
    let mut updated_did_doc = original_did_doc.clone();
    updated_did_doc.service.clear(); // Remove all services

    let result = contract
        .update_did_document(updated_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Verify the updated DID Document
    let current_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(
        updated_did_doc, current_did_doc,
        "DID Document was not updated correctly"
    );

    // Verify that the service was removed
    assert_eq!(
        current_did_doc.service.len(),
        0,
        "Expected no services to remain"
    );
}