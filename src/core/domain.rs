use crate::LogTrait;
use num::Zero;
use std::ops::{Add, Deref, DerefMut, Sub};
pub type Spread<T> = (Option<<T as Order>::Price>, Option<<T as Order>::Price>);
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

    type Acknowledgment: LogTrait;
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

    fn ack(&mut self, reject: bool) -> Self::Acknowledgment;
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

/// Matchers defines the operation to match an incoming order and its respective book and attempts to find a set
/// of matching trades (bids to asks and vice-versa).
pub trait Match {
    type Error;
    type Output;
    fn matching<B>(
        book: &mut B,
        incoming_order: <B as OrderBook>::Order,
    ) -> Result<Self::Output, Self::Error>
    where
        B: OrderBook,
        <<B as OrderBook>::Order as Order>::Acknowledgment: 'static;
}

/// OrderBook defines the operations that can be performed by the order book. It
/// embodies the basic operations that are typical of an order book
pub trait OrderBook {
    type Matching: Match;

    type Order: Order + Trade<Self::Order>;

    type OrderRef<'e>: Deref<Target = Self::Order>
    where
        Self: 'e;
    type OrderRefMut<'e>: DerefMut<Target = Self::Order>
    where
        Self: 'e;

    // Returns an iterator over the given side of the order book.
    fn iter(
        &self,
        side: &<Self::Order as Order>::Side,
    ) -> impl Iterator<Item = Self::OrderRef<'_>> + '_;

    /// Place an order into the book.
    fn place(&mut self, order: Self::Order);

    /// Cancel an open order in the book. Cancelling a non-existent order will result in a no-op.
    fn cancel(&mut self, order: &<Self::Order as Order>::Id) -> Option<Self::Order>;

    /// Returns a reference to the order (ask or bid) at the top of the book (head of the ask queue)
    fn peek(&self, side: &<Self::Order as Order>::Side) -> Option<Self::OrderRef<'_>>;

    /// Returns a reference to the order (ask or bid) at the top of the book (head of the ask queue)
    fn peek_mut(&mut self, side: &<Self::Order as Order>::Side) -> Option<Self::OrderRefMut<'_>>;

    /// Removes the top bid or aks from the head of the queue
    fn pop(&mut self, side: &<Self::Order as Order>::Side) -> Option<Self::Order>;

    /// Gets the bid and ask at the top of the book (head of the bid queue)
    fn peek_top_of_book(&self) -> Spread<Self::Order>;

    /// Returns the number of shares being bid on or offered.
    fn len(&self) -> (usize, usize);

    /// Returns `true` if the order book contains no items.
    fn is_empty(&self) -> bool {
        self.len() == (0, 0)
    }

    fn volume(&self) -> Volume<Self::Order>;

    /// Attempt to match an incoming order.
    ///
    /// This method takes an order as input and attempts to match it against the
    /// existing limit orders in the order book.
    fn matching(
        &mut self,
        incoming_order: Self::Order,
    ) -> Result<<Self::Matching as Match>::Output, <Self::Matching as Match>::Error>
    where
        Self: OrderBook + Sized,
        <<Self as OrderBook>::Order as Order>::Acknowledgment: 'static,
    {
        <Self::Matching as Match>::matching(self, incoming_order)
    }
}
