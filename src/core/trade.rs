use crate::core::domain::{Order, Trade};
use crate::core::order::LimitOrder;
use crate::core::{PriceError, Side, SideError, StatusError, TradeError};

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
            (Side::Ask, Side::Bid) => (taker_limit_price, maker_limit_price),
            (Side::Bid, Side::Ask) => (maker_limit_price, taker_limit_price),
            _ => return Err(SideError::Conflict)?,
        };

        (bid_price >= ask_price)
            .then_some(())
            .ok_or(PriceError::Incompatible)
            .map_err(Into::into)
    }
}

#[derive(Debug)]
pub struct TradeImpl {
    pub buy_user_id: u64,
    pub buy_order_id: u64,
    pub sell_user_id: u64,
    pub sell_order_id: u64,
    pub amount: u64,
    pub price: u64,
}

impl TradeImpl {
    /// Constructs a new `Trade`, returning an error if something fails.
    pub fn try_new(
        maker: &mut LimitOrder,
        taker: &mut LimitOrder,
    ) -> Result<TradeImpl, TradeError> {
        maker.matches(&*taker)?;

        let exchanged = taker.remaining().min(maker.remaining());
        let price = maker.limit_price().expect("maker must always have a price");

        maker.fill(exchanged);
        taker.fill(exchanged);

        Ok(TradeImpl {
            buy_user_id: taker.user_id,
            buy_order_id: taker.id(),
            sell_user_id: maker.user_id,
            sell_order_id: maker.id(),
            amount: exchanged,
            price,
        })
    }
}
