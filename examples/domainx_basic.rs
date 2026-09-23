#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! 最小消费者路径：值对象构造 + DX-VAL 校验 + 佣金聚合。
//!
//! ```bash
//! cargo run -p domainx --example domainx_basic
//! DOMAINX_LIVE_PROFILE=production cargo run -p domainx --example domainx_basic
//! ```

use std::env;
use std::str::FromStr;

use domainx::{
    Commission, Decimal, InstrumentKey, Order, OrderSide, OrderStatus, OrderType, TimeInForce,
    aggregate_commissions_by_asset, validate_order,
};

fn profile_label() -> &'static str {
    match env::var("DOMAINX_LIVE_PROFILE").as_deref() {
        Ok("production") => "production",
        _ => "development",
    }
}

fn sample_order() -> Order {
    Order {
        order_id: "ord-live-001".into(),
        instrument: InstrumentKey { exchange: "binance".into(), symbol: "BTCUSDT".into() },
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        status: OrderStatus::New,
        price: Some(Decimal::from_str("50000.00").expect("price")),
        stop_price: None,
        quantity: Decimal::from_str("1.5000").expect("quantity"),
        filled_quantity: Decimal::from_str("0.0000").expect("filled"),
        remaining_quantity: Decimal::from_str("1.5000").expect("remaining"),
        avg_fill_price: None,
        time_in_force: TimeInForce::Gtc,
        created_at: 1_700_000_000_000,
        updated_at: 1_700_000_000_000,
        client_order_id: Some("cli-live-1".into()),
    }
}

fn main() {
    let order = sample_order();
    validate_order(&order).expect("sample order passes DX-VAL");

    let commissions = vec![
        Commission { amount: Decimal::from_str("0.0010").expect("fee1"), asset: "BNB".into() },
        Commission { amount: Decimal::from_str("0.0005").expect("fee2"), asset: "BNB".into() },
        Commission { amount: Decimal::from_str("1.25").expect("fee3"), asset: "USDT".into() },
    ];
    let aggregated = aggregate_commissions_by_asset(&commissions).expect("commission aggregate");
    assert_eq!(aggregated.len(), 2, "BNB + USDT buckets");

    println!(
        "domainx-consumer: ok profile={} commissions={} instrument={}/{}",
        profile_label(),
        aggregated.len(),
        order.instrument.exchange,
        order.instrument.symbol,
    );
}
