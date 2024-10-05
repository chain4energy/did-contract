use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Did format error")]
    DidFormatError(),

    #[error("Controller format error")]
    ControllerFormatError(),

    #[error("Did document not found")]
    DidDocumentNotFound(StdError),

    #[error("Did controller not found")]
    DidControllerNotFound(),

    #[error("Did document has no controller")]
    DidDocumentNoController(),

    #[error("Self controlled did not allowed")]
    SelfControlledDidDocumentNotAllowed(),

    #[error("Did document error")]
    DidDocumentError(StdError),

    #[error("Did document unsignable")]
    DidDocumentUnsignable(),
    
    #[error("Did document is controller")]
    DidDocumentIsController(),

    #[error("Did document already exists")]
    DidDocumentAlreadyExists,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Did document controller already existsr")]
    DidDocumentControllerAlreadyExists,

    #[error("Did document controller not existsr")]
    DidDocumentControllerNotExists,

    #[error("Did document service already existsr")]
    DidDocumentServiceAlreadyExists,

    #[error("Did document service not existsr")]
    DidDocumentServiceNotExists,
}