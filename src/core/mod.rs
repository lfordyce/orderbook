use thiserror::Error;
mod depth;
mod engine;
mod instrument;
mod matcher;
mod orderbook;
mod types;

mod order;
mod trade;

pub use order::{OrderRequest, Side};
pub use engine::Engine;

#[derive(Debug, Error)]
pub enum OrderRequestError {
    #[error("order type mismatch")]
    MismatchType,
}

#[derive(Debug, Error)]
pub enum OrderError {
    #[error("empty filling is not allowed")]
    NoFill,
    #[error("filling amount exceeds remaining amount")]
    Overfill,
}

#[derive(Debug, Error)]
pub enum TradeError {
    #[error(transparent)]
    Price(#[from] PriceError),
    #[error(transparent)]
    Side(#[from] SideError),
    #[error(transparent)]
    Status(#[from] StatusError),
}

#[derive(Debug, Error)]
pub enum PriceError {
    #[error("prices do not match each other")]
    Incompatible,
    #[error("limit price is a must")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum SideError {
    #[error("taker and maker must be at opposite sides")]
    Conflict,
}

#[derive(Debug, Error)]
pub enum StatusError {
    #[error("taker and maker cannot be closed")]
    Closed,
}