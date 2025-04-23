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
        if controllers.is_empty() {
            return Err(ContractError::NoControllers);
        }
        for c in &controllers {
            c.ensure_valid(ctx.deps.api)?;
        }
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
        controller.ensure_valid(ctx.deps.api)?;
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
        did_doc.ensure_controllers_not_duplicated()?;
        did_doc.ensure_services_not_duplicated()?;
        if self
            .did_docs
            .has(ctx.deps.storage, did_doc.id.value().to_string())
        {
            return Err(ContractError::DidDocumentAlreadyExists(
                did_doc.id.value().to_string(),
            ));
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

        self.did_docs
            .save(ctx.deps.storage, new_doc.id.to_string(), &new_doc)
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
        new_did_doc.ensure_controllers_not_duplicated()?;
        new_did_doc.ensure_services_not_duplicated()?;
        new_did_doc.ensure_not_self_controlled()?;
        let did_doc = self.get_did_doc(ctx.deps.storage, new_did_doc.id.value())?;
        let sender: Controller = ctx.info.sender.to_string().into();
        did_doc.authorize(ctx.deps.storage, &self.did_docs, &sender)?;

        new_did_doc.ensure_controllers_exist(ctx.deps.storage, &self.did_docs)?;
        new_did_doc.ensure_signability(ctx.deps.storage, &self.did_docs)?; // TODO maybe optimoze by joining with ensure_controllers_exist

        self.did_docs
            .save(ctx.deps.storage, new_did_doc.id.to_string(), &new_did_doc)
            .map_err(|e| ContractError::DidDocumentError(e))?;

        self.unindex_controllers(ctx.deps.storage, &did_doc);
        self.index_controllers(ctx.deps.storage, &new_did_doc)?;

        let mut response = Response::default();

        let mut event = Event::new("update_did_document")
            .add_attribute("did", new_did_doc.id.to_string())
            .add_attribute("old_controllers", did_doc.controller.to_event_data())
            .add_attribute("new_controllers", new_did_doc.controller.to_event_data());

        if did_doc.service.len() > 0 {
            event = event.add_attribute("old_services", did_doc.service.to_event_data());
        }
        if new_did_doc.service.len() > 0 {
            event = event.add_attribute("new_services", new_did_doc.service.to_event_data());
        }

        response = response.add_event(event);
        Ok(response)
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
            return Err(ContractError::DidDocumentControllerAlreadyExists(
                controller.to_string(),
            ));
        }

        did_doc.controller.push(controller.clone());
        did_doc.ensure_not_self_controlled()?;

        controller.ensure_exist(ctx.deps.storage, &self.did_docs)?;

        self.did_docs
            .save(ctx.deps.storage, did_doc.id.to_string(), &did_doc)
            .map_err(|e| ContractError::DidDocumentError(e))?;

        self.index_controller(ctx.deps.storage, &did, &controller)?;

        let mut response = Response::default();

        let event = Event::new("add_controller")
            .add_attribute("did", did.to_string())
            .add_attribute("new_controller", controller.to_string());
        response = response.add_event(event);
        Ok(response)
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
            return Err(ContractError::DidDocumentControllerNotExists(
                controller.to_string(),
            ));
        }

        // did_doc.controller.mut_controllers().retain(|s| *s != controller);
        did_doc.controller.retain(|s| *s != controller);
        did_doc.ensure_controller()?;
        did_doc.ensure_signability(ctx.deps.storage, &self.did_docs)?;

        self.did_docs
            .save(ctx.deps.storage, did_doc.id.to_string(), &did_doc)
            .map_err(|e| ContractError::DidDocumentError(e))?;

        self.unindex_controller(ctx.deps.storage, &did, &controller);

        let mut response = Response::default();

        let event = Event::new("delete_controller")
            .add_attribute("did", did.to_string())
            .add_attribute("old_controller", controller.to_string());
        response = response.add_event(event);
        Ok(response)
    }

    #[sv::msg(exec)]
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
            return Err(ContractError::DidDocumentServiceAlreadyExists(
                service.id.to_string(),
            ));
        }

        did_doc.service.push(service.clone());

        self.did_docs
            .save(ctx.deps.storage, did_doc.id.to_string(), &did_doc)
            .map_err(|e| ContractError::DidDocumentError(e))?;

        let mut response = Response::default();

        let event = Event::new("add_service")
            .add_attribute("did", did.to_string())
            .add_attribute("new_service", service.id.to_string());
        response = response.add_event(event);
        Ok(response)
    }

    #[sv::msg(exec)]
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
            return Err(ContractError::DidDocumentServiceNotExists(
                service_did.to_string(),
            ));
        }

        did_doc.service.retain(|s| s.id != service_did);

        self.did_docs
            .save(ctx.deps.storage, did_doc.id.to_string(), &did_doc)
            .map_err(|e| ContractError::DidDocumentError(e))?;

        let mut response = Response::default();

        let event = Event::new("delete_service")
            .add_attribute("did", did.to_string())
            .add_attribute("old_service", service_did);
        response = response.add_event(event);
        Ok(response)
    }

    #[sv::msg(exec)]
    pub fn delete_did_document(&self, ctx: ExecCtx, did: Did) -> Result<Response, ContractError> {
        did.ensure_valid()?;
        let did_doc: DidDocument = self.get_did_doc(ctx.deps.storage, did.value())?;
        if !self.controllers.is_empty(ctx.deps.storage, did.value()) {
            return Err(ContractError::DidDocumentIsController(did.to_string()));
        }
        let sender: Controller = ctx.info.sender.to_string().into(); // Get sender's address as a string

        did_doc.authorize(ctx.deps.storage, &self.did_docs, &sender)?;

        self.did_docs.remove(ctx.deps.storage, did.to_string());
        self.unindex_controllers(ctx.deps.storage, &did_doc);
        let mut response = Response::default();

        let event = Event::new("delete_did_document").add_attribute("did", did.to_string());
        response = response.add_event(event);
        Ok(response)
    }

    fn get_did_doc(&self, store: &dyn Storage, did: &str) -> Result<DidDocument, ContractError> {
        self.did_docs.load(store, did.into()).map_err(|e| match e {
            StdError::NotFound { .. } => ContractError::DidDocumentNotFound(e),
            _ => ContractError::DidDocumentError(e),
        })
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
        self
            .controllers
            .save(store, &controller.to_string(), &did.to_string()).map_err(|e| {
                ContractError::DidDocumentError(e)
            })

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
