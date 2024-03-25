use thiserror::Error;
mod depth;
pub mod domain;
mod engine;
mod matcher;
pub mod order;
mod orderbook;
mod trade;

pub use engine::{Engine, EngineError};
pub use order::{OrderRequest, Side};
pub use orderbook::Book;
pub use domain::OrderBook;

#[derive(Debug, Error)]
pub enum OrderRequestError {
    #[error("order type mismatch")]
    MismatchType,
    #[error("invalid order side `{0}`")]
    InvalidOrderSide(String),
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
