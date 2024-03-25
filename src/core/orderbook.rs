use std::collections::btree_map::Entry;
use std::collections::HashMap;

use num::Zero;

use crate::core::depth::{OrdersById, OrdersBySide};
use crate::core::engine::MatchingEngine;
use crate::core::instrument::{Order, OrderBook, Spread, SpreadOption, Volume};
use crate::core::order::{LimitOrder, OrderIndex, OrderQueue, PriceTimePriorityOrderQueue};
use crate::core::Side;

pub struct Book {
    orders_by_id: OrdersById<LimitOrder>,
    orders_by_side: OrdersBySide<LimitOrder>,
}

impl Default for Book {
    #[inline]
    fn default() -> Self {
        Self {
            orders_by_id: Default::default(),
            orders_by_side: Default::default(),
        }
    }
}

impl Book {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl OrderBook for Book {
    type Matching = MatchingEngine;
    type Order = LimitOrder;
    // type IncomingOrder = LimitOrder;
    type OrderRef<'e> = &'e LimitOrder where Self: 'e;
    type OrderRefMut<'e> = &'e mut LimitOrder where Self: 'e;

    fn iter(
        &self,
        side: &<Self::Order as Order>::Side,
    ) -> impl Iterator<Item = Self::OrderRef<'_>> + '_ {
        let order_id_to_order = move |order_id: &<LimitOrder as Order>::Id| -> Self::OrderRef<'_> {
            self.orders_by_id
                .get(order_id)
                .expect("every order in tree must also be in index")
        };

        self.orders_by_side.iter(side).map(order_id_to_order)
    }

    fn insert(&mut self, order: Self::Order) {
        self.orders_by_side[order.side()]
            .entry(
                order
                    .limit_price()
                    .expect("bookable orders must have a limit price"),
            )
            .or_default()
            .push_back(order.id());

        self.orders_by_id.insert(order.id(), order);
    }

    fn remove(&mut self, order_id: &<Self::Order as Order>::Id) -> Option<Self::Order> {
        let order = self.orders_by_id.remove(order_id)?;

        let limit_price = order
            .limit_price()
            .expect("bookable orders must have a limit price");

        let Entry::Occupied(mut level) = self.orders_by_side[order.side()].entry(limit_price)
        else {
            unreachable!("orders that lives in index must also be in the tree");
        };

        // This prevents dangling levels (level with no orders).
        if level.get().len() == 1 {
            level.remove().pop_front()
        } else {
            level
                .get()
                .iter()
                .position(|&order_id| order.id() == order_id)
                .and_then(|index| level.get_mut().remove(index))
        }
        .expect("indexed orders must be in the book tree");

        assert_eq!(
            &order.id(),
            order_id,
            "order id must be the same; something is wrong otherwise"
        );

        order.into()
    }

    fn peek(&self, side: &<Self::Order as Order>::Side) -> Option<Self::OrderRef<'_>> {
        let order_id = self.orders_by_side.peek(side)?;

        self.orders_by_id
            .get(order_id)
            .expect("every order that lives in tree must also be in the index")
            .into()
    }

    fn peek_mut(&mut self, side: &<Self::Order as Order>::Side) -> Option<Self::OrderRefMut<'_>> {
        let order_id = self.orders_by_side.peek(side)?;

        self.orders_by_id
            .get_mut(order_id)
            .expect("every order that lives in tree must also be in the index")
            .into()
    }

    fn pop(&mut self, side: &<Self::Order as Order>::Side) -> Option<Self::Order> {
        let mut level = match side {
            side @ Side::Ask => self.orders_by_side[side].first_entry(),
            side @ Side::Bid => self.orders_by_side[side].last_entry(),
        }?;

        let order_id = if level.get().len() == 1 {
            // This prevents dangling levels (level with no orders).
            level.remove().pop_front()
        } else {
            level.get_mut().pop_front()
        }
        .expect("level should always have an order");

        self.orders_by_id
            .remove(&order_id)
            .expect("every order that lives in tree must also be in the index")
            .into()
    }

    fn spread(&self) -> Option<Spread<Self::Order>> {
        // let ask_side = self.peek(&Side::Ask);
        Some((
            self.peek(&Side::Ask)?.limit_price()?,
            self.peek(&Side::Bid)?.limit_price()?,
        ))
    }

    fn spread_option(&self) -> SpreadOption<Self::Order> {
        (
            if let Some(order) = self.peek(&Side::Ask) {
                order.limit_price()
            } else {
                None
            },
            if let Some(order) = self.peek(&Side::Bid) {
                order.limit_price()
            } else {
                None
            },
        )
    }

    fn len(&self) -> (usize, usize) {
        (
            self.orders_by_side[Side::Ask]
                .iter()
                .fold(0, |acc, (_, level)| acc + level.len()),
            self.orders_by_side[Side::Bid]
                .iter()
                .fold(0, |acc, (_, level)| acc + level.len()),
        )
    }

    fn volume(&self) -> Volume<Self::Order> {
        let ask = self
            .iter(&Side::Ask)
            .map(Order::remaining)
            .reduce(|acc, curr| acc + curr)
            .unwrap_or_else(Zero::zero);

        let bid = self
            .iter(&Side::Bid)
            .map(Order::remaining)
            .reduce(|acc, curr| acc + curr)
            .unwrap_or_else(Zero::zero);

        (ask, bid)
    }
}

#[derive(Debug)]
pub struct Books {
    order_symbol: String,
    bids: PriceTimePriorityOrderQueue<OrderIndex>,
    asks: PriceTimePriorityOrderQueue<OrderIndex>,
    orders: HashMap<u64, LimitOrder>,
    // _trade: PhantomData<Trade>
}

/// This trait defines the operations that can be performed by the orderbook. It
/// embodies the basic operations that are typical of an orderbook
pub trait OrderBookAlt {
    /// Cancel an open order in the book. Cancelling a non-existent order should fail
    fn cancel(&mut self, orderid: u64) -> Result<(), ()>;

    /// Place an order into the book, should the order already exists it should also fail
    fn place(&mut self, order: LimitOrder) -> Result<(), ()>;

    /// Gets the ask at the top of the book (head of the ask queue)
    fn peek_top_ask(&self) -> Option<&LimitOrder>;

    /// Gets the bid at the top of the book (head of the bid queue)
    fn peek_top_bid(&self) -> Option<&LimitOrder>;

    /// Allows for the modification of the order quantity in-place
    fn modify_quantity(&mut self, orderid: u64, qty: u64);

    /// Removes the top bid from the head of the queue
    fn pop_top_bid(&mut self) -> Option<LimitOrder>;

    /// Removes the top ask from the head of the ask queue
    fn pop_top_ask(&mut self) -> Option<LimitOrder>;
}

impl Books {
    pub fn new(order_symbol: String) -> Self {
        Books {
            order_symbol,
            bids: PriceTimePriorityOrderQueue::with_capacity(1000),
            asks: PriceTimePriorityOrderQueue::with_capacity(1000),
            orders: HashMap::with_capacity(1000),
        }
    }
}

impl OrderBookAlt for Books {
    fn cancel(&mut self, order_id: u64) -> Result<(), ()> {
        match self.orders.remove(&order_id) {
            Some(order) => {
                match order.side {
                    Side::Bid => self.bids.remove(OrderIndex::from(order)),
                    Side::Ask => self.asks.remove(OrderIndex::from(order)),
                };
                return Ok(());
            }
            None => Ok(()),
        }
    }

    fn place(&mut self, order: LimitOrder) -> Result<(), ()> {
        // if OrderType::Market == order.order_type {
        //     return Err(Failure::OrderRejected(
        //         "Only limit orders can be placed in the orderbook".to_string(),
        //     ));
        // }
        // if self.trading_pair != order.trading_pair {
        //     return Err(Failure::InvalidOrderForBook);
        // }

        self.orders.insert(order.order_id, order.clone());
        match order.side {
            Side::Bid => self.bids.push(OrderIndex::from(order)),
            Side::Ask => self.asks.push(OrderIndex::from(order)),
        };
        Ok(())
    }

    fn peek_top_ask(&self) -> Option<&LimitOrder> {
        if let Some(key) = self.asks.peek() {
            return self.orders.get(&key.order_id);
        }
        None
    }

    fn peek_top_bid(&self) -> Option<&LimitOrder> {
        if let Some(key) = self.bids.peek() {
            return self.orders.get(&key.order_id);
        }
        None
    }

    fn modify_quantity(&mut self, orderid: u64, quantity: u64) {
        if let Some(order) = self.orders.get_mut(&orderid) {
            order.quantity = quantity
        }
    }

    fn pop_top_bid(&mut self) -> Option<LimitOrder> {
        if let Some(key) = self.bids.pop() {
            return self.orders.remove(&key.order_id);
        }
        None
    }

    fn pop_top_ask(&mut self) -> Option<LimitOrder> {
        if let Some(key) = self.asks.pop() {
            return self.orders.remove(&key.order_id);
        }
        None
    }
}
