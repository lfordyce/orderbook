use thiserror::Error;

use crate::core::domain::{Match, Opposite, Order, OrderBook, Trade};
use crate::LogTrait;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DefaultMatchingError {}

pub struct MatchingEngine;

impl Match for MatchingEngine {
    type Error = DefaultMatchingError;
    type Output = (Box<dyn LogTrait>, bool);

    fn matching<B>(
        book: &mut B,
        mut incoming_order: <B as OrderBook>::Order,
    ) -> Result<Self::Output, Self::Error>
    where
        B: OrderBook,
        <<B as OrderBook>::Order as Order>::Acknowledgment: 'static,
    {
        while !incoming_order.is_closed() {
            let Some(mut top_order) = book.peek_mut(&incoming_order.side().opposite()) else {
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
                // We must explicit drop to reuse the order book.
                drop(top_order);
                // As long as top order is completed, it can be safely removed from order book.
                book.cancel(&top_order_id).expect("order should be `Some`");
            }
        }

        if incoming_order.is_closed() {
            let reject = incoming_order.ack(true);
            Ok((Box::new(reject), false))
        } else {
            let ack = incoming_order.ack(false);
            book.place(incoming_order);
            Ok((Box::new(ack), true))
        }
    }
}
