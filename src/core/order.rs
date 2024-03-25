

use std::borrow::Borrow;
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use num::Zero;

use crate::core::instrument::{Opposite, Order, Trade};
use crate::core::{OrderError, OrderRequestError, PriceError, SideError, StatusError, TradeError};
use crate::core::trade::TradeImpl;

#[derive(Debug)]
pub enum OrderRequest {
    Create {
        user_id: u64,
        symbol: String,
        price: u64,
        qty: u64,
        side: Side,
        user_order_id: u64,
        unix_nano: u128,
    },
    Cancel {
        user_id: u64,
        user_order_id: u64,
        unix_nano: u128,
    },
    FlushBook,
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Debug, Copy)]
pub enum Side {
    Ask,
    Bid,
}

impl FromStr for Side {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "S" => Ok(Side::Ask),
            "B" => Ok(Side::Bid),
            _ => Err(()),
        }
    }
}

impl Display for Side {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Ask => write!(f, "A"),
            Side::Bid => write!(f, "B"),
        }
    }
}

impl Opposite for Side {
    #[inline]
    fn opposite(&self) -> Self {
        match self {
            Side::Ask => Side::Bid,
            Side::Bid => Side::Ask,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OrderStatus {
    #[default]
    Open,
    Partial,
    Cancelled,
    Closed,
    Completed,
}

#[derive(Clone, Debug)]
pub struct LimitOrder {
    pub user_id: u64,
    pub order_id: u64,
    pub price: u64,
    pub quantity: u64,
    pub side: Side,
    pub order_symbol: String,
    pub timestamp: u128,
    pub filled: u64,
    pub status: OrderStatus,
    // pub order_type: OrderType,
}

impl LimitOrder {
    pub fn fill(&mut self, amount: u64) {
        self.try_fill(amount)
            .expect("order does not have available amount to fill")
    }

    fn try_fill(&mut self, amount: u64) -> Result<(), OrderError> {
        if amount.is_zero() {
            return Err(OrderError::NoFill);
        }
        if amount > self.remaining() {
            return Err(OrderError::Overfill);
        }
        self.filled += amount;

        self.status = if self.remaining().is_zero() {
            OrderStatus::Completed
        } else {
            OrderStatus::Partial
        };
        Ok(())
    }
}

impl Borrow<LimitOrder> for Reverse<LimitOrder> {
    #[inline]
    fn borrow(&self) -> &LimitOrder {
        &self.0
    }
}

impl PartialEq for LimitOrder {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.order_id.eq(&other.order_id)
    }
}
impl Eq for LimitOrder {}

impl PartialOrd for LimitOrder {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.limit_price().partial_cmp(&other.limit_price())
    }
}

impl TryFrom<OrderRequest> for LimitOrder {
    type Error = OrderRequestError;

    fn try_from(order_request: OrderRequest) -> Result<Self, Self::Error> {
        match order_request {
            OrderRequest::Create {
                user_order_id,
                user_id,
                price,
                qty,
                symbol,
                side,
                unix_nano,
            } => Ok(LimitOrder {
                user_id,
                order_id: user_order_id,
                price,
                quantity: qty,
                order_symbol: symbol,
                side,
                timestamp: unix_nano,
                filled: 0,
                status: OrderStatus::Open,
            }),
            _ => Err(OrderRequestError::MismatchType),
        }
    }
}

impl Order for LimitOrder {
    type Amount = u64;
    type Id = u64;
    type UserId = u64;
    type Price = u64;
    type Side = Side;
    type OrderStatus = OrderStatus;
    type Trade = TradeImpl;
    type TradeError = TradeError;

    fn id(&self) -> Self::Id {
        self.order_id
    }

    fn user_id(&self) -> Self::UserId {
        self.user_id
    }

    fn side(&self) -> Self::Side {
        self.side
    }

    fn remaining(&self) -> Self::Amount {
        self.quantity - self.filled
    }

    fn status(&self) -> Self::OrderStatus {
        self.status
    }

    fn is_closed(&self) -> bool {
        matches!(
            self.status(),
            OrderStatus::Cancelled | OrderStatus::Closed | OrderStatus::Completed
        )
    }

    fn limit_price(&self) -> Option<Self::Price> {
        Some(self.price)
    }

    fn cancel(&mut self) {
        match self.status() {
            OrderStatus::Open => self.status = OrderStatus::Cancelled,
            OrderStatus::Partial => self.status = OrderStatus::Closed,
            _ => (),
        }
    }
}

impl Trade<LimitOrder> for LimitOrder {
    fn trade(&mut self, other: &mut LimitOrder) -> Result<Self::Trade, Self::TradeError> {
        let (maker, taker) = (self, other);

        Self::Trade::try_new(maker, taker)
    }

    fn matches(&self, other: &LimitOrder) -> Result<(), Self::TradeError> {
        let (maker, taker) = (self, other);

        // Matching cannot occur between closed orders.
        if taker.is_closed() || maker.is_closed() {
            return Err(StatusError::Closed)?;
        }

        let maker_limit_price = maker
            .limit_price()
            .expect("market makers always have a limit price");

        let Some(taker_limit_price) = taker.limit_price() else {
            return Ok(());
        };

        let (ask_price, bid_price) = match (taker.side(), maker.side()) {
            (Side::Ask, Side::Bid) => {
                (taker_limit_price, maker_limit_price)
            }
            (Side::Bid, Side::Ask) => {
                (maker_limit_price, taker_limit_price)
            }
            _ => return Err(SideError::Conflict)?,
        };

        (bid_price >= ask_price)
            .then_some(())
            .ok_or(PriceError::Incompatible)
            .map_err(Into::into)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TradeS {
    pub order_id: u64,
    pub side: Side,
    pub price: u64,
    // pub status: OrderStatus,
    pub quantity: u64,
    pub timestamp: u128,
}

impl From<LimitOrder> for OrderIndex {
    fn from(value: LimitOrder) -> Self {
        OrderIndex {
            order_id: value.order_id,
            price: value.price,
            side: value.side,
            timestamp: value.timestamp,
        }
    }
}

#[derive(Clone, Eq, Copy, Debug)]
pub struct OrderIndex {
    pub order_id: u64,
    pub price: u64,
    pub timestamp: u128,
    pub side: Side,
}

// The ordering determines how the orders are arranged in the queue. For price time priority
// ordering, we want orders inserted based on the price and the time of entry. For Bids this
// means the highest price gets the top priority, for Asks the lowest price gets the top priority
// For orders with the same price, the longest staying in the queue gets the higher priority
impl Ord for OrderIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.price > other.price {
            match self.side {
                Side::Bid => Ordering::Greater,
                Side::Ask => Ordering::Less,
            }
        } else if self.price < other.price {
            match self.side {
                Side::Bid => Ordering::Less,
                Side::Ask => Ordering::Greater,
            }
        } else {
            other.timestamp.cmp(&self.timestamp)
        }
    }
}

impl PartialOrd for OrderIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for OrderIndex {
    fn eq(&self, other: &Self) -> bool {
        self.order_id == other.order_id
            && self.price == other.price
            && self.side == other.side
            && self.timestamp == other.timestamp
    }
}

/// Encapsulates a priority queue of Orders, ordered by OrderIndex.
/// A key index is a structure that defines some ordering, as well as information that
/// allows implementations of the order queue determine priority of items
pub trait KeyIndx: Clone + Ord + PartialEq + Copy {}

/// This trait defines the operations that should be performed by the order queue. It is
/// expected that the backing implemenation be a priority queue.
///
/// It is genric over type [T], which is any trait that implements the [KeyIndx] trait.
///
/// [KeyIndx] provides the ordering, which determines how items are prioritized in the queue
///
pub trait OrderQueue<T: KeyIndx> {
    /// Pushes an item into the queue
    fn push(&mut self, item: T);

    // Gets the item at the head of the queue
    fn peek(&self) -> Option<&T>;

    /// Removes the item at the head of the queue
    fn pop(&mut self) -> Option<T>;

    /// Removes the specified item from the queue. This operation balances the queue
    fn remove(&mut self, item: T) -> Option<T>;
}

/// Simple implementation of the order queue. Uses a binary heap as a priority queue
/// Orders are prioritized by price and time
#[derive(Debug)]
pub struct PriceTimePriorityOrderQueue<T> {
    heap: BinaryHeap<T>,
}

impl<T> PriceTimePriorityOrderQueue<T>
where
    T: KeyIndx,
{
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::with_capacity(16),
        }
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            heap: BinaryHeap::with_capacity(capacity),
        }
    }
}

impl<T> OrderQueue<T> for PriceTimePriorityOrderQueue<T>
where
    T: KeyIndx,
{
    fn push(&mut self, item: T) {
        self.heap.push(item)
    }

    fn peek(&self) -> Option<&T> {
        self.heap.peek()
    }

    fn pop(&mut self) -> Option<T> {
        self.heap.pop()
    }

    fn remove(&mut self, item: T) -> Option<T> {
        let mut key_vec = self.heap.to_owned().into_vec();
        key_vec.retain(|k| *k != item);
        self.heap = key_vec.into();
        Some(item)
    }
}

impl KeyIndx for OrderIndex {}
