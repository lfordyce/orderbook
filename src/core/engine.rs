use crate::core::engine::EngineError::MarketUnsupported;
use crate::core::instrument::{Matchers, Opposite, Order, OrderBook, Trade};
use crate::core::order::LimitOrder;
use crate::core::orderbook::Book;
use crate::core::OrderRequest;
use csv::StringRecord;
use num::Zero;
use std::convert::TryFrom;
use thiserror::Error;
use orderbook::{LogTrait, Row};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DefaultExchangeError {}

pub struct MatchingEngine;

impl Matchers for MatchingEngine {
    type Error = DefaultExchangeError;
    type Output = ();

    fn matching<E>(
        exchange: &mut E,
        mut incoming_order: <E as OrderBook>::Order,
    ) -> Result<(), DefaultExchangeError>
    where
        E: OrderBook,
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

        exchange.insert(incoming_order);

        Ok(())
    }
}

pub struct Engine
{
    orderbook: Book,
    log_sender: std::sync::mpsc::Sender<Box<dyn LogTrait>>,
}

impl Engine
{
    #[inline]
    pub fn new(_pair: &str, log_sender: std::sync::mpsc::Sender<Box<dyn LogTrait>>) -> Self {
        Self {
            orderbook: Book::new(),
            log_sender,
        }
    }

    #[inline]
    pub fn process(&mut self, incoming_order: OrderRequest) -> Result<(), EngineError> {
        match incoming_order {
            OrderRequest::Create { price, .. } => {
                if price.is_zero() {
                    Err(MarketUnsupported)?;
                }

                let order = LimitOrder::try_from(incoming_order).unwrap();
                let _ = self.orderbook.matching(order);
            }
            OrderRequest::Cancel { user_order_id, .. } => {
                self.orderbook.remove(&user_order_id);
            }
            OrderRequest::FlushBook => {
                let (ask_length, bid_length) = self.orderbook.len();
                if let Some((ask_price, bid_price)) = self.orderbook.spread() {
                    // Row {
                    //     label: "spread".to_string(),
                    //     values: vec![ask_price, bid_price],
                    // }
                    // println!("SPREAD -- ASK: {:?} BID: {:?}", ask_price, bid_price);
                    self.log_sender
                        .send(Box::new(Row {
                            label: "spread".to_string(),
                            values: vec![ask_price, bid_price],
                        }))
                        .unwrap_or_else(|e| eprintln!("{}", e));
                }
                self.log_sender
                    .send(Box::new(Row {
                        label: "book_length".to_string(),
                        values: vec![ask_length as u64, bid_length as u64],
                    }))
                    .unwrap_or_else(|e| eprintln!("{}", e));
                // println!("ASK LENGTH{:?} BID LENGTH {:?}", ask_length, bid_length);
                self.orderbook = Book::new()
            }
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
