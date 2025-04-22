use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Did format error: {0}")]
    DidFormatError(String),

    #[error("Controller format error: {0}")]
    ControllerFormatError(String),

    #[error("Did document not found: {0}")]
    DidDocumentNotFound(StdError),

    #[error("Did controller not found: {0}")]
    DidControllerNotFound(String),

    #[error("Did document {0} has no controller")]
    DidDocumentNoController(String),

    #[error("Self controlled did document not allowed: {0}")]
    SelfControlledDidDocumentNotAllowed(String),

    #[error("Did document error")]
    DidDocumentError(StdError),

    #[error("Did document unsignable: {0}")]
    DidDocumentUnsignable(String),
    
    #[error("Did document is controller")]
    DidDocumentIsController(),

    #[error("Did document already exists: {0}")]
    DidDocumentAlreadyExists(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Did document controller already exists: {0}")]
    DidDocumentControllerAlreadyExists(String),

    #[error("Did document controller not exists: {0}")]
    DidDocumentControllerNotExists(String),

    #[error("Did document service already exists: {0}")]
    DidDocumentServiceAlreadyExists(String),

    #[error("Did document service does not exist")]
    DidDocumentServiceNotExists(String),

    #[error("Duplicated controller: {0}")]
    DuplicatedController(String),

    #[error("Service id format error: {0}")]
    ServiceIdFormatError(String),
}