#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! T-CONCURRENCY：`Send + Sync` 编译期断言 + `std::thread` 并发只读校验。
//!
//! 禁止 tokio。

use domainx::{
    Commission, Decimal, InstrumentKey, Order, OrderSide, OrderStatus, OrderType, Portfolio,
    Position, TimeInForce, Trade, ValidationError, aggregate_commissions_by_asset, validate_order,
};
use static_assertions::assert_impl_all;

assert_impl_all!(Order: Send, Sync);
assert_impl_all!(Position: Send, Sync);
assert_impl_all!(Trade: Send, Sync);
assert_impl_all!(Portfolio: Send, Sync);
assert_impl_all!(ValidationError: Send, Sync);

fn sample_order() -> Order {
    Order {
        order_id: "o-cc".into(),
        instrument: InstrumentKey { exchange: "binance".into(), symbol: "BTCUSDT".into() },
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        status: OrderStatus::New,
        price: Some(Decimal::new(50000, 0)),
        stop_price: None,
        quantity: Decimal::new(2, 0),
        filled_quantity: Decimal::ZERO,
        remaining_quantity: Decimal::new(2, 0),
        avg_fill_price: None,
        time_in_force: TimeInForce::Gtc,
        created_at: 1_700_000_000_000,
        updated_at: 1_700_000_000_000,
        client_order_id: None,
    }
}

#[test]
fn concurrent_validate_order_and_aggregate() {
    let order = sample_order();
    let items = [
        Commission { amount: Decimal::new(1, 2), asset: "USDT".into() },
        Commission { amount: Decimal::new(2, 2), asset: "USDT".into() },
        Commission { amount: Decimal::new(3, 4), asset: "BNB".into() },
    ];

    std::thread::scope(|scope| {
        for i in 0..8 {
            let order = &order;
            let items = &items;
            scope.spawn(move || {
                validate_order(order).unwrap_or_else(|e| panic!("线程 {i} validate_order: {e}"));
                let aggregated = aggregate_commissions_by_asset(items)
                    .unwrap_or_else(|e| panic!("线程 {i} aggregate: {e}"));
                assert_eq!(aggregated.len(), 2, "线程 {i} 应按两资产汇总");
            });
        }
    });
}
