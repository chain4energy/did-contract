use cosmwasm_std::{Response, StdResult};
// use cw_storage_plus::Item;
use cw_storage_plus::Map;
use sylvia::{contract, entry_points};
use sylvia::types::{InstantiateCtx, QueryCtx, ExecCtx};



use crate::state::DidDocument;
use crate::state::Did;
pub struct DidContract {
    pub(crate) did_docs: Map<String, DidDocument>,
}

#[entry_points]
#[contract]
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
    pub fn get_did_document(&self, ctx: QueryCtx, did: String) -> StdResult<DidDocument> {
        // Load and return the current value of the counter
        let did_doc: DidDocument = self.did_docs.load(ctx.deps.storage, did)?;
        Ok(did_doc)
    }

    #[sv::msg(exec)]
    pub fn create_did_document(&self, ctx: ExecCtx, did_doc: DidDocument) -> StdResult<Response> {
        self.did_docs.save(ctx.deps.storage, did_doc.id.value().to_string(), &did_doc)?;
        // self.count
        //     .update(ctx.deps.storage, |count| -> StdResult<u32> {
        //         Ok(count + 1)
        //     })?;
        Ok(Response::default())
    }

    #[sv::msg(exec)]
    pub fn delete_did_document(&self, ctx: ExecCtx, did: String) -> StdResult<Response> {
         // Load the DID document from storage
    let did_doc = self.did_docs.may_load(ctx.deps.storage, (&did).clone())?;

    // Check if the DID document exists
    if let Some(did_doc) = did_doc {
        // Ensure the sender is the controller
        let sender = ctx.info.sender.to_string(); // Get sender's address as a string
        let sender_did = Did::new_address(sender.as_str());
        if did_doc.has_controller(&sender_did) {
            // If sender is the controller, remove the DID document
            self.did_docs.remove(ctx.deps.storage, did);
            Ok(Response::default())
        } else {
            // Return an error if the sender is not the controller
            Err(cosmwasm_std::StdError::generic_err("Unauthorized: Only the controller can delete this DID document"))
        }
    } else {
        // Return an error if the DID document does not exist
        Err(cosmwasm_std::StdError::not_found("DID Document"))
    }
    }
}

#[cfg(test)]
mod tests {
    use sylvia::cw_multi_test::IntoAddr;
    use sylvia::multitest::App;

    use crate::{contract::sv::mt::{CodeId, DidContractProxy}, state::{Did, DidDocument, Service}};

    #[test]
    fn instantiate() {
        let app = App::default();
        let code_id = CodeId::store_code(&app);
    
        let owner = "owner".into_addr();
    
        let contract = code_id.instantiate().call(&owner).unwrap();
    
        let did_owner = "did_owner";
        // let did_document = contract.get_did_document("did_owner".to_string()).unwrap();
        // assert_eq!(count, 42);

        let new_did_doc = DidDocument{
            id: Did::new("new_did"),
            controller: vec![Did::new(did_owner)],
            service: vec![Service{
                a_type: "".to_string(),
                id: Did::new("dfdsfs"),
                service_endpoint: "dfdsfs".to_string()
            }]
        };
        let result = contract.create_did_document(new_did_doc.clone()).call(&owner).unwrap();

        let did_document = contract.get_did_document("new_did".to_string()).unwrap();
        assert_eq!(new_did_doc.clone(), did_document.clone());


    }
}
