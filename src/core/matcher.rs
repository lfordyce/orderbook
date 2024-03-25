use crate::core::order::{LimitOrder, TradeS};
use crate::core::orderbook::OrderBookAlt;
use crate::core::Side;

/// A match is a structure which contains a list of executed orders (trades) as well as fields
/// indicating if the match was done in full or partially, along with the quantity left
#[derive(Debug)]
pub struct Match<T> {
    /// list of matches found by the matcher
    matches: Vec<T>,

    /// the state of the match run, can be partial, full or no-match
    state: MatchState,

    /// number of items left to complete a full match
    qty_left: u64,
}

impl<T> Match<T>
where
    T: Clone + std::fmt::Debug + Copy,
{
    pub fn new() -> Self {
        Self {
            matches: Vec::with_capacity(4),
            state: MatchState::NoMatch,
            qty_left: 0,
        }
    }

    pub fn add_match(&mut self, trade: T) {
        self.matches.push(trade)
    }

    pub fn get_matches(&self) -> Vec<T> {
        self.matches.clone()
    }

    pub fn update_state(&mut self, state: MatchState) {
        match state {
            MatchState::Full | MatchState::NoMatch => self.update_qty_left(0),
            MatchState::Partial => (),
        }
        self.state = state
    }

    pub fn get_state(&self) -> MatchState {
        self.state.clone()
    }

    pub fn update_qty_left(&mut self, qty: u64) {
        self.qty_left = qty
    }

    pub fn get_qty_left(&self) -> u64 {
        self.qty_left
    }

    pub fn is_partial(&self) -> bool {
        self.state == MatchState::Partial
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum MatchState {
    Full,
    Partial,
    NoMatch,
}
/// Implements a matcher with takes an order and its respective book and attempts to find a set
/// of matching trades (bids to asks and vice-versa)
#[derive(Debug)]
pub struct Matcher;

impl Matcher {
    pub fn match_order<T: OrderBookAlt>(&self, order: LimitOrder, orderbook: &mut T) -> Match<TradeS> {
        let mut matches = Match::new();
        // match order.order_type {
        //     // a market order is matched immediately at the best available price. In cases
        //     // where the engine is unable to fill the match completely, the order is partially
        //     // filled and the remaining part of the order is left on the book
        //     OrderType::Market => {
        //         if let Some(opp_order) = Self::get_opposite_order(order.side, orderbook) {
        //             Self::do_match(order, opp_order.clone(), orderbook, &mut matches)
        //         }
        //         // an early return with the state being MatchState::NoMatch
        //         return matches;
        //     }
        //     // a limit order is first matched immediately if possible and if not it is placed into
        //     // the limit order book to be filled at a later time, when a matching market order is found
        //     OrderType::Limit => {
        //         if let Some(opp_order) = Self::get_opposite_order(order.side, orderbook) {
        //             // first we do price check to ensure the price variant of the limit order is maintained
        //             if Self::is_within_price_limit(order, *opp_order) {
        //                 Self::do_match(order, opp_order.clone(), orderbook, &mut matches);
        //                 // if there's a partial match we want to place the remnants on the orderbook
        //                 if MatchState::Partial == matches.get_state() {
        //                     let mut left_over = order.clone();
        //                     left_over.quantity = matches.get_qty_left();
        //                     let _ = orderbook.place(left_over);
        //                 }
        //                 return matches;
        //             }
        //         }
        //         let _ = orderbook.place(order);
        //         // an early return with the state being MatchState::NoMatch
        //         return matches;
        //     }
        //     OrderType::Stop => todo!(),
        // }
        if let Some(opp_order) = Self::get_opposite_order(order.side, orderbook) {
            // first we do price check to ensure the price variant of the limit order is maintained
            if Self::is_within_price_limit(&order, opp_order) {
                Self::do_match(order.clone(), opp_order.clone(), orderbook, &mut matches);
                // if there's a partial match we want to place the remnants on the orderbook
                if MatchState::Partial == matches.get_state() {
                    let mut left_over = order.clone();
                    left_over.quantity = matches.get_qty_left();
                    let _ = orderbook.place(left_over);
                }
                return matches;
            }
        }
        let _ = orderbook.place(order);
        // an early return with the state being MatchState::NoMatch
        return matches;
    }

    fn get_opposite_order(side: Side, orderbook: &mut dyn OrderBookAlt) -> Option<&LimitOrder> {
        match side {
            Side::Bid => orderbook.peek_top_ask(),
            Side::Ask => orderbook.peek_top_bid(),
        }
    }

    fn is_within_price_limit(order: &LimitOrder, opp_order: &LimitOrder) -> bool {
        match order.side {
            Side::Bid => order.price >= opp_order.price,
            Side::Ask => order.price <= opp_order.price,
        }
    }

    fn do_match(
        mut incoming_order: LimitOrder,
        opposite_order: LimitOrder,
        orderbook: &mut dyn OrderBookAlt,
        matches: &mut Match<TradeS>,
    ) {
        if incoming_order.quantity < opposite_order.quantity {
            matches.add_match(TradeS {
                order_id: incoming_order.order_id,
                side: incoming_order.side,
                price: opposite_order.price,
                // status: OrderStatus::Filled,
                quantity: incoming_order.quantity,
                timestamp: 0,
            });

            matches.add_match(TradeS {
                order_id: opposite_order.order_id,
                side: opposite_order.side,
                price: opposite_order.price,
                // status: OrderStatus::PartialFill,
                quantity: incoming_order.quantity,
                timestamp: 0,
            });

            orderbook.modify_quantity(
                opposite_order.order_id,
                opposite_order.quantity - incoming_order.quantity,
            );
            // the state is full because the engine was able to fully match the incoming order
            matches.update_state(MatchState::Full);
        } else if incoming_order.quantity > opposite_order.quantity {
            matches.add_match(TradeS {
                order_id: incoming_order.order_id,
                side: incoming_order.side,
                price: opposite_order.price,
                // status: OrderStatus::PartialFill,
                quantity: opposite_order.quantity,
                timestamp: 0,
            });

            matches.add_match(TradeS {
                order_id: opposite_order.order_id,
                side: opposite_order.side,
                price: opposite_order.price,
                // status: OrderStatus::Filled,
                quantity: opposite_order.quantity,
                timestamp: 0,
            });

            // update the quantity of the partially filled order
            incoming_order.quantity -= opposite_order.quantity;

            // we update the quantity left to match for the primary order
            matches.update_qty_left(incoming_order.quantity);

            // since the incoming order was partially filled, the state is updated accordingly
            matches.update_state(MatchState::Partial);

            let some_order = match incoming_order.side {
                Side::Bid => {
                    // pop off the current top ask, since it has already been filled
                    orderbook.pop_top_ask();
                    // get the current top ask on the book
                    orderbook.peek_top_ask()
                }
                Side::Ask => {
                    // pop the current top bid since it has been filled
                    orderbook.pop_top_bid();
                    // get the current top bid and attempt to fill
                    orderbook.peek_top_bid()
                }
            };

            // attempt to fill the rest of the partially filled order
            if let Some(opposite) = some_order {
                Self::do_match(incoming_order, opposite.clone(), orderbook, matches)
            }
        } else {
            matches.add_match(TradeS {
                order_id: incoming_order.order_id,
                side: incoming_order.side,
                price: opposite_order.price,
                // status: OrderStatus::Filled,
                quantity: incoming_order.quantity,
                timestamp: 0,
            });

            matches.add_match(TradeS {
                order_id: opposite_order.order_id,
                side: opposite_order.side,
                price: opposite_order.price,
                // status: OrderStatus::Filled,
                quantity: opposite_order.quantity,
                timestamp: 0,
            });

            matches.update_state(MatchState::Full);

            match incoming_order.side {
                Side::Bid => orderbook.pop_top_ask(),
                Side::Ask => orderbook.pop_top_bid(),
            };
        }
    }
}
