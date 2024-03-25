use std::collections::btree_map::Entry;

use num::Zero;

use crate::core::depth::{OrdersById, OrdersBySide};
use crate::core::domain::{Order, OrderBook, Spread, Volume};
use crate::core::matcher::MatchingEngine;
use crate::core::order::LimitOrder;
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

    pub fn flush(&mut self) {
        self.orders_by_id.clear();
        self.orders_by_side.flush();
    }
}

impl OrderBook for Book {
    type Matching = MatchingEngine;
    type Order = LimitOrder;
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

    fn place(&mut self, order: Self::Order) {
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

    fn cancel(&mut self, order_id: &<Self::Order as Order>::Id) -> Option<Self::Order> {
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
                .position(|&inner_order_id| order.id() == inner_order_id)
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

    fn peek_top_of_book(&self) -> Spread<Self::Order> {
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
