use std::convert::TryFrom;

use num::Zero;
use thiserror::Error;

use crate::core::domain::OrderBook;
use crate::core::order::LimitOrder;
use crate::core::orderbook::Book;
use crate::core::{OrderRequest, OrderRequestError};
use crate::{Acknowledgment, BookTop, LogTrait};

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("market order unsupported")]
    MarketUnsupported,
    #[error(transparent)]
    InboundOrderError(#[from] OrderRequestError),
    #[error(transparent)]
    ReportingError(#[from] std::sync::mpsc::SendError<Box<dyn LogTrait>>),
}

pub struct Engine {
    orderbook: Book,
    log_sender: std::sync::mpsc::Sender<Box<dyn LogTrait>>,
}

impl Engine {
    pub fn new(log_sender: std::sync::mpsc::Sender<Box<dyn LogTrait>>) -> Self {
        Self {
            orderbook: Book::new(),
            log_sender,
        }
    }

    pub fn process(&mut self, incoming_order: OrderRequest) -> Result<(), EngineError> {
        match incoming_order {
            OrderRequest::Create { price, .. } => {
                if price.is_zero() {
                    Err(EngineError::MarketUnsupported)?;
                }

                let order = LimitOrder::try_from(incoming_order)?;
                if let Ok((r, accepted)) = self.orderbook.matching(order) {
                    self.log_sender.send(r)?;
                    if accepted {
                        let (ask_volume, bib_volume) = self.orderbook.volume();
                        let (side, qty, price) = match self.orderbook.peek_top_of_book() {
                            (Some(ask_price), Some(bid_price)) => {
                                if ask_price > bid_price {
                                    ("S", ask_volume, ask_price)
                                } else {
                                    ("B", bib_volume, bid_price)
                                }
                            }
                            (Some(ask_price), None) => ("S", ask_volume, ask_price),
                            (None, Some(bid_price)) => ("B", bib_volume, bid_price),
                            _ => ("-", 0, 0),
                        };
                        self.log_sender.send(Box::new(BookTop {
                            label: "B".to_owned(),
                            side: side.to_string(),
                            price,
                            total_qty: qty,
                        }))?;
                    }
                }
            }
            OrderRequest::Cancel { user_order_id, .. } => {
                if let Some(canceled_order) = self.orderbook.cancel(&user_order_id) {
                    self.log_sender.send(Box::new(Acknowledgment {
                        label: "A".to_owned(),
                        user_id: canceled_order.user_id,
                        user_order_id: canceled_order.order_id,
                    }))?;
                }
            }
            OrderRequest::FlushBook => {
                self.orderbook.flush();
            }
        };

        Ok(())
    }
}
