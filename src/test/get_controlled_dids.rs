use sylvia::cw_multi_test::IntoAddr;
use sylvia::multitest::App;

use crate::{
    contract::sv::mt::{CodeId, DidContractProxy},
    state::{Did, DidDocument, Service, DID_PREFIX},
};

#[test]
fn get_controlled_dids() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let owner2 = "owner2".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // let did_owner = "did_owner";
    let did1 = format!("{}{}", DID_PREFIX, "new_did11111111111111111111111111");
    let new_did_doc = DidDocument {
        id: Did::new(&did1),
        // controller: Controllers(vec![owner.to_string().into()]),
        controller: vec![owner.to_string().into()],
        service: vec![Service {
            a_type: "".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "ffffff")),
            service_endpoint: "dfdsfs".to_string(),
        }],
    };
    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    let did2 = format!("{}{}", DID_PREFIX, "new_did22222222222222222222222222");
    let new_did_doc = DidDocument {
        id: Did::new(&did2),
        // controller: Controllers(vec![owner.to_string().into()]),
        controller: vec![owner.to_string().into()],
        service: vec![Service {
            a_type: "".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "ffffff")),
            service_endpoint: "dfdsfs".to_string(),
        }],
    };

    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    let did3 = format!("{}{}", DID_PREFIX, "new_did333333333333333333333333333");
    let new_did_doc = DidDocument {
        id: Did::new(&did3),
        // controller: Controllers(vec![owner2.to_string().into()]),
        controller: vec![owner2.to_string().into()],
        service: vec![Service {
            a_type: "".to_string(),
            id: Did::new(&format!("{}{}", DID_PREFIX, "ffffff")),
            service_endpoint: "dfdsfs".to_string(),
        }],
    };

    let result = contract
        .create_did_document(new_did_doc.clone())
        .call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    let dids = contract
        .get_controlled_dids(owner.to_string().into(), Some(1), None)
        .unwrap();
    for d in &dids {
        println!("AAA {}", d)
    }
    assert_eq!(&vec![did1.to_string()], &dids);

    let dids = contract
        .get_controlled_dids(owner.to_string().into(), None, None)
        .unwrap();
    for d in &dids {
        println!("BBB {}", d)
    }
    assert_eq!(&vec![did1.to_string(), did2.to_string()], &dids);

    let dids = contract
        .get_controlled_dids(owner2.to_string().into(), None, None)
        .unwrap();
    for d in &dids {
        println!("CCC {}", d)
    }
    assert_eq!(&vec![did3.to_string()], &dids);
}


#[test]
fn retrieve_controlled_dids_for_single_controller() {
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
    let result = contract.create_did_document(did_doc1.clone()).call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    let result = contract.create_did_document(did_doc2.clone()).call(&owner);
    assert!(result.is_ok(), "Expected Ok, but got an Err");

    // Retrieve controlled DIDs
    let controlled_dids = contract
        .get_controlled_dids(owner.to_string().into(), None, None)
        .unwrap();
    assert_eq!(
        controlled_dids,
        vec![Did::new(&did1), Did::new(&did2)],
        "Expected controlled DIDs to match"
    );
}

#[test]
fn retrieve_controlled_dids_for_controller_with_no_dids() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let controller = "controller_with_no_dids".into_addr();

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Retrieve controlled DIDs for a controller with no DIDs
    let controlled_dids = contract
        .get_controlled_dids(controller.to_string().into(), None, None)
        .unwrap();
    assert!(
        controlled_dids.is_empty(),
        "Expected no controlled DIDs, but got some"
    );
}

#[test]
fn retrieve_controlled_dids_with_pagination() {
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

    // Retrieve controlled DIDs with a limit of 2
    let controlled_dids = contract
        .get_controlled_dids(owner.to_string().into(), Some(2), None)
        .unwrap();
    assert_eq!(
        controlled_dids,
        vec![Did::new(&did1), Did::new(&did2)],
        "Expected first two controlled DIDs"
    );

    // Retrieve the next set of controlled DIDs
    let controlled_dids = contract
        .get_controlled_dids(owner.to_string().into(), Some(2), Some(did2.clone()))
        .unwrap();
    assert_eq!(
        controlled_dids,
        vec![Did::new(&did3)],
        "Expected the last controlled DID"
    );
}

#[test]
fn retrieve_controlled_dids_with_invalid_controller_format() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let invalid_controller = "invalid_controller_format";

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Attempt to retrieve controlled DIDs with an invalid controller format
    let result = contract.get_controlled_dids(invalid_controller.into(), None, None);
    assert!(result.is_err(), "Expected Err, but got an Ok");

    // Verify the error message
    assert_eq!(
        format!("Generic error: Querier contract error: Controller format error: {}", invalid_controller),
        result.err().unwrap().to_string()
    );
}

#[test]
fn retrieve_controlled_dids_for_non_existing_controller() {
    let app = App::default();
    let code_id = CodeId::store_code(&app);

    let owner = "owner".into_addr();
    let non_existing_controller = format!("{}{}", DID_PREFIX, "non_existing_controller");

    let contract = code_id.instantiate().call(&owner).unwrap();

    // Retrieve controlled DIDs for a non-existing controller
    let controlled_dids = contract
        .get_controlled_dids(non_existing_controller.into(), None, None)
        .unwrap();
    assert!(
        controlled_dids.is_empty(),
        "Expected no controlled DIDs, but got some"
    );
}