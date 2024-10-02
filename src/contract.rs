use cosmwasm_std::{Response, StdError, StdResult};
// use cw_storage_plus::Item;
use cw_storage_plus::Map;
use sylvia::{contract, entry_points};
use sylvia::types::{InstantiateCtx, QueryCtx, ExecCtx};
use crate::error::ContractError;
use crate::state::{Controller, DidDocument, Service};
use crate::state::Did;
pub struct DidContract {
    pub did_docs: Map<String, DidDocument>,
}

#[entry_points]
#[contract]
#[sv::error(ContractError)]
impl DidContract {
    pub const fn new() -> Self {
        Self {
            did_docs: Map::new("dids"),
        }
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
        Ok(Response::default())
    }

    #[sv::msg(query)]
    pub fn get_did_document(&self, ctx: QueryCtx, did: String) -> Result<DidDocument, ContractError> {
        let did_doc_result = self.did_docs.load(ctx.deps.storage, did);
        match did_doc_result {
            Ok(did_document) => Ok(did_document),
            Err(e) => match e {
                StdError::NotFound{ .. } => Err(ContractError::DidDocumentNotFound(e)),
                _ => Err(ContractError::DidDocumentError(e)),
            },
        }
    }

    #[sv::msg(query)]
    pub fn is_did_controller(&self, ctx: QueryCtx, did: String, controller: Controller) -> Result<bool, ContractError> {
        let did_doc_result: Result<DidDocument, StdError> = self.did_docs.load(ctx.deps.storage, did);
        match did_doc_result {
            Ok(did_document) => did_document.is_controlled_by(ctx.deps.storage, &self.did_docs, &controller),
            Err(e) => match e {
                StdError::NotFound{ .. } => Err(ContractError::DidDocumentNotFound(e)),
                _ => Err(ContractError::DidDocumentError(e)),
            },
        }
    }


    // TODO add DID and Controller validation in all methods
    #[sv::msg(exec)]
    pub fn create_did_document(&self, ctx: ExecCtx, did_doc: DidDocument) -> Result<Response, ContractError> {
        if self.did_docs.has(ctx.deps.storage, did_doc.id.value().to_string()) {
            return Err(ContractError::DidDocumentAlreadyExists);
        }

        let r = self.did_docs.save(ctx.deps.storage, did_doc.id.to_string(), &did_doc);
        match r {
            Ok(_) => Ok(Response::default()),
            Err(e) => Err(ContractError::DidDocumentError(e))
        }
    }

    #[sv::msg(exec)]
    pub fn update_did_document(&self, ctx: ExecCtx, new_did_doc: DidDocument) -> Result<Response, ContractError> {
        let did_doc = self.did_docs.load(ctx.deps.storage, new_did_doc.id.to_string());
        let did_doc = match did_doc {
            Ok(did_document) => did_document,
            Err(e) => return match e {
                StdError::NotFound{ .. } => Err(ContractError::DidDocumentNotFound(e)),
                _ => Err(ContractError::DidDocumentError(e)),
            },
        };
        let sender: Controller = ctx.info.sender.to_string().into(); // Get sender's address as a string
        // let sender = Did::new_address(sender.as_str());
        if !did_doc.is_controlled_by(ctx.deps.storage, &self.did_docs, &sender)? {
            return Err(ContractError::Unauthorized);
        }

        let r = self.did_docs.save(ctx.deps.storage, new_did_doc.id.to_string(), &new_did_doc);
        match r {
            Ok(_) => Ok(Response::default()),
            Err(e) => Err(ContractError::DidDocumentError(e))
        }
    }

    #[sv::msg(exec)]
    pub fn add_controller(&self, ctx: ExecCtx, did: String, controller: Controller) -> Result<Response, ContractError> {
        let did_doc = self.did_docs.load(ctx.deps.storage, did);
        let mut did_doc = match did_doc {
            Ok(did_document) => did_document,
            Err(e) => return match e {
                StdError::NotFound{ .. } => Err(ContractError::DidDocumentNotFound(e)),
                _ => Err(ContractError::DidDocumentError(e)),
            },
        };
        let sender: Controller = ctx.info.sender.to_string().into(); // Get sender's address as a string
        // let sender = Did::new_address(sender.as_str());
        if !did_doc.is_controlled_by(ctx.deps.storage, &self.did_docs, &sender)? {
            return Err(ContractError::Unauthorized);
        }

        if did_doc.has_controller(&controller) {
            return Err(ContractError::DidDocumentControllerAlreadyExists);
        }

        did_doc.controller.push(controller);

        let r = self.did_docs.save(ctx.deps.storage, did_doc.id.to_string(), &did_doc);
        match r {
            Ok(_) => Ok(Response::default()),
            Err(e) => Err(ContractError::DidDocumentError(e))
        }
    }

    #[sv::msg(exec)]
    pub fn delete_controller(&self, ctx: ExecCtx, did: String, controller: Controller) -> Result<Response, ContractError> {
        let did_doc = self.did_docs.load(ctx.deps.storage, did);
        let mut did_doc = match did_doc {
            Ok(did_document) => did_document,
            Err(e) => return match e {
                StdError::NotFound{ .. } => Err(ContractError::DidDocumentNotFound(e)),
                _ => Err(ContractError::DidDocumentError(e)),
            },
        };
        let sender: Controller = ctx.info.sender.to_string().into(); // Get sender's address as a string
        // let sender = Did::new_address(sender.as_str());
        if !did_doc.is_controlled_by(ctx.deps.storage, &self.did_docs, &sender)? {
            return Err(ContractError::Unauthorized);
        }

        if !did_doc.has_controller(&controller) {
            return Err(ContractError::DidDocumentControllerNotExists);
        }

        did_doc.controller.retain(|s| *s != controller);

        let r = self.did_docs.save(ctx.deps.storage, did_doc.id.to_string(), &did_doc);
        match r {
            Ok(_) => Ok(Response::default()),
            Err(e) => Err(ContractError::DidDocumentError(e))
        }
    }

    pub fn add_service(&self, ctx: ExecCtx, did: String, service: Service) -> Result<Response, ContractError> {
        let did_doc = self.did_docs.load(ctx.deps.storage, did);
        let mut did_doc = match did_doc {
            Ok(did_document) => did_document,
            Err(e) => return match e {
                StdError::NotFound{ .. } => Err(ContractError::DidDocumentNotFound(e)),
                _ => Err(ContractError::DidDocumentError(e)),
            },
        };
        let sender: Controller = ctx.info.sender.to_string().into(); // Get sender's address as a string
        // let sender = Did::new_address(sender.as_str());
        if !did_doc.is_controlled_by(ctx.deps.storage, &self.did_docs, &sender)? {
            return Err(ContractError::Unauthorized);
        }

        if did_doc.has_service(&service.id) {
            return Err(ContractError::DidDocumentServiceAlreadyExists);
        }

        did_doc.service.push(service);

        let r = self.did_docs.save(ctx.deps.storage, did_doc.id.to_string(), &did_doc);
        match r {
            Ok(_) => Ok(Response::default()),
            Err(e) => Err(ContractError::DidDocumentError(e))
        }
    }

    pub fn delete_service(&self, ctx: ExecCtx, did: String, service_did: Did) -> Result<Response, ContractError> {
        let did_doc = self.did_docs.load(ctx.deps.storage, did);
        let mut did_doc = match did_doc {
            Ok(did_document) => did_document,
            Err(e) => return match e {
                StdError::NotFound{ .. } => Err(ContractError::DidDocumentNotFound(e)),
                _ => Err(ContractError::DidDocumentError(e)),
            },
        };
        let sender: Controller = ctx.info.sender.to_string().into(); // Get sender's address as a string
        // let sender = Did::new_address(sender.as_str());
        if !did_doc.is_controlled_by(ctx.deps.storage, &self.did_docs, &sender)? {
            return Err(ContractError::Unauthorized);
        }

        if !did_doc.has_service(&service_did) {
            return Err(ContractError::DidDocumentServiceNotExists);
        }

        did_doc.service.retain(|s| s.id != service_did);

        let r = self.did_docs.save(ctx.deps.storage, did_doc.id.to_string(), &did_doc);
        match r {
            Ok(_) => Ok(Response::default()),
            Err(e) => Err(ContractError::DidDocumentError(e))
        }
    }

    #[sv::msg(exec)]
    pub fn delete_did_document(&self, ctx: ExecCtx, did: String) -> Result<Response, ContractError> {
         // Load the DID document from storage
        let did_doc = self.did_docs.load(ctx.deps.storage, (&did).clone());
        let did_doc = match did_doc {
            Ok(did_document) => did_document,
            Err(e) => return match e {
                StdError::NotFound{ .. } => Err(ContractError::DidDocumentNotFound(e)),
                _ => Err(ContractError::DidDocumentError(e)),
            },
        };

        // Ensure the sender is the controller
        let sender: Controller = ctx.info.sender.to_string().into(); // Get sender's address as a string
        // let sender = Did::new_address(sender.as_str());
        
        if did_doc.is_controlled_by(ctx.deps.storage, &self.did_docs, &sender)? {
            // If sender is the controller, remove the DID document
            self.did_docs.remove(ctx.deps.storage, did);
            Ok(Response::default()) // TODO add some informations
        } else {
            // Return an error if the sender is not the controller
            Err(ContractError::Unauthorized)
        }
    }
    
}

#[cfg(test)]
mod tests {
    use sylvia::cw_multi_test::IntoAddr;
    use sylvia::multitest::App;

    use crate::{contract::sv::mt::{CodeId, DidContractProxy}, state::{Did, DidDocument, Service}};

    #[test]
    fn get_document_not_found() {
        let app = App::default();
        let code_id = CodeId::store_code(&app);
    
        let owner = "owner".into_addr();
    
        let contract = code_id.instantiate().call(&owner).unwrap();
    
        let did = "did";
        let no_did = contract.get_did_document(did.to_string());
        assert!(no_did.is_err(), "Expected Err, but got an Ok");
        assert_eq!("Generic error: Querier contract error: Did document not found", no_did.err().unwrap().to_string());
    }

    #[test]
    fn create_and_get_document() {
        let app = App::default();
        let code_id = CodeId::store_code(&app);
    
        let owner = "owner".into_addr();
    
        let contract = code_id.instantiate().call(&owner).unwrap();
    
        // let did_owner = "did_owner";
        let did = "new_did";
        let new_did_doc = DidDocument{
            id: Did::new(did),
            controller: vec![owner.to_string().into()],
            service: vec![Service{
                a_type: "".to_string(),
                id: Did::new("dfdsfs"),
                service_endpoint: "dfdsfs".to_string()
            }]
        };
        let result = contract.create_did_document(new_did_doc.clone()).call(&owner);
        assert!(result.is_ok(), "Expected Ok, but got an Err");

        let did_document = contract.get_did_document(did.to_string()).unwrap();
        assert_eq!(new_did_doc.clone(), did_document.clone());
    }

    #[test]
    fn is_did_controller() {
        let app = App::default();
        let code_id = CodeId::store_code(&app);
    
        let owner = "owner".into_addr();
    
        let contract = code_id.instantiate().call(&owner).unwrap();
    
        // let did_owner = "did_owner";
        let did_simple = "didc4e:c4e:did_simple";
        let did_doc_simple = DidDocument{
            id: Did::new(did_simple),
            controller: vec![owner.to_string().into()],
            service: vec![Service{
                a_type: "".to_string(),
                id: Did::new("dfdsfs"),
                service_endpoint: "dfdsfs".to_string()
            }]
        };
        
        let result = contract.create_did_document(did_doc_simple.clone()).call(&owner);
        assert!(result.is_ok(), "Expected Ok, but got an Err");

        let is_controller = contract.is_did_controller(did_simple.to_string(), owner.to_string().into());
        assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
        assert!(is_controller.unwrap(), "Expected true, but got false");

        let is_controller = contract.is_did_controller(did_simple.to_string(), "unknown".to_string().into());
        assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
        assert!(!is_controller.unwrap(), "Expected false, but got true");

        let is_controller = contract.is_did_controller(did_simple.to_string(), "didc4e:c4e:unknown".to_string().into());
        assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
        assert!(!is_controller.unwrap(), "Expected false, but got true");

        let did_controlled_by_itself = "didc4e:c4e:did_controlled_by_himself";
        let did_doc_controlled_by_itself = DidDocument{
            id: Did::new(did_controlled_by_itself),
            controller: vec![did_controlled_by_itself.to_string().into()],
            service: vec![Service{
                a_type: "".to_string(),
                id: Did::new("dfdsfs"),
                service_endpoint: "dfdsfs".to_string()
            }]
        };

        let result = contract.create_did_document(did_doc_controlled_by_itself.clone()).call(&owner);
        assert!(result.is_ok(), "Expected Ok, but got an Err");

        let is_controller = contract.is_did_controller(did_controlled_by_itself.to_string(), did_controlled_by_itself.to_string().into());
        assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
        assert!(is_controller.unwrap(), "Expected true, but got false");

        let is_controller = contract.is_did_controller(did_controlled_by_itself.to_string(), "didc4e:c4e:unknown".to_string().into());
        assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
        assert!(!is_controller.unwrap(), "Expected false, but got true");
        
        let did_looped_1 = "didc4e:c4e:did_looped_1";
        let did_looped_2 = "didc4e:c4e:did_looped_2";
        let did_doc_looped_1 = DidDocument{
            id: Did::new(did_looped_1),
            controller: vec![did_looped_2.to_string().into()],
            service: vec![Service{
                a_type: "".to_string(),
                id: Did::new("dfdsfs"),
                service_endpoint: "dfdsfs".to_string()
            }]
        };

        let did_doc_looped_2 = DidDocument{
            id: Did::new(did_looped_2),
            controller: vec![did_looped_1.to_string().into()],
            service: vec![Service{
                a_type: "".to_string(),
                id: Did::new("dfdsfs"),
                service_endpoint: "dfdsfs".to_string()
            }]
        };

        let result = contract.create_did_document(did_doc_looped_1.clone()).call(&owner);
        assert!(result.is_ok(), "Expected Ok, but got an Err");

        let result = contract.create_did_document(did_doc_looped_2.clone()).call(&owner);
        assert!(result.is_ok(), "Expected Ok, but got an Err");

        let is_controller = contract.is_did_controller(did_looped_1.to_string(), "didc4e:c4e:unknown".to_string().into());
        assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
        assert!(!is_controller.unwrap(), "Expected false, but got true");

        let is_controller = contract.is_did_controller(did_looped_1.to_string(), did_looped_2.to_string().into());
        assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
        assert!(is_controller.unwrap(), "Expected true, but got false");

        let is_controller = contract.is_did_controller(did_looped_1.to_string(), did_looped_1.to_string().into());
        assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
        assert!(is_controller.unwrap(), "Expected true, but got false");

        let did_controlled_by_simple = "didc4e:c4e:did_controlled_by_simple";
        let did_doc_controlled_by_simple = DidDocument{
            id: Did::new(did_controlled_by_simple),
            controller: vec![did_simple.to_string().into()],
            service: vec![Service{
                a_type: "".to_string(),
                id: Did::new("dfdsfs"),
                service_endpoint: "dfdsfs".to_string()
            }]
        };

        let result = contract.create_did_document(did_doc_controlled_by_simple.clone()).call(&owner);
        assert!(result.is_ok(), "Expected Ok, but got an Err");

        let is_controller = contract.is_did_controller(did_controlled_by_simple.to_string(), "didc4e:c4e:unknown".to_string().into());
        assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
        assert!(!is_controller.unwrap(), "Expected false, but got true");

        let is_controller = contract.is_did_controller(did_controlled_by_simple.to_string(), did_simple.to_string().into());
        assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
        assert!(is_controller.unwrap(), "Expected true, but got false");

        let is_controller = contract.is_did_controller(did_controlled_by_simple.to_string(), owner.to_string().into());
        assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
        assert!(is_controller.unwrap(), "Expected true, but got false");

    }

    #[test]
    fn replacing_document() {
        let app = App::default();
        let code_id = CodeId::store_code(&app);
    
        let owner = "owner".into_addr();
    
        let contract = code_id.instantiate().call(&owner).unwrap();
    
        // let did_owner = "did_owner";
        let did = "new_did";
        let mut new_did_doc = DidDocument{
            id: Did::new(did),
            controller: vec![owner.to_string().into()],
            service: vec![Service{
                a_type: "".to_string(),
                id: Did::new("dfdsfs"),
                service_endpoint: "dfdsfs".to_string()
            }]
        };
        let mut result = contract.create_did_document(new_did_doc.clone()).call(&owner);
        assert!(result.is_ok(), "Expected Ok, but got an Err");

        new_did_doc = DidDocument{
            id: Did::new(did),
            controller: vec![owner.to_string().into()],
            service: vec![Service{
                a_type: "".to_string(),
                id: Did::new("AAAA"),
                service_endpoint: "BBBBB".to_string()
            }]
        };

        result = contract.create_did_document(new_did_doc.clone()).call(&owner);
        assert!(result.is_err(), "Expected Err, but got an Ok");
        assert_eq!("Did document already exists", result.err().unwrap().to_string());
    }

    #[test]
    fn delete_did_document_not_found() {
        let app = App::default();
        let code_id = CodeId::store_code(&app);
    
        let owner = "owner".into_addr();
    
        let contract = code_id.instantiate().call(&owner).unwrap();
    
        let did = "did";
        let no_did = contract.delete_did_document(did.to_string()).call(&owner);
        assert!(no_did.is_err(), "Expected Err, but got an Ok");
        assert_eq!("Did document not found", no_did.err().unwrap().to_string());
    }


    #[test]
    fn delete_did_document() {
        let app = App::default();
        let code_id = CodeId::store_code(&app);
    
        // let did_owner = "did_owner";
        let owner_addr = "did_owner".into_addr();
    
        let contract = code_id.instantiate().call(&owner_addr).unwrap();
    
        // let did_owner = "did_owner";
        let did = "new_did";
        let new_did_doc = DidDocument{
            id: Did::new(did),
            controller: vec![owner_addr.to_string().into()],
            service: vec![Service{
                a_type: "".to_string(),
                id: Did::new("dfdsfs"),
                service_endpoint: "dfdsfs".to_string()
            }]
        };
        let result = contract.create_did_document(new_did_doc.clone()).call(&owner_addr);
        assert!(result.is_ok(), "Expected Ok, but got an Err");

        let did_document = contract.get_did_document(did.to_string()).unwrap();
        assert_eq!(new_did_doc.clone(), did_document.clone());

        let result = contract.delete_did_document(did.to_string()).call(&owner_addr);
        assert!(result.is_ok(), "Expected Ok, but got an Err");

        let result = contract.get_did_document(did.to_string());
        assert!(result.is_err(), "Expected Err, but got an Ok");
        assert_eq!("Generic error: Querier contract error: Did document not found", result.err().unwrap().to_string());
    }

    #[test]
    fn delete_did_document_wrong_owner() {
        let app = App::default();
        let code_id = CodeId::store_code(&app);
    
        // let did_owner = "did_owner";
        let owner_addr = "did_owner".into_addr();
        let wrong_owner_addr = "wrong_did_owner".into_addr();

        let contract = code_id.instantiate().call(&owner_addr).unwrap();
    
        // let did_owner = "did_owner";
        let did = "new_did";
        let new_did_doc = DidDocument{
            id: Did::new(did),
            controller: vec![owner_addr.to_string().into()],
            service: vec![Service{
                a_type: "".to_string(),
                id: Did::new("dfdsfs"),
                service_endpoint: "dfdsfs".to_string()
            }]
        };
        let result = contract.create_did_document(new_did_doc.clone()).call(&owner_addr);
        assert!(result.is_ok(), "Expected Ok, but got an Err");

        let did_document = contract.get_did_document(did.to_string()).unwrap();
        assert_eq!(new_did_doc.clone(), did_document.clone());

        let result = contract.delete_did_document(did.to_string()).call(&wrong_owner_addr);
        assert!(result.is_err(), "Expected Err, but got an Ok");
        assert_eq!("Unauthorized", result.err().unwrap().to_string());

    }
}
