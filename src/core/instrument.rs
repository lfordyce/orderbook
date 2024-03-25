use num::Zero;
use std::convert::TryFrom;
use std::ops::{Add, Deref, DerefMut, Sub};

pub type Spread<T> = (<T as Order>::Price, <T as Order>::Price);
pub type Volume<T> = (<T as Order>::Amount, <T as Order>::Amount);

pub trait Order: PartialOrd {
    type Amount: Add<Output = Self::Amount> + Sub<Output = Self::Amount> + Copy + Ord + Zero;

    /// User Order Id
    type Id: Copy + Eq + Ord;
    /// User ID
    type UserId: Copy + Eq + Ord;
    /// Order price.
    type Price: Copy + Ord;
    /// Order side.
    type Side: Opposite;
    type OrderStatus: Copy + Eq;
    type Trade;
    type TradeError: std::error::Error;
    /// Return order unique identifier.
    fn id(&self) -> Self::Id;
    fn user_id(&self) -> Self::UserId;
    /// Return order side.
    fn side(&self) -> Self::Side;
    fn remaining(&self) -> Self::Amount;
    fn status(&self) -> Self::OrderStatus;
    fn is_closed(&self) -> bool;
    /// Return order limit price.
    fn limit_price(&self) -> Option<Self::Price>;
    /// Cancel the order.
    fn cancel(&mut self);
}

pub trait Trade<Rhs>: Order
where
    Rhs: Order,
{
    /// Execute a trade.
    fn trade(&mut self, other: &mut Rhs) -> Result<Self::Trade, Self::TradeError>;
    /// Returns `Ok` if orders match.
    fn matches(&self, other: &Rhs) -> Result<(), Self::TradeError>;
}

/// The logical opposite of a value.
pub trait Opposite<Opposite = Self> {
    /// Returns the opposite value.
    fn opposite(&self) -> Opposite;
}

pub trait Matchers {
    type Error;
    type Output;
    fn matching<E>(
        exchange: &mut E,
        incoming_order: <E as OrderBook>::Order,
    ) -> Result<Self::Output, Self::Error>
    where
        E: OrderBook;
        // <E as OrderBook>::Order: TryFrom<<E as OrderBook>::IncomingOrder>;
}

pub trait OrderBook {
    type Matching: Matchers;

    type Order: Order + Trade<Self::Order>;

    // type IncomingOrder: Order<
    //     Amount = <Self::Order as Order>::Amount,
    //     Id = <Self::Order as Order>::Id,
    //     Price = <Self::Order as Order>::Price,
    //     Side = <Self::Order as Order>::Side,
    //     OrderStatus = <Self::Order as Order>::OrderStatus,
    // >;
    type OrderRef<'e>: Deref<Target = Self::Order>
    where
        Self: 'e;
    type OrderRefMut<'e>: DerefMut<Target = Self::Order>
    where
        Self: 'e;

    // Returns an iterator over the given side of the exchange.
    fn iter(
        &self,
        side: &<Self::Order as Order>::Side,
    ) -> impl Iterator<Item = Self::OrderRef<'_>> + '_;

    fn insert(&mut self, order: Self::Order);

    /// Removes an order from the exchange.
    fn remove(&mut self, order: &<Self::Order as Order>::Id) -> Option<Self::Order>;

    /// Returns a reference of the most relevant order in the exchange.
    fn peek(&self, side: &<Self::Order as Order>::Side) -> Option<Self::OrderRef<'_>>;

    /// Returns a mutable reference of the most relevant order in the exchange.
    fn peek_mut(&mut self, side: &<Self::Order as Order>::Side) -> Option<Self::OrderRefMut<'_>>;

    /// Removes the most relevant order in the exchange.
    fn pop(&mut self, side: &<Self::Order as Order>::Side) -> Option<Self::Order>;

    /// Returns the difference or gap that exists between bid and ask
    /// prices.
    fn spread(&self) -> Option<Spread<Self::Order>>;

    /// Returns the number of shares being bid on or offered.
    fn len(&self) -> (usize, usize);

    /// Returns `true` if the exchange contains no items.
    fn is_empty(&self) -> bool {
        self.len() == (0, 0)
    }

    fn volume(&self) -> Volume<Self::Order>;

    /// Attempt to match an incoming order.
    ///
    /// This method takes an order as input and attempts to match it against the
    /// existing limit orders in the orderbook. Matching is done in a specific
    /// order based on the orderbook's rules, such as price-time priority.
    fn matching(
        &mut self,
        incoming_order: Self::Order,
    ) -> Result<<Self::Matching as Matchers>::Output, <Self::Matching as Matchers>::Error>
        where
            Self: OrderBook + Sized,
    {
        <Self::Matching as Matchers>::matching(self, incoming_order)
    }
    // fn matching(
    //     &mut self,
    //     incoming_order: Self::IncomingOrder,
    // ) -> Result<<Self::Matching as Matchers>::Output, <Self::Matching as Matchers>::Error>
    // where
    //     Self: OrderBook + Sized,
    //     Self::Order: TryFrom<Self::IncomingOrder>,
    // {
    //     <Self::Matching as Matchers>::matching(self, incoming_order)
    // }
}
