use sylvia::cw_multi_test::IntoAddr;
use sylvia::multitest::App;

use crate::{
    contract::sv::mt::{CodeId, DidContractProxy},
    state::{Did, DidDocument, Service, DID_PREFIX},
};

#[test]
fn retrieve_all_controlled_did_documents_for_single_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Create two DID Documents controlled by the same controller
    let did1 = format!("{}{}", DID_PREFIX, "did1");
    let did2 = format!("{}{}", DID_PREFIX, "did2");

    let did_doc1 = DidDocument {
        id: Did::new(&did1),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };

    let did_doc2 = DidDocument {
        id: Did::new(&did2),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };

    // Save the DID Documents
    contract.create_did_document(did_doc1.clone()).call(&owner).unwrap();
    contract.create_did_document(did_doc2.clone()).call(&owner).unwrap();

    // Retrieve controlled DID Documents
    let controlled_did_docs = contract
        .get_controlled_did_documents(owner.to_string().into(), None, None)
        .unwrap();
    assert_eq!(
        controlled_did_docs,
        vec![did_doc1, did_doc2],
        "Expected controlled DID Documents to match"
    );
}

#[test]
fn retrieve_controlled_did_documents_for_controller_with_no_dids() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let controller = "controller_with_no_dids".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Retrieve controlled DID Documents for a controller with no DIDs
    let controlled_did_docs = contract
        .get_controlled_did_documents(controller.to_string().into(), None, None)
        .unwrap();
    assert!(
        controlled_did_docs.is_empty(),
        "Expected no controlled DID Documents, but got some"
    );
}

#[test]
fn retrieve_controlled_did_documents_with_pagination() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Create three DID Documents controlled by the same controller
    let did1 = format!("{}{}", DID_PREFIX, "did1");
    let did2 = format!("{}{}", DID_PREFIX, "did2");
    let did3 = format!("{}{}", DID_PREFIX, "did3");

    let did_doc1 = DidDocument {
        id: Did::new(&did1),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };

    let did_doc2 = DidDocument {
        id: Did::new(&did2),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };

    let did_doc3 = DidDocument {
        id: Did::new(&did3),
        controller: vec![owner.to_string().into()],
        service: vec![],
    };

    // Save the DID Documents
    contract.create_did_document(did_doc1.clone()).call(&owner).unwrap();
    contract.create_did_document(did_doc2.clone()).call(&owner).unwrap();
    contract.create_did_document(did_doc3.clone()).call(&owner).unwrap();

    // Retrieve controlled DID Documents with a limit of 2
    let controlled_did_docs = contract
        .get_controlled_did_documents(owner.to_string().into(), Some(2), None)
        .unwrap();
    assert_eq!(
        controlled_did_docs,
        vec![did_doc1.clone(), did_doc2.clone()],
        "Expected first two controlled DID Documents"
    );

    // Retrieve the next set of controlled DID Documents
    let controlled_did_docs = contract
        .get_controlled_did_documents(owner.to_string().into(), Some(2), Some(did2.clone()))
        .unwrap();
    assert_eq!(
        controlled_did_docs,
        vec![did_doc3.clone()],
        "Expected the last controlled DID Document"
    );
}

#[test]
fn retrieve_controlled_did_documents_for_non_existing_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let non_existing_controller = format!("{}{}", DID_PREFIX, "non_existing_controller");

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Retrieve controlled DID Documents for a non-existing controller
    let controlled_did_docs = contract
        .get_controlled_did_documents(non_existing_controller.into(), None, None)
        .unwrap();
    assert!(
        controlled_did_docs.is_empty(),
        "Expected no controlled DID Documents, but got some"
    );
}

#[test]
fn retrieve_controlled_did_documents_with_invalid_controller_format() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let invalid_controller = "invalid_controller_format";

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Attempt to retrieve controlled DID Documents with an invalid controller format
    let result = contract.get_controlled_did_documents(invalid_controller.into(), None, None);
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        format!("Generic error: Querier contract error: Controller format error: {}", invalid_controller),
        result.err().unwrap().to_string()
    );
}

#[test]
fn retrieve_controlled_did_documents_with_services() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Create a DID Document with services
    let did = format!("{}{}", DID_PREFIX, "did_with_services");
    let service1 = Service {
        id: Did::new(&format!("{}{}", DID_PREFIX, "service1")),
        a_type: "ServiceType1".to_string(),
        service_endpoint: "https://service1.com".to_string(),
    };
    let service2 = Service {
        id: Did::new(&format!("{}{}", DID_PREFIX, "service2")),
        a_type: "ServiceType2".to_string(),
        service_endpoint: "https://service2.com".to_string(),
    };

    let did_doc = DidDocument {
        id: Did::new(&did),
        controller: vec![owner.to_string().into()],
        service: vec![service1.clone(), service2.clone()],
    };

    // Save the DID Document
    contract.create_did_document(did_doc.clone()).call(&owner).unwrap();

    // Retrieve controlled DID Documents
    let controlled_did_docs = contract
        .get_controlled_did_documents(owner.to_string().into(), None, None)
        .unwrap();
    assert_eq!(
        controlled_did_docs,
        vec![did_doc],
        "Expected controlled DID Document with services"
    );
}