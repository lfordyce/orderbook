use crate::core::instrument::Order;
use crate::core::order::LimitOrder;
use crate::core::Side;
use either::Either;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::ops::{Deref, DerefMut, Index, IndexMut};

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
    #[inline]
    fn index_mut(&mut self, side: S) -> &mut Self::Output {
        match side.borrow() {
            Side::Ask => &mut self.ask,
            Side::Bid => &mut self.bid,
        }
    }
}

pub trait PriceTimeOrder {
    fn new(price: u64, time: u128) -> Self;
}

pub struct Depth<T: PriceTimeOrder + Ord> {
    pub orders: HashMap<u64, LimitOrder>,
    pub queue: BTreeMap<T, u128>,
}

impl<T: PriceTimeOrder + Ord> Depth<T> {
    pub fn add(&mut self, order: &LimitOrder) {
        self.orders.insert(order.order_id, order.clone());
        self.queue
            .insert(T::new(order.price, order.timestamp), order.timestamp);
    }

    // pub fn decr_size(&mut self, order_id: u64, qty: u64) -> Result<(), ()> {
    //     return match self.orders.get(&order_id) {
    //         Some(order) => {
    //             let mut order = order.clone();
    //             match
    //             Ok(())
    //         },
    //         None => {Err()}
    //     }
    // }
}

pub struct PriceTimeKeyAsc {
    price: u64,
    time: u128,
}

impl PriceTimeOrder for PriceTimeKeyAsc {
    fn new(price: u64, time: u128) -> Self {
        PriceTimeKeyAsc { price, time }
    }
}

impl Eq for PriceTimeKeyAsc {}

impl PartialEq<Self> for PriceTimeKeyAsc {
    fn eq(&self, other: &Self) -> bool {
        self.price.eq(&other.price) && self.time.eq(&other.time)
    }
}

impl PartialOrd<Self> for PriceTimeKeyAsc {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriceTimeKeyAsc {
    fn cmp(&self, other: &Self) -> Ordering {
        return match self.price.cmp(&other.price) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => match self.time.cmp(&other.time) {
                Ordering::Less => Ordering::Less,
                Ordering::Equal => Ordering::Equal,
                Ordering::Greater => Ordering::Greater,
            },
        };
    }
}

pub struct PriceTimeKeyDesc {
    price: u64,
    time: u128,
}

impl PriceTimeOrder for PriceTimeKeyDesc {
    fn new(price: u64, time: u128) -> Self {
        PriceTimeKeyDesc { price, time }
    }
}

impl Eq for PriceTimeKeyDesc {}

impl PartialEq for PriceTimeKeyDesc {
    fn eq(&self, other: &Self) -> bool {
        self.price.eq(&other.price) && self.time.eq(&other.time)
    }
}

impl PartialOrd<Self> for PriceTimeKeyDesc {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriceTimeKeyDesc {
    fn cmp(&self, other: &Self) -> Ordering {
        return match self.price.cmp(&other.price) {
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
            Ordering::Equal => match self.time.cmp(&other.time) {
                Ordering::Less => Ordering::Greater,
                Ordering::Greater => Ordering::Less,
                Ordering::Equal => Ordering::Equal,
            },
        };
    }
}
