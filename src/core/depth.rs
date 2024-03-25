use std::borrow::Borrow;
use std::collections::{BTreeMap, VecDeque};
use std::ops::{Deref, DerefMut, Index, IndexMut};

use either::Either;

use crate::core::domain::Order;
use crate::core::Side;

pub struct OrdersByPrice<T: Order>(BTreeMap<<T as Order>::Price, VecDeque<<T as Order>::Id>>);
pub struct OrdersById<T: Order>(BTreeMap<<T as Order>::Id, T>);

impl<T: Order> Default for OrdersByPrice<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Order> Deref for OrdersByPrice<T> {
    type Target = BTreeMap<<T as Order>::Price, VecDeque<<T as Order>::Id>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Order> DerefMut for OrdersByPrice<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Order> Default for OrdersById<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Order> Deref for OrdersById<T> {
    type Target = BTreeMap<<T as Order>::Id, T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Order> DerefMut for OrdersById<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct OrdersBySide<T: Order> {
    ask: OrdersByPrice<T>,
    bid: OrdersByPrice<T>,
}

impl<T: Order> OrdersBySide<T>
where
    T: Order<Side = Side>,
{
    pub fn iter(
        &self,
        side: &<T as Order>::Side,
    ) -> impl Iterator<Item = &<T as Order>::Id> {
        match side {
            Side::Ask => Either::Left(self[side].deref().values().flat_map(VecDeque::iter)),
            Side::Bid => Either::Right(self[side].deref().values().rev().flat_map(VecDeque::iter)),
        }
    }

    pub fn peek(&self, side: &<T as Order>::Side) -> Option<&<T as Order>::Id> {
        self.iter(side).next()
    }
}

impl<T: Order> Default for OrdersBySide<T> {
    fn default() -> Self {
        Self {
            ask: Default::default(),
            bid: Default::default(),
        }
    }
}

impl<T, S> Index<S> for OrdersBySide<T>
where
    T: Order<Side = Side>,
    S: Borrow<<T as Order>::Side>,
{
    type Output = OrdersByPrice<T>;

    fn index(&self, side: S) -> &Self::Output {
        match *side.borrow() {
            Side::Ask => &self.ask,
            Side::Bid => &self.bid,
        }
    }
}

impl<T, S> IndexMut<S> for OrdersBySide<T>
where
    T: Order<Side = Side>,
    S: Borrow<<T as Order>::Side>,
{
    fn index_mut(&mut self, side: S) -> &mut Self::Output {
        match side.borrow() {
            Side::Ask => &mut self.ask,
            Side::Bid => &mut self.bid,
        }
    }
}