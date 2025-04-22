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
    assert_eq!(
        "Did format error: new_did",
        result.err().unwrap().to_string()
    );

    let did = format!("{}{}", DID_PREFIX, "new_did");
    new_did_doc.id = Did::new(&did);

    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!(
        "Service id format error: Did format error: dfdsfs",
        result.err().unwrap().to_string()
    );

    new_did_doc.service[0].id = Did::new(&format!("{}{}", DID_PREFIX, "ffffff"));

    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    let did_document = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(new_did_doc.clone(), did_document.clone());
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
    assert_eq!(
        "Did format error: invalid_did_format",
        result.err().unwrap().to_string()
    );
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
fn create_document_with_duplicated_did() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "duplicated_did");
    let new_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    // First creation should succeed
    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Attempt to create a document with the same DID
    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!(
        "Did document already exists: did:c4e:duplicated_did",
        result.err().unwrap().to_string()
    );
}

#[test]
fn create_document_with_invalid_controller_format() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "new_did");
    let new_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec!["invalid_controller_format".to_string().into()], // Invalid controller format
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
        "Controller format error: invalid_controller_format",
        result.err().unwrap().to_string()
    );
}

#[test]
fn create_document_with_nonexistent_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "new_did");
    let new_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![format!("{}{}", DID_PREFIX, "nonexistent_controller").into()], // Nonexistent controller
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
        "Did controller not found: did:c4e:nonexistent_controller",
        result.err().unwrap().to_string()
    );
}

#[test]
fn create_document_with_invalid_service_id() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "new_did");
    let new_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new("invalid_service_id"), // Invalid service ID
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_err(), "Expected Err, but got an Ok");
    assert_eq!(
        "Service id format error: Did format error: invalid_service_id",
        result.err().unwrap().to_string()
    );
}

#[test]
fn create_document_not_self_controlled() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "self_controlled_did");
    let new_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![did.to_string().into()], // Self-controlled
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
        "Self controlled did document not allowed: did:c4e:self_controlled_did",
        result.err().unwrap().to_string()
    );
}

#[test]
fn create_document_with_unsignable_did() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    const DID_STORE: Map<String, DidDocument> = Map::new("dids");

    let did_controlled_by_itself = format!("{}{}", DID_PREFIX, "did_controlled_by_himself");
    let did_doc_controlled_by_itself = DidDocument {
        id: Did::new(&did_controlled_by_itself),
        // controller: Controllers(vec![did_controlled_by_itself.to_string().into()]),
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
    let new_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![did_controlled_by_itself.into()], // Nonexistent controller
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
        "Did document unsignable: did:c4e:unsignable_did",
        result.err().unwrap().to_string()
    );
}

#[test]
fn create_document_with_event_checks() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "event_test_did");
    let service_did = format!("{}{}", DID_PREFIX, "service1");
    let new_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: service_did.to_string().into(),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    // Create the DID Document
    let result = contract
        .create_did_document(new_did_doc.clone())
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

    assert_eq!(res.events[1].attributes.len(), 4);

    assert_eq!(res.events[1].ty, "wasm-create_did_document");
    assert_eq!(res.events[1].attributes[0].key, "_contract_address");
    assert_eq!(
        res.events[1].attributes[0].value,
        contract.contract_addr.to_string()
    );
    assert_eq!(res.events[1].attributes[1].key, "did");
    assert_eq!(res.events[1].attributes[1].value, did.to_string());
    assert_eq!(res.events[1].attributes[2].key, "controllers");
    assert_eq!(res.events[1].attributes[2].value, owner.to_string());
    assert_eq!(res.events[1].attributes[3].key, "services");
    assert_eq!(res.events[1].attributes[3].value, service_did);
    // assert_eq!(res.events[1].attributes[2].value, admin1.to_string());
}

#[test]
fn create_document_with_event_checks_many_controllers_and_services() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let contorller2 = "contorller2".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "event_test_did");
    let service_did: String = format!("{}{}", DID_PREFIX, "service1");
    let service_did2: String = format!("{}{}", DID_PREFIX, "service2");

    let new_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into(), contorller2.to_string().into()],
        service: vec![
            Service {
                a_type: "ServiceType".to_string(),
                id: service_did.to_string().into(),
                service_endpoint: "http://example.com".to_string(),
            },
            Service {
                a_type: "ServiceType".to_string(),
                id: service_did2.to_string().into(),
                service_endpoint: "http://example.com".to_string(),
            },
        ],
    };

    // Create the DID Document
    let result = contract
        .create_did_document(new_did_doc.clone())
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

    assert_eq!(res.events[1].attributes.len(), 4);

    assert_eq!(res.events[1].ty, "wasm-create_did_document");
    assert_eq!(res.events[1].attributes[0].key, "_contract_address");
    assert_eq!(
        res.events[1].attributes[0].value,
        contract.contract_addr.to_string()
    );
    assert_eq!(res.events[1].attributes[1].key, "did");
    assert_eq!(res.events[1].attributes[1].value, did.to_string());
    assert_eq!(res.events[1].attributes[2].key, "controllers");
    assert_eq!(
        res.events[1].attributes[2].value,
        format!("{},{}", owner.to_string(), contorller2.to_string())
    );
    assert_eq!(res.events[1].attributes[3].key, "services");
    assert_eq!(
        res.events[1].attributes[3].value,
        format!("{},{}", service_did, service_did2)
    );
    // assert_eq!(res.events[1].attributes[2].value, admin1.to_string());
}

#[test]
fn create_document_with_no_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "no_controller_did");
    let new_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![], // No controllers provided
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    // Create the DID Document
    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Retrieve the DID Document
    let did_document = contract.get_did_document(Did::new(&did)).unwrap();

    // Verify that the owner was added as the controller
    assert_eq!(
        did_document.controller,
        vec![owner.to_string().into()],
        "Expected the owner to be added as the controller"
    );

    // Verify that the rest of the document matches
    assert_eq!(did_document.service, new_did_doc.service);
}


#[test]
fn create_document_with_normal_address_as_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let normal_address = "normal_address".into_addr(); // Normal address as controller

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "normal_address_controller_did");
    let new_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![normal_address.to_string().into()], // Normal address as controller
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    // Create the DID Document
    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Retrieve the DID Document
    let did_document = contract.get_did_document(Did::new(&did)).unwrap();

    // Verify that the controller matches the normal address
    assert_eq!(
        did_document.controller,
        vec![normal_address.to_string().into()],
        "Expected the controller to match the normal address"
    );

    // Verify that the rest of the document matches
    assert_eq!(did_document.service, new_did_doc.service);
}

#[test]
fn create_document_with_other_did_as_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let normal_address = "normal_address".into_addr(); // Normal address as controller

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Create the first DID Document with a normal address as its controller
    let controller_did = format!("{}{}", DID_PREFIX, "controller_did");
    let controller_did_doc = DidDocument {
        id: Did::new(&controller_did),
        controller: vec![normal_address.to_string().into()], // Normal address as controller
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
    let did = format!("{}{}", DID_PREFIX, "new_did_with_did_controller");
    let new_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![controller_did.to_string().into()], // Another DID as controller
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "service2")),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Retrieve the DID Document
    let did_document = contract.get_did_document(Did::new(&did)).unwrap();

    // Verify that the controller matches the first DID
    assert_eq!(
        did_document.controller,
        vec![controller_did.to_string().into()],
        "Expected the controller to match the first DID"
    );

    // Verify that the rest of the document matches
    assert_eq!(did_document.service, new_did_doc.service);
}

#[test]
fn create_did_document_with_index_verification() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let controller2 = "controller2".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "indexed_did");
    let service_did = format!("{}{}", DID_PREFIX, "service1");
    let new_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into(), controller2.to_string().into()],
        service: vec![Service {
            a_type: "ServiceType".to_string(),
            id: Did::new(&service_did),
            service_endpoint: "http://example.com".to_string(),
        }],
    };

    // Create the DID Document
    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Verify that the DID is indexed under the first controller
    let controlled_dids = contract
        .get_controlled_dids(owner.to_string().into(), None, None)
        .unwrap();
    assert_eq!(
        controlled_dids,
        vec![did.clone().into()],
        "Expected the DID to be indexed under the first controller"
    );

    // Verify that the DID is indexed under the second controller
    let controlled_dids = contract
        .get_controlled_dids(controller2.to_string().into(), None, None)
        .unwrap();
    assert_eq!(
        controlled_dids,
        vec![did.clone().into()],
        "Expected the DID to be indexed under the second controller"
    );

    // Verify the created DID Document
    let created_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(
        new_did_doc, created_did_doc,
        "DID Document was not created correctly"
    );
}

#[test]
fn create_did_document_with_no_services() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    let did = format!("{}{}", DID_PREFIX, "no_services_did");
    let new_did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()], // Single controller
        service: vec![], // No services
    };

    // Create the DID Document
    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Verify the created DID Document
    let created_did_doc = contract.get_did_document(Did::new(&did)).unwrap();
    assert_eq!(
        new_did_doc, created_did_doc,
        "DID Document was not created correctly"
    );

    // Verify that the DID is indexed under the controller
    let controlled_dids = contract
        .get_controlled_dids(owner.to_string().into(), None, None)
        .unwrap();
    assert_eq!(
        controlled_dids,
        vec![did.clone().into()],
        "Expected the DID to be indexed under the controller"
    );
}