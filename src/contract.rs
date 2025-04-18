use cosmwasm_std::{Deps, Event, Order, Response, StdError, StdResult, Storage};
// use cw_storage_plus::Item;
use crate::error::ContractError;
use crate::multiset::MultiSet;
use crate::state::{self, Controller, Controllers, Did, DidDocument, Service, ToEventData};
use cw_storage_plus::{Bound, Map};
use sylvia::contract;
use sylvia::ctx::{ExecCtx, InstantiateCtx, QueryCtx};
pub struct DidContract {
    pub did_docs: Map<String, DidDocument>,
    pub controllers: MultiSet, // TODO optimize indexing on controllers
}

#[cfg(not(feature = "library"))]
use sylvia::entry_points;
// TODO update error handling
// TODO responses for msgs
// TODO response Events

// #[entry_points]

#[cfg_attr(not(feature = "library"), entry_points)]
#[contract]
// #[cfg_attr(not(feature = "library"), contract)]
#[sv::error(ContractError)]
impl DidContract {
    pub const fn new() -> Self {
        Self {
            did_docs: Map::new("dids"),
            controllers: MultiSet::new("controllers"),
        }
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(&self, _ctx: InstantiateCtx) -> StdResult<Response> {
        Ok(Response::default())
    }

    #[sv::msg(query)]
    pub fn get_did_document(&self, ctx: QueryCtx, did: Did) -> Result<DidDocument, ContractError> {
        did.ensure_valid()?;
        self.get_did_doc(ctx.deps.storage, &did.to_string())
    }

    #[sv::msg(query)]
    pub fn is_did_controller(
        &self,
        ctx: QueryCtx,
        did: Did,
        controller: Controller,
    ) -> Result<bool, ContractError> {
        did.ensure_valid()?;
        controller.ensure_valid(ctx.deps.api)?;
        let doc = self.get_did_doc(ctx.deps.storage, did.value())?;
        doc.is_controlled_by(ctx.deps.storage, &self.did_docs, &controller)
    }

    #[sv::msg(query)]
    pub fn is_controller_of(
        &self,
        ctx: QueryCtx,
        controllers: Vec<Controller>,
        controller: Controller,
    ) -> Result<bool, ContractError> {
        controller.ensure_valid(ctx.deps.api)?;
        for c in &controllers {
            c.ensure_valid(ctx.deps.api)?;
        }
        state::is_controller_of(ctx.deps.storage, &self.did_docs, &controllers, &controller)
    }

    #[sv::msg(query)]
    pub fn do_controllers_exist(
        &self,
        ctx: QueryCtx,
        controllers: Vec<Controller>,
    ) -> Result<bool, ContractError> {
        match controllers.ensure_exist(ctx.deps.storage, &self.did_docs) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    #[sv::msg(query)]
    pub fn does_controller_exist(
        &self,
        ctx: QueryCtx,
        controller: Controller,
    ) -> Result<bool, ContractError> {
        match controller.ensure_exist(ctx.deps.storage, &self.did_docs) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    // TODO ??????
    // #[sv::msg(query)]
    // pub fn authotize(&self, ctx: QueryCtx, controllers: Vec<Controller>, controller: Controller) -> Result<bool, ContractError> {
    //     controller.ensure_valid(ctx.deps.api)?;
    //     for c in &controllers {
    //         c.ensure_valid(ctx.deps.api)?;
    //     }
    //     state::is_controller_of(ctx.deps.storage, &self.did_docs, &controllers, &controller)
    // }

    #[sv::msg(query)]
    pub fn get_controlled_dids(
        &self,
        ctx: QueryCtx,
        controller: Controller,
        limit: Option<usize>,
        start_after: Option<String>,
    ) -> Result<Vec<Did>, ContractError> {
        controller.ensure_valid(ctx.deps.api)?;
        let res =
            self.get_controlled_dids_strings(ctx.deps, controller.value(), limit, start_after)?;
        let dids = res.into_iter().map(|s| Did::new(&s)).collect();
        Ok(dids)
    }

    #[sv::msg(query)]
    pub fn get_controlled_did_documents(
        &self,
        ctx: QueryCtx,
        controller: Controller,
        limit: Option<usize>,
        start_after: Option<String>,
    ) -> Result<Vec<DidDocument>, ContractError> {
        controller.ensure_valid(ctx.deps.api)?;
        let res =
            self.get_controlled_dids_strings(ctx.deps, controller.value(), limit, start_after)?;
        let mut docs: Vec<DidDocument> = Vec::new();
        for did in &res {
            let doc = self.get_did_doc(ctx.deps.storage, did)?;
            docs.push(doc);
        }
        Ok(docs)
    }

    #[sv::msg(exec)]
    pub fn create_did_document(
        &self,
        ctx: ExecCtx,
        did_doc: DidDocument,
    ) -> Result<Response, ContractError> {
        did_doc.ensure_valid(ctx.deps.api)?;
        if self
            .did_docs
            .has(ctx.deps.storage, did_doc.id.value().to_string())
        {
            return Err(ContractError::DidDocumentAlreadyExists);
        }
        let mut new_doc = did_doc;
        if !new_doc.has_any_controller() {
            // new_doc.controller.mut_controllers().push(Controller::new(&ctx.info.sender.to_string()));
            new_doc
                .controller
                .push(Controller::new(&ctx.info.sender.to_string()));
        }
        new_doc.ensure_not_self_controlled()?;
        new_doc.ensure_controllers_exist(ctx.deps.storage, &self.did_docs)?;
        new_doc.ensure_signability(ctx.deps.storage, &self.did_docs)?;

        self.did_docs.save(ctx.deps.storage, new_doc.id.to_string(), &new_doc)
            .map_err(|e| ContractError::DidDocumentError(e))?;

        self.index_controllers(ctx.deps.storage, &new_doc)?;
        let mut response = Response::default();

        let mut event = Event::new("create_did_document")
            .add_attribute("did", new_doc.id.to_string())
            .add_attribute("controllers", new_doc.controller.to_event_data());

        if new_doc.service.len() > 0 {
            event = event.add_attribute("services", new_doc.service.to_event_data());
        }

        response = response.add_event(event);

        Ok(response)
    }

    #[sv::msg(exec)]
    pub fn update_did_document(
        &self,
        ctx: ExecCtx,
        new_did_doc: DidDocument,
    ) -> Result<Response, ContractError> {
        new_did_doc.ensure_valid(ctx.deps.api)?;
        new_did_doc.ensure_controller()?;
        new_did_doc.ensure_not_self_controlled()?;
        let did_doc = self.get_did_doc(ctx.deps.storage, new_did_doc.id.value())?;
        let sender: Controller = ctx.info.sender.to_string().into(); // Get sender's address as a string
        did_doc.authorize(ctx.deps.storage, &self.did_docs, &sender)?;

        new_did_doc.ensure_controllers_exist(ctx.deps.storage, &self.did_docs)?;
        new_did_doc.ensure_signability(ctx.deps.storage, &self.did_docs)?; // TODO maybe optimoze by joining with ensure_controllers_exist

        let r = self
            .did_docs
            .save(ctx.deps.storage, new_did_doc.id.to_string(), &new_did_doc);
        match r {
            Ok(_) => {
                self.unindex_controllers(ctx.deps.storage, &did_doc);
                self.index_controllers(ctx.deps.storage, &new_did_doc)?;
                Ok(Response::default())
            }
            Err(e) => Err(ContractError::DidDocumentError(e)),
        }
    }

    #[sv::msg(exec)]
    pub fn add_controller(
        &self,
        ctx: ExecCtx,
        did: Did,
        controller: Controller,
    ) -> Result<Response, ContractError> {
        did.ensure_valid()?;
        controller.ensure_valid(ctx.deps.api)?;
        let mut did_doc: DidDocument = self.get_did_doc(ctx.deps.storage, did.value())?;
        let sender: Controller = ctx.info.sender.to_string().into(); // Get sender's address as a string
        did_doc.authorize(ctx.deps.storage, &self.did_docs, &sender)?;

        if did_doc.has_controller(&controller) {
            return Err(ContractError::DidDocumentControllerAlreadyExists);
        }

        did_doc.controller.push(controller.clone());
        did_doc.ensure_not_self_controlled()?;

        controller.ensure_exist(ctx.deps.storage, &self.did_docs)?;

        let r = self
            .did_docs
            .save(ctx.deps.storage, did_doc.id.to_string(), &did_doc);
        match r {
            Ok(_) => {
                self.index_controller(ctx.deps.storage, &did, &controller)?;
                Ok(Response::default())
            }
            Err(e) => Err(ContractError::DidDocumentError(e)),
        }
    }

    #[sv::msg(exec)]
    pub fn delete_controller(
        &self,
        ctx: ExecCtx,
        did: Did,
        controller: Controller,
    ) -> Result<Response, ContractError> {
        did.ensure_valid()?;
        controller.ensure_valid(ctx.deps.api)?;
        let mut did_doc: DidDocument = self.get_did_doc(ctx.deps.storage, did.value())?;
        let sender: Controller = ctx.info.sender.to_string().into(); // Get sender's address as a string
        did_doc.authorize(ctx.deps.storage, &self.did_docs, &sender)?;

        if !did_doc.has_controller(&controller) {
            return Err(ContractError::DidDocumentControllerNotExists);
        }

        // did_doc.controller.mut_controllers().retain(|s| *s != controller);
        did_doc.controller.retain(|s| *s != controller);
        did_doc.ensure_controller()?;
        did_doc.ensure_signability(ctx.deps.storage, &self.did_docs)?;

        let r = self
            .did_docs
            .save(ctx.deps.storage, did_doc.id.to_string(), &did_doc);
        match r {
            Ok(_) => {
                self.unindex_controller(ctx.deps.storage, &did, &controller);
                Ok(Response::default())
            }
            Err(e) => Err(ContractError::DidDocumentError(e)),
        }
    }

    pub fn add_service(
        &self,
        ctx: ExecCtx,
        did: Did,
        service: Service,
    ) -> Result<Response, ContractError> {
        did.ensure_valid()?;
        service.ensure_valid()?;
        let mut did_doc: DidDocument = self.get_did_doc(ctx.deps.storage, did.value())?;

        let sender: Controller = ctx.info.sender.to_string().into();
        did_doc.authorize(ctx.deps.storage, &self.did_docs, &sender)?;

        if did_doc.has_service(&service.id) {
            return Err(ContractError::DidDocumentServiceAlreadyExists);
        }

        did_doc.service.push(service);

        let r = self
            .did_docs
            .save(ctx.deps.storage, did_doc.id.to_string(), &did_doc);
        match r {
            Ok(_) => Ok(Response::default()),
            Err(e) => Err(ContractError::DidDocumentError(e)),
        }
    }

    pub fn delete_service(
        &self,
        ctx: ExecCtx,
        did: Did,
        service_did: Did,
    ) -> Result<Response, ContractError> {
        did.ensure_valid()?;
        service_did.ensure_valid()?;
        let mut did_doc: DidDocument = self.get_did_doc(ctx.deps.storage, did.value())?;

        let sender: Controller = ctx.info.sender.to_string().into();
        did_doc.authorize(ctx.deps.storage, &self.did_docs, &sender)?;

        if !did_doc.has_service(&service_did) {
            return Err(ContractError::DidDocumentServiceNotExists);
        }

        did_doc.service.retain(|s| s.id != service_did);

        let r = self
            .did_docs
            .save(ctx.deps.storage, did_doc.id.to_string(), &did_doc);
        match r {
            Ok(_) => Ok(Response::default()),
            Err(e) => Err(ContractError::DidDocumentError(e)),
        }
    }

    #[sv::msg(exec)]
    pub fn delete_did_document(&self, ctx: ExecCtx, did: Did) -> Result<Response, ContractError> {
        did.ensure_valid()?;
        let did_doc: DidDocument = self.get_did_doc(ctx.deps.storage, did.value())?;
        if !self.controllers.is_empty(ctx.deps.storage, did.value()) {
            return Err(ContractError::DidDocumentIsController());
        }
        let sender: Controller = ctx.info.sender.to_string().into(); // Get sender's address as a string

        did_doc.authorize(ctx.deps.storage, &self.did_docs, &sender)?;

        self.did_docs.remove(ctx.deps.storage, did.to_string());
        self.unindex_controllers(ctx.deps.storage, &did_doc);
        Ok(Response::default())
    }

    fn get_did_doc(&self, store: &dyn Storage, did: &str) -> Result<DidDocument, ContractError> {
        let did_doc_result = self.did_docs.load(store, did.into());
        match did_doc_result {
            Ok(did_document) => Ok(did_document),
            Err(e) => match e {
                StdError::NotFound { .. } => Err(ContractError::DidDocumentNotFound(e)),
                _ => Err(ContractError::DidDocumentError(e)),
            },
        }
    }

    fn index_controllers(
        &self,
        store: &mut dyn Storage,
        did_doc: &DidDocument,
    ) -> Result<(), ContractError> {
        // for c in did_doc.controller.controllers() {
        for c in &did_doc.controller {
            let r = self
                .controllers
                .save(store, &c.to_string(), &did_doc.id.to_string());
            if let Err(e) = r {
                return Err(ContractError::DidDocumentError(e));
            }
        }
        Ok(())
    }

    fn unindex_controllers(&self, store: &mut dyn Storage, did_doc: &DidDocument) {
        // for c in did_doc.controller.controllers() {
        for c in &did_doc.controller {
            self.controllers
                .remove(store, &c.to_string(), &did_doc.id.to_string());
        }
    }

    fn index_controller(
        &self,
        store: &mut dyn Storage,
        did: &Did,
        controller: &Controller,
    ) -> Result<(), ContractError> {
        let r = self
            .controllers
            .save(store, &controller.to_string(), &did.to_string());
        if let Err(e) = r {
            return Err(ContractError::DidDocumentError(e));
        }
        Ok(())
    }

    fn unindex_controller(&self, store: &mut dyn Storage, did: &Did, controller: &Controller) {
        self.controllers
            .remove(store, &controller.to_string(), &did.to_string());
    }

    fn get_controlled_dids_strings(
        &self,
        deps: Deps,
        controller: &str,
        limit: Option<usize>,
        start_after: Option<String>,
    ) -> Result<Vec<String>, ContractError> {
        const DEFAULT_LIMIT: usize = 50;
        const MAX_LIMIT: usize = 200;
        let mut limit = limit.unwrap_or(DEFAULT_LIMIT);
        limit = {
            if limit == 0 {
                DEFAULT_LIMIT
            } else {
                limit
            }
        }
        .min(MAX_LIMIT) as usize;

        let start = start_after.map(Bound::exclusive);

        let res: Result<Vec<_>, _> = self
            .controllers
            .get_values(deps.storage, controller, start, None, Order::Ascending)
            .take(limit)
            .collect();
        match res {
            Ok(dids) => Ok(dids),
            Err(e) => match e {
                StdError::NotFound { .. } => Err(ContractError::DidDocumentError(e)),
                _ => Err(ContractError::DidDocumentError(e)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use cw_storage_plus::Map;
    use sylvia::cw_multi_test::IntoAddr;
    use sylvia::multitest::App;

    use crate::{
        contract::sv::mt::{CodeId, DidContractProxy},
        state::{Did, DidDocument, Service, DID_PREFIX},
    };

    // TODO add tests for verfying Did and Controller formats in case of every method
    // TODO add authorization tests for every method
    // TODO add checking if all cotroller realations are indexed
    // TODO add chcecki if did doc can be rmoeved - if is controler of other did
    // TODO add tests chcecking for checking if ocntroller exists when setting in did doc - for each required method

    #[test]
    fn get_document_not_found() {
        let app = App::default();
        let code_id = CodeId::store_code(&app);

        let owner = "owner".into_addr();

        let contract = code_id.instantiate().call(&owner).unwrap();

        let did = "did";
        let no_did = contract.get_did_document(Did::new(did));
        assert!(no_did.is_err(), "Expected Err, but got an Ok");
        assert_eq!(
            "Generic error: Querier contract error: Did format error",
            no_did.err().unwrap().to_string()
        );

        let did = format!("{}{}", DID_PREFIX, "did");
        let no_did = contract.get_did_document(Did::new(&did));
        assert!(no_did.is_err(), "Expected Err, but got an Ok");
        assert_eq!(
            "Generic error: Querier contract error: Did document not found",
            no_did.err().unwrap().to_string()
        );
    }

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
        assert_eq!("Did format error", result.err().unwrap().to_string());

        let did = format!("{}{}", DID_PREFIX, "new_did");
        new_did_doc.id = Did::new(&did);

        let result = contract
            .create_did_document(new_did_doc.clone())
            .call(&owner);
        assert!(result.is_err(), "Expected Err, but got an Ok");
        assert_eq!("Did format error", result.err().unwrap().to_string());

        new_did_doc.service[0].id = Did::new(&format!("{}{}", DID_PREFIX, "ffffff"));

        let result = contract
            .create_did_document(new_did_doc.clone())
            .call(&owner);
        assert!(result.is_ok(), "Expected Ok, but got an Err");

        let did_document = contract.get_did_document(Did::new(&did)).unwrap();
        assert_eq!(new_did_doc.clone(), did_document.clone());
    }

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

        let is_controller =
            contract.is_did_controller(Did::new(&did_simple), owner.to_string().into());
        assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
        assert!(is_controller.unwrap(), "Expected true, but got false");

        let is_controller =
            contract.is_did_controller(Did::new(&did_simple), unknow_addr.to_string().into());
        assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
        assert!(!is_controller.unwrap(), "Expected false, but got true");

        let is_controller = contract.is_did_controller(Did::new(&did_simple), unknow_did.into());
        assert!(is_controller.is_ok(), "Expected Ok, but got an Err");
        assert!(!is_controller.unwrap(), "Expected false, but got true");

        let did_controlled_by_itself = "didc4e:c4e:did_controlled_by_himself";
        let did_doc_controlled_by_itself = DidDocument {
            id: Did::new(did_controlled_by_itself),
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
            Did::new(did_controlled_by_itself),
            did_controlled_by_itself.to_string().into(),
        );
        assert!(
            is_controller.is_ok(),
            "Expected Ok, but got an Err: {}",
            is_controller.unwrap_err()
        );
        assert!(is_controller.unwrap(), "Expected true, but got false");

        let is_controller =
            contract.is_did_controller(Did::new(did_controlled_by_itself), unknow_did.into());
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

        let is_controller = contract
            .is_did_controller(Did::new(did_controlled_by_simple), owner.to_string().into());
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
        let did = &format!("{}{}", DID_PREFIX, "new_did");
        let mut new_did_doc = DidDocument {
            id: Did::new(did),
            // controller: Controllers(vec![owner.to_string().into()]),
            controller: vec![owner.to_string().into()],
            service: vec![Service {
                a_type: "".to_string(),
                id: Did::new(&format!("{}{}", DID_PREFIX, "dfdsfs")),
                service_endpoint: "dfdsfs".to_string(),
            }],
        };
        let mut result = contract
            .create_did_document(new_did_doc.clone())
            .call(&owner);
        assert!(result.is_ok(), "Expected Ok, but got an Err");

        new_did_doc = DidDocument {
            id: Did::new(did),
            // controller: Controllers(vec![owner.to_string().into()]),
            controller: vec![owner.to_string().into()],
            service: vec![Service {
                a_type: "".to_string(),
                id: Did::new(&format!("{}{}", DID_PREFIX, "AAAA")),
                service_endpoint: "BBBBB".to_string(),
            }],
        };

        result = contract
            .create_did_document(new_did_doc.clone())
            .call(&owner);
        assert!(result.is_err(), "Expected Err, but got an Ok");
        assert_eq!(
            "Did document already exists",
            result.err().unwrap().to_string()
        );
    }

    #[test]
    fn delete_did_document_not_found() {
        let app = App::default();
        let code_id = CodeId::store_code(&app);

        let owner = "owner".into_addr();

        let contract = code_id.instantiate().call(&owner).unwrap();

        let did = &format!("{}{}", DID_PREFIX, "did");
        let no_did = contract.delete_did_document(Did::new(did)).call(&owner);
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
        let did = &format!("{}{}", DID_PREFIX, "new_did");
        let new_did_doc = DidDocument {
            id: Did::new(did),
            // controller: Controllers(vec![owner_addr.to_string().into()]),
            controller: vec![owner_addr.to_string().into()],
            service: vec![Service {
                a_type: "".to_string(),
                id: Did::new(&format!("{}{}", DID_PREFIX, "dfdsfs")),
                service_endpoint: "dfdsfs".to_string(),
            }],
        };
        let result = contract
            .create_did_document(new_did_doc.clone())
            .call(&owner_addr);
        assert!(result.is_ok(), "Expected Ok, but got an Err");

        let did_document = contract.get_did_document(Did::new(did)).unwrap();
        assert_eq!(new_did_doc.clone(), did_document.clone());

        let result = contract
            .delete_did_document(Did::new(did))
            .call(&owner_addr);
        assert!(result.is_ok(), "Expected Ok, but got an Err");

        let result = contract.get_did_document(Did::new(did));
        assert!(result.is_err(), "Expected Err, but got an Ok");
        assert_eq!(
            "Generic error: Querier contract error: Did document not found",
            result.err().unwrap().to_string()
        );
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
        let did = &format!("{}{}", DID_PREFIX, "new_did");
        let new_did_doc = DidDocument {
            id: Did::new(did),
            // controller: Controllers(vec![owner_addr.to_string().into()]),
            controller: vec![owner_addr.to_string().into()],
            service: vec![Service {
                a_type: "".to_string(),
                id: Did::new(&format!("{}{}", DID_PREFIX, "dfdsfs")),
                service_endpoint: "dfdsfs".to_string(),
            }],
        };
        let result = contract
            .create_did_document(new_did_doc.clone())
            .call(&owner_addr);
        assert!(result.is_ok(), "Expected Ok, but got an Err");

        let did_document = contract.get_did_document(Did::new(did)).unwrap();
        assert_eq!(new_did_doc.clone(), did_document.clone());

        let result = contract
            .delete_did_document(Did::new(did))
            .call(&wrong_owner_addr);
        assert!(result.is_err(), "Expected Err, but got an Ok");
        assert_eq!("Unauthorized", result.err().unwrap().to_string());
    }
}
