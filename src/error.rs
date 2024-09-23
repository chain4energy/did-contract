use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("did document not found.")]
    DidDocumentNotFound,

    #[error("did document error.")]
    DidDocumentError,
}