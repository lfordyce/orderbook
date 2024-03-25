use std::convert::TryFrom;
use std::ptr::read;

use num::Zero;
use thiserror::Error;

use orderbook::{Acknowledgment, BookTop, LogTrait};

use crate::core::engine::EngineError::MarketUnsupported;
use crate::core::instrument::{Matchers, Opposite, Order, OrderBook, SpreadOption, Trade};
use crate::core::order::LimitOrder;
use crate::core::orderbook::Book;
use crate::core::OrderRequest;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DefaultExchangeError {}

pub struct MatchingEngine;

impl Matchers for MatchingEngine {
    type Error = DefaultExchangeError;
    type Output = Box<dyn LogTrait>;

    fn matching<E>(
        exchange: &mut E,
        mut incoming_order: <E as OrderBook>::Order,
    ) -> Result<Self::Output, Self::Error>
    where
        E: OrderBook,
        <<E as OrderBook>::Order as Order>::Acknowledgment: 'static,
        // <E as OrderBook>::Order: TryFrom<<E as OrderBook>::IncomingOrder>,
    {
        while !incoming_order.is_closed() {
            let Some(mut top_order) = exchange.peek_mut(&incoming_order.side().opposite()) else {
                // Since there is no opposite order anymore, we can move on.
                break;
            };

            let Ok(_trade) = top_order.trade(&mut incoming_order) else {
                // Since incoming order is not matching to top order
                // anymore, we can also move on.
                break;
            };

            if top_order.is_closed() {
                let top_order_id = top_order.id();

                // We must explicity drop to reuse the `exchange`.
                drop(top_order);

                // As long as top order is completed, it can be safely removed
                // from orderbook.
                exchange
                    .remove(&top_order_id)
                    .expect("order should be `Some`");
            }
        }

        // let order_id = incoming_order.id();
        // let user_id =  incoming_order.user_id();
        let ack = incoming_order.ack(false);
        exchange.insert(incoming_order);

        Ok(Box::new(ack))
    }
}

pub struct Engine {
    orderbook: Book,
    log_sender: std::sync::mpsc::Sender<Box<dyn LogTrait>>,
}

impl Engine {
    pub fn new(_symbol: &str, log_sender: std::sync::mpsc::Sender<Box<dyn LogTrait>>) -> Self {
        Self {
            orderbook: Book::new(),
            log_sender,
        }
    }

    pub fn process(&mut self, incoming_order: OrderRequest) -> Result<(), EngineError> {
        match incoming_order {
            OrderRequest::Create { price, .. } => {
                if price.is_zero() {
                    Err(MarketUnsupported)?;
                }

                let order = LimitOrder::try_from(incoming_order).unwrap();
                if let Ok(r) = self.orderbook.matching(order) {
                    self.log_sender
                        .send(r)
                        .unwrap_or_else(|e| eprintln!("{}", e));
                }
                let (a, b) = self.orderbook.volume();
                let (side, qty, price) = match self.orderbook.spread_option() {
                    (Some(ask_price), Some(bid_price)) => {
                        if ask_price > bid_price {
                            ("S", a, ask_price)
                        } else {
                            ("B", b, bid_price)
                        }
                    }
                    (Some(ask_price), None) => {
                        ("S", a, ask_price)
                    }
                    (None, Some(bid_price)) => {
                        ("B", b, bid_price)
                    }
                    _ => ("NONE", a, 0),
                };
                self.log_sender
                    .send(Box::new(BookTop {
                        label: "B".to_string(),
                        side: side.to_string(),
                        values: vec![price, qty],
                    }))
                    .unwrap_or_else(|e| eprintln!("{}", e));

                // if let Some((ask_price, bid_price)) = self.orderbook.spread() {
                //     let (side, qty) = if ask_price > bid_price {
                //         ("S", a)
                //     } else {
                //         ("B", b)
                //     };
                //     self.log_sender
                //         .send(Box::new(BookTop {
                //             label: "B".to_string(),
                //             side: side.to_string(),
                //             values: vec![ask_price, qty],
                //         }))
                //         .unwrap_or_else(|e| eprintln!("{}", e));
                // }
            }
            OrderRequest::Cancel {
                user_id,
                user_order_id,
                ..
            } => {
                self.orderbook.remove(&user_order_id);
                self.log_sender
                    .send(Box::new(Acknowledgment {
                        label: "A".to_string(),
                        values: vec![user_id, user_order_id],
                    }))
                    .unwrap_or_else(|e| eprintln!("{}", e));
            }
            OrderRequest::FlushBook => self.orderbook = Book::new(),
        };

        Ok(())
    }

    #[inline]
    pub fn orderbook(&self) -> &Book {
        &self.orderbook
    }
}

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("market order unsupported")]
    MarketUnsupported,
}
