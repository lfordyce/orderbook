use crate::core::instrument::{Order, Trade};
use crate::core::order::LimitOrder;
use crate::core::{PriceError, Side, SideError, StatusError, TradeError};

#[derive(Debug)]
pub struct TradeImpl {
    pub taker_id: u64,
    pub maker_id: u64,
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
            taker_id: taker.id(),
            maker_id: maker.id(),
            amount: exchanged,
            price,
        })
    }

    /// Returns the traded price.
    pub fn price(&self) -> u64 {
        self.price
    }
}
