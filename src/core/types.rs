
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrderType {
    /// Limit orders are both the default and basic order type. A limit order
    /// requires specifying a price and size. The size is the number of bitcoin
    /// to buy or sell, and the price is the price per bitcoin. The limit order
    /// will be filled at the price specified or better.
    Limit {
        limit_price: u64,
        /// Time in force policies provide guarantees about the lifetime of an
        /// [order](Order).
        time_in_force: TimeInForce,
        amount: u64,
        filled: u64,
    },
    /// Market orders differ from limit orders in that they provide no pricing
    /// guarantees. They however do provide a way to buy or sell specific
    /// amounts of bitcoin or fiat without having to specify the price. Market
    /// orders execute immediately and no part of the market order will go on
    /// the open order book.
    Market {
        /// The `all or none` flag indicates that the orders are rejected if
        /// the entire size cannot be matched. When this is `true`, the order
        /// is considered a fill or kill order.
        all_or_none: bool,
        amount: u64,
        filled: u64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeInForce {
    /// An order will be on the book unless the order is canceled.
    GoodTillCancel {
        /// The post-only flag indicates that the order should only make
        /// liquidity. If any part of the order results in taking liquidity,
        /// the order will be rejected and no part of it will execute.
        post_only: bool,
    },
    /// An order will try to fill the order as much as it can before the order
    /// expires.
    ImmediateOrCancel {
        /// The `all-or-none` flag indicates that the orders are rejected if
        /// the entire size cannot be matched. When this is `true`, the order
        /// is considered a fill or kill order.
        all_or_none: bool,
    },
}

impl Default for TimeInForce {
    fn default() -> Self {
        Self::GoodTillCancel { post_only: false }
    }
}

pub struct InboundOrder {

}

