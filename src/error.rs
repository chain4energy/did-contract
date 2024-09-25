use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Did document not found")]
    DidDocumentNotFound(StdError),

    #[error("Did document error")]
    DidDocumentError(StdError),

    #[error("Did document already exists")]
    DidDocumentAlreadyExists,

    #[error("Did document - wrong owner")]
    DidDocumentWrongOwner,

    #[error("Did document controller already existsr")]
    DidDocumentControllerAlreadyExists,

    #[error("Did document controller not existsr")]
    DidDocumentControllerNotExists,

    #[error("Did document service already existsr")]
    DidDocumentServiceAlreadyExists,

    #[error("Did document service not existsr")]
    DidDocumentServiceNotExists,
}