use std::borrow::Borrow;
use std::cmp::{Ordering, Reverse};
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use num::Zero;

use crate::Acknowledgment;
use crate::core::{OrderError, OrderRequestError, TradeError};
use crate::core::domain::{Opposite, Order};
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
    type Err = OrderRequestError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "S" => Ok(Side::Ask),
            "B" => Ok(Side::Bid),
            _ => Err(OrderRequestError::InvalidOrderSide(input.to_owned())),
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
    type Acknowledgment = Acknowledgment;

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

    fn ack(&mut self, reject: bool) -> Self::Acknowledgment {
        Acknowledgment {
            label: if reject {
                "R".to_string()
            } else {
                "A".to_string()
            },
            user_id: self.user_id,
            user_order_id: self.order_id,
        }
    }
}
