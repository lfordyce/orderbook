use orderbook::core::{Book, OrderBook, Side};
use tap::Tap;

mod valid {
    use super::*;
    use orderbook::core::order::LimitOrder;
    #[test]
    fn generate_reject() {
        let mut orderbook = Book::new().tap_mut(|orderbook| {
            let limit_order = LimitOrder {
                user_id: 1,
                order_id: 1,
                price: 10,
                quantity: 100,
                side: Side::Bid,
                order_symbol: "IBM".to_string(),
                timestamp: 1711396383937299000,
                filled: 0,
                status: Default::default(),
            };
            assert!(orderbook.matching(limit_order).is_ok());

            let limit_order = LimitOrder {
                user_id: 1,
                order_id: 2,
                price: 12,
                quantity: 100,
                side: Side::Ask,
                order_symbol: "IBM".to_string(),
                timestamp: 1711396383937305000,
                filled: 0,
                status: Default::default(),
            };
            assert!(orderbook.matching(limit_order).is_ok());

            let limit_order = LimitOrder {
                user_id: 2,
                order_id: 101,
                price: 9,
                quantity: 100,
                side: Side::Bid,
                order_symbol: "IBM".to_string(),
                timestamp: 1711396383937306000,
                filled: 0,
                status: Default::default(),
            };
            assert!(orderbook.matching(limit_order).is_ok());

            let limit_order = LimitOrder {
                user_id: 2,
                order_id: 102,
                price: 11,
                quantity: 100,
                side: Side::Ask,
                order_symbol: "IBM".to_string(),
                timestamp: 1711396383937307000,
                filled: 0,
                status: Default::default(),
            };
            assert!(orderbook.matching(limit_order).is_ok());
        });

        let first_rejected_limit_order = LimitOrder {
            user_id: 1,
            order_id: 3,
            price: 11,
            quantity: 100,
            side: Side::Bid,
            order_symbol: "IBM".to_string(),
            timestamp: 1711396383937308000,
            filled: 0,
            status: Default::default(),
        };

        let first_reject = orderbook.matching(first_rejected_limit_order);
        assert!(first_reject.is_ok());
        assert!(!first_reject.unwrap().1);

        let second_rejected_limit_order = LimitOrder {
            user_id: 2,
            order_id: 103,
            price: 10,
            quantity: 100,
            side: Side::Ask,
            order_symbol: "IBM".to_string(),
            timestamp: 1711396383937309000,
            filled: 0,
            status: Default::default(),
        };

        let second_reject = orderbook.matching(second_rejected_limit_order);
        assert!(second_reject.is_ok());
        assert!(!second_reject.unwrap().1);
    }

    #[test]
    fn cancel_order() {
        let mut orderbook = Book::new().tap_mut(|orderbook| {
            let limit_order = LimitOrder {
                user_id: 1,
                order_id: 1,
                price: 10,
                quantity: 100,
                side: Side::Bid,
                order_symbol: "IBM".to_string(),
                timestamp: 1711396383937299000,
                filled: 0,
                status: Default::default(),
            };
            assert!(orderbook.matching(limit_order).is_ok());

            let limit_order = LimitOrder {
                user_id: 1,
                order_id: 2,
                price: 12,
                quantity: 100,
                side: Side::Ask,
                order_symbol: "IBM".to_string(),
                timestamp: 1711396383937305000,
                filled: 0,
                status: Default::default(),
            };
            assert!(orderbook.matching(limit_order).is_ok());

            let limit_order = LimitOrder {
                user_id: 2,
                order_id: 101,
                price: 9,
                quantity: 100,
                side: Side::Bid,
                order_symbol: "IBM".to_string(),
                timestamp: 1711396383937306000,
                filled: 0,
                status: Default::default(),
            };
            assert!(orderbook.matching(limit_order).is_ok());

            let limit_order = LimitOrder {
                user_id: 2,
                order_id: 102,
                price: 11,
                quantity: 100,
                side: Side::Ask,
                order_symbol: "IBM".to_string(),
                timestamp: 1711396383937307000,
                filled: 0,
                status: Default::default(),
            };
            assert!(orderbook.matching(limit_order).is_ok());
        });

        let order_id: u64 = 2;
        let result = orderbook.cancel(&order_id);
        assert!(result.is_some());
        assert_eq!(result.unwrap().order_id, 2);

        // order id doesn't exist
        let nonexistent_order_id = 5;
        assert!(orderbook.cancel(&nonexistent_order_id).is_none());
    }
}
